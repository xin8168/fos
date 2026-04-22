//! 配置类型定义

use serde::{Deserialize, Serialize};

/// 锁配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockConfig {
    /// 锁超时时间（秒）
    pub lock_timeout_secs: u64,
    /// 获取锁等待时间（秒）
    pub wait_timeout_secs: u64,
    /// 最大等待队列长度
    pub max_wait_queue_size: usize,
    /// 是否启用死锁检测
    pub deadlock_detection: bool,
    /// 死锁检测间隔（秒）
    pub deadlock_check_interval_secs: u64,
}

impl Default for LockConfig {
    fn default() -> Self {
        Self {
            lock_timeout_secs: 30,
            wait_timeout_secs: 10,
            max_wait_queue_size: 100,
            deadlock_detection: true,
            deadlock_check_interval_secs: 60,
        }
    }
}

impl LockConfig {
    /// 创建新配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置锁超时时间
    pub fn with_lock_timeout(mut self, secs: u64) -> Self {
        self.lock_timeout_secs = secs;
        self
    }

    /// 设置等待超时时间
    pub fn with_wait_timeout(mut self, secs: u64) -> Self {
        self.wait_timeout_secs = secs;
        self
    }

    /// 设置最大等待队列长度
    pub fn with_max_queue_size(mut self, size: usize) -> Self {
        self.max_wait_queue_size = size;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LockConfig::default();
        assert_eq!(config.lock_timeout_secs, 30);
        assert_eq!(config.wait_timeout_secs, 10);
        assert!(config.deadlock_detection);
    }

    #[test]
    fn test_config_builder() {
        let config =
            LockConfig::new().with_lock_timeout(60).with_wait_timeout(30).with_max_queue_size(50);

        assert_eq!(config.lock_timeout_secs, 60);
        assert_eq!(config.wait_timeout_secs, 30);
        assert_eq!(config.max_wait_queue_size, 50);
    }
}
