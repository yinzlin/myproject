use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use tracing::Level;

pub fn init_logger() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .try_init()
        .ok();
}

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

pub fn init_logger_with_filter(filter_str: &str) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(filter_str));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .try_init()
        .ok();
}

pub fn init_logger_json() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().json())
        .try_init()
        .ok();
}

pub fn init_logger_compact() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().compact())
        .try_init()
        .ok();
}

pub fn init_logger_pretty() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().pretty())
        .try_init()
        .ok();
}

pub fn level_to_string(level: Level) -> &'static str {
    match level {
        Level::TRACE => "trace",
        Level::DEBUG => "debug",
        Level::INFO => "info",
        Level::WARN => "warn",
        Level::ERROR => "error",
    }
}

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
    fn test_level_to_string() {
        assert_eq!(level_to_string(Level::TRACE), "trace");
        assert_eq!(level_to_string(Level::DEBUG), "debug");
        assert_eq!(level_to_string(Level::INFO), "info");
        assert_eq!(level_to_string(Level::WARN), "warn");
        assert_eq!(level_to_string(Level::ERROR), "error");
    }

    #[test]
    fn test_init_logger_with_filter() {
        init_logger_with_filter("info,common=debug");
        tracing::info!("使用自定义过滤器初始化日志");
        tracing::debug!("DEBUG级别日志");
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
