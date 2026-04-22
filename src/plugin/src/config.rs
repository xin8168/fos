//! 配置类型定义

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 插件目录
    pub plugin_dir: String,
    /// 是否启用热加载
    pub hot_reload: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { plugin_dir: "plugins".to_string(), hot_reload: false }
    }
}
