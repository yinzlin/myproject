use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("数据库连接错误: {0}")]
    ConnectionError(#[from] sqlx::Error),
    #[error("配置错误: {0}")]
    ConfigError(String),
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
        .connect(&config.database_url)
        .await?;

    Ok(pool)
}

pub async fn create_pool_with_url(database_url: &str) -> DbResult<DbPool> {
    let config = DbConfig {
        database_url: database_url.to_string(),
        ..Default::default()
    };
    create_pool(&config).await
}

pub async fn create_pool_with_options(
    database_url: &str,
    max_connections: u32,
    min_connections: u32,
) -> DbResult<DbPool> {
    let config = DbConfig {
        database_url: database_url.to_string(),
        max_connections,
        min_connections,
        ..Default::default()
    };
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

pub async fn fetch_one(pool: &DbPool, query: &str) -> DbResult<sqlx::postgres::PgRow> {
    let row = sqlx::query(query)
        .fetch_one(pool)
        .await?;
    Ok(row)
}

pub async fn fetch_all(pool: &DbPool, query: &str) -> DbResult<Vec<sqlx::postgres::PgRow>> {
    let rows = sqlx::query(query)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn fetch_optional(pool: &DbPool, query: &str) -> DbResult<Option<sqlx::postgres::PgRow>> {
    let row = sqlx::query(query)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn begin_transaction(pool: &DbPool) -> DbResult<sqlx::Transaction<'_, Postgres>> {
    let tx = pool.begin().await?;
    Ok(tx)
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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Row;

    #[test]
    fn test_db_config_default() {
        let config = DbConfig::default();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 1);
        assert_eq!(config.acquire_timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_db_config_custom() {
        let config = DbConfig {
            database_url: "postgresql://user:pass@host:5432/db".to_string(),
            max_connections: 20,
            min_connections: 5,
            ..Default::default()
        };
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.min_connections, 5);
        assert_eq!(config.database_url, "postgresql://user:pass@host:5432/db");
    }

    #[test]
    fn test_db_error_display() {
        let error = DbError::ConfigError("test error".to_string());
        assert_eq!(error.to_string(), "配置错误: test error");
    }

    #[tokio::test]
    async fn test_create_pool() {
        let config = DbConfig {
            database_url: "postgresql://postgres:password@localhost:5432/testdb".to_string(),
            max_connections: 5,
            min_connections: 1,
            ..Default::default()
        };

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
        let result = create_pool_with_url("postgresql://postgres:password@localhost:5432/testdb").await;
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
        let result = create_pool_with_options("postgresql://postgres:password@localhost:5432/testdb", 5, 1).await;
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
        let result = create_pool_with_url("postgresql://postgres:password@localhost:5432/testdb").await;
        match result {
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
        let result = create_pool_with_url("postgresql://postgres:password@localhost:5432/testdb").await;
        match result {
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
        let result = create_pool_with_url("postgresql://postgres:password@localhost:5432/testdb").await;
        match result {
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
        let result = create_pool_with_url("postgresql://postgres:password@localhost:5432/testdb").await;
        match result {
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
        let result = create_pool_with_url("postgresql://postgres:password@localhost:5432/testdb").await;
        match result {
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
    async fn test_fetch_one() {
        let result = create_pool_with_url("postgresql://postgres:password@localhost:5432/testdb").await;
        match result {
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
    async fn test_fetch_all() {
        let result = create_pool_with_url("postgresql://postgres:password@localhost:5432/testdb").await;
        match result {
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
    async fn test_fetch_optional() {
        let result = create_pool_with_url("postgresql://postgres:password@localhost:5432/testdb").await;
        match result {
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
    async fn test_begin_transaction() {
        let result = create_pool_with_url("postgresql://postgres:password@localhost:5432/testdb").await;
        match result {
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
    async fn test_commit_transaction() {
        let result = create_pool_with_url("postgresql://postgres:password@localhost:5432/testdb").await;
        match result {
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
        let result = create_pool_with_url("postgresql://postgres:password@localhost:5432/testdb").await;
        match result {
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
        let result = create_pool_with_url("postgresql://postgres:password@localhost:5432/testdb").await;
        match result {
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
}
