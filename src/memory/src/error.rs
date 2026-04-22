//! # Memory 错误处理模块

use thiserror::Error;

/// Memory 错误类型
#[derive(Debug, Error)]
pub enum MemoryError {
    /// 事件不存在
    #[error("事件不存在: {0}")]
    EventNotFound(String),

    /// 存储错误
    #[error("存储错误: {0}")]
    StorageError(String),

    /// 查询错误
    #[error("查询错误: {0}")]
    QueryError(String),

    /// 连接错误
    #[error("数据库连接错误: {0}")]
    ConnectionError(String),

    /// 序列化错误
    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),

    /// 数据库错误
    #[error("数据库错误: {0}")]
    Database(String),

    /// 配置错误
    #[error("配置错误: {0}")]
    ConfigError(String),

    /// 内部错误
    #[error("内部错误: {0}")]
    Internal(String),
}

/// Memory 结果类型
pub type Result<T> = std::result::Result<T, MemoryError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_not_found_error() {
        let error = MemoryError::EventNotFound("event-001".to_string());
        assert_eq!(error.to_string(), "事件不存在: event-001");
    }

    #[test]
    fn test_storage_error() {
        let error = MemoryError::StorageError("写入失败".to_string());
        assert_eq!(error.to_string(), "存储错误: 写入失败");
    }

    #[test]
    fn test_database_error() {
        let error = MemoryError::Database("连接超时".to_string());
        assert_eq!(error.to_string(), "数据库错误: 连接超时");
    }
}
