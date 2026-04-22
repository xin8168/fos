//! # FOS Permission - 权限中心模块
//!
//! ## 核心职责
//! - 权限验证与授权
//! - 角色管理
//! - 资源访问控制
//!
//! ## 安全铁律
//! - 不做规则判断

pub mod checker;
pub mod config;
pub mod error;
pub mod policy;
pub mod role;

pub use checker::{
    PermissionChecker, PermissionCheckerBuilder, PermissionRequest, PermissionResult,
};
pub use config::Config;
pub use error::{Error, Result};
pub use policy::{
    ActionId, Policy, PolicyEffect, PolicyId, PolicyManager, PolicyStatus, PolicyUpdate,
    ResourceId, ResourceType,
};
pub use role::{PermissionId, Role, RoleId, RoleManager, RoleStatus, RoleType, RoleUpdate};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
