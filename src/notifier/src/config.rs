//! 配置类型定义

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 重试次数
    pub max_retries: u32,
    /// 重试间隔（毫秒）
    pub retry_interval_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self { max_retries: 3, retry_interval_ms: 1000 }
    }
}
