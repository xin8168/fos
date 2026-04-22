//! Audit 错误处理

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuditError {
    #[error("日志不存在: {0}")]
    NotFound(String),

    #[error("查询失败: {0}")]
    QueryFailed(String),

    #[error("存储失败: {0}")]
    StorageFailed(String),
}

pub type Result<T> = std::result::Result<T, AuditError>;
