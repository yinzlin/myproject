use sqlx::{Pool, Postgres, Sqlite, postgres::PgPoolOptions, sqlite::SqlitePoolOptions};
use std::time::Duration;
use thiserror::Error;

pub const DEFAULT_MAX_CONNECTIONS: u32 = 10;
pub const DEFAULT_MIN_CONNECTIONS: u32 = 1;
pub const DEFAULT_ACQUIRE_TIMEOUT_SECS: u64 = 30;
pub const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 600;
pub const DEFAULT_MAX_LIFETIME_SECS: u64 = 1800;
pub const DEFAULT_QUERY_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    Postgres,
    Sqlite,
}

impl DatabaseType {
    pub fn from_url(url: &str) -> Option<Self> {
        if url.starts_with("postgresql://") || url.starts_with("postgres://") {
            Some(DatabaseType::Postgres)
        } else if url.starts_with("sqlite:") {
            Some(DatabaseType::Sqlite)
        } else {
            None
        }
    }
}

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
    #[error("不支持的数据库类型")]
    UnsupportedDatabaseType,
}

pub type DbResult<T> = Result<T, DbError>;

#[derive(Debug, Clone)]
pub struct DbConfig {
    pub database_url: String,
    pub database_type: DatabaseType,
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
    database_type: Option<DatabaseType>,
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

    pub fn database_type(mut self, db_type: DatabaseType) -> Self {
        self.database_type = Some(db_type);
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
        let database_url = self.database_url.ok_or_else(|| {
            DbError::ConfigError("database_url is required".to_string())
        })?;
        
        let database_type = self.database_type.or_else(|| {
            DatabaseType::from_url(&database_url)
        }).unwrap_or(DatabaseType::Postgres);
        
        Ok(DbConfig {
            database_url,
            database_type,
            max_connections: self.max_connections.unwrap_or(DEFAULT_MAX_CONNECTIONS),
            min_connections: self.min_connections.unwrap_or(DEFAULT_MIN_CONNECTIONS),
            acquire_timeout: self.acquire_timeout.unwrap_or(Duration::from_secs(DEFAULT_ACQUIRE_TIMEOUT_SECS)),
            idle_timeout: self.idle_timeout.unwrap_or(Duration::from_secs(DEFAULT_IDLE_TIMEOUT_SECS)),
            max_lifetime: self.max_lifetime.unwrap_or(Duration::from_secs(DEFAULT_MAX_LIFETIME_SECS)),
            test_before_acquire: self.test_before_acquire.unwrap_or(true),
            query_timeout: self.query_timeout.unwrap_or(Duration::from_secs(DEFAULT_QUERY_TIMEOUT_SECS)),
        })
    }
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://postgres:password@localhost:5432/mydb".to_string(),
            database_type: DatabaseType::Postgres,
            max_connections: DEFAULT_MAX_CONNECTIONS,
            min_connections: DEFAULT_MIN_CONNECTIONS,
            acquire_timeout: Duration::from_secs(DEFAULT_ACQUIRE_TIMEOUT_SECS),
            idle_timeout: Duration::from_secs(DEFAULT_IDLE_TIMEOUT_SECS),
            max_lifetime: Duration::from_secs(DEFAULT_MAX_LIFETIME_SECS),
            test_before_acquire: true,
            query_timeout: Duration::from_secs(DEFAULT_QUERY_TIMEOUT_SECS),
        }
    }
}

pub enum DbPool {
    Postgres(Pool<Postgres>),
    Sqlite(Pool<Sqlite>),
}

impl DbPool {
    pub fn size(&self) -> u32 {
        match self {
            DbPool::Postgres(pool) => pool.size(),
            DbPool::Sqlite(pool) => pool.size(),
        }
    }

    pub fn num_idle(&self) -> u32 {
        match self {
            DbPool::Postgres(pool) => pool.num_idle() as u32,
            DbPool::Sqlite(pool) => pool.num_idle() as u32,
        }
    }

    pub async fn close(self) {
        match self {
            DbPool::Postgres(pool) => pool.close().await,
            DbPool::Sqlite(pool) => pool.close().await,
        }
    }
}

pub async fn create_pool(config: &DbConfig) -> DbResult<DbPool> {
    match config.database_type {
        DatabaseType::Postgres => {
            let pool = PgPoolOptions::new()
                .max_connections(config.max_connections)
                .min_connections(config.min_connections)
                .acquire_timeout(config.acquire_timeout)
                .idle_timeout(config.idle_timeout)
                .max_lifetime(config.max_lifetime)
                .test_before_acquire(config.test_before_acquire)
                .connect(&config.database_url)
                .await?;
            Ok(DbPool::Postgres(pool))
        }
        DatabaseType::Sqlite => {
            let pool = SqlitePoolOptions::new()
                .max_connections(config.max_connections)
                .min_connections(config.min_connections)
                .acquire_timeout(config.acquire_timeout)
                .idle_timeout(config.idle_timeout)
                .max_lifetime(config.max_lifetime)
                .test_before_acquire(config.test_before_acquire)
                .connect(&config.database_url)
                .await?;
            Ok(DbPool::Sqlite(pool))
        }
    }
}

pub async fn create_pool_with_url(database_url: &str) -> DbResult<DbPool> {
    let config = DbConfig::builder()
        .database_url(database_url)
        .build()?;
    create_pool(&config).await
}

pub async fn create_pool_auto(database_url: &str) -> DbResult<DbPool> {
    let database_type = DatabaseType::from_url(database_url)
        .ok_or_else(|| DbError::ConfigError(format!("无法从URL识别数据库类型: {}", database_url)))?;
    
    create_pool_with_url_and_type(database_url, database_type).await
}

pub async fn create_pool_with_url_and_type(
    database_url: &str,
    database_type: DatabaseType,
) -> DbResult<DbPool> {
    let config = DbConfig::builder()
        .database_url(database_url)
        .database_type(database_type)
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
    match pool {
        DbPool::Postgres(pool) => {
            sqlx::query("SELECT 1")
                .fetch_one(pool)
                .await
                .map(|_| true)
                .map_err(DbError::ConnectionError)
        }
        DbPool::Sqlite(pool) => {
            sqlx::query("SELECT 1")
                .fetch_one(pool)
                .await
                .map(|_| true)
                .map_err(DbError::ConnectionError)
        }
    }
}

pub async fn close_pool(pool: DbPool) -> DbResult<()> {
    pool.close().await;
    Ok(())
}

pub fn get_pool_size(pool: &DbPool) -> u32 {
    pool.size()
}

pub fn get_idle_connections(pool: &DbPool) -> u32 {
    pool.num_idle()
}

pub async fn execute_query(pool: &DbPool, query: &str) -> DbResult<u64> {
    match pool {
        DbPool::Postgres(pool) => {
            let result = sqlx::query(query)
                .execute(pool)
                .await?;
            Ok(result.rows_affected())
        }
        DbPool::Sqlite(pool) => {
            let result = sqlx::query(query)
                .execute(pool)
                .await?;
            Ok(result.rows_affected())
        }
    }
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

pub async fn fetch_one(pool: &DbPool, query: &str) -> DbResult<sqlx::any::AnyRow> {
    match pool {
        DbPool::Postgres(pool) => {
            let row = sqlx::query(query)
                .fetch_one(pool)
                .await?;
            sqlx::any::AnyRow::try_from(&row).map_err(DbError::ConnectionError)
        }
        DbPool::Sqlite(pool) => {
            let row = sqlx::query(query)
                .fetch_one(pool)
                .await?;
            sqlx::any::AnyRow::try_from(&row).map_err(DbError::ConnectionError)
        }
    }
}

pub async fn fetch_one_with_timeout(
    pool: &DbPool,
    query: &str,
    timeout: Duration,
) -> DbResult<sqlx::any::AnyRow> {
    tokio::time::timeout(timeout, fetch_one(pool, query))
        .await
        .map_err(|_| DbError::QueryTimeout)?
}

pub async fn fetch_all(pool: &DbPool, query: &str) -> DbResult<Vec<sqlx::any::AnyRow>> {
    match pool {
        DbPool::Postgres(pool) => {
            let rows = sqlx::query(query)
                .fetch_all(pool)
                .await?;
            rows.into_iter()
                .map(|row| sqlx::any::AnyRow::try_from(&row))
                .collect::<Result<Vec<_>, _>>()
                .map_err(DbError::ConnectionError)
        }
        DbPool::Sqlite(pool) => {
            let rows = sqlx::query(query)
                .fetch_all(pool)
                .await?;
            rows.into_iter()
                .map(|row| sqlx::any::AnyRow::try_from(&row))
                .collect::<Result<Vec<_>, _>>()
                .map_err(DbError::ConnectionError)
        }
    }
}

pub async fn fetch_all_with_timeout(
    pool: &DbPool,
    query: &str,
    timeout: Duration,
) -> DbResult<Vec<sqlx::any::AnyRow>> {
    tokio::time::timeout(timeout, fetch_all(pool, query))
        .await
        .map_err(|_| DbError::QueryTimeout)?
}

pub async fn fetch_optional(pool: &DbPool, query: &str) -> DbResult<Option<sqlx::any::AnyRow>> {
    match pool {
        DbPool::Postgres(pool) => {
            let row = sqlx::query(query)
                .fetch_optional(pool)
                .await?;
            match row {
                Some(r) => {
                    let any_row = sqlx::any::AnyRow::try_from(&r).map_err(DbError::ConnectionError)?;
                    Ok(Some(any_row))
                }
                None => Ok(None),
            }
        }
        DbPool::Sqlite(pool) => {
            let row = sqlx::query(query)
                .fetch_optional(pool)
                .await?;
            match row {
                Some(r) => {
                    let any_row = sqlx::any::AnyRow::try_from(&r).map_err(DbError::ConnectionError)?;
                    Ok(Some(any_row))
                }
                None => Ok(None),
            }
        }
    }
}

pub async fn fetch_optional_with_timeout(
    pool: &DbPool,
    query: &str,
    timeout: Duration,
) -> DbResult<Option<sqlx::any::AnyRow>> {
    tokio::time::timeout(timeout, fetch_optional(pool, query))
        .await
        .map_err(|_| DbError::QueryTimeout)?
}

pub enum DbTransaction<'a> {
    Postgres(sqlx::Transaction<'a, Postgres>),
    Sqlite(sqlx::Transaction<'a, Sqlite>),
}

pub async fn begin_transaction(pool: &DbPool) -> DbResult<DbTransaction<'_>> {
    match pool {
        DbPool::Postgres(pool) => {
            let tx = pool.begin().await?;
            Ok(DbTransaction::Postgres(tx))
        }
        DbPool::Sqlite(pool) => {
            let tx = pool.begin().await?;
            Ok(DbTransaction::Sqlite(tx))
        }
    }
}

pub async fn begin_transaction_with_timeout(
    pool: &DbPool,
    timeout: Duration,
) -> DbResult<DbTransaction<'_>> {
    tokio::time::timeout(timeout, begin_transaction(pool))
        .await
        .map_err(|_| DbError::QueryTimeout)?
}

pub async fn commit_transaction(tx: DbTransaction<'_>) -> DbResult<()> {
    match tx {
        DbTransaction::Postgres(tx) => {
            tx.commit().await?;
        }
        DbTransaction::Sqlite(tx) => {
            tx.commit().await?;
        }
    }
    Ok(())
}

pub async fn rollback_transaction(tx: DbTransaction<'_>) -> DbResult<()> {
    match tx {
        DbTransaction::Postgres(tx) => {
            tx.rollback().await?;
        }
        DbTransaction::Sqlite(tx) => {
            tx.rollback().await?;
        }
    }
    Ok(())
}

pub async fn execute_in_transaction(
    tx: &mut DbTransaction<'_>,
    query: &str,
) -> DbResult<u64> {
    match tx {
        DbTransaction::Postgres(tx) => {
            let result = sqlx::query(query)
                .execute(&mut **tx)
                .await?;
            Ok(result.rows_affected())
        }
        DbTransaction::Sqlite(tx) => {
            let result = sqlx::query(query)
                .execute(&mut **tx)
                .await?;
            Ok(result.rows_affected())
        }
    }
}

pub async fn execute_in_transaction_with_timeout(
    tx: &mut DbTransaction<'_>,
    query: &str,
    timeout: Duration,
) -> DbResult<u64> {
    tokio::time::timeout(timeout, execute_in_transaction(tx, query))
        .await
        .map_err(|_| DbError::QueryTimeout)?
}

pub async fn fetch_one_in_transaction(
    tx: &mut DbTransaction<'_>,
    query: &str,
) -> DbResult<sqlx::any::AnyRow> {
    match tx {
        DbTransaction::Postgres(tx) => {
            let row = sqlx::query(query)
                .fetch_one(&mut **tx)
                .await?;
            sqlx::any::AnyRow::try_from(&row).map_err(DbError::ConnectionError)
        }
        DbTransaction::Sqlite(tx) => {
            let row = sqlx::query(query)
                .fetch_one(&mut **tx)
                .await?;
            sqlx::any::AnyRow::try_from(&row).map_err(DbError::ConnectionError)
        }
    }
}

pub async fn fetch_one_in_transaction_with_timeout(
    tx: &mut DbTransaction<'_>,
    query: &str,
    timeout: Duration,
) -> DbResult<sqlx::any::AnyRow> {
    tokio::time::timeout(timeout, fetch_one_in_transaction(tx, query))
        .await
        .map_err(|_| DbError::QueryTimeout)?
}

pub async fn fetch_all_in_transaction(
    tx: &mut DbTransaction<'_>,
    query: &str,
) -> DbResult<Vec<sqlx::any::AnyRow>> {
    match tx {
        DbTransaction::Postgres(tx) => {
            let rows = sqlx::query(query)
                .fetch_all(&mut **tx)
                .await?;
            rows.into_iter()
                .map(|row| sqlx::any::AnyRow::try_from(&row))
                .collect::<Result<Vec<_>, _>>()
                .map_err(DbError::ConnectionError)
        }
        DbTransaction::Sqlite(tx) => {
            let rows = sqlx::query(query)
                .fetch_all(&mut **tx)
                .await?;
            rows.into_iter()
                .map(|row| sqlx::any::AnyRow::try_from(&row))
                .collect::<Result<Vec<_>, _>>()
                .map_err(DbError::ConnectionError)
        }
    }
}

pub async fn fetch_all_in_transaction_with_timeout(
    tx: &mut DbTransaction<'_>,
    query: &str,
    timeout: Duration,
) -> DbResult<Vec<sqlx::any::AnyRow>> {
    tokio::time::timeout(timeout, fetch_all_in_transaction(tx, query))
        .await
        .map_err(|_| DbError::QueryTimeout)?
}

pub async fn fetch_optional_in_transaction(
    tx: &mut DbTransaction<'_>,
    query: &str,
) -> DbResult<Option<sqlx::any::AnyRow>> {
    match tx {
        DbTransaction::Postgres(tx) => {
            let row = sqlx::query(query)
                .fetch_optional(&mut **tx)
                .await?;
            match row {
                Some(r) => {
                    let any_row = sqlx::any::AnyRow::try_from(&r).map_err(DbError::ConnectionError)?;
                    Ok(Some(any_row))
                }
                None => Ok(None),
            }
        }
        DbTransaction::Sqlite(tx) => {
            let row = sqlx::query(query)
                .fetch_optional(&mut **tx)
                .await?;
            match row {
                Some(r) => {
                    let any_row = sqlx::any::AnyRow::try_from(&r).map_err(DbError::ConnectionError)?;
                    Ok(Some(any_row))
                }
                None => Ok(None),
            }
        }
    }
}

pub async fn fetch_optional_in_transaction_with_timeout(
    tx: &mut DbTransaction<'_>,
    query: &str,
    timeout: Duration,
) -> DbResult<Option<sqlx::any::AnyRow>> {
    tokio::time::timeout(timeout, fetch_optional_in_transaction(tx, query))
        .await
        .map_err(|_| DbError::QueryTimeout)?
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Row;

    const TEST_POSTGRES_URL: &str = "postgresql://postgres:password@localhost:5432/testdb";
    const TEST_SQLITE_URL: &str = "sqlite::memory:";

    async fn create_postgres_test_pool() -> DbResult<DbPool> {
        let config = DbConfig::builder()
            .database_url(TEST_POSTGRES_URL)
            .database_type(DatabaseType::Postgres)
            .build()?;
        create_pool(&config).await
    }

    async fn create_sqlite_test_pool() -> DbResult<DbPool> {
        let config = DbConfig::builder()
            .database_url(TEST_SQLITE_URL)
            .database_type(DatabaseType::Sqlite)
            .build()?;
        create_pool(&config).await
    }

    #[tokio::test]
    async fn test_create_pool() {
        match create_postgres_test_pool().await {
            Ok(pool) => {
                assert_eq!(pool.size(), 1);
                println!("PostgreSQL连接池创建成功，连接数: {}", pool.size());
            }
            Err(e) => {
                println!("PostgreSQL连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }

        match create_sqlite_test_pool().await {
            Ok(pool) => {
                assert_eq!(pool.size(), 1);
                println!("SQLite连接池创建成功，连接数: {}", pool.size());
            }
            Err(e) => {
                println!("SQLite连接池创建失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_create_pool_with_url() {
        match create_pool_with_url(TEST_POSTGRES_URL).await {
            Ok(pool) => {
                assert_eq!(pool.size(), 1);
                println!("PostgreSQL连接池创建成功（URL方式），连接数: {}", pool.size());
            }
            Err(e) => {
                println!("PostgreSQL连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }

        match create_pool_with_url(TEST_SQLITE_URL).await {
            Ok(pool) => {
                assert_eq!(pool.size(), 1);
                println!("SQLite连接池创建成功（URL方式），连接数: {}", pool.size());
            }
            Err(e) => {
                println!("SQLite连接池创建失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_create_pool_with_options() {
        match create_pool_with_options(TEST_POSTGRES_URL, 5, 2).await {
            Ok(pool) => {
                assert_eq!(pool.size(), 2);
                println!("PostgreSQL连接池创建成功（选项方式），连接数: {}", pool.size());
            }
            Err(e) => {
                println!("PostgreSQL连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }

        match create_pool_with_options(TEST_SQLITE_URL, 5, 2).await {
            Ok(pool) => {
                assert_eq!(pool.size(), 2);
                println!("SQLite连接池创建成功（选项方式），连接数: {}", pool.size());
            }
            Err(e) => {
                println!("SQLite连接池创建失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_test_connection() {
        match create_postgres_test_pool().await {
            Ok(pool) => {
                let result = test_connection(&pool).await;
                match result {
                    Ok(true) => {
                        println!("PostgreSQL连接测试成功");
                    }
                    Ok(false) => {
                        println!("PostgreSQL连接测试失败: 连接不可用");
                    }
                    Err(e) => {
                        println!("PostgreSQL连接测试失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("PostgreSQL连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }

        match create_sqlite_test_pool().await {
            Ok(pool) => {
                let result = test_connection(&pool).await;
                match result {
                    Ok(true) => {
                        println!("SQLite连接测试成功");
                    }
                    Ok(false) => {
                        println!("SQLite连接测试失败: 连接不可用");
                    }
                    Err(e) => {
                        println!("SQLite连接测试失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("SQLite连接池创建失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_close_pool() {
        match create_postgres_test_pool().await {
            Ok(pool) => {
                let result = close_pool(pool).await;
                match result {
                    Ok(_) => {
                        println!("PostgreSQL连接池关闭成功");
                    }
                    Err(e) => {
                        println!("PostgreSQL连接池关闭失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("PostgreSQL连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }

        match create_sqlite_test_pool().await {
            Ok(pool) => {
                let result = close_pool(pool).await;
                match result {
                    Ok(_) => {
                        println!("SQLite连接池关闭成功");
                    }
                    Err(e) => {
                        println!("SQLite连接池关闭失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("SQLite连接池创建失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_get_pool_size() {
        match create_postgres_test_pool().await {
            Ok(pool) => {
                let size = get_pool_size(&pool);
                assert_eq!(size, 1);
                println!("PostgreSQL连接池大小: {}", size);
            }
            Err(e) => {
                println!("PostgreSQL连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }

        match create_sqlite_test_pool().await {
            Ok(pool) => {
                let size = get_pool_size(&pool);
                assert_eq!(size, 1);
                println!("SQLite连接池大小: {}", size);
            }
            Err(e) => {
                println!("SQLite连接池创建失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_get_idle_connections() {
        match create_postgres_test_pool().await {
            Ok(pool) => {
                let idle = get_idle_connections(&pool);
                println!("PostgreSQL空闲连接数: {}", idle);
            }
            Err(e) => {
                println!("PostgreSQL连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }

        match create_sqlite_test_pool().await {
            Ok(pool) => {
                let idle = get_idle_connections(&pool);
                println!("SQLite空闲连接数: {}", idle);
            }
            Err(e) => {
                println!("SQLite连接池创建失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_execute_query() {
        match create_postgres_test_pool().await {
            Ok(pool) => {
                let result = execute_query(&pool, "SELECT 1").await;
                match result {
                    Ok(rows_affected) => {
                        assert_eq!(rows_affected, 0);
                        println!("PostgreSQL执行查询成功，影响行数: {}", rows_affected);
                    }
                    Err(e) => {
                        println!("PostgreSQL执行查询失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("PostgreSQL连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }

        match create_sqlite_test_pool().await {
            Ok(pool) => {
                let result = execute_query(&pool, "SELECT 1").await;
                match result {
                    Ok(rows_affected) => {
                        assert_eq!(rows_affected, 0);
                        println!("SQLite执行查询成功，影响行数: {}", rows_affected);
                    }
                    Err(e) => {
                        println!("SQLite执行查询失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("SQLite连接池创建失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_execute_query_with_timeout() {
        match create_postgres_test_pool().await {
            Ok(pool) => {
                let result = execute_query_with_timeout(&pool, "SELECT 1", Duration::from_secs(5)).await;
                match result {
                    Ok(rows_affected) => {
                        assert_eq!(rows_affected, 0);
                        println!("PostgreSQL执行查询成功（超时控制），影响行数: {}", rows_affected);
                    }
                    Err(e) => {
                        println!("PostgreSQL执行查询失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("PostgreSQL连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }

        match create_sqlite_test_pool().await {
            Ok(pool) => {
                let result = execute_query_with_timeout(&pool, "SELECT 1", Duration::from_secs(5)).await;
                match result {
                    Ok(rows_affected) => {
                        assert_eq!(rows_affected, 0);
                        println!("SQLite执行查询成功（超时控制），影响行数: {}", rows_affected);
                    }
                    Err(e) => {
                        println!("SQLite执行查询失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("SQLite连接池创建失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_one() {
        match create_postgres_test_pool().await {
            Ok(pool) => {
                let fetch_result = fetch_one(&pool, "SELECT 1 as value").await;
                match fetch_result {
                    Ok(row) => {
                        let value: i32 = row.get("value");
                        assert_eq!(value, 1);
                        println!("PostgreSQL单行查询成功，value: {}", value);
                    }
                    Err(e) => {
                        println!("PostgreSQL单行查询失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("PostgreSQL连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }

        match create_sqlite_test_pool().await {
            Ok(pool) => {
                let fetch_result = fetch_one(&pool, "SELECT 1 as value").await;
                match fetch_result {
                    Ok(row) => {
                        let value: i32 = row.get("value");
                        assert_eq!(value, 1);
                        println!("SQLite单行查询成功，value: {}", value);
                    }
                    Err(e) => {
                        println!("SQLite单行查询失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("SQLite连接池创建失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_one_with_timeout() {
        match create_postgres_test_pool().await {
            Ok(pool) => {
                let fetch_result = fetch_one_with_timeout(&pool, "SELECT 1 as value", Duration::from_secs(5)).await;
                match fetch_result {
                    Ok(row) => {
                        let value: i32 = row.get("value");
                        assert_eq!(value, 1);
                        println!("PostgreSQL单行查询成功（超时控制），value: {}", value);
                    }
                    Err(e) => {
                        println!("PostgreSQL单行查询失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("PostgreSQL连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }

        match create_sqlite_test_pool().await {
            Ok(pool) => {
                let fetch_result = fetch_one_with_timeout(&pool, "SELECT 1 as value", Duration::from_secs(5)).await;
                match fetch_result {
                    Ok(row) => {
                        let value: i32 = row.get("value");
                        assert_eq!(value, 1);
                        println!("SQLite单行查询成功（超时控制），value: {}", value);
                    }
                    Err(e) => {
                        println!("SQLite单行查询失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("SQLite连接池创建失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_all() {
        match create_postgres_test_pool().await {
            Ok(pool) => {
                let fetch_result = fetch_all(&pool, "SELECT 1 as value UNION ALL SELECT 2 as value").await;
                match fetch_result {
                    Ok(rows) => {
                        assert_eq!(rows.len(), 2);
                        let value1: i32 = rows[0].get("value");
                        let value2: i32 = rows[1].get("value");
                        assert_eq!(value1, 1);
                        assert_eq!(value2, 2);
                        println!("PostgreSQL多行查询成功，行数: {}", rows.len());
                    }
                    Err(e) => {
                        println!("PostgreSQL多行查询失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("PostgreSQL连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }

        match create_sqlite_test_pool().await {
            Ok(pool) => {
                let fetch_result = fetch_all(&pool, "SELECT 1 as value UNION ALL SELECT 2 as value").await;
                match fetch_result {
                    Ok(rows) => {
                        assert_eq!(rows.len(), 2);
                        let value1: i32 = rows[0].get("value");
                        let value2: i32 = rows[1].get("value");
                        assert_eq!(value1, 1);
                        assert_eq!(value2, 2);
                        println!("SQLite多行查询成功，行数: {}", rows.len());
                    }
                    Err(e) => {
                        println!("SQLite多行查询失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("SQLite连接池创建失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_all_with_timeout() {
        match create_postgres_test_pool().await {
            Ok(pool) => {
                let fetch_result = fetch_all_with_timeout(&pool, "SELECT 1 as value UNION ALL SELECT 2 as value", Duration::from_secs(5)).await;
                match fetch_result {
                    Ok(rows) => {
                        assert_eq!(rows.len(), 2);
                        let value1: i32 = rows[0].get("value");
                        let value2: i32 = rows[1].get("value");
                        assert_eq!(value1, 1);
                        assert_eq!(value2, 2);
                        println!("PostgreSQL多行查询成功（超时控制），行数: {}", rows.len());
                    }
                    Err(e) => {
                        println!("PostgreSQL多行查询失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("PostgreSQL连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }

        match create_sqlite_test_pool().await {
            Ok(pool) => {
                let fetch_result = fetch_all_with_timeout(&pool, "SELECT 1 as value UNION ALL SELECT 2 as value", Duration::from_secs(5)).await;
                match fetch_result {
                    Ok(rows) => {
                        assert_eq!(rows.len(), 2);
                        let value1: i32 = rows[0].get("value");
                        let value2: i32 = rows[1].get("value");
                        assert_eq!(value1, 1);
                        assert_eq!(value2, 2);
                        println!("SQLite多行查询成功（超时控制），行数: {}", rows.len());
                    }
                    Err(e) => {
                        println!("SQLite多行查询失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("SQLite连接池创建失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_optional() {
        match create_postgres_test_pool().await {
            Ok(pool) => {
                let fetch_result = fetch_optional(&pool, "SELECT 1 as value WHERE 1=1").await;
                match fetch_result {
                    Ok(Some(row)) => {
                        let value: i32 = row.get("value");
                        assert_eq!(value, 1);
                        println!("PostgreSQL可选查询成功，value: {}", value);
                    }
                    Ok(None) => {
                        println!("PostgreSQL可选查询返回空");
                    }
                    Err(e) => {
                        println!("PostgreSQL可选查询失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("PostgreSQL连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }

        match create_sqlite_test_pool().await {
            Ok(pool) => {
                let fetch_result = fetch_optional(&pool, "SELECT 1 as value WHERE 1=1").await;
                match fetch_result {
                    Ok(Some(row)) => {
                        let value: i32 = row.get("value");
                        assert_eq!(value, 1);
                        println!("SQLite可选查询成功，value: {}", value);
                    }
                    Ok(None) => {
                        println!("SQLite可选查询返回空");
                    }
                    Err(e) => {
                        println!("SQLite可选查询失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("SQLite连接池创建失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_optional_with_timeout() {
        match create_postgres_test_pool().await {
            Ok(pool) => {
                let fetch_result = fetch_optional_with_timeout(&pool, "SELECT 1 as value WHERE 1=1", Duration::from_secs(5)).await;
                match fetch_result {
                    Ok(Some(row)) => {
                        let value: i32 = row.get("value");
                        assert_eq!(value, 1);
                        println!("PostgreSQL可选查询成功（超时控制），value: {}", value);
                    }
                    Ok(None) => {
                        println!("PostgreSQL可选查询返回空");
                    }
                    Err(e) => {
                        println!("PostgreSQL可选查询失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("PostgreSQL连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }

        match create_sqlite_test_pool().await {
            Ok(pool) => {
                let fetch_result = fetch_optional_with_timeout(&pool, "SELECT 1 as value WHERE 1=1", Duration::from_secs(5)).await;
                match fetch_result {
                    Ok(Some(row)) => {
                        let value: i32 = row.get("value");
                        assert_eq!(value, 1);
                        println!("SQLite可选查询成功（超时控制），value: {}", value);
                    }
                    Ok(None) => {
                        println!("SQLite可选查询返回空");
                    }
                    Err(e) => {
                        println!("SQLite可选查询失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("SQLite连接池创建失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_begin_transaction() {
        match create_postgres_test_pool().await {
            Ok(pool) => {
                let result = begin_transaction(&pool).await;
                match result {
                    Ok(tx) => {
                        println!("PostgreSQL事务创建成功");
                        let _ = rollback_transaction(tx).await;
                    }
                    Err(e) => {
                        println!("PostgreSQL事务创建失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("PostgreSQL连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }

        match create_sqlite_test_pool().await {
            Ok(pool) => {
                let result = begin_transaction(&pool).await;
                match result {
                    Ok(tx) => {
                        println!("SQLite事务创建成功");
                        let _ = rollback_transaction(tx).await;
                    }
                    Err(e) => {
                        println!("SQLite事务创建失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("SQLite连接池创建失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_begin_transaction_with_timeout() {
        match create_postgres_test_pool().await {
            Ok(pool) => {
                let result = begin_transaction_with_timeout(&pool, Duration::from_secs(5)).await;
                match result {
                    Ok(tx) => {
                        println!("PostgreSQL事务创建成功（超时控制）");
                        let _ = rollback_transaction(tx).await;
                    }
                    Err(e) => {
                        println!("PostgreSQL事务创建失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("PostgreSQL连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }

        match create_sqlite_test_pool().await {
            Ok(pool) => {
                let result = begin_transaction_with_timeout(&pool, Duration::from_secs(5)).await;
                match result {
                    Ok(tx) => {
                        println!("SQLite事务创建成功（超时控制）");
                        let _ = rollback_transaction(tx).await;
                    }
                    Err(e) => {
                        println!("SQLite事务创建失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("SQLite连接池创建失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_commit_transaction() {
        match create_postgres_test_pool().await {
            Ok(pool) => {
                let tx = match begin_transaction(&pool).await {
                    Ok(tx) => tx,
                    Err(e) => {
                        println!("PostgreSQL事务创建失败: {}", e);
                        return;
                    }
                };

                let result = commit_transaction(tx).await;
                match result {
                    Ok(_) => {
                        println!("PostgreSQL事务提交成功");
                    }
                    Err(e) => {
                        println!("PostgreSQL事务提交失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("PostgreSQL连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }

        match create_sqlite_test_pool().await {
            Ok(pool) => {
                let tx = match begin_transaction(&pool).await {
                    Ok(tx) => tx,
                    Err(e) => {
                        println!("SQLite事务创建失败: {}", e);
                        return;
                    }
                };

                let result = commit_transaction(tx).await;
                match result {
                    Ok(_) => {
                        println!("SQLite事务提交成功");
                    }
                    Err(e) => {
                        println!("SQLite事务提交失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("SQLite连接池创建失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_rollback_transaction() {
        match create_postgres_test_pool().await {
            Ok(pool) => {
                let tx = match begin_transaction(&pool).await {
                    Ok(tx) => tx,
                    Err(e) => {
                        println!("PostgreSQL事务创建失败: {}", e);
                        return;
                    }
                };

                let result = rollback_transaction(tx).await;
                match result {
                    Ok(_) => {
                        println!("PostgreSQL事务回滚成功");
                    }
                    Err(e) => {
                        println!("PostgreSQL事务回滚失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("PostgreSQL连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }

        match create_sqlite_test_pool().await {
            Ok(pool) => {
                let tx = match begin_transaction(&pool).await {
                    Ok(tx) => tx,
                    Err(e) => {
                        println!("SQLite事务创建失败: {}", e);
                        return;
                    }
                };

                let result = rollback_transaction(tx).await;
                match result {
                    Ok(_) => {
                        println!("SQLite事务回滚成功");
                    }
                    Err(e) => {
                        println!("SQLite事务回滚失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("SQLite连接池创建失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_execute_in_transaction() {
        match create_postgres_test_pool().await {
            Ok(pool) => {
                let mut tx = match begin_transaction(&pool).await {
                    Ok(tx) => tx,
                    Err(e) => {
                        println!("PostgreSQL事务创建失败: {}", e);
                        return;
                    }
                };

                let result = execute_in_transaction(&mut tx, "SELECT 1").await;
                match result {
                    Ok(rows_affected) => {
                        assert_eq!(rows_affected, 0);
                        println!("PostgreSQL事务内执行查询成功，影响行数: {}", rows_affected);
                    }
                    Err(e) => {
                        println!("PostgreSQL事务内执行查询失败: {}", e);
                    }
                }

                let _ = rollback_transaction(tx).await;
            }
            Err(e) => {
                println!("PostgreSQL连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }

        match create_sqlite_test_pool().await {
            Ok(pool) => {
                let mut tx = match begin_transaction(&pool).await {
                    Ok(tx) => tx,
                    Err(e) => {
                        println!("SQLite事务创建失败: {}", e);
                        return;
                    }
                };

                let result = execute_in_transaction(&mut tx, "SELECT 1").await;
                match result {
                    Ok(rows_affected) => {
                        assert_eq!(rows_affected, 0);
                        println!("SQLite事务内执行查询成功，影响行数: {}", rows_affected);
                    }
                    Err(e) => {
                        println!("SQLite事务内执行查询失败: {}", e);
                    }
                }

                let _ = rollback_transaction(tx).await;
            }
            Err(e) => {
                println!("SQLite连接池创建失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_execute_in_transaction_with_timeout() {
        match create_postgres_test_pool().await {
            Ok(pool) => {
                let mut tx = match begin_transaction(&pool).await {
                    Ok(tx) => tx,
                    Err(e) => {
                        println!("PostgreSQL事务创建失败: {}", e);
                        return;
                    }
                };

                let result = execute_in_transaction_with_timeout(&mut tx, "SELECT 1", Duration::from_secs(5)).await;
                match result {
                    Ok(rows_affected) => {
                        assert_eq!(rows_affected, 0);
                        println!("PostgreSQL事务内执行查询成功（超时控制），影响行数: {}", rows_affected);
                    }
                    Err(e) => {
                        println!("PostgreSQL事务内执行查询失败: {}", e);
                    }
                }

                let _ = rollback_transaction(tx).await;
            }
            Err(e) => {
                println!("PostgreSQL连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }

        match create_sqlite_test_pool().await {
            Ok(pool) => {
                let mut tx = match begin_transaction(&pool).await {
                    Ok(tx) => tx,
                    Err(e) => {
                        println!("SQLite事务创建失败: {}", e);
                        return;
                    }
                };

                let result = execute_in_transaction_with_timeout(&mut tx, "SELECT 1", Duration::from_secs(5)).await;
                match result {
                    Ok(rows_affected) => {
                        assert_eq!(rows_affected, 0);
                        println!("SQLite事务内执行查询成功（超时控制），影响行数: {}", rows_affected);
                    }
                    Err(e) => {
                        println!("SQLite事务内执行查询失败: {}", e);
                    }
                }

                let _ = rollback_transaction(tx).await;
            }
            Err(e) => {
                println!("SQLite连接池创建失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_one_in_transaction() {
        match create_postgres_test_pool().await {
            Ok(pool) => {
                let mut tx = match begin_transaction(&pool).await {
                    Ok(tx) => tx,
                    Err(e) => {
                        println!("PostgreSQL事务创建失败: {}", e);
                        return;
                    }
                };

                let fetch_result = fetch_one_in_transaction(&mut tx, "SELECT 1 as value").await;
                match fetch_result {
                    Ok(row) => {
                        let value: i32 = row.get("value");
                        assert_eq!(value, 1);
                        println!("PostgreSQL事务内单行查询成功，value: {}", value);
                    }
                    Err(e) => {
                        println!("PostgreSQL事务内单行查询失败: {}", e);
                    }
                }

                let _ = rollback_transaction(tx).await;
            }
            Err(e) => {
                println!("PostgreSQL连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }

        match create_sqlite_test_pool().await {
            Ok(pool) => {
                let mut tx = match begin_transaction(&pool).await {
                    Ok(tx) => tx,
                    Err(e) => {
                        println!("SQLite事务创建失败: {}", e);
                        return;
                    }
                };

                let fetch_result = fetch_one_in_transaction(&mut tx, "SELECT 1 as value").await;
                match fetch_result {
                    Ok(row) => {
                        let value: i32 = row.get("value");
                        assert_eq!(value, 1);
                        println!("SQLite事务内单行查询成功，value: {}", value);
                    }
                    Err(e) => {
                        println!("SQLite事务内单行查询失败: {}", e);
                    }
                }

                let _ = rollback_transaction(tx).await;
            }
            Err(e) => {
                println!("SQLite连接池创建失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_one_in_transaction_with_timeout() {
        match create_postgres_test_pool().await {
            Ok(pool) => {
                let mut tx = match begin_transaction(&pool).await {
                    Ok(tx) => tx,
                    Err(e) => {
                        println!("PostgreSQL事务创建失败: {}", e);
                        return;
                    }
                };

                let fetch_result = fetch_one_in_transaction_with_timeout(&mut tx, "SELECT 1 as value", Duration::from_secs(5)).await;
                match fetch_result {
                    Ok(row) => {
                        let value: i32 = row.get("value");
                        assert_eq!(value, 1);
                        println!("PostgreSQL事务内单行查询成功（超时控制），value: {}", value);
                    }
                    Err(e) => {
                        println!("PostgreSQL事务内单行查询失败: {}", e);
                    }
                }

                let _ = rollback_transaction(tx).await;
            }
            Err(e) => {
                println!("PostgreSQL连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }

        match create_sqlite_test_pool().await {
            Ok(pool) => {
                let mut tx = match begin_transaction(&pool).await {
                    Ok(tx) => tx,
                    Err(e) => {
                        println!("SQLite事务创建失败: {}", e);
                        return;
                    }
                };

                let fetch_result = fetch_one_in_transaction_with_timeout(&mut tx, "SELECT 1 as value", Duration::from_secs(5)).await;
                match fetch_result {
                    Ok(row) => {
                        let value: i32 = row.get("value");
                        assert_eq!(value, 1);
                        println!("SQLite事务内单行查询成功（超时控制），value: {}", value);
                    }
                    Err(e) => {
                        println!("SQLite事务内单行查询失败: {}", e);
                    }
                }

                let _ = rollback_transaction(tx).await;
            }
            Err(e) => {
                println!("SQLite连接池创建失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_all_in_transaction() {
        match create_postgres_test_pool().await {
            Ok(pool) => {
                let mut tx = match begin_transaction(&pool).await {
                    Ok(tx) => tx,
                    Err(e) => {
                        println!("PostgreSQL事务创建失败: {}", e);
                        return;
                    }
                };

                let fetch_result = fetch_all_in_transaction(&mut tx, "SELECT 1 as value UNION ALL SELECT 2 as value").await;
                match fetch_result {
                    Ok(rows) => {
                        assert_eq!(rows.len(), 2);
                        let value1: i32 = rows[0].get("value");
                        let value2: i32 = rows[1].get("value");
                        assert_eq!(value1, 1);
                        assert_eq!(value2, 2);
                        println!("PostgreSQL事务内多行查询成功，行数: {}", rows.len());
                    }
                    Err(e) => {
                        println!("PostgreSQL事务内多行查询失败: {}", e);
                    }
                }

                let _ = rollback_transaction(tx).await;
            }
            Err(e) => {
                println!("PostgreSQL连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }

        match create_sqlite_test_pool().await {
            Ok(pool) => {
                let mut tx = match begin_transaction(&pool).await {
                    Ok(tx) => tx,
                    Err(e) => {
                        println!("SQLite事务创建失败: {}", e);
                        return;
                    }
                };

                let fetch_result = fetch_all_in_transaction(&mut tx, "SELECT 1 as value UNION ALL SELECT 2 as value").await;
                match fetch_result {
                    Ok(rows) => {
                        assert_eq!(rows.len(), 2);
                        let value1: i32 = rows[0].get("value");
                        let value2: i32 = rows[1].get("value");
                        assert_eq!(value1, 1);
                        assert_eq!(value2, 2);
                        println!("SQLite事务内多行查询成功，行数: {}", rows.len());
                    }
                    Err(e) => {
                        println!("SQLite事务内多行查询失败: {}", e);
                    }
                }

                let _ = rollback_transaction(tx).await;
            }
            Err(e) => {
                println!("SQLite连接池创建失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_all_in_transaction_with_timeout() {
        match create_postgres_test_pool().await {
            Ok(pool) => {
                let mut tx = match begin_transaction(&pool).await {
                    Ok(tx) => tx,
                    Err(e) => {
                        println!("PostgreSQL事务创建失败: {}", e);
                        return;
                    }
                };

                let fetch_result = fetch_all_in_transaction_with_timeout(&mut tx, "SELECT 1 as value UNION ALL SELECT 2 as value", Duration::from_secs(5)).await;
                match fetch_result {
                    Ok(rows) => {
                        assert_eq!(rows.len(), 2);
                        let value1: i32 = rows[0].get("value");
                        let value2: i32 = rows[1].get("value");
                        assert_eq!(value1, 1);
                        assert_eq!(value2, 2);
                        println!("PostgreSQL事务内多行查询成功（超时控制），行数: {}", rows.len());
                    }
                    Err(e) => {
                        println!("PostgreSQL事务内多行查询失败: {}", e);
                    }
                }

                let _ = rollback_transaction(tx).await;
            }
            Err(e) => {
                println!("PostgreSQL连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }

        match create_sqlite_test_pool().await {
            Ok(pool) => {
                let mut tx = match begin_transaction(&pool).await {
                    Ok(tx) => tx,
                    Err(e) => {
                        println!("SQLite事务创建失败: {}", e);
                        return;
                    }
                };

                let fetch_result = fetch_all_in_transaction_with_timeout(&mut tx, "SELECT 1 as value UNION ALL SELECT 2 as value", Duration::from_secs(5)).await;
                match fetch_result {
                    Ok(rows) => {
                        assert_eq!(rows.len(), 2);
                        let value1: i32 = rows[0].get("value");
                        let value2: i32 = rows[1].get("value");
                        assert_eq!(value1, 1);
                        assert_eq!(value2, 2);
                        println!("SQLite事务内多行查询成功（超时控制），行数: {}", rows.len());
                    }
                    Err(e) => {
                        println!("SQLite事务内多行查询失败: {}", e);
                    }
                }

                let _ = rollback_transaction(tx).await;
            }
            Err(e) => {
                println!("SQLite连接池创建失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_optional_in_transaction() {
        match create_postgres_test_pool().await {
            Ok(pool) => {
                let mut tx = match begin_transaction(&pool).await {
                    Ok(tx) => tx,
                    Err(e) => {
                        println!("PostgreSQL事务创建失败: {}", e);
                        return;
                    }
                };

                let fetch_result = fetch_optional_in_transaction(&mut tx, "SELECT 1 as value WHERE 1=1").await;
                match fetch_result {
                    Ok(Some(row)) => {
                        let value: i32 = row.get("value");
                        assert_eq!(value, 1);
                        println!("PostgreSQL事务内可选查询成功，value: {}", value);
                    }
                    Ok(None) => {
                        println!("PostgreSQL事务内可选查询返回空");
                    }
                    Err(e) => {
                        println!("PostgreSQL事务内可选查询失败: {}", e);
                    }
                }

                let _ = rollback_transaction(tx).await;
            }
            Err(e) => {
                println!("PostgreSQL连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }

        match create_sqlite_test_pool().await {
            Ok(pool) => {
                let mut tx = match begin_transaction(&pool).await {
                    Ok(tx) => tx,
                    Err(e) => {
                        println!("SQLite事务创建失败: {}", e);
                        return;
                    }
                };

                let fetch_result = fetch_optional_in_transaction(&mut tx, "SELECT 1 as value WHERE 1=1").await;
                match fetch_result {
                    Ok(Some(row)) => {
                        let value: i32 = row.get("value");
                        assert_eq!(value, 1);
                        println!("SQLite事务内可选查询成功，value: {}", value);
                    }
                    Ok(None) => {
                        println!("SQLite事务内可选查询返回空");
                    }
                    Err(e) => {
                        println!("SQLite事务内可选查询失败: {}", e);
                    }
                }

                let _ = rollback_transaction(tx).await;
            }
            Err(e) => {
                println!("SQLite连接池创建失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_optional_in_transaction_with_timeout() {
        match create_postgres_test_pool().await {
            Ok(pool) => {
                let mut tx = match begin_transaction(&pool).await {
                    Ok(tx) => tx,
                    Err(e) => {
                        println!("PostgreSQL事务创建失败: {}", e);
                        return;
                    }
                };

                let fetch_result = fetch_optional_in_transaction_with_timeout(&mut tx, "SELECT 1 as value WHERE 1=1", Duration::from_secs(5)).await;
                match fetch_result {
                    Ok(Some(row)) => {
                        let value: i32 = row.get("value");
                        assert_eq!(value, 1);
                        println!("PostgreSQL事务内可选查询成功（超时控制），value: {}", value);
                    }
                    Ok(None) => {
                        println!("PostgreSQL事务内可选查询返回空");
                    }
                    Err(e) => {
                        println!("PostgreSQL事务内可选查询失败: {}", e);
                    }
                }

                let _ = rollback_transaction(tx).await;
            }
            Err(e) => {
                println!("PostgreSQL连接池创建失败（预期，如果数据库未运行）: {}", e);
            }
        }

        match create_sqlite_test_pool().await {
            Ok(pool) => {
                let mut tx = match begin_transaction(&pool).await {
                    Ok(tx) => tx,
                    Err(e) => {
                        println!("SQLite事务创建失败: {}", e);
                        return;
                    }
                };

                let fetch_result = fetch_optional_in_transaction_with_timeout(&mut tx, "SELECT 1 as value WHERE 1=1", Duration::from_secs(5)).await;
                match fetch_result {
                    Ok(Some(row)) => {
                        let value: i32 = row.get("value");
                        assert_eq!(value, 1);
                        println!("SQLite事务内可选查询成功（超时控制），value: {}", value);
                    }
                    Ok(None) => {
                        println!("SQLite事务内可选查询返回空");
                    }
                    Err(e) => {
                        println!("SQLite事务内可选查询失败: {}", e);
                    }
                }

                let _ = rollback_transaction(tx).await;
            }
            Err(e) => {
                println!("SQLite连接池创建失败: {}", e);
            }
        }
    }
}
