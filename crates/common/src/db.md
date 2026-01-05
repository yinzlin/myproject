# db.rs 数据库模块文档

## 模块概述

db.rs 模块提供了 PostgreSQL 数据库连接池管理、事务处理和查询执行的完整功能。该模块基于 sqlx 库实现，提供了类型安全的数据库访问接口，支持异步操作、超时控制和连接池管理。

## 核心组件

### 1. 错误类型 (DbError)

```rust
pub enum DbError {
    ConnectionError(#[from] sqlx::Error),
    ConfigError(String),
    QueryTimeout,
    TransactionError(String),
}
```

**说明**：
- `ConnectionError`: 数据库连接错误，包装 sqlx::Error
- `ConfigError`: 配置错误，包含错误描述字符串
- `QueryTimeout`: 查询超时错误
- `TransactionError`: 事务错误，包含错误描述字符串

**最佳实践**：
- 使用 `?` 操作符自动转换 sqlx::Error 为 DbError::ConnectionError
- 配置错误应在构建时验证，避免运行时错误
- 超时错误应记录日志并考虑重试策略

### 2. 数据库配置 (DbConfig)

```rust
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
```

**字段说明**：
- `database_url`: PostgreSQL 连接字符串
- `max_connections`: 连接池最大连接数
- `min_connections`: 连接池最小连接数
- `acquire_timeout`: 获取连接超时时间
- `idle_timeout`: 空闲连接超时时间
- `max_lifetime`: 连接最大生命周期
- `test_before_acquire`: 获取连接前是否测试连接
- `query_timeout`: 查询默认超时时间

**最佳实践**：
- 使用 Builder 模式创建配置，确保必填字段
- 根据应用负载调整连接池大小
- 生产环境建议启用 `test_before_acquire`
- 根据查询复杂度设置合理的超时时间

### 3. 配置构建器 (DbConfigBuilder)

```rust
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
```

**方法列表**：
- `database_url(url)`: 设置数据库连接 URL（必填）
- `max_connections(max)`: 设置最大连接数
- `min_connections(min)`: 设置最小连接数
- `acquire_timeout(timeout)`: 设置获取连接超时
- `idle_timeout(timeout)`: 设置空闲连接超时
- `max_lifetime(lifetime)`: 设置连接最大生命周期
- `test_before_acquire(test)`: 设置是否测试连接
- `query_timeout(timeout)`: 设置查询超时
- `build()`: 构建配置对象

**使用示例**：
```rust
let config = DbConfig::builder()
    .database_url("postgresql://user:pass@localhost:5432/db")
    .max_connections(20)
    .min_connections(5)
    .test_before_acquire(true)
    .build()?;
```

## 函数文档

### 连接池管理

#### create_pool

```rust
pub async fn create_pool(config: &DbConfig) -> DbResult<DbPool>
```

**功能**：根据配置创建数据库连接池

**参数**：
- `config`: 数据库配置对象

**返回**：
- `DbResult<DbPool>`: 连接池对象或错误

**最佳实践**：
- 应用启动时创建连接池，全局共享
- 根据应用负载调整连接池大小
- 使用 Builder 模式创建配置

#### create_pool_with_url

```rust
pub async fn create_pool_with_url(database_url: &str) -> DbResult<DbPool>
```

**功能**：使用默认配置创建连接池

**参数**：
- `database_url`: 数据库连接 URL

**返回**：
- `DbResult<DbPool>`: 连接池对象或错误

**适用场景**：
- 快速原型开发
- 测试环境
- 不需要自定义配置的场景

#### create_pool_with_options

```rust
pub async fn create_pool_with_options(
    database_url: &str,
    max_connections: u32,
    min_connections: u32,
) -> DbResult<DbPool>
```

**功能**：使用自定义连接数创建连接池

**参数**：
- `database_url`: 数据库连接 URL
- `max_connections`: 最大连接数
- `min_connections`: 最小连接数

**返回**：
- `DbResult<DbPool>`: 连接池对象或错误

**适用场景**：
- 需要控制连接池大小
- 其他参数使用默认值的场景

#### test_connection

```rust
pub async fn test_connection(pool: &DbPool) -> DbResult<bool>
```

**功能**：测试数据库连接是否正常

**参数**：
- `pool`: 数据库连接池

**返回**：
- `DbResult<bool>`: 连接状态或错误

**最佳实践**：
- 应用启动时测试连接
- 健康检查接口使用
- 连接失败时记录详细日志

#### close_pool

```rust
pub async fn close_pool(pool: DbPool) -> DbResult<()>
```

**功能**：关闭数据库连接池

**参数**：
- `pool`: 数据库连接池

**返回**：
- `DbResult<()>`: 成功或错误

**最佳实践**：
- 应用关闭时调用
- 确保所有查询完成后再关闭
- 使用 tokio::spawn 异步关闭避免阻塞

#### get_pool_size

```rust
pub fn get_pool_size(pool: &DbPool) -> u32
```

**功能**：获取连接池当前连接数

**参数**：
- `pool`: 数据库连接池

**返回**：
- `u32`: 当前连接数

**适用场景**：
- 监控连接池状态
- 性能分析
- 调试连接泄漏

#### get_idle_connections

```rust
pub fn get_idle_connections(pool: &DbPool) -> u32
```

**功能**：获取连接池空闲连接数

**参数**：
- `pool`: 数据库连接池

**返回**：
- `u32`: 空闲连接数

**适用场景**：
- 监控连接池利用率
- 调整连接池大小
- 性能优化

### 查询执行

#### execute_query

```rust
pub async fn execute_query(pool: &DbPool, query: &str) -> DbResult<u64>
```

**功能**：执行 SQL 查询并返回影响的行数

**参数**：
- `pool`: 数据库连接池
- `query`: SQL 查询语句

**返回**：
- `DbResult<u64>`: 影响的行数或错误

**适用场景**：
- INSERT、UPDATE、DELETE 操作
- DDL 语句执行

**最佳实践**：
- 使用参数化查询防止 SQL 注入
- 批量操作考虑使用事务
- 捕获并记录错误

#### execute_query_with_timeout

```rust
pub async fn execute_query_with_timeout(
    pool: &DbPool,
    query: &str,
    timeout: Duration,
) -> DbResult<u64>
```

**功能**：执行带超时的 SQL 查询

**参数**：
- `pool`: 数据库连接池
- `query`: SQL 查询语句
- `timeout`: 超时时间

**返回**：
- `DbResult<u64>`: 影响的行数或错误

**适用场景**：
- 需要控制查询执行时间
- 防止慢查询阻塞
- 外部 API 调用

#### fetch_one

```rust
pub async fn fetch_one(pool: &DbPool, query: &str) -> DbResult<sqlx::postgres::PgRow>
```

**功能**：获取单行查询结果

**参数**：
- `pool`: 数据库连接池
- `query`: SQL 查询语句

**返回**：
- `DbResult<sqlx::postgres::PgRow>`: 单行结果或错误

**适用场景**：
- 查询单条记录
- 主键查询
- 聚合查询

**最佳实践**：
- 确保查询只返回一行
- 使用 LIMIT 1 限制结果
- 处理 RowNotFound 错误

#### fetch_one_with_timeout

```rust
pub async fn fetch_one_with_timeout(
    pool: &DbPool,
    query: &str,
    timeout: Duration,
) -> DbResult<sqlx::postgres::PgRow>
```

**功能**：获取带超时的单行查询结果

**参数**：
- `pool`: 数据库连接池
- `query`: SQL 查询语句
- `timeout`: 超时时间

**返回**：
- `DbResult<sqlx::postgres::PgRow>`: 单行结果或错误

#### fetch_all

```rust
pub async fn fetch_all(pool: &DbPool, query: &str) -> DbResult<Vec<sqlx::postgres::PgRow>>
```

**功能**：获取多行查询结果

**参数**：
- `pool`: 数据库连接池
- `query`: SQL 查询语句

**返回**：
- `DbResult<Vec<sqlx::postgres::PgRow>>`: 多行结果或错误

**适用场景**：
- 查询多条记录
- 列表查询
- 报表查询

**最佳实践**：
- 使用 LIMIT 限制结果数量
- 考虑分页查询
- 大数据集使用流式查询

#### fetch_all_with_timeout

```rust
pub async fn fetch_all_with_timeout(
    pool: &DbPool,
    query: &str,
    timeout: Duration,
) -> DbResult<Vec<sqlx::postgres::PgRow>>
```

**功能**：获取带超时的多行查询结果

**参数**：
- `pool`: 数据库连接池
- `query`: SQL 查询语句
- `timeout`: 超时时间

**返回**：
- `DbResult<Vec<sqlx::postgres::PgRow>>`: 多行结果或错误

#### fetch_optional

```rust
pub async fn fetch_optional(pool: &DbPool, query: &str) -> DbResult<Option<sqlx::postgres::PgRow>>
```

**功能**：获取可选的单行查询结果

**参数**：
- `pool`: 数据库连接池
- `query`: SQL 查询语句

**返回**：
- `DbResult<Option<sqlx::postgres::PgRow>>`: 可选单行结果或错误

**适用场景**：
- 查询可能不存在的记录
- 可选配置查询
- 条件查询

**最佳实践**：
- 使用 LIMIT 1 限制结果
- 处理 None 情况
- 避免使用 SELECT *

#### fetch_optional_with_timeout

```rust
pub async fn fetch_optional_with_timeout(
    pool: &DbPool,
    query: &str,
    timeout: Duration,
) -> DbResult<Option<sqlx::postgres::PgRow>>
```

**功能**：获取带超时的可选单行查询结果

**参数**：
- `pool`: 数据库连接池
- `query`: SQL 查询语句
- `timeout`: 超时时间

**返回**：
- `DbResult<Option<sqlx::postgres::PgRow>>`: 可选单行结果或错误

### 事务管理

#### begin_transaction

```rust
pub async fn begin_transaction(pool: &DbPool) -> DbResult<sqlx::Transaction<'_, Postgres>>
```

**功能**：开始一个数据库事务

**参数**：
- `pool`: 数据库连接池

**返回**：
- `DbResult<sqlx::Transaction<'_, Postgres>>`: 事务对象或错误

**最佳实践**：
- 使用 `?` 操作符处理错误
- 确保事务及时提交或回滚
- 避免长时间持有事务

#### begin_transaction_with_timeout

```rust
pub async fn begin_transaction_with_timeout(
    pool: &DbPool,
    timeout: Duration,
) -> DbResult<sqlx::Transaction<'_, Postgres>>
```

**功能**：开始带超时的数据库事务

**参数**：
- `pool`: 数据库连接池
- `timeout`: 超时时间

**返回**：
- `DbResult<sqlx::Transaction<'_, Postgres>>`: 事务对象或错误

#### commit_transaction

```rust
pub async fn commit_transaction(tx: sqlx::Transaction<'_, Postgres>) -> DbResult<()>
```

**功能**：提交事务

**参数**：
- `tx`: 事务对象

**返回**：
- `DbResult<()>`: 成功或错误

**最佳实践**：
- 所有操作成功后提交
- 捕获提交错误
- 使用 `?` 操作符传播错误

#### rollback_transaction

```rust
pub async fn rollback_transaction(tx: sqlx::Transaction<'_, Postgres>) -> DbResult<()>
```

**功能**：回滚事务

**参数**：
- `tx`: 事务对象

**返回**：
- `DbResult<()>`: 成功或错误

**最佳实践**：
- 发生错误时回滚
- 使用 `?` 操作符传播错误
- 记录回滚原因

### 事务内查询

#### execute_in_transaction

```rust
pub async fn execute_in_transaction(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    query: &str,
) -> DbResult<u64>
```

**功能**：在事务内执行 SQL 查询

**参数**：
- `tx`: 事务对象的可变引用
- `query`: SQL 查询语句

**返回**：
- `DbResult<u64>`: 影响的行数或错误

**最佳实践**：
- 使用参数化查询
- 批量操作使用事务
- 错误时回滚事务

#### execute_in_transaction_with_timeout

```rust
pub async fn execute_in_transaction_with_timeout(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    query: &str,
    timeout: Duration,
) -> DbResult<u64>
```

**功能**：在事务内执行带超时的 SQL 查询

**参数**：
- `tx`: 事务对象的可变引用
- `query`: SQL 查询语句
- `timeout`: 超时时间

**返回**：
- `DbResult<u64>`: 影响的行数或错误

#### fetch_one_in_transaction

```rust
pub async fn fetch_one_in_transaction(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    query: &str,
) -> DbResult<sqlx::postgres::PgRow>
```

**功能**：在事务内获取单行查询结果

**参数**：
- `tx`: 事务对象的可变引用
- `query`: SQL 查询语句

**返回**：
- `DbResult<sqlx::postgres::PgRow>`: 单行结果或错误

#### fetch_one_in_transaction_with_timeout

```rust
pub async fn fetch_one_in_transaction_with_timeout(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    query: &str,
    timeout: Duration,
) -> DbResult<sqlx::postgres::PgRow>
```

**功能**：在事务内获取带超时的单行查询结果

**参数**：
- `tx`: 事务对象的可变引用
- `query`: SQL 查询语句
- `timeout`: 超时时间

**返回**：
- `DbResult<sqlx::postgres::PgRow>`: 单行结果或错误

#### fetch_all_in_transaction

```rust
pub async fn fetch_all_in_transaction(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    query: &str,
) -> DbResult<Vec<sqlx::postgres::PgRow>>
```

**功能**：在事务内获取多行查询结果

**参数**：
- `tx`: 事务对象的可变引用
- `query`: SQL 查询语句

**返回**：
- `DbResult<Vec<sqlx::postgres::PgRow>>`: 多行结果或错误

#### fetch_all_in_transaction_with_timeout

```rust
pub async fn fetch_all_in_transaction_with_timeout(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    query: &str,
    timeout: Duration,
) -> DbResult<Vec<sqlx::postgres::PgRow>>
```

**功能**：在事务内获取带超时的多行查询结果

**参数**：
- `tx`: 事务对象的可变引用
- `query`: SQL 查询语句
- `timeout`: 超时时间

**返回**：
- `DbResult<Vec<sqlx::postgres::PgRow>>`: 多行结果或错误

#### fetch_optional_in_transaction

```rust
pub async fn fetch_optional_in_transaction(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    query: &str,
) -> DbResult<Option<sqlx::postgres::PgRow>>
```

**功能**：在事务内获取可选的单行查询结果

**参数**：
- `tx`: 事务对象的可变引用
- `query`: SQL 查询语句

**返回**：
- `DbResult<Option<sqlx::postgres::PgRow>>`: 可选单行结果或错误

#### fetch_optional_in_transaction_with_timeout

```rust
pub async fn fetch_optional_in_transaction_with_timeout(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    query: &str,
    timeout: Duration,
) -> DbResult<Option<sqlx::postgres::PgRow>>
```

**功能**：在事务内获取带超时的可选单行查询结果

**参数**：
- `tx`: 事务对象的可变引用
- `query`: SQL 查询语句
- `timeout`: 超时时间

**返回**：
- `DbResult<Option<sqlx::postgres::PgRow>>`: 可选单行结果或错误

## 使用示例

### 基本使用

```rust
use common::db::{create_pool_with_url, execute_query, fetch_one};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_pool_with_url("postgresql://user:pass@localhost:5432/db").await?;
    
    let rows_affected = execute_query(&pool, "INSERT INTO users (name) VALUES ('Alice')").await?;
    println!("插入 {} 行", rows_affected);
    
    let row = fetch_one(&pool, "SELECT * FROM users WHERE name = 'Alice'").await?;
    let name: String = row.get("name");
    println!("用户名: {}", name);
    
    Ok(())
}
```

### 使用 Builder 模式

```rust
use common::db::{DbConfig, create_pool};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = DbConfig::builder()
        .database_url("postgresql://user:pass@localhost:5432/db")
        .max_connections(20)
        .min_connections(5)
        .test_before_acquire(true)
        .query_timeout(Duration::from_secs(30))
        .build()?;
    
    let pool = create_pool(&config).await?;
    Ok(())
}
```

### 事务使用

```rust
use common::db::{create_pool_with_url, begin_transaction, execute_in_transaction, commit_transaction};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_pool_with_url("postgresql://user:pass@localhost:5432/db").await?;
    
    let mut tx = begin_transaction(&pool).await?;
    execute_in_transaction(&mut tx, "INSERT INTO users (name) VALUES ('Alice')").await?;
    execute_in_transaction(&mut tx, "INSERT INTO users (name) VALUES ('Bob')").await?;
    commit_transaction(tx).await?;
    
    Ok(())
}
```

### 带超时的查询

```rust
use common::db::{create_pool_with_url, fetch_one_with_timeout};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_pool_with_url("postgresql://user:pass@localhost:5432/db").await?;
    
    let row = fetch_one_with_timeout(&pool, "SELECT * FROM users WHERE id = 1", Duration::from_secs(5)).await?;
    let name: String = row.get("name");
    println!("用户名: {}", name);
    
    Ok(())
}
```

## 最佳实践

### 1. 连接池配置

- 根据应用负载调整连接池大小
- 生产环境启用 `test_before_acquire`
- 设置合理的超时时间
- 监控连接池状态

### 2. 查询优化

- 使用参数化查询防止 SQL 注入
- 避免 SELECT *，只查询需要的字段
- 使用 LIMIT 限制结果数量
- 大数据集使用分页查询

### 3. 事务管理

- 保持事务简短
- 及时提交或回滚
- 避免在事务中执行耗时操作
- 使用适当的隔离级别

### 4. 错误处理

- 捕获并记录所有错误
- 使用 `?` 操作符传播错误
- 区分可恢复和不可恢复错误
- 实现重试机制

### 5. 性能优化

- 使用连接池减少连接开销
- 批量操作使用事务
- 使用索引优化查询
- 考虑使用缓存

### 6. 安全性

- 使用参数化查询
- 限制数据库用户权限
- 加密敏感数据
- 定期更新依赖

## 性能考虑

### 连接池大小

- CPU 密集型应用：连接数 = CPU 核心数
- IO 密集型应用：连接数 = CPU 核心数 * 2
- 根据实际负载调整

### 超时设置

- 获取连接超时：5-30 秒
- 查询超时：根据查询复杂度设置
- 事务超时：根据业务逻辑设置

### 查询优化

- 使用索引加速查询
- 避免 N+1 查询
- 使用批量操作
- 考虑使用视图或物化视图

## 测试

模块包含 39 个单元测试，覆盖所有主要功能：

- 配置构建和验证
- 连接池创建和管理
- 查询执行（普通和带超时）
- 事务管理
- 事务内查询

运行测试：
```bash
cargo test --package common
```

## 依赖

- sqlx: PostgreSQL 数据库驱动
- tokio: 异步运行时
- thiserror: 错误处理

## 版本历史

### v1.0.0 (当前版本)
- 添加 Builder 模式支持
- 添加查询超时功能
- 添加连接池健康检查
- 优化错误处理
- 完善测试覆盖
