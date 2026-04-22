//! 错误类型定义

use thiserror::Error;

/// Shutdown错误类型
#[derive(Debug, Error)]
pub enum Error {
    /// 配置错误
    #[error("配置错误: {0}")]
    Config(String),

    /// 关闭错误
    #[error("关闭错误: {0}")]
    Shutdown(String),

    /// 超时错误
    #[error("关闭超时: {0}")]
    Timeout(String),

    /// 任务等待错误
    #[error("任务等待失败: {0}")]
    TaskWaitFailed(String),

    /// 资源清理错误
    #[error("资源清理失败: {0}")]
    CleanupFailed(String),

    /// 信号错误
    #[error("信号处理失败: {0}")]
    SignalError(String),

    /// 内部错误
    #[error("内部错误: {0}")]
    Internal(String),
}

/// Result类型别名
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::Timeout("30秒超时".to_string());
        assert!(err.to_string().contains("超时"));
    }
}
