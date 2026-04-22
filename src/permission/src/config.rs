//! 配置类型定义

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 是否启用权限缓存
    pub enable_cache: bool,
    /// 缓存过期时间（秒）
    pub cache_ttl_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self { enable_cache: true, cache_ttl_secs: 300 }
    }
}
