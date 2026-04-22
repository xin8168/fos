//! Bootstrap配置类型定义

use serde::{Deserialize, Serialize};

/// Bootstrap配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapConfig {
    /// 启动超时时间（秒）
    pub timeout_secs: u64,
    /// 是否启用健康检查
    pub enable_health_check: bool,
    /// 是否启用依赖检查
    pub enable_dependency_check: bool,
    /// 是否启用模块初始化
    pub enable_module_init: bool,
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试间隔（毫秒）
    pub retry_interval_ms: u64,
}

impl Default for BootstrapConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 300,
            enable_health_check: true,
            enable_dependency_check: true,
            enable_module_init: true,
            max_retries: 3,
            retry_interval_ms: 1000,
        }
    }
}

impl BootstrapConfig {
    /// 创建新配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置超时时间
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// 禁用健康检查
    pub fn without_health_check(mut self) -> Self {
        self.enable_health_check = false;
        self
    }

    /// 禁用依赖检查
    pub fn without_dependency_check(mut self) -> Self {
        self.enable_dependency_check = false;
        self
    }

    /// 设置重试参数
    pub fn with_retry(mut self, max_retries: u32, interval_ms: u64) -> Self {
        self.max_retries = max_retries;
        self.retry_interval_ms = interval_ms;
        self
    }
}

/// 配置别名（兼容）
pub type Config = BootstrapConfig;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BootstrapConfig::default();

        assert!(config.enable_health_check);
        assert!(config.enable_dependency_check);
        assert!(config.enable_module_init);
        assert!(config.timeout_secs > 0);
    }

    #[test]
    fn test_config_builder() {
        let config =
            BootstrapConfig::new().with_timeout(60).without_health_check().with_retry(5, 500);

        assert_eq!(config.timeout_secs, 60);
        assert!(!config.enable_health_check);
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.retry_interval_ms, 500);
    }

    #[test]
    fn test_config_serialization() {
        let config = BootstrapConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: BootstrapConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.timeout_secs, parsed.timeout_secs);
    }
}
