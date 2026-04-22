//! 配置类型定义

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 每秒请求数限制
    pub requests_per_second: u32,
    /// 突发流量限制
    pub burst_size: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self { requests_per_second: 100, burst_size: 200 }
    }
}
