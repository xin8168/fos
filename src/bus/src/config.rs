//! # Bus 配置模块

use serde::{Deserialize, Serialize};

/// Bus 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusConfig {
    /// 工作线程数
    pub workers: usize,

    /// 任务超时（秒）
    pub task_timeout_secs: u64,

    /// 队列最大容量
    pub max_queue_size: usize,

    /// 心跳间隔（秒）
    pub heartbeat_interval_secs: u64,

    /// 最大重试次数
    pub max_retries: u32,
}

impl Default for BusConfig {
    fn default() -> Self {
        Self {
            workers: 4,
            task_timeout_secs: 300,
            max_queue_size: 1000,
            heartbeat_interval_secs: 10,
            max_retries: 3,
        }
    }
}

impl BusConfig {
    /// 创建新配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 验证配置
    pub fn validate(&self) -> Result<(), String> {
        if self.workers == 0 {
            return Err("workers must be greater than 0".to_string());
        }
        if self.task_timeout_secs == 0 {
            return Err("task_timeout_secs must be greater than 0".to_string());
        }
        if self.max_queue_size == 0 {
            return Err("max_queue_size must be greater than 0".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BusConfig::default();
        assert_eq!(config.workers, 4);
        assert_eq!(config.task_timeout_secs, 300);
    }

    #[test]
    fn test_config_validation() {
        let config = BusConfig::default();
        assert!(config.validate().is_ok());
    }
}
