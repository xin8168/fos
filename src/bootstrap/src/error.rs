//! 错误类型定义

use thiserror::Error;

/// Bootstrap错误类型
#[derive(Debug, Error)]
pub enum Error {
    /// 配置错误
    #[error("配置错误: {0}")]
    Config(String),

    /// 启动错误
    #[error("启动错误: {0}")]
    Startup(String),

    /// 依赖检查错误
    #[error("依赖检查失败: {0}")]
    DependencyCheckFailed(String),

    /// 模块初始化错误
    #[error("模块初始化失败: {0}")]
    ModuleInitFailed(String),

    /// 超时错误
    #[error("启动超时: {0}")]
    Timeout(String),

    /// 阶段执行错误
    #[error("阶段执行失败: {phase} - {message}")]
    PhaseFailed { phase: String, message: String },

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
        let err = Error::Config("测试配置错误".to_string());
        assert!(err.to_string().contains("配置错误"));
    }

    #[test]
    fn test_error_timeout() {
        let err = Error::Timeout("30秒超时".to_string());
        assert!(err.to_string().contains("启动超时"));
    }

    #[test]
    fn test_error_phase_failed() {
        let err = Error::PhaseFailed {
            phase: "Config".to_string(),
            message: "配置加载失败".to_string(),
        };
        assert!(err.to_string().contains("阶段执行失败"));
    }
}
