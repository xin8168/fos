//! # FOS Rollback - 回滚引擎模块
//!
//! ## 核心职责
//! - 提供操作回滚能力
//! - 管理回滚动作链
//! - 支持多级回滚
//!
//! ## 安全铁律
//! - 不做规则判断

pub mod config;
pub mod error;
pub mod executor;
pub mod snapshot;
pub mod verifier;

pub use config::Config;
pub use error::{Error, Result};
pub use executor::{
    RollbackAction, RollbackActionStatus, RollbackActionType, RollbackExecutor, RollbackResult,
};
pub use snapshot::{Snapshot, SnapshotId, SnapshotManager, SnapshotStatus, SnapshotType};
pub use verifier::{
    RollbackVerifier, VerificationCheck, VerificationCheckType, VerificationResult,
    VerificationStatus,
};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
