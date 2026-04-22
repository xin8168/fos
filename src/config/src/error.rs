//! 配置错误处理

use thiserror::Error;

/// 配置错误类型
#[derive(Debug, Error)]
pub enum ConfigError {
    /// 配置文件不存在
    #[error("配置文件不存在: {0}")]
    FileNotFound(String),

    /// 配置解析失败
    #[error("配置解析失败: {0}")]
    ParseError(String),

    /// 配置验证失败
    #[error("配置验证失败: {0}")]
    ValidationError(String),

    /// 配置项不存在
    #[error("配置项不存在: {0}")]
    NotFound(String),

    /// 环境变量错误
    #[error("环境变量错误: {0}")]
    EnvironmentError(String),

    /// IO错误
    #[error("IO错误: {0}")]
    IoError(String),

    /// 内部错误
    #[error("内部错误: {0}")]
    Internal(String),
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::IoError(err.to_string())
    }
}

impl From<serde_yaml::Error> for ConfigError {
    fn from(err: serde_yaml::Error) -> Self {
        ConfigError::ParseError(err.to_string())
    }
}

/// 配置结果类型
pub type Result<T> = std::result::Result<T, ConfigError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ConfigError::FileNotFound("config.yaml".to_string());
        assert!(err.to_string().contains("配置文件不存在"));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let config_err: ConfigError = io_err.into();
        assert!(matches!(config_err, ConfigError::IoError(_)));
    }
}
