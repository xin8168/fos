//! # FOS Idempotency - 幂等控制模块
//!
//! 提供请求幂等性保证，防止重复操作
//!
//! ## 核心职责
//! - 请求幂等性保证
//! - 幂等键管理
//! - 重复请求检测
//! - 结果缓存复用
//!
//! ## 安全铁律
//! - 不做规则判断
//! - 不执行业务逻辑
//! - 只负责幂等控制

pub mod error;
pub mod config;
pub mod key;
pub mod checker;
pub mod cache;
pub mod manager;

pub use error::{Error, Result};
pub use config::IdempotencyConfig;
pub use key::{IdempotencyKey, KeyStatus};
pub use checker::IdempotencyChecker;
pub use cache::ResultCache;
pub use manager::IdempotencyManager;

/// 模块版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 模块名称
pub const NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_name() {
        assert_eq!(NAME, "fos-idempotency");
    }
}
