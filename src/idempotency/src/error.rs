//! 错误类型定义

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("配置错误: {0}")]
    Config(String),

    #[error("幂等检查错误: {0}")]
    Idempotency(String),

    #[error("重复请求: {0}")]
    Duplicate(String),

    #[error("处理中: {0}")]
    Processing(String),

    #[error("内部错误: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::Idempotency("test".into());
        assert!(err.to_string().contains("幂等检查错误"));
    }
}
