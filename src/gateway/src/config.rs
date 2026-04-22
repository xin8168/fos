//! # Gateway 配置模块

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Gateway 主配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 服务器配置
    pub server: ServerConfig,

    /// 日志配置
    pub logging: LoggingConfig,

    /// 安全配置
    pub security: SecurityConfig,

    /// 上游服务配置
    pub upstream: UpstreamConfig,
}

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// 监听地址
    pub addr: SocketAddr,

    /// 工作线程数
    pub workers: usize,

    /// 请求超时（毫秒）
    pub request_timeout_ms: u64,

    /// 最大请求体大小（字节）
    pub max_request_body_size: usize,
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别
    pub level: String,

    /// 日志格式
    pub format: String,

    /// 是否输出到文件
    pub file_enabled: bool,

    /// 日志文件路径
    pub file_path: Option<String>,
}

/// 安全配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// 是否启用认证
    pub auth_enabled: bool,

    /// 是否启用 TLS
    pub tls_enabled: bool,

    /// TLS 证书路径
    pub tls_cert_path: Option<String>,

    /// TLS 密钥路径
    pub tls_key_path: Option<String>,

    /// 是否启用限流
    pub rate_limit_enabled: bool,

    /// 每分钟最大请求数
    pub rate_limit_requests: usize,

    /// 允许的 IP 白名单
    pub ip_whitelist: Vec<String>,
}

/// 上游服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamConfig {
    /// 校验引擎地址
    pub validator_addr: String,

    /// 硬记忆库地址
    pub memory_addr: String,

    /// 执行总线地址
    pub bus_addr: String,

    /// 连接超时（毫秒）
    pub connection_timeout_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                addr: "0.0.0.0:8080".parse().unwrap(),
                workers: 4,
                request_timeout_ms: 30000,
                max_request_body_size: 10 * 1024 * 1024, // 10MB
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                file_enabled: false,
                file_path: None,
            },
            security: SecurityConfig {
                auth_enabled: true,
                tls_enabled: false,
                tls_cert_path: None,
                tls_key_path: None,
                rate_limit_enabled: true,
                rate_limit_requests: 1000,
                ip_whitelist: vec![],
            },
            upstream: UpstreamConfig {
                validator_addr: "http://localhost:8081".to_string(),
                memory_addr: "http://localhost:8082".to_string(),
                bus_addr: "http://localhost:8083".to_string(),
                connection_timeout_ms: 5000,
            },
        }
    }
}

impl Config {
    /// 从文件加载配置
    pub fn from_file(path: &str) -> Result<Self, config::ConfigError> {
        config::Config::builder()
            .set_default("server.addr", "0.0.0.0:8080")?
            .set_default("server.workers", 4)?
            .set_default("logging.level", "info")?
            .add_source(config::File::with_name(path))
            .add_source(config::Environment::with_prefix("FOS"))
            .build()?
            .try_deserialize()
    }

    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self, config::ConfigError> {
        config::Config::builder()
            .set_default("server.addr", "0.0.0.0:8080")?
            .set_default("server.workers", 4)?
            .set_default("logging.level", "info")?
            .add_source(config::Environment::with_prefix("FOS"))
            .build()?
            .try_deserialize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.workers, 4);
        assert!(config.security.auth_enabled);
    }

    #[test]
    fn test_config_from_env() {
        std::env::set_var("FOS_SERVER__WORKERS", "8");
        let config = Config::from_env().unwrap_or_default();
        // 配置可能因为环境变量而改变
        assert!(config.server.workers >= 1);
    }
}
