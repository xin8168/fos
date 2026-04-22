//! 配置类型定义

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdempotencyConfig {
    pub key_expiry_secs: u64,
    pub auto_cleanup: bool,
    pub max_cache_entries: usize,
}

impl Default for IdempotencyConfig {
    fn default() -> Self {
        Self { key_expiry_secs: 86400, auto_cleanup: true, max_cache_entries: 10000 }
    }
}

impl IdempotencyConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_expiry(mut self, secs: u64) -> Self {
        self.key_expiry_secs = secs;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = IdempotencyConfig::default();
        assert_eq!(config.key_expiry_secs, 86400);
    }
}
