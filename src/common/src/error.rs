//! FOS 全局统一错误类型

use thiserror::Error;

/// FOS 错误类型
#[derive(Debug, Error)]
pub enum Error {
    #[error("验证错误: {0}")]
    Validation(String),

    #[error("执行错误: {0}")]
    Execution(String),

    #[error("超时错误: {0}")]
    Timeout(String),

    #[error("权限错误: {0}")]
    Permission(String),

    #[error("资源不存在: {0}")]
    NotFound(String),

    #[error("冲突错误: {0}")]
    Conflict(String),

    #[error("内部错误: {0}")]
    Internal(String),

    #[error("序列化错误: {0}")]
    Serialization(String),

    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("配置错误: {0}")]
    Config(String),
}

/// 错误类型分类
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// 客户端错误 (4xx)
    Client,
    /// 服务器错误 (5xx)
    Server,
    /// 业务逻辑错误
    Business,
    /// 系统错误
    System,
}

impl Error {
    /// 获取错误类型分类
    pub fn kind(&self) -> ErrorKind {
        match self {
            Error::Validation(_)
            | Error::Permission(_)
            | Error::NotFound(_)
            | Error::Conflict(_) => ErrorKind::Client,
            Error::Execution(_)
            | Error::Timeout(_)
            | Error::Internal(_)
            | Error::Serialization(_) => ErrorKind::Server,
            Error::Config(_) => ErrorKind::Business,
            Error::Io(_) => ErrorKind::System,
        }
    }

    /// 获取错误码
    pub fn code(&self) -> u16 {
        match self {
            Error::Validation(_) => 400,
            Error::Permission(_) => 403,
            Error::NotFound(_) => 404,
            Error::Conflict(_) => 409,
            Error::Execution(_) => 500,
            Error::Timeout(_) => 504,
            Error::Internal(_) => 500,
            Error::Serialization(_) => 500,
            Error::Io(_) => 500,
            Error::Config(_) => 500,
        }
    }
}

/// FOS 全局 Result 类型
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_kind() {
        let validation_err = Error::Validation("无效输入".to_string());
        assert_eq!(validation_err.kind(), ErrorKind::Client);
        assert_eq!(validation_err.code(), 400);

        let internal_err = Error::Internal("内部错误".to_string());
        assert_eq!(internal_err.kind(), ErrorKind::Server);
        assert_eq!(internal_err.code(), 500);

        let not_found_err = Error::NotFound("资源不存在".to_string());
        assert_eq!(not_found_err.kind(), ErrorKind::Client);
        assert_eq!(not_found_err.code(), 404);
    }

    #[test]
    fn test_error_display() {
        let err = Error::Validation("测试错误".to_string());
        assert_eq!(format!("{}", err), "验证错误: 测试错误");
    }
}
