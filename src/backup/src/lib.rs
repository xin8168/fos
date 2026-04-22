//! # FOS Backup - 数据备份模块
//!
//! ## 核心职责
//! - 数据备份执行
//! - 备份恢复管理
//! - 备份策略配置
//! - 增量备份和差异备份
//!
//! ## 安全铁律
//! - 不做规则判断

pub mod backup;
pub mod config;
pub mod error;
pub mod executor;
pub mod incremental;
pub mod scheduler;

/// 导出所有公共类型
pub use backup::{
    BackupEnvironment, BackupItem, BackupMetadata, BackupPlan, BackupRetention, BackupSchedule,
    BackupStatus, BackupTarget, BackupTargetType, BackupType,
};
pub use config::Config;
pub use error::{Error, Result};
pub use executor::FsBackupExecutor;
pub use incremental::{
    calculate_file_checksum, calculate_incremental_changes, compress_file, decompress_file,
    merge_incremental_backups, scan_directory, DiffStrategy, FileChange, FileChangeType,
    FileMetadata, IncrementalManifest,
};
pub use scheduler::{BackupExecutor, BackupNotification, BackupNotificationType, BackupScheduler};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
