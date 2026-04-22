//! FOS Migration Module
//!
//! 数据版本管理和迁移模块

pub mod config;
pub mod error;
pub mod manager;
pub mod version;

pub use config::Config;
pub use error::{MigrationError, Result as MigrationResult};
pub use manager::{MigrationExecutor, MigrationManager, MigrationVersionStatus};
pub use version::{
    MigrationDirection, MigrationRecord, MigrationStatus, MigrationType, MigrationVersion,
};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
