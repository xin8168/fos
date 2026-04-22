//! # FOS Lock - 分布式锁模块
//!
//! 提供分布式锁管理能力，确保并发安全
//!
//! ## 核心职责
//! - 分布式锁管理
//! - 锁超时控制
//! - 锁重入支持
//! - 死锁检测
//!
//! ## 安全铁律
//! - 不做规则判断
//! - 不执行业务逻辑
//! - 只负责锁资源管理

pub mod config;
pub mod error;
pub mod lock;
pub mod manager;
pub mod queue;

pub use config::LockConfig;
pub use error::{Error, Result};
pub use lock::{Lock, LockId, LockState, LockType};
pub use manager::LockManager;
pub use queue::LockWaitQueue;

/// 模块版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 模块名称
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// 锁持有者标识
pub type LockOwner = String;

/// 锁键名
pub type LockKey = String;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_name() {
        assert_eq!(NAME, "fos-lock");
    }
}
