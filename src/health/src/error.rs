//! 错误类型定义

use thiserror::Error;

/// 健康检查错误
#[derive(Debug, Error)]
pub enum Error {
    /// 配置错误
    #[error("配置错误: {0}")]
    Config(String),

    /// 健康检查错误
    #[error("健康检查错误: {0}")]
    Health(String),

    /// 检查超时
    #[error("健康检查超时: {0}")]
    Timeout(String),

    /// 自愈失败
    #[error("自愈失败: {0}")]
    HealingFailed(String),

    /// 检查项不存在
    #[error("检查项不存在: {0}")]
    CheckNotFound(String),

    /// 内部错误
    #[error("内部错误: {0}")]
    Internal(String),
}

/// 健康检查结果类型
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// 是否为配置错误
    pub fn is_config(&self) -> bool {
        matches!(self, Error::Config(_))
    }

    /// 是否为超时错误
    pub fn is_timeout(&self) -> bool {
        matches!(self, Error::Timeout(_))
    }

    /// 是否为自愈错误
    pub fn is_healing(&self) -> bool {
        matches!(self, Error::HealingFailed(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::Config("test error".to_string());
        assert!(err.to_string().contains("配置错误"));
    }

    #[test]
    fn test_error_is_config() {
        let err = Error::Config("test".to_string());
        assert!(err.is_config());
        assert!(!err.is_timeout());
    }

    #[test]
    fn test_error_is_timeout() {
        let err = Error::Timeout("check timed out".to_string());
        assert!(err.is_timeout());
        assert!(!err.is_config());
    }
}
