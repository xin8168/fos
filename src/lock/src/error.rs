//! 错误类型定义

use thiserror::Error;

/// 锁错误
#[derive(Debug, Error)]
pub enum Error {
    /// 配置错误
    #[error("配置错误: {0}")]
    Config(String),

    /// 锁操作错误
    #[error("锁操作错误: {0}")]
    Lock(String),

    /// 锁超时
    #[error("锁超时: {0}")]
    Timeout(String),

    /// 死锁检测
    #[error("检测到死锁: {0}")]
    Deadlock(String),

    /// 内部错误
    #[error("内部错误: {0}")]
    Internal(String),
}

/// 锁结果类型
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::Lock("test error".to_string());
        assert!(err.to_string().contains("锁操作错误"));
    }
}
