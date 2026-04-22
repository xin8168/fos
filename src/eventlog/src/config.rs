//! 配置类型定义

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 日志保留天数
    pub retention_days: u32,
    /// 是否启用异步写入
    pub async_write: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { retention_days: 30, async_write: true }
    }
}
