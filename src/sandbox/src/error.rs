//! Sandbox 错误处理

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SandboxError {
    #[error("沙箱创建失败: {0}")]
    CreationFailed(String),

    #[error("沙箱执行失败: {0}")]
    ExecutionFailed(String),

    #[error("沙箱超时: {0}")]
    Timeout(String),

    #[error("资源限制超限: {0}")]
    ResourceLimitExceeded(String),

    #[error("隔离失败: {0}")]
    IsolationFailed(String),

    #[error("回滚失败: {0}")]
    RollbackFailed(String),

    #[error("快照错误: {0}")]
    Snapshot(String),

    #[error("内部错误: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, SandboxError>;
