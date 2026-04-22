//! 配置类型定义

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 备份目录
    pub backup_dir: String,
    /// 备份保留天数
    pub retention_days: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self { backup_dir: "backups".to_string(), retention_days: 30 }
    }
}
