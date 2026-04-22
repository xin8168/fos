//! 事务错误类型定义

use thiserror::Error;

/// 事务错误
#[derive(Debug, Error)]
pub enum Error {
    /// 配置错误
    #[error("配置错误: {0}")]
    Config(String),

    /// 事务错误
    #[error("事务错误: {0}")]
    Transaction(String),

    /// 参与者错误
    #[error("参与者错误: {0}")]
    Participant(String),

    /// 执行错误
    #[error("执行错误: {0}")]
    Execution(String),

    /// 超时错误
    #[error("事务超时: {0}")]
    Timeout(String),

    /// 补偿错误
    #[error("补偿失败: {0}")]
    Compensation(String),

    /// 日志错误
    #[error("日志错误: {0}")]
    Log(String),

    /// 序列化错误
    #[error("序列化错误: {0}")]
    Serialization(String),

    /// 内部错误
    #[error("内部错误: {0}")]
    Internal(String),
}

/// 事务结果类型
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// 是否可重试
    pub fn is_retryable(&self) -> bool {
        matches!(self, Error::Timeout(_) | Error::Execution(_) | Error::Internal(_))
    }

    /// 是否需要补偿
    pub fn needs_compensation(&self) -> bool {
        matches!(self, Error::Execution(_) | Error::Timeout(_) | Error::Participant(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::Transaction("test error".to_string());
        assert!(err.to_string().contains("事务错误"));
    }

    #[test]
    fn test_is_retryable() {
        assert!(Error::Timeout("test".to_string()).is_retryable());
        assert!(!Error::Config("test".to_string()).is_retryable());
    }

    #[test]
    fn test_needs_compensation() {
        assert!(Error::Execution("test".to_string()).needs_compensation());
        assert!(!Error::Serialization("test".to_string()).needs_compensation());
    }
}
