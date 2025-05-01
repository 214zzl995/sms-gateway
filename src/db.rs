use anyhow::Result;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{migrate, pool};
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::{FromRow, QueryBuilder};
use std::sync::OnceLock;

const MAX_BATCH_SIZE: usize = 500;

static POOL: OnceLock<SqlitePool> = OnceLock::new();

/// Represents a single SMS message
#[derive(Debug, FromRow, Deserialize, Serialize, Default)]
pub struct SMS {
    pub id: i64, // SQLite auto-increment ID
    pub contact_id: i64,
    pub timestamp: NaiveDateTime,
    pub message: String,
    pub device: String,
    pub send: bool,
    pub read: bool,
}

pub struct ModemSMS {
    pub contact: String,
    pub timestamp: NaiveDateTime,
    pub message: String,
    pub device: String,
    pub send: bool,
}

#[derive(Debug, FromRow, Deserialize, Serialize, Default)]
pub struct Contact {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, FromRow, Deserialize, Serialize, Default)]
pub struct SMSPreview {
    pub message: String,
    pub timestamp: NaiveDateTime,
    pub read: bool,
}

#[derive(Debug, FromRow, Deserialize, Serialize, Default)]
pub struct Conversation {
    #[sqlx(flatten)]
    pub contact: Contact,
    #[sqlx(flatten)]
    pub sms_preview: SMSPreview,
}

impl SMS {
    /// Retrieves all SMS records
    pub async fn _all() -> Result<Vec<Self>> {
        let pool = get_pool()?;
        let sms_list = sqlx::query_as(
            r#"
            SELECT id, sender, receiver, timestamp, message, device, 
                   local_send 
            FROM sms 
            ORDER BY timestamp DESC
            "#,
        )
        .fetch_all(pool)
        .await?;
        Ok(sms_list)
    }

    pub async fn count() -> Result<i64> {
        let pool = get_pool()?;
        let count = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM sms
            "#,
        )
        .fetch_one(pool)
        .await?;
        Ok(count)
    }

    pub async fn device_count(device: &str) -> Result<i64> {
        let pool = get_pool()?;
        let count = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM sms WHERE device = ?
            "#,
        )
        .bind(device)
        .fetch_one(pool)
        .await?;
        Ok(count)
    }

    /// Retrieves paginated SMS records
    pub async fn paginate(page: u32, per_page: u32) -> Result<(Vec<Self>, i64)> {
        if page == 0 {
            return Err(anyhow::anyhow!("Page number must be greater than 0"));
        }
        let offset = (page - 1) * per_page;
        let pool = get_pool()?;

        let sms_list = sqlx::query_as(
            r#"
            SELECT id, sender, receiver, timestamp, message, device, 
                   local_send 
            FROM sms 
            ORDER BY timestamp DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(per_page as i32)
        .bind(offset as i32)
        .fetch_all(pool)
        .await?;

        let total = SMS::count().await?;

        Ok((sms_list, total))
    }

    pub async fn paginate_by_device(
        device: &str,
        page: u32,
        per_page: u32,
    ) -> Result<(Vec<Self>, i64)> {
        if page == 0 {
            return Err(anyhow::anyhow!("Page number must be greater than 0"));
        }
        let offset = (page - 1) * per_page;
        let pool = get_pool()?;

        let sms_list = sqlx::query_as(
            r#"
            SELECT id, sender, receiver, timestamp, message, device, 
                   local_send 
            FROM sms 
            WHERE device = ?
            ORDER BY timestamp DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(device)
        .bind(per_page as i32)
        .bind(offset as i32)
        .fetch_all(pool)
        .await?;

        let total = SMS::device_count(device).await?;

        Ok((sms_list, total))
    }

    /// Inserts a single SMS record into the database
    pub async fn _insert(&self) -> Result<()> {
        let pool = get_pool()?;

        sqlx::query(
            r#"
            INSERT INTO sms (contact_id, timestamp, message, device, send)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&self.contact_id)
        .bind(&self.timestamp)
        .bind(&self.message)
        .bind(&self.device)
        .bind(self.send)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Inserts multiple SMS records in bulk with batch size limitation
    pub async fn _bulk_insert(records: &[Self]) -> Result<()> {
        let pool = get_pool()?;

        // Process records in batches of MAX_BATCH_SIZE
        for chunk in records.chunks(MAX_BATCH_SIZE) {
            let mut query_builder = QueryBuilder::new(
                "INSERT INTO sms (contact_id, timestamp, message, device, send) ",
            );

            query_builder.push_values(chunk, |mut b, sms| {
                b.push_bind(&sms.contact_id)
                    .push_bind(&sms.timestamp)
                    .push_bind(&sms.message)
                    .push_bind(&sms.device)
                    .push_bind(sms.send);
            });

            query_builder.build().execute(pool).await?;
        }

        Ok(())
    }
}


impl ModemSMS {
    pub async fn get_contact_id(&self) -> Result<i64>{



        Ok(0)
    }

    pub async fn insert(&self) -> Result<i64>{
       

       Ok(0) 
    }

    pub async fn bulk_insert(records: &[Self]) -> Result<()> {
        // 先提取出所有的 contact先插入 拿个引用到时候装一下 每一条的contact_id 就行 (Self,i64)


        Ok(())
    }
}


/// Initializes SQLite database
pub async fn db_init() -> Result<()> {
    let db_path = "sqlite:///var/lib/sms-gateway/data.db";

    if !sqlx::Sqlite::database_exists(db_path).await? {
        sqlx::Sqlite::create_database(db_path).await?;
    };

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_path)
        .await?;

    migrate!("./migrations").run(&pool).await?;

    POOL.set(pool)
        .map_err(|_| anyhow::anyhow!("Failed to initialize database connection pool"))?;

    Ok(())
}

/// Retrieves the database connection pool
fn get_pool() -> Result<&'static SqlitePool> {
    POOL.get()
        .ok_or(anyhow::anyhow!("Database not initialized"))
}

