use anyhow::Result;
use chrono::NaiveDateTime;
use log;
use serde::{Deserialize, Serialize};
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use sqlx::{migrate, Sqlite, Transaction};
use sqlx::{FromRow, QueryBuilder};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;
use uuid::Uuid;

const MAX_BATCH_SIZE: usize = 500;

static POOL: OnceLock<SqlitePool> = OnceLock::new();

/// Represents a single SMS message
#[derive(Debug, FromRow, Deserialize, Serialize, Default)]
pub struct SMS {
    pub id: i64,
    pub contact_id: String,
    pub timestamp: NaiveDateTime,
    pub message: String,
    pub device: String,
    pub send: bool,
    pub status: SmsStatus,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, sqlx::Type, Default)]
#[repr(i32)]
#[serde(into = "i32", from = "i32")]
pub enum SmsStatus {
    #[default]
    Unread = 0,
    Read = 1,
    Loading = 2,
    Failed = 3,
}

impl From<i32> for SmsStatus {
    fn from(value: i32) -> Self {
        match value {
            1 => SmsStatus::Read,
            2 => SmsStatus::Loading,
            3 => SmsStatus::Failed,
            _ => SmsStatus::Unread,
        }
    }
}

impl From<SmsStatus> for i32 {
    fn from(status: SmsStatus) -> Self {
        status as i32
    }
}

#[derive(Debug,Clone)]
pub struct ModemSMS {
    pub contact: String,
    pub timestamp: NaiveDateTime,
    pub message: String,
    pub device: String,
    pub send: bool,
}

#[derive(Debug, FromRow, Deserialize, Serialize, Default, Clone)]
pub struct Contact {
    pub id: String,
    pub name: String,
}

#[derive(Debug, FromRow, Deserialize, Serialize, Default, Clone)]
pub struct SMSPreview {
    pub device: String,
    pub message: String,
    pub timestamp: NaiveDateTime,
    pub status: SmsStatus,
}

#[derive(Debug, FromRow, Deserialize, Serialize, Default, Clone)]
pub struct Conversation {
    #[sqlx(flatten)]
    pub contact: Contact,
    #[sqlx(flatten)]
    pub sms_preview: SMSPreview,
}

impl SMS {
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

    pub async fn count_by_device(device: &str) -> Result<i64> {
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
    pub async fn count_by_contact_id(contact_id: &str) -> Result<i64> {
        let pool = get_pool()?;
        let count = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM sms WHERE contact_id = ?
            "#,
        )
        .bind(contact_id)
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
            SELECT id, contact_id, timestamp, message, device, send, status
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
    pub async fn paginate_by_contact_id(
        contact_id: &str,
        page: u32,
        per_page: u32,
    ) -> Result<(Vec<Self>, i64)> {
        if page == 0 {
            return Err(anyhow::anyhow!("Page number must be greater than 0"));
        }
        let offset = (page - 1) * per_page;
        let pool = get_pool()?;

        let mut tx = pool.begin().await?;

        let sms_list = sqlx::query_as(
            r#"
            SELECT id, contact_id, timestamp, message, device, send, status
            FROM sms 
            WHERE contact_id = ?
            ORDER BY timestamp DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(contact_id)
        .bind(per_page as i32)
        .bind(offset as i32)
        .fetch_all(&mut *tx)
        .await?;

        if page == 1 {
            sqlx::query(
                r#"
                UPDATE sms
                SET status = ?
                WHERE contact_id = ? AND status = ?
                "#,
            )
            .bind(SmsStatus::Read as i32)
            .bind(contact_id)
            .bind(SmsStatus::Unread as i32)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        let total = SMS::count_by_contact_id(contact_id).await?;

        Ok((sms_list, total))
    }

    pub async fn insert(&self) -> Result<i64> {
        let pool = get_pool()?;
        let sms_id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO sms (contact_id, timestamp, message, device, send, status)
            VALUES (?, ?, ?, ?, ?, ?) RETURNING id
            "#,
        )
        .bind(&self.contact_id)
        .bind(&self.timestamp)
        .bind(&self.message)
        .bind(&self.device)
        .bind(&self.send)
        .bind(self.status as i32)
        .fetch_one(pool)
        .await?;

        Ok(sms_id)
    }
    pub async fn query_unread_by_contact_id(contact_id: &str) -> Result<Vec<Self>> {
        let pool = get_pool()?;
        let mut tx = pool.begin().await?;
        let sms_list = sqlx::query_as(
            r#"
            SELECT id, contact_id, timestamp, message, device, send, status
            FROM sms 
            WHERE contact_id = ? AND status = ?
            ORDER BY timestamp DESC
            "#,
        )
        .bind(contact_id)
        .bind(SmsStatus::Unread as i32)
        .fetch_all(&mut *tx)
        .await?;

        sqlx::query(
            r#"
                UPDATE sms
                SET status = ?
                WHERE contact_id = ? AND status = ?
                "#,
        )
        .bind(SmsStatus::Read as i32)
        .bind(contact_id)
        .bind(SmsStatus::Unread as i32)
        .execute(&mut *tx)
        .await?;

        Ok(sms_list)
    }

    pub async fn _update_status(&self, status: SmsStatus) -> Result<()> {
        let pool = get_pool()?;
        sqlx::query(
            r#"
            UPDATE sms
            SET status = ?
            WHERE id = ?
            "#,
        )
        .bind(status as i32)
        .bind(self.id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update_status_by_id(id: i64, status: SmsStatus) -> Result<()> {
        let pool = get_pool()?;
        sqlx::query(
            r#"
            UPDATE sms
            SET status = ?
            WHERE id = ?
            "#,
        )
        .bind(status as i32)
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }
}

impl Contact {
    pub async fn query_all() -> Result<Vec<Self>> {
        let pool = get_pool()?;
        let contacts = sqlx::query_as("SELECT id, name FROM contacts")
            .fetch_all(pool)
            .await?;

        Ok(contacts)
    }
    pub async fn query_by_id(id: &str) -> Result<Self> {
        let pool = get_pool()?;
        let contact = sqlx::query_as("SELECT id, name FROM contacts WHERE id = ?")
            .bind(id)
            .fetch_one(pool)
            .await?;

        Ok(contact)
    }
    pub async fn insert(&self) -> Result<()> {
        let pool = get_pool()?;

        sqlx::query(
            r#"
            INSERT INTO contacts (id, name) VALUES (?, ?)
            "#,
        )
        .bind(&self.id)
        .bind(&self.name)
        .execute(pool)
        .await?;

        Ok(())
    }    pub async fn find_or_create(&mut self) -> Result<()> {
        let pool = get_pool()?;

        let existing_id = sqlx::query_scalar::<_, Option<String>>(
            r#"
            SELECT id FROM contacts WHERE name = ?
            "#,
        )
        .bind(&self.name)
        .fetch_one(pool)
        .await;

        if let Ok(Some(id)) = existing_id {
            self.id = id;
            return Ok(());
        }

        sqlx::query(
            r#"
            INSERT INTO contacts (id, name) VALUES (?, ?)
            "#,
        )
        .bind(&self.id)
        .bind(&self.name)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete_contacts_without_messages() -> Result<u64> {
        let pool = get_pool()?;

        let affected_rows = sqlx::query(
            r#"
            DELETE FROM contacts 
            WHERE id NOT IN (SELECT DISTINCT contact_id FROM sms)
            "#,
        )
        .execute(pool)
        .await?;

        Ok(affected_rows.rows_affected())
    }
    pub async fn delete_by_id(id: &str) -> Result<bool> {
        let pool = get_pool()?;

        sqlx::query(
            r#"
            DELETE FROM sms WHERE contact_id = ?
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        let result = sqlx::query(
            r#"
            DELETE FROM contacts WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

}

impl Conversation {
    pub async fn query_all() -> Result<Vec<Self>> {
        let pool = get_pool()?;

        let conversations = sqlx::query_as(
              "SELECT id, name, timestamp, message, status , device FROM v_contacts ORDER BY timestamp DESC"
        )
        .fetch_all(pool)
        .await?;

        Ok(conversations)
    }

    pub async fn _query_unread() -> Result<Vec<Self>> {
        let pool = get_pool()?;

        let conversations = sqlx::query_as(
              "SELECT id, name, timestamp, message, status, device FROM v_contacts where status = ? ORDER BY timestamp DESC"
        )
        .bind(SmsStatus::Unread as i32)
        .fetch_all(pool)
        .await?;

        Ok(conversations)
    }
    pub async fn query_by_contact_ids(contact_ids: &[String]) -> Result<Vec<Self>> {
        let pool = get_pool()?;

        if contact_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut query_builder = QueryBuilder::new(
            "SELECT id, name, timestamp, message, status, device FROM v_contacts WHERE id IN (",
        );

        let mut separated = query_builder.separated(", ");
        for id in contact_ids {
            separated.push_bind(id);
        }
        separated.push_unseparated(") ORDER BY timestamp DESC");

        let conversations = query_builder.build_query_as().fetch_all(pool).await?;

        Ok(conversations)
    }

    pub async fn _mark_as_read(&self) -> Result<()> {
        let pool = get_pool()?;

        sqlx::query(
            r#"
            UPDATE sms 
            SET status = ? 
            WHERE contact_id = ? 
            AND timestamp = (
                SELECT timestamp 
                FROM sms 
                WHERE contact_id = ? 
                ORDER BY timestamp DESC 
                LIMIT 1
            )"#,
        )
        .bind(SmsStatus::Read as i32)
        .bind(&self.contact.id)
        .bind(&self.contact.id)
        .execute(pool)
        .await?;

        Ok(())
    }
}

impl ModemSMS {
    pub async fn get_contact_id<'a>(
        &self,
        transaction: &'a mut Transaction<'_, Sqlite>,
    ) -> Result<String> {
        let contact_id = sqlx::query_scalar::<_, String>(
            r#"
            SELECT id FROM contacts WHERE name = ?
            "#,
        )
        .bind(&self.contact)
        .fetch_optional(&mut **transaction)
        .await?;

        if let Some(contact_id) = contact_id {
            Ok(contact_id)
        } else {
            let uuid = Uuid::new_v4().to_string();

            sqlx::query(
                r#"
                INSERT INTO contacts (id, name) VALUES (?, ?)
                "#,
            )
            .bind(&uuid)
            .bind(&self.contact)
            .execute(&mut **transaction)
            .await?;

            Ok(uuid)
        }
    }

    pub async fn insert(&self) -> Result<i64> {
        let pool = get_pool()?;

        let mut transaction = pool.begin().await?;
        let sms_id = self.insert_transaction(&mut transaction).await?;
        transaction.commit().await?;
        Ok(sms_id)
    }

    pub async fn insert_transaction(
        &self,
        transaction: &mut Transaction<'_, Sqlite>,
    ) -> Result<i64> {
        let contact_id = self.get_contact_id(transaction).await?;

        //When send is true, status defaults to Read
        let sms_id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO sms (contact_id, timestamp, message, device, send, status)
            VALUES (?, ?, ?, ?, ?, ?) RETURNING id
            "#,
        )
        .bind(contact_id)
        .bind(&self.timestamp)
        .bind(&self.message)
        .bind(&self.device)
        .bind(&self.send)
        .bind(if self.send {
            SmsStatus::Read as i32
        } else {
            SmsStatus::Unread as i32
        })
        .fetch_one(&mut **transaction)
        .await?;

        Ok(sms_id)
    }
    pub async fn bulk_insert(records: &[Self]) -> Result<Vec<String>> {
        let pool = get_pool()?;

        let mut transaction = pool.begin().await?;

        let mut contact_names = HashSet::new();
        for record in records {
            contact_names.insert(record.contact.clone());
        }

        let contact_names = contact_names
            .iter()
            .map(|contact| contact.clone())
            .collect::<Vec<String>>();

        // 查询已存在的联系人
        let mut query_builder = QueryBuilder::new("SELECT id, name FROM contacts WHERE name IN (");

        let mut separated = query_builder.separated(", ");
        for contact in contact_names.iter() {
            separated.push_bind(contact);
        }
        separated.push_unseparated(") ");

        let rows = query_builder.build().fetch_all(&mut *transaction).await?;

        let mut contact_map: HashMap<String, String> = rows
            .into_iter()
            .map(|row| {
                let id: String = row.try_get(0).unwrap();
                let name: String = row.try_get(1).unwrap();

                (name, id)
            })
            .collect();

        // 插入新联系人
        for contact_name in contact_names.iter() {
            if !contact_map.contains_key(contact_name) {
                let uuid = Uuid::new_v4().to_string();

                sqlx::query(
                    r#"
                    INSERT INTO contacts (id, name) VALUES (?, ?)
                    "#,
                )
                .bind(&uuid)
                .bind(contact_name)
                .execute(&mut *transaction)
                .await?;

                contact_map.insert(contact_name.clone(), uuid);
            }
        }

        for chunk in records.chunks(MAX_BATCH_SIZE) {
            let mut query_builder = QueryBuilder::new(
                "INSERT INTO sms (contact_id, timestamp, message, device, send, status) ",
            );

            query_builder.push_values(chunk, |mut b, sms| {
                b.push_bind(contact_map.get(&sms.contact))
                    .push_bind(&sms.timestamp)
                    .push_bind(&sms.message)
                    .push_bind(&sms.device)
                    .push_bind(&sms.send)
                    .push_bind(if sms.send {
                        SmsStatus::Read as i32
                    } else {
                        SmsStatus::Unread as i32
                    });
            });

            query_builder.build().execute(&mut *transaction).await?;
        }

        transaction.commit().await?;

        Ok(contact_map.into_iter().map(|(_, id)| id).collect())
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

    tokio::spawn(async {
        match Contact::delete_contacts_without_messages().await {
            Ok(count) => {
                log::info!("{} contacts without messages have been cleaned up", count);
            }
            Err(e) => {
                log::error!("Failed to clean up contacts without messages: {}", e);
            }
        }
    });

    Ok(())
}

/// Retrieves the database connection pool
fn get_pool() -> Result<&'static SqlitePool> {
    POOL.get()
        .ok_or(anyhow::anyhow!("Database not initialized"))
}
