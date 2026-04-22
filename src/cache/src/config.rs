//! 配置类型定义

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 缓存最大容量
    pub max_capacity: usize,
    /// 默认过期时间（秒）
    pub default_ttl_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self { max_capacity: 10000, default_ttl_secs: 300 }
    }
}
