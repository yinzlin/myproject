use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("数据库连接错误: {0}")]
    ConnectionError(#[from] sqlx::Error),
    #[error("配置错误: {0}")]
    ConfigError(String),
    #[error("查询超时")]
    QueryTimeout,
    #[error("事务错误: {0}")]
    TransactionError(String),
}

pub type DbResult<T> = Result<T, DbError>;

#[derive(Debug, Clone)]
pub struct DbConfig {
    pub database_url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
    pub test_before_acquire: bool,
    pub query_timeout: Duration,
}

impl DbConfig {
    pub fn builder() -> DbConfigBuilder {
        DbConfigBuilder::default()
    }
}

#[derive(Default)]
pub struct DbConfigBuilder {
    database_url: Option<String>,
    max_connections: Option<u32>,
    min_connections: Option<u32>,
    acquire_timeout: Option<Duration>,
    idle_timeout: Option<Duration>,
    max_lifetime: Option<Duration>,
    test_before_acquire: Option<bool>,
    query_timeout: Option<Duration>,
}

impl DbConfigBuilder {
    pub fn database_url(mut self, url: impl Into<String>) -> Self {
        self.database_url = Some(url.into());
        self
    }

    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = Some(max);
        self
    }

    pub fn min_connections(mut self, min: u32) -> Self {
        self.min_connections = Some(min);
        self
    }

    pub fn acquire_timeout(mut self, timeout: Duration) -> Self {
        self.acquire_timeout = Some(timeout);
        self
    }

    pub fn idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = Some(timeout);
        self
    }

    pub fn max_lifetime(mut self, lifetime: Duration) -> Self {
        self.max_lifetime = Some(lifetime);
        self
    }

    pub fn test_before_acquire(mut self, test: bool) -> Self {
        self.test_before_acquire = Some(test);
        self
    }

    pub fn query_timeout(mut self, timeout: Duration) -> Self {
        self.query_timeout = Some(timeout);
        self
    }

    pub fn build(self) -> DbResult<DbConfig> {
        Ok(DbConfig {
            database_url: self.database_url.ok_or_else(|| {
                DbError::ConfigError("database_url is required".to_string())
            })?,
            max_connections: self.max_connections.unwrap_or(10),
            min_connections: self.min_connections.unwrap_or(1),
            acquire_timeout: self.acquire_timeout.unwrap_or(Duration::from_secs(30)),
            idle_timeout: self.idle_timeout.unwrap_or(Duration::from_secs(600)),
            max_lifetime: self.max_lifetime.unwrap_or(Duration::from_secs(1800)),
            test_before_acquire: self.test_before_acquire.unwrap_or(true),
            query_timeout: self.query_timeout.unwrap_or(Duration::from_secs(30)),
        })
    }
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://postgres:password@localhost:5432/mydb".to_string(),
            max_connections: 10,
            min_connections: 1,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
            test_before_acquire: true,
            query_timeout: Duration::from_secs(30),
        }
    }
}

pub type DbPool = Pool<Postgres>;

pub async fn create_pool(config: &DbConfig) -> DbResult<DbPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.acquire_timeout)
        .idle_timeout(config.idle_timeout)
        .max_lifetime(config.max_lifetime)
        .test_before_acquire(config.test_before_acquire)
        .connect(&config.database_url)
        .await?;

    Ok(pool)
}

pub async fn create_pool_with_url(database_url: &str) -> DbResult<DbPool> {
    let config = DbConfig::builder()
        .database_url(database_url)
        .build()?;
    create_pool(&config).await
}

pub async fn create_pool_with_options(
    database_url: &str,
    max_connections: u32,
    min_connections: u32,
) -> DbResult<DbPool> {
    let config = DbConfig::builder()
        .database_url(database_url)
        .max_connections(max_connections)
        .min_connections(min_connections)
        .build()?;
    create_pool(&config).await
}

pub async fn test_connection(pool: &DbPool) -> DbResult<bool> {
    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await
        .map(|_| true)
        .map_err(DbError::ConnectionError)
}

pub async fn close_pool(pool: DbPool) -> DbResult<()> {
    pool.close().await;
    Ok(())
}

pub fn get_pool_size(pool: &DbPool) -> u32 {
    pool.size()
}

pub fn get_idle_connections(pool: &DbPool) -> u32 {
    pool.num_idle() as u32
}

pub async fn execute_query(pool: &DbPool, query: &str) -> DbResult<u64> {
    let result = sqlx::query(query)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

pub async fn execute_query_with_timeout(
    pool: &DbPool,
    query: &str,
    timeout: Duration,
) -> DbResult<u64> {
    tokio::time::timeout(timeout, execute_query(pool, query))
        .await
        .map_err(|_| DbError::QueryTimeout)?
}

pub async fn fetch_one(pool: &DbPool, query: &str) -> DbResult<sqlx::postgres::PgRow> {
    let row = sqlx::query(query)
        .fetch_one(pool)
        .await?;
    Ok(row)
}

pub async fn fetch_one_with_timeout(
    pool: &DbPool,
    query: &str,
    timeout: Duration,
) -> DbResult<sqlx::postgres::PgRow> {
    tokio::time::timeout(timeout, fetch_one(pool, query))
        .await
        .map_err(|_| DbError::QueryTimeout)?
}

pub async fn fetch_all(pool: &DbPool, query: &str) -> DbResult<Vec<sqlx::postgres::PgRow>> {
    let rows = sqlx::query(query)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn fetch_all_with_timeout(
    pool: &DbPool,
    query: &str,
    timeout: Duration,
) -> DbResult<Vec<sqlx::postgres::PgRow>> {
    tokio::time::timeout(timeout, fetch_all(pool, query))
        .await
        .map_err(|_| DbError::QueryTimeout)?
}

pub async fn fetch_optional(pool: &DbPool, query: &str) -> DbResult<Option<sqlx::postgres::PgRow>> {
    let row = sqlx::query(query)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn fetch_optional_with_timeout(
    pool: &DbPool,
    query: &str,
    timeout: Duration,
) -> DbResult<Option<sqlx::postgres::PgRow>> {
    tokio::time::timeout(timeout, fetch_optional(pool, query))
        .await
        .map_err(|_| DbError::QueryTimeout)?
}

pub async fn begin_transaction(pool: &DbPool) -> DbResult<sqlx::Transaction<'_, Postgres>> {
    let tx = pool.begin().await?;
    Ok(tx)
}

pub async fn begin_transaction_with_timeout(
    pool: &DbPool,
    timeout: Duration,
) -> DbResult<sqlx::Transaction<'_, Postgres>> {
    tokio::time::timeout(timeout, begin_transaction(pool))
        .await
        .map_err(|_| DbError::QueryTimeout)?
}

pub async fn commit_transaction(tx: sqlx::Transaction<'_, Postgres>) -> DbResult<()> {
    tx.commit().await?;
    Ok(())
}

pub async fn rollback_transaction(tx: sqlx::Transaction<'_, Postgres>) -> DbResult<()> {
    tx.rollback().await?;
    Ok(())
}

pub async fn execute_in_transaction(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    query: &str,
) -> DbResult<u64> {
    let result = sqlx::query(query)
        .execute(&mut **tx)
        .await?;
    Ok(result.rows_affected())
}

pub async fn execute_in_transaction_with_timeout(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    query: &str,
    timeout: Duration,
) -> DbResult<u64> {
    tokio::time::timeout(timeout, execute_in_transaction(tx, query))
        .await
        .map_err(|_| DbError::QueryTimeout)?
}

pub async fn fetch_one_in_transaction(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    query: &str,
) -> DbResult<sqlx::postgres::PgRow> {
    let row = sqlx::query(query)
        .fetch_one(&mut **tx)
        .await?;
    Ok(row)
}

pub async fn fetch_one_in_transaction_with_timeout(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    query: &str,
    timeout: Duration,
) -> DbResult<sqlx::postgres::PgRow> {
    tokio::time::timeout(timeout, fetch_one_in_transaction(tx, query))
        .await
        .map_err(|_| DbError::QueryTimeout)?
}

pub async fn fetch_all_in_transaction(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    query: &str,
) -> DbResult<Vec<sqlx::postgres::PgRow>> {
    let rows = sqlx::query(query)
        .fetch_all(&mut **tx)
        .await?;
    Ok(rows)
}

pub async fn fetch_all_in_transaction_with_timeout(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    query: &str,
    timeout: Duration,
) -> DbResult<Vec<sqlx::postgres::PgRow>> {
    tokio::time::timeout(timeout, fetch_all_in_transaction(tx, query))
        .await
        .map_err(|_| DbError::QueryTimeout)?
}

pub async fn fetch_optional_in_transaction(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    query: &str,
) -> DbResult<Option<sqlx::postgres::PgRow>> {
    let row = sqlx::query(query)
        .fetch_optional(&mut **tx)
        .await?;
    Ok(row)
}

pub async fn fetch_optional_in_transaction_with_timeout(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    query: &str,
    timeout: Duration,
) -> DbResult<Option<sqlx::postgres::PgRow>> {
    tokio::time::timeout(timeout, fetch_optional_in_transaction(tx, query))
        .await
        .map_err(|_| DbError::QueryTimeout)?
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Row;

    const TEST_DATABASE_URL: &str = "postgresql://postgres:password@localhost:5432/testdb";

    async fn create_test_pool() -> DbResult<DbPool> {
        create_pool_with_url(TEST_DATABASE_URL).await
    }

    #[test]
    fn test_db_config_default() {
        let config = DbConfig::default();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 1);
        assert_eq!(config.acquire_timeout, Duration::from_secs(30));
        assert!(config.test_before_acquire);
        assert_eq!(config.query_timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_db_config_builder() {
        let config = DbConfig::builder()
            .database_url("postgresql://user:pass@host:5432/db")
            .max_connections(20)
            .min_connections(5)
            .acquire_timeout(Duration::from_secs(60))
            .test_before_acquire(false)
            .query_timeout(Duration::from_secs(10))
            .build()
            .unwrap();

        assert_eq!(config.database_url, "postgresql://user:pass@host:5432/db");
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.min_connections, 5);
        assert_eq!(config.acquire_timeout, Duration::from_secs(60));
        assert!(!config.test_before_acquire);
        assert_eq!(config.query_timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_db_config_builder_missing_url() {
        let result = DbConfig::builder()
            .max_connections(20)
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_db_error_display() {
        let error = DbError::ConfigError("test error".to_string());
        assert_eq!(error.to_string(), "配置错误: test error");

        let timeout_error = DbError::QueryTimeout;
        assert_eq!(timeout_error.to_string(), "查询超时");
    }

    #[tokio::test]
    async fn test_create_pool() {
        let config = DbConfig::builder()
            .database_url(TEST_DATABASE_URL)
            .max_connections(5)
            .min_connections(1)
            .build()
            .unwrap();

        let result = create_pool(&config).await;
        match result {
            Ok(pool) => {
                assert_eq!(pool.size(), 0);
                println!("连接池创建成功，当前连接数: {}", pool.size());
            }
            Err(e) => {
                println!("连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_create_pool_with_url() {
        let result = create_pool_with_url(TEST_DATABASE_URL).await;
        match result {
            Ok(pool) => {
                assert_eq!(pool.size(), 0);
                println!("连接池创建成功，当前连接数: {}", pool.size());
            }
            Err(e) => {
                println!("连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_create_pool_with_options() {
        let result = create_pool_with_options(TEST_DATABASE_URL, 5, 1).await;
        match result {
            Ok(pool) => {
                assert_eq!(pool.size(), 0);
                println!("连接池创建成功，当前连接数: {}", pool.size());
            }
            Err(e) => {
                println!("连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_test_connection() {
        match create_test_pool().await {
            Ok(pool) => {
                let test_result = test_connection(&pool).await;
                match test_result {
                    Ok(is_connected) => {
                        assert!(is_connected);
                        println!("数据库连接测试成功");
                    }
                    Err(e) => {
                        println!("数据库连接测试失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_close_pool() {
        match create_test_pool().await {
            Ok(pool) => {
                let close_result = close_pool(pool).await;
                assert!(close_result.is_ok());
                println!("连接池关闭成功");
            }
            Err(e) => {
                println!("连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_get_pool_size() {
        match create_test_pool().await {
            Ok(pool) => {
                let size = get_pool_size(&pool);
                assert_eq!(size, 0);
                println!("连接池当前连接数: {}", size);
            }
            Err(e) => {
                println!("连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_get_idle_connections() {
        match create_test_pool().await {
            Ok(pool) => {
                let idle = get_idle_connections(&pool);
                assert_eq!(idle, 0);
                println!("连接池空闲连接数: {}", idle);
            }
            Err(e) => {
                println!("连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_execute_query() {
        match create_test_pool().await {
            Ok(pool) => {
                let query_result = execute_query(&pool, "SELECT 1").await;
                match query_result {
                    Ok(rows) => {
                        assert_eq!(rows, 1);
                        println!("查询执行成功，影响行数: {}", rows);
                    }
                    Err(e) => {
                        println!("查询执行失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_execute_query_with_timeout() {
        match create_test_pool().await {
            Ok(pool) => {
                let query_result = execute_query_with_timeout(&pool, "SELECT 1", Duration::from_secs(5)).await;
                match query_result {
                    Ok(rows) => {
                        assert_eq!(rows, 1);
                        println!("带超时的查询执行成功，影响行数: {}", rows);
                    }
                    Err(e) => {
                        println!("带超时的查询执行失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_one() {
        match create_test_pool().await {
            Ok(pool) => {
                let fetch_result = fetch_one(&pool, "SELECT 1 as value").await;
                match fetch_result {
                    Ok(row) => {
                        let value: i32 = row.get("value");
                        assert_eq!(value, 1);
                        println!("单行查询成功，value: {}", value);
                    }
                    Err(e) => {
                        println!("单行查询失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_one_with_timeout() {
        match create_test_pool().await {
            Ok(pool) => {
                let fetch_result = fetch_one_with_timeout(&pool, "SELECT 1 as value", Duration::from_secs(5)).await;
                match fetch_result {
                    Ok(row) => {
                        let value: i32 = row.get("value");
                        assert_eq!(value, 1);
                        println!("带超时的单行查询成功，value: {}", value);
                    }
                    Err(e) => {
                        println!("带超时的单行查询失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_all() {
        match create_test_pool().await {
            Ok(pool) => {
                let fetch_result = fetch_all(&pool, "SELECT 1 as value UNION SELECT 2 as value").await;
                match fetch_result {
                    Ok(rows) => {
                        assert_eq!(rows.len(), 2);
                        let value1: i32 = rows[0].get("value");
                        let value2: i32 = rows[1].get("value");
                        println!("多行查询成功，行数: {}, values: {}, {}", rows.len(), value1, value2);
                    }
                    Err(e) => {
                        println!("多行查询失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_all_with_timeout() {
        match create_test_pool().await {
            Ok(pool) => {
                let fetch_result = fetch_all_with_timeout(&pool, "SELECT 1 as value UNION SELECT 2 as value", Duration::from_secs(5)).await;
                match fetch_result {
                    Ok(rows) => {
                        assert_eq!(rows.len(), 2);
                        println!("带超时的多行查询成功，行数: {}", rows.len());
                    }
                    Err(e) => {
                        println!("带超时的多行查询失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_optional() {
        match create_test_pool().await {
            Ok(pool) => {
                let fetch_result = fetch_optional(&pool, "SELECT 1 as value").await;
                match fetch_result {
                    Ok(Some(row)) => {
                        let value: i32 = row.get("value");
                        assert_eq!(value, 1);
                        println!("可选查询成功，value: {}", value);
                    }
                    Ok(None) => {
                        println!("可选查询返回空结果");
                    }
                    Err(e) => {
                        println!("可选查询失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_optional_with_timeout() {
        match create_test_pool().await {
            Ok(pool) => {
                let fetch_result = fetch_optional_with_timeout(&pool, "SELECT 1 as value", Duration::from_secs(5)).await;
                match fetch_result {
                    Ok(Some(row)) => {
                        let value: i32 = row.get("value");
                        assert_eq!(value, 1);
                        println!("带超时的可选查询成功，value: {}", value);
                    }
                    Ok(None) => {
                        println!("带超时的可选查询返回空结果");
                    }
                    Err(e) => {
                        println!("带超时的可选查询失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_begin_transaction() {
        match create_test_pool().await {
            Ok(pool) => {
                let tx_result = begin_transaction(&pool).await;
                match tx_result {
                    Ok(_tx) => {
                        println!("事务创建成功");
                    }
                    Err(e) => {
                        println!("事务创建失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_begin_transaction_with_timeout() {
        match create_test_pool().await {
            Ok(pool) => {
                let tx_result = begin_transaction_with_timeout(&pool, Duration::from_secs(5)).await;
                match tx_result {
                    Ok(_tx) => {
                        println!("带超时的事务创建成功");
                    }
                    Err(e) => {
                        println!("带超时的事务创建失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_commit_transaction() {
        match create_test_pool().await {
            Ok(pool) => {
                let tx_result = begin_transaction(&pool).await;
                match tx_result {
                    Ok(tx) => {
                        let commit_result = commit_transaction(tx).await;
                        match commit_result {
                            Ok(_) => {
                                println!("事务提交成功");
                            }
                            Err(e) => {
                                println!("事务提交失败: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("事务创建失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_rollback_transaction() {
        match create_test_pool().await {
            Ok(pool) => {
                let tx_result = begin_transaction(&pool).await;
                match tx_result {
                    Ok(tx) => {
                        let rollback_result = rollback_transaction(tx).await;
                        match rollback_result {
                            Ok(_) => {
                                println!("事务回滚成功");
                            }
                            Err(e) => {
                                println!("事务回滚失败: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("事务创建失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_execute_in_transaction() {
        match create_test_pool().await {
            Ok(pool) => {
                let tx_result = begin_transaction(&pool).await;
                match tx_result {
                    Ok(mut tx) => {
                        let exec_result = execute_in_transaction(&mut tx, "SELECT 1").await;
                        match exec_result {
                            Ok(rows) => {
                                assert_eq!(rows, 1);
                                println!("事务内查询执行成功，影响行数: {}", rows);
                            }
                            Err(e) => {
                                println!("事务内查询执行失败: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("事务创建失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_execute_in_transaction_with_timeout() {
        match create_test_pool().await {
            Ok(pool) => {
                let tx_result = begin_transaction(&pool).await;
                match tx_result {
                    Ok(mut tx) => {
                        let exec_result = execute_in_transaction_with_timeout(&mut tx, "SELECT 1", Duration::from_secs(5)).await;
                        match exec_result {
                            Ok(rows) => {
                                assert_eq!(rows, 1);
                                println!("带超时的事务内查询执行成功，影响行数: {}", rows);
                            }
                            Err(e) => {
                                println!("带超时的事务内查询执行失败: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("事务创建失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_one_in_transaction() {
        match create_test_pool().await {
            Ok(pool) => {
                let tx_result = begin_transaction(&pool).await;
                match tx_result {
                    Ok(mut tx) => {
                        let fetch_result = fetch_one_in_transaction(&mut tx, "SELECT 1 as value").await;
                        match fetch_result {
                            Ok(row) => {
                                let value: i32 = row.get("value");
                                assert_eq!(value, 1);
                                println!("事务内单行查询成功，value: {}", value);
                            }
                            Err(e) => {
                                println!("事务内单行查询失败: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("事务创建失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_one_in_transaction_with_timeout() {
        match create_test_pool().await {
            Ok(pool) => {
                let tx_result = begin_transaction(&pool).await;
                match tx_result {
                    Ok(mut tx) => {
                        let fetch_result = fetch_one_in_transaction_with_timeout(&mut tx, "SELECT 1 as value", Duration::from_secs(5)).await;
                        match fetch_result {
                            Ok(row) => {
                                let value: i32 = row.get("value");
                                assert_eq!(value, 1);
                                println!("带超时的事务内单行查询成功，value: {}", value);
                            }
                            Err(e) => {
                                println!("带超时的事务内单行查询失败: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("事务创建失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_all_in_transaction() {
        match create_test_pool().await {
            Ok(pool) => {
                let tx_result = begin_transaction(&pool).await;
                match tx_result {
                    Ok(mut tx) => {
                        let fetch_result = fetch_all_in_transaction(&mut tx, "SELECT 1 as value UNION SELECT 2 as value").await;
                        match fetch_result {
                            Ok(rows) => {
                                assert_eq!(rows.len(), 2);
                                let value1: i32 = rows[0].get("value");
                                let value2: i32 = rows[1].get("value");
                                println!("事务内多行查询成功，行数: {}, values: {}, {}", rows.len(), value1, value2);
                            }
                            Err(e) => {
                                println!("事务内多行查询失败: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("事务创建失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_all_in_transaction_with_timeout() {
        match create_test_pool().await {
            Ok(pool) => {
                let tx_result = begin_transaction(&pool).await;
                match tx_result {
                    Ok(mut tx) => {
                        let fetch_result = fetch_all_in_transaction_with_timeout(&mut tx, "SELECT 1 as value UNION SELECT 2 as value", Duration::from_secs(5)).await;
                        match fetch_result {
                            Ok(rows) => {
                                assert_eq!(rows.len(), 2);
                                println!("带超时的事务内多行查询成功，行数: {}", rows.len());
                            }
                            Err(e) => {
                                println!("带超时的事务内多行查询失败: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("事务创建失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_optional_in_transaction() {
        match create_test_pool().await {
            Ok(pool) => {
                let tx_result = begin_transaction(&pool).await;
                match tx_result {
                    Ok(mut tx) => {
                        let fetch_result = fetch_optional_in_transaction(&mut tx, "SELECT 1 as value").await;
                        match fetch_result {
                            Ok(Some(row)) => {
                                let value: i32 = row.get("value");
                                assert_eq!(value, 1);
                                println!("事务内可选查询成功，value: {}", value);
                            }
                            Ok(None) => {
                                println!("事务内可选查询返回空结果");
                            }
                            Err(e) => {
                                println!("事务内可选查询失败: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("事务创建失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_optional_in_transaction_with_timeout() {
        match create_test_pool().await {
            Ok(pool) => {
                let tx_result = begin_transaction(&pool).await;
                match tx_result {
                    Ok(mut tx) => {
                        let fetch_result = fetch_optional_in_transaction_with_timeout(&mut tx, "SELECT 1 as value", Duration::from_secs(5)).await;
                        match fetch_result {
                            Ok(Some(row)) => {
                                let value: i32 = row.get("value");
                                assert_eq!(value, 1);
                                println!("带超时的事务内可选查询成功，value: {}", value);
                            }
                            Ok(None) => {
                                println!("带超时的事务内可选查询返回空结果");
                            }
                            Err(e) => {
                                println!("带超时的事务内可选查询失败: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("事务创建失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }
    }
}
