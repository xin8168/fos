//! FOS Config - 配置管理模块
//!
//! 提供统一的配置管理能力，支持多源配置加载、热重载、验证
//!
//! ## 核心职责
//! - 多源配置加载（文件、环境变量、默认值）
//! - 配置热重载
//! - 配置验证
//! - 类型安全的配置访问
//!
//! ## 安全铁律
//! - 不执行业务操作
//! - 不修改配置源
//! - 不暴露敏感信息

pub mod error;
pub mod loader;
pub mod sources;
pub mod validator;
pub mod watcher;

pub use error::{ConfigError, Result};
pub use loader::ConfigLoader;
pub use sources::{ConfigSource, EnvironmentSource, FileSource};
pub use validator::ConfigValidator;
pub use watcher::ConfigWatcher;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// 模块版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 模块名称
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// FOS 主配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FosConfig {
    /// 服务配置
    pub server: ServerConfig,
    /// 日志配置
    pub logging: LoggingConfig,
    /// 数据库配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<DatabaseConfig>,
    /// 模块配置
    #[serde(default)]
    pub modules: HashMap<String, serde_yaml::Value>,
}

impl Default for FosConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            logging: LoggingConfig::default(),
            database: None,
            modules: HashMap::new(),
        }
    }
}

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// 服务名称
    #[serde(default = "default_server_name")]
    pub name: String,
    /// 监听地址
    #[serde(default = "default_host")]
    pub host: String,
    /// 监听端口
    #[serde(default = "default_port")]
    pub port: u16,
    /// 工作线程数
    #[serde(default = "default_workers")]
    pub workers: usize,
    /// 请求超时（秒）
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    /// 是否启用TLS
    #[serde(default)]
    pub tls: bool,
}

fn default_server_name() -> String {
    "fos-server".to_string()
}
fn default_host() -> String {
    "0.0.0.0".to_string()
}
fn default_port() -> u16 {
    8080
}
fn default_workers() -> usize {
    4
}
fn default_timeout() -> u64 {
    30
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            name: default_server_name(),
            host: default_host(),
            port: default_port(),
            workers: default_workers(),
            timeout_secs: default_timeout(),
            tls: false,
        }
    }
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别
    #[serde(default = "default_log_level")]
    pub level: String,
    /// 日志格式
    #[serde(default = "default_log_format")]
    pub format: String,
    /// 日志输出路径
    #[serde(default = "default_log_path")]
    pub path: String,
    /// 是否输出到控制台
    #[serde(default = "default_console")]
    pub console: bool,
}

fn default_log_level() -> String {
    "info".to_string()
}
fn default_log_format() -> String {
    "json".to_string()
}
fn default_log_path() -> String {
    "logs/fos.log".to_string()
}
fn default_console() -> bool {
    true
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
            path: default_log_path(),
            console: default_console(),
        }
    }
}

/// 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// 数据库类型
    #[serde(default = "default_db_type")]
    pub db_type: String,
    /// 连接地址
    pub url: String,
    /// 最大连接数
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    /// 最小连接数
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
    /// 连接超时（秒）
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout_secs: u64,
}

fn default_db_type() -> String {
    "postgres".to_string()
}
fn default_max_connections() -> u32 {
    10
}
fn default_min_connections() -> u32 {
    1
}
fn default_connection_timeout() -> u64 {
    30
}

/// 配置管理器
pub struct ConfigManager {
    /// 配置源
    sources: Vec<Box<dyn ConfigSource + Send + Sync>>,
    /// 当前配置
    config: std::sync::RwLock<FosConfig>,
    /// 配置验证器
    validator: ConfigValidator,
    /// 配置路径
    config_path: Option<PathBuf>,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            config: std::sync::RwLock::new(FosConfig::default()),
            validator: ConfigValidator::new(),
            config_path: None,
        }
    }

    /// 添加配置源
    pub fn with_source<S: ConfigSource + Send + Sync + 'static>(mut self, source: S) -> Self {
        self.sources.push(Box::new(source));
        self
    }

    /// 从文件加载配置
    pub fn from_file<P: Into<PathBuf>>(path: P) -> Self {
        let mut manager = Self::new();
        manager.config_path = Some(path.into());
        manager.sources.push(Box::new(FileSource::new(manager.config_path.clone().unwrap())));
        manager
    }

    /// 从环境变量加载配置
    pub fn from_env() -> Self {
        let mut manager = Self::new();
        manager.sources.push(Box::new(EnvironmentSource::new()));
        manager
    }

    /// 合并多个配置源
    pub fn with_defaults(mut self) -> Self {
        // 按优先级合并配置
        self.merge_sources();
        self
    }

    /// 合并配置源
    fn merge_sources(&mut self) {
        let mut merged = FosConfig::default();

        for source in &self.sources {
            if let Ok(partial) = source.load() {
                merged = self.merge_configs(merged, partial);
            }
        }

        *self.config.write().unwrap() = merged;
    }

    /// 合并两个配置
    fn merge_configs(&self, base: FosConfig, overlay: serde_yaml::Value) -> FosConfig {
        // 使用 serde_yaml 进行深度合并
        let base_value = serde_yaml::to_value(&base).unwrap_or_default();

        if let (Some(mut base_map), Some(overlay_map)) =
            (base_value.as_mapping().cloned(), overlay.as_mapping().cloned())
        {
            for (k, v) in overlay_map {
                base_map.insert(k, v);
            }
            if let Ok(merged) = serde_yaml::from_value(serde_yaml::Value::Mapping(base_map)) {
                return merged;
            }
        }
        base
    }

    /// 加载配置
    pub fn load(&mut self) -> Result<()> {
        self.merge_sources();
        self.validator.validate(&self.config.read().unwrap())?;
        Ok(())
    }

    /// 获取当前配置
    pub fn get(&self) -> std::sync::RwLockReadGuard<'_, FosConfig> {
        self.config.read().unwrap()
    }

    /// 重新加载配置
    pub fn reload(&mut self) -> Result<()> {
        self.load()
    }

    /// 获取配置路径
    pub fn config_path(&self) -> Option<&PathBuf> {
        self.config_path.as_ref()
    }

    /// 获取模块配置
    pub fn get_module_config<T: for<'de> Deserialize<'de>>(&self, module: &str) -> Result<T> {
        let config = self.config.read().unwrap();
        if let Some(value) = config.modules.get(module) {
            serde_yaml::from_value(value.clone())
                .map_err(|e| ConfigError::ParseError(format!("模块配置解析失败: {}", e)))
        } else {
            Err(ConfigError::ValidationError(format!("模块 {} 配置不存在", module)))
        }
    }

    /// 设置配置（用于测试）
    #[cfg(test)]
    pub fn set(&self, config: FosConfig) {
        *self.config.write().unwrap() = config;
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = FosConfig::default();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.workers, 4);
    }

    #[test]
    fn test_config_manager_creation() {
        let manager = ConfigManager::new();
        let config = manager.get();
        assert_eq!(config.server.name, "fos-server");
    }

    #[test]
    fn test_server_config_defaults() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert!(!config.tls);
    }

    #[test]
    fn test_logging_config_defaults() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, "info");
        assert!(config.console);
    }
}
