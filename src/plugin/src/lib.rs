//! # FOS Plugin - 插件系统模块
//!
//! ## 核心职责
//! - 插件生命周期管理
//! - 插件依赖解析
//! - 插件沙箱隔离
//!
//! ## 安全铁律
//! - 不做规则判断

pub mod config;
pub mod error;
pub mod lifecycle;
pub mod loader;
pub mod plugin;
pub mod sandbox;

pub use config::Config;
pub use error::{Error, Result};
pub use lifecycle::{EventListener, LifecycleEvent, PluginLifecycleManager};
pub use loader::PluginLoader;
pub use plugin::{PluginMetadata, PluginState, PluginStats, PluginStatus, PluginType};
pub use sandbox::{
    PluginPermissions, ResourceLimits, ResourceUsage, Sandbox, SandboxConfig, SandboxManager,
};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
