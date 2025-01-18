use anyhow::{Context, Result};
use log::info;
use serde::Deserialize;
use sqlx::any::AnyPoolOptions;
use sqlx::migrate::{MigrateDatabase, Migrator};
use sqlx::{AnyPool, FromRow, Row};
use std::collections::HashMap;

/// SMS struct, representing a single SMS message
#[derive(Debug, FromRow, Deserialize)]
struct SMS {
    uuid: String,
    message: String,
    mobile: String,
    status: i32,
    retries: i32,
    device: Option<String>,
    created_at: Option<String>,
    updated_at: Option<String>,
}

/// Initialize the database
pub async fn db_init() -> Result<AnyPool> {
    sqlx::any::install_default_drivers();
    let db_path = "sqlite:///var/sms-gateway/data.db";

    if !sqlx::Any::database_exists(db_path).await? {
        sqlx::Any::create_database(db_path).await?;
    };

    let pool = AnyPoolOptions::new()
        .max_connections(5)
        .connect(&db_path)
        .await?;

    let _backend_name = pool.acquire().await?.backend_name();

    let migrate: Migrator = sqlx::migrate!("./migrations/sqlite");
    migrate.run(&pool).await?;

    Ok(pool)
}

/// Insert a new SMS message into the database
async fn insert_message(pool: &AnyPool, sms: &SMS) -> Result<()> {
    info!("Inserting message: {:?}", sms);
    sqlx::query(
        r#"
        INSERT INTO messages (uuid, message, mobile)
        VALUES (?, ?, ?)
        "#,
    )
    .bind(&sms.uuid)
    .bind(&sms.message)
    .bind(&sms.mobile)
    .execute(pool)
    .await
    .context("Failed to insert message")?;

    Ok(())
}

/// Update the status of an SMS message
async fn update_message_status(pool: &AnyPool, sms: &SMS) -> Result<()> {
    info!("Updating message status: {:?}", sms);
    sqlx::query(
        r#"
        UPDATE messages
        SET status = ?, retries = ?, device = ?, updated_at = CURRENT_TIMESTAMP
        WHERE uuid = ?
        "#,
    )
    .bind(sms.status)
    .bind(sms.retries)
    .bind(&sms.device)
    .bind(&sms.uuid)
    .execute(pool)
    .await
    .context("Failed to update message status")?;

    Ok(())
}

/// Fetch pending SMS messages (status != processed and retries < limit)
async fn get_pending_messages(pool: &AnyPool, buffer_size: i32) -> Result<Vec<SMS>> {
    info!("Fetching pending messages");
    let messages = sqlx::query_as::<_, SMS>(
        r#"
        SELECT uuid, message, mobile, status, retries
        FROM messages
        WHERE status != ? AND retries < ?
        LIMIT ?
        "#,
    )
    .bind(1) // SMSProcessed status
    .bind(3) // SMSRetryLimit
    .bind(buffer_size)
    .fetch_all(pool)
    .await
    .context("Failed to fetch pending messages")?;

    Ok(messages)
}

/// Fetch SMS messages with a custom filter
async fn get_messages(pool: &AnyPool, filter: &str) -> Result<Vec<SMS>> {
    info!("Fetching messages with filter: {}", filter);
    let query = format!(
        r#"
        SELECT uuid, message, mobile, status, retries, device, created_at, updated_at
        FROM messages {}
        "#,
        filter
    );
    let messages = sqlx::query_as::<_, SMS>(&query)
        .fetch_all(pool)
        .await
        .context("Failed to fetch messages")?;

    Ok(messages)
}

/// Fetch the count of SMS messages sent in the last 7 days
async fn get_last_7_days_message_count(pool: &AnyPool) -> Result<HashMap<String, i32>> {
    info!("Fetching last 7 days message count");
    let rows = sqlx::query(
        r#"
        SELECT strftime('%Y-%m-%d', created_at) as datestamp, COUNT(id) as messagecount
        FROM messages
        GROUP BY datestamp
        ORDER BY datestamp DESC
        LIMIT 7
        "#,
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch last 7 days message count")?;

    let mut day_count = HashMap::new();
    for row in rows {
        let day: String = row.get("datestamp");
        let count: i32 = row.get("messagecount");
        day_count.insert(day, count);
    }

    Ok(day_count)
}

/// Fetch a summary of SMS messages by status
async fn get_status_summary(pool: &AnyPool) -> Result<Vec<i32>> {
    info!("Fetching status summary");
    let rows = sqlx::query(
        r#"
        SELECT status, COUNT(id) as messagecount
        FROM messages
        GROUP BY status
        ORDER BY status
        "#,
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch status summary")?;

    let mut status_summary = vec![0; 3]; // Assuming 3 statuses
    for row in rows {
        let status: i32 = row.get("status");
        let count: i32 = row.get("messagecount");
        if status >= 0 && status < 3 {
            status_summary[status as usize] = count;
        }
    }

    Ok(status_summary)
}
