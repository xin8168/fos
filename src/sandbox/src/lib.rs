//! # FOS Sandbox - 反射弧隔离层
//!
//! FOS神经元控制器的反射弧机制：危险动作在隔离环境预执行
//!
//! 类比于人体神经系统的脊髓反射弧：危险刺激直接反射，不上传大脑
//!
//! ## 核心职责
//! - 隔离执行环境（沙箱）
//! - 反射弧机制（预执行验证）
//! - 快照管理（状态保存）
//! - 环境验证（执行前检查）
//!
//! ## 隔离级别
//! - Full: 完全隔离（文件系统+网络+进程）
//! - Filesystem: 文件系统隔离
//! - Network: 网络隔离
//! - Process: 进程隔离
//!
//! ## 安全铁律
//! - 危险动作必须先在沙箱预执行
//! - 失败动作反射执行，不上传运动神经层

pub mod error;
pub mod executor;
pub mod isolation;
pub mod sandbox;
pub mod snapshot;
pub mod validator;

pub use error::{Result, SandboxError};
pub use isolation::{
    FilesystemIsolation, FilesystemIsolationStatus, IsolationConfig, IsolationManager,
    NetworkAccess, NetworkIsolation, NetworkIsolationStatus, PathAccess, ProcessIsolation,
    ProcessIsolationStatus,
};
pub use sandbox::Sandbox;
pub use snapshot::{SandboxSnapshot, SnapshotId, SnapshotManager, SnapshotStatus, SnapshotType};
pub use validator::{
    CheckResult, EnvironmentValidator, IsolationValidator, StatusValidator, ValidationResult,
};

use serde::{Deserialize, Serialize};

/// 沙箱状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SandboxStatus {
    /// 空闲
    Idle,
    /// 运行中
    Running,
    /// 成功
    Success,
    /// 失败
    Failed,
    /// 已销毁
    Destroyed,
}

/// 隔离级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IsolationLevel {
    /// 完全隔离（文件系统、网络、进程）
    Full,
    /// 文件系统隔离
    Filesystem,
    /// 网络隔离
    Network,
    /// 进程隔离
    Process,
}

/// 沙箱配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// 沙箱ID
    pub id: String,

    /// 隔离级别
    pub isolation_level: IsolationLevel,

    /// 超时时间（秒）
    pub timeout_secs: u64,

    /// 内存限制（MB）
    pub memory_limit_mb: u64,

    /// CPU 限制（百分比）
    pub cpu_limit_percent: u64,

    /// 允许的网络访问
    pub network_allowed: bool,

    /// 允许的文件路径
    pub allowed_paths: Vec<String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            isolation_level: IsolationLevel::Full,
            timeout_secs: 300,
            memory_limit_mb: 512,
            cpu_limit_percent: 50,
            network_allowed: false,
            allowed_paths: vec![],
        }
    }
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert_eq!(config.timeout_secs, 300);
        assert_eq!(config.isolation_level, IsolationLevel::Full);
    }
}
