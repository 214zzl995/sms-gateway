use anyhow::Result;
use chrono::NaiveDateTime;
use serde::Deserialize;
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::{migrate::Migrator, FromRow, QueryBuilder};
use std::sync::OnceLock;

const MAX_BATCH_SIZE: usize = 500;

static POOL: OnceLock<SqlitePool> = OnceLock::new();

/// Represents a single SMS message
#[derive(Debug, FromRow, Deserialize)]
pub struct SMS {
    pub id: Option<i64>, // SQLite auto-increment ID
    #[sqlx(default)] // Client-side sequence number (not stored in DB)
    pub index: u32,
    pub sender: String,
    pub timestamp: NaiveDateTime,
    pub message: String,
    pub device: String,
    pub local_send: bool,
}

impl SMS {
    /// Retrieves all SMS records
    pub async fn all() -> Result<Vec<Self>> {
        let pool = get_pool()?;
        let sms_list = sqlx::query_as(
            r#"
            SELECT id, sender, timestamp, message, device, 
                   local_send 
            FROM sms 
            ORDER BY timestamp DESC
            "#,
        )
        .fetch_all(pool)
        .await?;
        Ok(sms_list)
    }

    /// Retrieves paginated SMS records
    pub async fn paginate(page: u32, per_page: u32) -> Result<Vec<Self>> {
        if page == 0 {
            return Err(anyhow::anyhow!("Page number must be greater than 0"));
        }
        let offset = (page - 1) * per_page;
        let pool = get_pool()?;

        let sms_list = sqlx::query_as(
            r#"
            SELECT id, sender, timestamp, message, device, 
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

        Ok(sms_list)
    }

    /// Inserts a single SMS record into the database
    pub async fn insert(&self) -> Result<()> {
        let pool = get_pool()?;

        sqlx::query(
            r#"
            INSERT INTO sms (sender, timestamp, message, device, local_send)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&self.sender)
        .bind(&self.timestamp)
        .bind(&self.message)
        .bind(&self.device)
        .bind(self.local_send)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Inserts multiple SMS records in bulk with batch size limitation
    pub async fn bulk_insert(records: &[Self]) -> Result<()> {
        let pool = get_pool()?;

        // Process records in batches of MAX_BATCH_SIZE
        for chunk in records.chunks(MAX_BATCH_SIZE) {
            let mut query_builder = QueryBuilder::new(
                "INSERT INTO sms (sender, timestamp, message, device, local_send) ",
            );

            query_builder.push_values(chunk, |mut b, sms| {
                b.push_bind(&sms.sender)
                    .push_bind(&sms.timestamp)
                    .push_bind(&sms.message)
                    .push_bind(&sms.device)
                    .push_bind(sms.local_send);
            });

            query_builder.build().execute(pool).await?;
        }

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

    // Run migrations
    Migrator::new(std::path::Path::new("./migrations"))
        .await?
        .run(&pool)
        .await?;

    POOL.set(pool)
        .map_err(|_| anyhow::anyhow!("Failed to initialize database connection pool"))?;

    Ok(())
}

/// Retrieves the database connection pool
fn get_pool() -> Result<&'static SqlitePool> {
    POOL.get()
        .ok_or(anyhow::anyhow!("Database not initialized"))
}
