//! MCP 错误处理

use thiserror::Error;

#[derive(Debug, Error)]
pub enum McpError {
    #[error("设备不存在: {0}")]
    DeviceNotFound(String),

    #[error("设备离线: {0}")]
    DeviceOffline(String),

    #[error("设备不支持此操作: {0}")]
    OperationNotSupported(String),

    #[error("协议转换失败: {0}")]
    ProtocolConversionFailed(String),

    #[error("连接失败: {0}")]
    ConnectionFailed(String),

    #[error("连接不存在: {0}")]
    ConnectionNotFound(String),

    #[error("会话不存在: {0}")]
    SessionNotFound(String),

    #[error("数量限制: {0}")]
    LimitExceeded(String),

    #[error("内部错误: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, McpError>;
