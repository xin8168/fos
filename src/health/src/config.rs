//! 配置类型定义

use serde::{Deserialize, Serialize};

/// 健康检查配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 检查间隔（秒）
    pub check_interval_secs: u64,
    /// 检查超时（秒）
    pub check_timeout_secs: u64,
    /// 是否启用自愈
    pub enable_healing: bool,
    /// 最大自愈重试次数
    pub max_healing_retries: u32,
    /// 检查失败阈值（连续失败多少次后标记为不健康）
    pub failure_threshold: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            check_interval_secs: 10,
            check_timeout_secs: 5,
            enable_healing: true,
            max_healing_retries: 3,
            failure_threshold: 3,
        }
    }
}

impl Config {
    /// 创建新的配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置检查间隔
    pub fn with_interval(mut self, secs: u64) -> Self {
        self.check_interval_secs = secs;
        self
    }

    /// 设置检查超时
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.check_timeout_secs = secs;
        self
    }

    /// 设置是否启用自愈
    pub fn with_healing(mut self, enable: bool) -> Self {
        self.enable_healing = enable;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.check_interval_secs, 10);
        assert_eq!(config.check_timeout_secs, 5);
        assert!(config.enable_healing);
    }

    #[test]
    fn test_config_builder() {
        let config = Config::new().with_interval(30).with_timeout(10).with_healing(false);

        assert_eq!(config.check_interval_secs, 30);
        assert_eq!(config.check_timeout_secs, 10);
        assert!(!config.enable_healing);
    }
}
