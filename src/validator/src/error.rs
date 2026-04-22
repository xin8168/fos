//! # Validator 错误处理模块

use thiserror::Error;

/// Validator 错误类型
#[derive(Debug, Error)]
pub enum ValidatorError {
    /// 规则不存在
    #[error("规则不存在: {0}")]
    RuleNotFound(String),

    /// 规则校验失败
    #[error("规则校验失败: {rule} - {reason}")]
    RuleValidationFailed { rule: String, reason: String },

    /// 权限不足
    #[error("权限不足: {0}")]
    PermissionDeniedSimple(String),

    /// 权限不足（详细）
    #[error("用户 {user} 权限不足: 需要 {required} - {reason}")]
    PermissionDenied { user: String, required: String, reason: String },

    /// 设备不可用
    #[error("设备不可用: {0}")]
    DeviceUnavailable(String),

    /// 上下文无效
    #[error("上下文无效: {0}")]
    InvalidContext(String),

    /// 配置错误
    #[error("配置错误: {0}")]
    ConfigError(String),

    /// 内部错误
    #[error("内部错误: {0}")]
    Internal(String),

    /// 序列化错误
    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),

    /// IO 错误
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
}

/// Validator 结果类型
pub type Result<T> = std::result::Result<T, ValidatorError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_not_found_error() {
        let error = ValidatorError::RuleNotFound("rule-001".to_string());
        assert_eq!(error.to_string(), "规则不存在: rule-001");
    }

    #[test]
    fn test_permission_denied_error() {
        let error = ValidatorError::PermissionDeniedSimple("需要管理员权限".to_string());
        assert_eq!(error.to_string(), "权限不足: 需要管理员权限");
    }

    #[test]
    fn test_permission_denied_detailed_error() {
        let error = ValidatorError::PermissionDenied {
            user: "user-001".to_string(),
            required: "file:write".to_string(),
            reason: "用户缺少写入权限".to_string(),
        };
        assert!(error.to_string().contains("user-001"));
    }

    #[test]
    fn test_device_unavailable_error() {
        let error = ValidatorError::DeviceUnavailable("设备离线".to_string());
        assert_eq!(error.to_string(), "设备不可用: 设备离线");
    }
}
