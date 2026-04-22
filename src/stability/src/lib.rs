//! FOS 稳定性测试模块
//!
//! 提供长时间运行、边界场景和异常恢复的稳定性测试

pub mod error;
pub mod runner;
pub mod tests;
pub mod utils;

pub use error::{Result, StabilityTestError};
pub use runner::{
    cache_stability, lock_stability, system_integration_stability, StabilityTestRunner,
};
pub use utils::{Metrics, TestConfig, TestResult};
