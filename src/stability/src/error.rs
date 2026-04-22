//! 稳定性测试错误处理

use thiserror::Error;

/// 稳定性测试错误类型
#[derive(Debug, Error)]
pub enum StabilityTestError {
    #[error("测试超时: {0}")]
    Timeout(String),

    #[error("内存使用超过限制: {0} bytes, 限制: {1} bytes")]
    MemoryLimitExceeded(u64, u64),

    #[error("资源泄漏: {0}")]
    ResourceLeak(String),

    #[error("数据不一致: {0}")]
    DataInconsistency(String),

    #[error("断言失败: {0}")]
    AssertionFailed(String),

    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("其他错误: {0}")]
    Other(String),
}

/// 稳定性测试结果
pub type Result<T> = std::result::Result<T, StabilityTestError>;
