//! # Bus 错误处理模块

use thiserror::Error;

/// Bus 错误类型
#[derive(Debug, Error)]
pub enum BusError {
    /// 任务不存在
    #[error("任务不存在: {0}")]
    TaskNotFound(String),

    /// 任务执行失败
    #[error("任务执行失败: {0}")]
    ExecutionFailed(String),

    /// 任务超时
    #[error("任务超时: {0}")]
    Timeout(String),

    /// 队列已满
    #[error("任务队列已满")]
    QueueFull,

    /// 执行器不可用
    #[error("执行器不可用: {0}")]
    ExecutorUnavailable(String),

    /// 配置错误
    #[error("配置错误: {0}")]
    ConfigError(String),

    /// 内部错误
    #[error("内部错误: {0}")]
    Internal(String),

    /// 序列化错误
    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Bus 结果类型
pub type Result<T> = std::result::Result<T, BusError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_not_found_error() {
        let error = BusError::TaskNotFound("task-001".to_string());
        assert_eq!(error.to_string(), "任务不存在: task-001");
    }

    #[test]
    fn test_execution_failed_error() {
        let error = BusError::ExecutionFailed("执行超时".to_string());
        assert_eq!(error.to_string(), "任务执行失败: 执行超时");
    }

    #[test]
    fn test_queue_full_error() {
        let error = BusError::QueueFull;
        assert_eq!(error.to_string(), "任务队列已满");
    }
}
