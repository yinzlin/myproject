use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use tracing::Level;

/// 初始化日志系统
/// 
/// 该函数会配置tracing日志系统，支持从环境变量RUST_LOG读取日志级别
/// 默认日志级别为info
pub fn init_logger() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .try_init()
        .ok();
}

/// 使用指定日志级别初始化日志系统
/// 
/// # 参数
/// - `level`: 日志级别，支持 trace, debug, info, warn, error
/// 
/// # 示例
/// ```no_run
/// use common::log::init_logger_with_level;
/// use tracing::Level;
/// 
/// init_logger_with_level(Level::DEBUG);
/// ```
pub fn init_logger_with_level(level: Level) {
    let level_str = match level {
        Level::TRACE => "trace",
        Level::DEBUG => "debug",
        Level::INFO => "info",
        Level::WARN => "warn",
        Level::ERROR => "error",
    };

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level_str));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .try_init()
        .ok();
}

/// 使用自定义过滤器字符串初始化日志系统
/// 
/// # 参数
/// - `filter_str`: 过滤器字符串，支持针对不同模块设置不同日志级别
/// 
/// # 示例
/// ```no_run
/// use common::log::init_logger_with_filter;
/// 
/// // 设置全局为info级别，特定模块为debug级别
/// init_logger_with_filter("info,my_module=debug");
/// 
/// // 设置多个模块的不同级别
/// init_logger_with_filter("warn,module_a=debug,module_b=trace");
/// ```
pub fn init_logger_with_filter(filter_str: &str) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(filter_str));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .try_init()
        .ok();
}

/// 使用JSON格式初始化日志系统
/// 
/// 该函数会配置tracing日志系统，输出JSON格式的日志，便于日志收集和分析
/// 支持从环境变量RUST_LOG读取日志级别，默认日志级别为info
/// 
/// # 示例
/// ```no_run
/// use common::log::init_logger_json;
/// 
/// init_logger_json();
/// tracing::info!("JSON格式日志");
/// ```
pub fn init_logger_json() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().json())
        .try_init()
        .ok();
}

/// 使用紧凑格式初始化日志系统
/// 
/// 该函数会配置tracing日志系统，使用紧凑格式输出日志，适合生产环境
/// 支持从环境变量RUST_LOG读取日志级别，默认日志级别为info
/// 
/// # 示例
/// ```no_run
/// use common::log::init_logger_compact;
/// 
/// init_logger_compact();
/// tracing::info!("紧凑格式日志");
/// ```
pub fn init_logger_compact() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().compact())
        .try_init()
        .ok();
}

/// 使用Pretty格式初始化日志系统
/// 
/// 该函数会配置tracing日志系统，使用Pretty格式输出日志，包含颜色、时间戳等信息
/// 适合开发环境使用，提供更好的可读性
/// 支持从环境变量RUST_LOG读取日志级别，默认日志级别为info
/// 
/// # 示例
/// ```no_run
/// use common::log::init_logger_pretty;
/// 
/// init_logger_pretty();
/// tracing::info!("Pretty格式日志");
/// ```
pub fn init_logger_pretty() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().pretty())
        .try_init()
        .ok();
}

/// 获取默认日志过滤器
/// 
/// 该函数返回一个基于环境变量RUST_LOG的EnvFilter，如果未设置则使用info级别
/// 
/// # 返回
/// 返回一个EnvFilter实例
/// 
/// # 示例
/// ```no_run
/// use common::log::get_default_filter;
/// 
/// let filter = get_default_filter();
/// ```
pub fn get_default_filter() -> EnvFilter {
    EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_logger() {
        init_logger();
        tracing::info!("日志初始化测试");
    }

    #[test]
    fn test_init_logger_with_level() {
        init_logger_with_level(Level::DEBUG);
        tracing::debug!("使用DEBUG级别初始化日志");
        tracing::info!("INFO级别日志");
    }

    #[test]
    fn test_level_conversion() {
        let level_str = match Level::DEBUG {
            Level::TRACE => "trace",
            Level::DEBUG => "debug",
            Level::INFO => "info",
            Level::WARN => "warn",
            Level::ERROR => "error",
        };
        assert_eq!(level_str, "debug");
    }

    #[test]
    fn test_init_logger_with_filter() {
        init_logger_with_filter("info,common=debug");
        tracing::info!("使用自定义过滤器初始化日志");
        tracing::debug!("DEBUG级别日志");
    }

    #[test]
    fn test_filter_string_parsing() {
        let filter_str = "info,common=debug,website=trace";
        let env_filter = EnvFilter::new(filter_str);
        assert!(env_filter.to_string().contains("info"));
    }

    #[test]
    fn test_init_logger_json() {
        init_logger_json();
        tracing::info!("JSON格式日志测试");
    }

    #[test]
    fn test_init_logger_compact() {
        init_logger_compact();
        tracing::info!("紧凑格式日志测试");
    }

    #[test]
    fn test_init_logger_pretty() {
        init_logger_pretty();
        tracing::info!("Pretty格式日志测试");
    }

    #[test]
    fn test_get_default_filter() {
        let filter = get_default_filter();
        assert!(filter.to_string().contains("info") || filter.to_string().contains("debug"));
    }
}
