//! 配置类型定义

use serde::{Deserialize, Serialize};

/// 事务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionConfig {
    /// 事务超时时间（秒）
    pub timeout_secs: u64,
    /// 最大重试次数
    pub max_retries: u32,
    /// 补偿重试次数
    pub compensate_retries: u32,
    /// 是否启用自动恢复
    pub auto_recovery: bool,
    /// 恢复检查间隔（秒）
    pub recovery_interval_secs: u64,
    /// 日志最大条目数
    pub max_log_entries: usize,
}

impl Default for TransactionConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 60,
            max_retries: 3,
            compensate_retries: 3,
            auto_recovery: true,
            recovery_interval_secs: 30,
            max_log_entries: 10000,
        }
    }
}

impl TransactionConfig {
    /// 创建新配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置超时时间
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// 设置最大重试次数
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// 设置自动恢复
    pub fn with_auto_recovery(mut self, enable: bool) -> Self {
        self.auto_recovery = enable;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TransactionConfig::default();
        assert_eq!(config.timeout_secs, 60);
        assert_eq!(config.max_retries, 3);
        assert!(config.auto_recovery);
    }

    #[test]
    fn test_config_builder() {
        let config = TransactionConfig::new()
            .with_timeout(120)
            .with_max_retries(5)
            .with_auto_recovery(false);

        assert_eq!(config.timeout_secs, 120);
        assert_eq!(config.max_retries, 5);
        assert!(!config.auto_recovery);
    }
}
