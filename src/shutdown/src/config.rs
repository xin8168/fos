//! 配置类型定义

use serde::{Deserialize, Serialize};

/// Shutdown配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownConfig {
    /// 关闭超时时间（秒）
    pub timeout_secs: u64,
    /// 是否等待任务完成
    pub wait_for_tasks: bool,
    /// 任务等待超时（秒）
    pub task_wait_timeout_secs: u64,
    /// 是否启用资源清理
    pub enable_cleanup: bool,
    /// 清理重试次数
    pub cleanup_retries: u32,
    /// 是否强制关闭
    pub force_shutdown: bool,
}

impl Default for ShutdownConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            wait_for_tasks: true,
            task_wait_timeout_secs: 10,
            enable_cleanup: true,
            cleanup_retries: 3,
            force_shutdown: false,
        }
    }
}

impl ShutdownConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    pub fn without_task_wait(mut self) -> Self {
        self.wait_for_tasks = false;
        self
    }
}

pub type Config = ShutdownConfig;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ShutdownConfig::default();
        assert!(config.wait_for_tasks);
        assert!(config.enable_cleanup);
    }
}
