//! # Gateway 错误处理模块

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Gateway 错误类型
#[derive(Debug, Error)]
pub enum GatewayError {
    /// 协议格式错误
    #[error("协议格式错误: {0}")]
    ProtocolFormat(String),

    /// 缺少必要字段
    #[error("缺少必要字段: {0}")]
    MissingField(String),

    /// 字段值无效
    #[error("字段值无效: {field} = {value}")]
    InvalidField { field: String, value: String },

    /// 校验失败
    #[error("校验失败: {0}")]
    ValidationFailed(String),

    /// 权限不足
    #[error("权限不足: {0}")]
    Unauthorized(String),

    /// 内部错误
    #[error("内部错误: {0}")]
    Internal(String),

    /// 超时
    #[error("请求超时")]
    Timeout,

    /// 服务不可用
    #[error("服务不可用: {0}")]
    ServiceUnavailable(String),

    /// 命令被拦截
    #[error("命令被拦截: {reason}")]
    CommandBlocked { reason: String },

    /// JSON 解析错误
    #[error("JSON 解析错误: {0}")]
    JsonParse(#[from] serde_json::Error),

    /// IO 错误
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
}

/// Gateway 结果类型
pub type Result<T> = std::result::Result<T, GatewayError>;

impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        let (status, error_message, error_type) = match self {
            GatewayError::ProtocolFormat(msg) => {
                (StatusCode::BAD_REQUEST, format!("协议格式错误: {}", msg), "protocol_format_error")
            },
            GatewayError::MissingField(field) => {
                (StatusCode::BAD_REQUEST, format!("缺少必要字段: {}", field), "missing_field_error")
            },
            GatewayError::InvalidField { field, value } => (
                StatusCode::BAD_REQUEST,
                format!("字段值无效: {} = {}", field, value),
                "invalid_field_error",
            ),
            GatewayError::ValidationFailed(msg) => {
                (StatusCode::BAD_REQUEST, format!("校验失败: {}", msg), "validation_error")
            },
            GatewayError::Unauthorized(msg) => {
                (StatusCode::UNAUTHORIZED, format!("权限不足: {}", msg), "unauthorized_error")
            },
            GatewayError::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("内部错误: {}", msg), "internal_error")
            },
            GatewayError::Timeout => {
                (StatusCode::REQUEST_TIMEOUT, "请求超时".to_string(), "timeout_error")
            },
            GatewayError::ServiceUnavailable(msg) => (
                StatusCode::SERVICE_UNAVAILABLE,
                format!("服务不可用: {}", msg),
                "service_unavailable_error",
            ),
            GatewayError::CommandBlocked { reason } => {
                (StatusCode::FORBIDDEN, format!("命令被拦截: {}", reason), "command_blocked_error")
            },
            GatewayError::JsonParse(e) => {
                (StatusCode::BAD_REQUEST, format!("JSON 解析错误: {}", e), "json_parse_error")
            },
            GatewayError::Io(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("IO 错误: {}", e), "io_error")
            },
        };

        let body = Json(json!({
            "error": {
                "code": status.as_u16(),
                "message": error_message,
                "type": error_type
            }
        }));

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_format_error() {
        let error = GatewayError::ProtocolFormat("缺少事件字段".to_string());
        assert_eq!(error.to_string(), "协议格式错误: 缺少事件字段");
    }

    #[test]
    fn test_missing_field_error() {
        let error = GatewayError::MissingField("event".to_string());
        assert_eq!(error.to_string(), "缺少必要字段: event");
    }

    #[test]
    fn test_invalid_field_error() {
        let error = GatewayError::InvalidField {
            field: "steps".to_string(),
            value: "空数组".to_string(),
        };
        assert_eq!(error.to_string(), "字段值无效: steps = 空数组");
    }

    #[test]
    fn test_command_blocked_error() {
        let error = GatewayError::CommandBlocked { reason: "违反安全规则".to_string() };
        assert_eq!(error.to_string(), "命令被拦截: 违反安全规则");
    }
}
