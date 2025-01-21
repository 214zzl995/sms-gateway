use anyhow::Result;
use serde::Deserialize;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::{FromRow, QueryBuilder, migrate::Migrator};
use std::sync::OnceLock;

const MAX_BATCH_SIZE: usize = 500;

static POOL: OnceLock<SqlitePool> = OnceLock::new();

/// SMS 结构体表示单条短信
#[derive(Debug, FromRow, Deserialize)]
pub struct SMS {
    pub id: Option<i64>,      // SQLite 自增ID
    #[sqlx(default)]          // 客户端使用的序号（数据库不存储）
    pub index: u32,
    pub sender: String,
    pub timestamp: i64,
    pub message: String,
    pub device: String,
    pub local_send: bool,
}

impl SMS {
    /// 查询全部记录
    pub async fn all() -> Result<Vec<Self>> {
        let pool = get_pool()?;
        let sms_list = sqlx::query_as(
            r#"
            SELECT id, sender, timestamp, message, device, 
                   local_send 
            FROM sms 
            ORDER BY timestamp DESC
            "#
        )
        .fetch_all(pool)
        .await?;
        Ok(sms_list)
    }

    /// 分页查询
    pub async fn paginate(page: u32, per_page: u32) -> Result<Vec<Self>> {
        if page == 0 {
            return Err(anyhow::anyhow!("页码必须大于0"));
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
            "#
        )
        .bind(per_page as i32)
        .bind(offset as i32)
        .fetch_all(pool)
        .await?;
        
        Ok(sms_list)
    }

    /// 插入单条记录
    pub async fn insert(&self) -> Result<()> {
        let pool = get_pool()?;
        
        sqlx::query(
            r#"
            INSERT INTO sms (sender, timestamp, message, device, local_send)
            VALUES (?, ?, ?, ?, ?)
            "#
        )
        .bind(&self.sender)
        .bind(self.timestamp)
        .bind(&self.message)
        .bind(&self.device)
        .bind(self.local_send) // 布尔转整数
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 批量插入（优化版）
    pub async fn bulk_insert(records: &[Self]) -> Result<()> {
        let pool = get_pool()?;
        let mut query_builder = QueryBuilder::new(
            "INSERT INTO sms (sender, timestamp, message, device, local_send) "
        );

        query_builder.push_values(records, |mut b, sms| {
            b.push_bind(&sms.sender)
             .push_bind(sms.timestamp)
             .push_bind(&sms.message)
             .push_bind(&sms.device)
             .push_bind(sms.local_send); // 批量转换布尔值
        });

        query_builder.build().execute(pool).await?;
        Ok(())
    }
}

/// 初始化SQLite数据库
pub async fn db_init() -> Result<()> {
    let db_path = "sqlite:///var/lib/sms-gateway/data.db";
    
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_path)
        .await?;

    // 执行迁移
    Migrator::new(std::path::Path::new("./migrations"))
        .await?
        .run(&pool)
        .await?;

    POOL.set(pool)
        .map_err(|_| anyhow::anyhow!("数据库连接池初始化失败"))?;

    Ok(())
}

/// 获取连接池
fn get_pool() -> Result<&'static SqlitePool> {
    POOL.get().ok_or(anyhow::anyhow!("数据库未初始化"))
}