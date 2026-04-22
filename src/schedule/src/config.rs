//! 配置类型定义

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 最大并发任务数
    pub max_concurrent_jobs: usize,
    /// 是否启用持久化
    pub persistent: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { max_concurrent_jobs: 100, persistent: false }
    }
}
