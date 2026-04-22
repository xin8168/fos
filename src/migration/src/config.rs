//! 配置类型定义

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 迁移脚本目录
    pub scripts_dir: String,
    /// 是否启用自动迁移
    pub auto_migrate: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { scripts_dir: "migrations".to_string(), auto_migrate: false }
    }
}
