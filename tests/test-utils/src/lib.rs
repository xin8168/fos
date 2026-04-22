//! # FOS 测试工具库
//!
//! 提供测试共用的工具函数、mock数据和断言宏
//!
//! ## 核心功能
//! - 测试夹具（Fixtures）
//! - Mock数据生成
//! - 测试断言宏
//! - 异步测试支持
//!
//! ## 使用方式
//! ```rust
//! use fos_test_utils::{assert_ok, mock_event, TestContext};
//!
//! #[tokio::test]
//! async fn test_example() {
//!     let ctx = TestContext::new();
//!     let event = mock_event("测试事件");
//!     assert_ok!(result);
//! }
//! ```

pub mod fixtures;
pub mod mocks;
pub mod assertions;
pub mod helpers;
pub mod context;

pub use fixtures::*;
pub use mocks::*;
pub use assertions::*;
pub use helpers::*;
pub use context::TestContext;

/// 模块版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 模块名称
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// 测试结果类型别名
pub type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
