# log.rs 模块文档

## 模块概述

`log.rs` 是common层的日志模块，基于 `tracing` 和 `tracing-subscriber` 库实现，提供了多种日志初始化方式和辅助函数。该模块支持不同格式的日志输出、灵活的日志级别控制，以及环境变量配置。

## 功能特性

- 支持多种日志格式：默认格式、JSON格式、紧凑格式、Pretty格式
- 支持环境变量 `RUST_LOG` 动态配置日志级别
- 提供不同场景下的日志初始化函数
- 避免重复初始化导致的panic
- 提供日志级别转换工具函数

## 依赖项

- `tracing`: Rust的日志框架
- `tracing-subscriber`: 日志订阅器实现，提供多种格式化选项

## 函数列表

### 1. init_logger()

初始化日志系统，使用默认格式输出。

**功能说明：**
- 配置tracing日志系统
- 支持从环境变量 `RUST_LOG` 读取日志级别
- 默认日志级别为 `info`
- 使用默认格式输出日志，包含时间戳、日志级别、模块名等信息

**使用场景：**
- 应用程序启动时的标准日志初始化
- 开发环境的日志配置

**环境变量配置示例：**
```bash
# 设置全局日志级别为debug
export RUST_LOG=debug

# 设置特定模块的日志级别
export RUST_LOG=info,common=debug,website=trace
```

**代码实现：**
```rust
pub fn init_logger() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .try_init()
        .ok();
}
```

**使用示例：**
```rust
use common::log::init_logger;

fn main() {
    init_logger();
    tracing::info!("应用程序启动");
}
```

---

### 2. init_logger_with_level(level: Level)

使用指定的日志级别初始化日志系统。

**功能说明：**
- 接受一个 `Level` 参数，支持 `trace`、`debug`、`info`、`warn`、`error` 五个级别
- 仍然支持环境变量 `RUST_LOG` 覆盖
- 使用默认格式输出日志
- 内部调用 `level_to_string()` 函数进行级别转换

**使用场景：**
- 需要固定日志级别的场景
- 根据配置文件动态设置日志级别

**代码实现：**
```rust
pub fn init_logger_with_level(level: Level) {
    let level_str = level_to_string(level);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level_str));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .try_init()
        .ok();
}
```

**使用示例：**
```rust
use common::log::init_logger_with_level;
use tracing::Level;

fn main() {
    init_logger_with_level(Level::DEBUG);
    tracing::debug!("调试信息");
    tracing::info!("普通信息");
}
```

---

### 3. init_logger_with_filter(filter_str: &str)

使用自定义过滤器字符串初始化日志系统。

**功能说明：**
- 接受一个过滤器字符串参数
- 支持针对不同模块设置不同的日志级别
- 仍然支持环境变量 `RUST_LOG` 覆盖
- 使用默认格式输出日志

**过滤器字符串格式：**
- 全局级别：`"info"`、`"debug"` 等
- 模块级别：`"module_name=level"`
- 组合配置：`"info,module_a=debug,module_b=trace"`

**使用场景：**
- 需要精细控制不同模块日志级别的场景
- 生产环境调试特定模块

**代码实现：**
```rust
pub fn init_logger_with_filter(filter_str: &str) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(filter_str));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .try_init()
        .ok();
}
```

**使用示例：**
```rust
use common::log::init_logger_with_filter;

fn main() {
    // 全局info级别，common模块debug级别
    init_logger_with_filter("info,common=debug");
    
    // 多个模块不同级别
    init_logger_with_filter("warn,website=debug,api=trace");
}
```

---

### 4. init_logger_json()

使用JSON格式初始化日志系统。

**功能说明：**
- 输出JSON格式的日志，便于日志收集和分析
- 支持从环境变量 `RUST_LOG` 读取日志级别
- 默认日志级别为 `info`
- 适合与日志收集系统（如ELK、Loki等）集成

**使用场景：**
- 生产环境日志收集
- 需要结构化日志的场景
- 与日志分析平台集成

**JSON日志格式示例：**
```json
{
  "timestamp": "2026-01-04T03:00:00.000Z",
  "level": "INFO",
  "target": "common::log::tests",
  "fields": {
    "message": "JSON格式日志测试"
  }
}
```

**代码实现：**
```rust
pub fn init_logger_json() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().json())
        .try_init()
        .ok();
}
```

**使用示例：**
```rust
use common::log::init_logger_json;

fn main() {
    init_logger_json();
    tracing::info!("JSON格式日志");
}
```

---

### 5. init_logger_compact()

使用紧凑格式初始化日志系统。

**功能说明：**
- 使用紧凑格式输出日志，减少日志输出体积
- 支持从环境变量 `RUST_LOG` 读取日志级别
- 默认日志级别为 `info`
- 适合生产环境使用，减少日志存储和传输开销

**使用场景：**
- 生产环境日志输出
- 日志量较大的场景
- 需要减少日志存储空间的场景

**紧凑格式示例：**
```
2026-01-04T03:00:00.000Z INFO common::log: 紧凑格式日志测试
```

**代码实现：**
```rust
pub fn init_logger_compact() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().compact())
        .try_init()
        .ok();
}
```

**使用示例：**
```rust
use common::log::init_logger_compact;

fn main() {
    init_logger_compact();
    tracing::info!("紧凑格式日志");
}
```

---

### 6. init_logger_pretty()

使用Pretty格式初始化日志系统。

**功能说明：**
- 使用Pretty格式输出日志，包含颜色、时间戳等信息
- 支持从环境变量 `RUST_LOG` 读取日志级别
- 默认日志级别为 `info`
- 适合开发环境使用，提供更好的可读性

**使用场景：**
- 开发环境日志输出
- 需要高可读性的场景
- 终端日志查看

**Pretty格式示例：**
```
2026-01-04T03:00:00.000Z INFO common::log: Pretty格式日志测试
```
（实际输出会包含颜色和更丰富的格式）

**代码实现：**
```rust
pub fn init_logger_pretty() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().pretty())
        .try_init()
        .ok();
}
```

**使用示例：**
```rust
use common::log::init_logger_pretty;

fn main() {
    init_logger_pretty();
    tracing::info!("Pretty格式日志");
}
```

---

### 7. level_to_string(level: Level) -> &'static str

将日志级别转换为字符串。

**功能说明：**
- 接受一个 `Level` 参数
- 返回对应的字符串表示
- 返回静态字符串，无需内存分配
- 提供统一的日志级别字符串转换方法

**使用场景：**
- 需要将日志级别转换为字符串的场景
- 配置文件输出
- 日志级别显示

**代码实现：**
```rust
pub fn level_to_string(level: Level) -> &'static str {
    match level {
        Level::TRACE => "trace",
        Level::DEBUG => "debug",
        Level::INFO => "info",
        Level::WARN => "warn",
        Level::ERROR => "error",
    }
}
```

**使用示例：**
```rust
use common::log::level_to_string;
use tracing::Level;

fn main() {
    let level_str = level_to_string(Level::DEBUG);
    println!("当前日志级别: {}", level_str);
}
```

---

### 8. get_default_filter() -> EnvFilter

获取默认日志过滤器。

**功能说明：**
- 返回一个基于环境变量 `RUST_LOG` 的 `EnvFilter` 实例
- 如果未设置环境变量，则使用 `info` 级别
- 可用于自定义日志配置场景

**使用场景：**
- 需要单独获取过滤器的场景
- 自定义日志配置
- 组合多个日志层

**代码实现：**
```rust
pub fn get_default_filter() -> EnvFilter {
    EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"))
}
```

**使用示例：**
```rust
use common::log::get_default_filter;
use tracing_subscriber::fmt;

fn main() {
    let filter = get_default_filter();
    
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer())
        .init();
}
```

---

## 测试覆盖

该模块包含完整的单元测试，覆盖所有函数：

1. `test_init_logger()` - 测试默认日志初始化
2. `test_init_logger_with_level()` - 测试指定级别初始化
3. `test_level_to_string()` - 测试级别字符串转换
4. `test_init_logger_with_filter()` - 测试自定义过滤器初始化
5. `test_init_logger_json()` - 测试JSON格式初始化
6. `test_init_logger_compact()` - 测试紧凑格式初始化
7. `test_init_logger_pretty()` - 测试Pretty格式初始化
8. `test_get_default_filter()` - 测试默认过滤器获取

**运行测试：**
```bash
cargo test --package common
```

---

## 最佳实践

### 1. 选择合适的日志格式

- **开发环境**：使用 `init_logger_pretty()`，提供更好的可读性
- **生产环境**：使用 `init_logger_compact()` 或 `init_logger_json()`
- **日志收集**：使用 `init_logger_json()`，便于结构化分析

### 2. 日志级别配置

```bash
# 开发环境
export RUST_LOG=debug

# 生产环境
export RUST_LOG=info

# 调试特定模块
export RUST_LOG=info,common=debug
```

### 3. 避免重复初始化

所有初始化函数都使用 `try_init()` 而不是 `init()`，避免重复初始化导致的panic。

### 4. 性能考虑

- 紧凑格式和JSON格式在生产环境中性能更好
- Pretty格式在开发环境中提供更好的体验
- 日志级别越高，性能影响越小

---

## 注意事项

1. **环境变量优先级**：所有初始化函数都支持环境变量 `RUST_LOG` 覆盖默认配置
2. **线程安全**：`tracing` 是线程安全的，可以在多线程环境中使用
3. **性能影响**：日志级别设置为 `trace` 或 `debug` 会显著影响性能
4. **初始化时机**：建议在应用程序启动时尽早初始化日志系统
5. **全局唯一**：日志系统只能初始化一次，多次调用不会报错但也不会生效

---

## 相关资源

- [tracing 官方文档](https://docs.rs/tracing/)
- [tracing-subscriber 官方文档](https://docs.rs/tracing-subscriber/)
- [EnvFilter 文档](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html)
