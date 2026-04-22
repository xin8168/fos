//! 配置类型定义

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 最大回滚层级
    pub max_rollback_levels: usize,
    /// 是否启用自动回滚
    pub auto_rollback: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { max_rollback_levels: 10, auto_rollback: false }
    }
}
