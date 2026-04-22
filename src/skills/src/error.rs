//! Skills 错误处理

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SkillsError {
    #[error("技能不存在: {0}")]
    NotFound(String),

    #[error("技能验证失败: {0}")]
    VerificationFailed(String),

    #[error("参数验证失败: {0}")]
    ValidationFailed(String),

    #[error("技能执行失败: {0}")]
    ExecutionFailed(String),

    #[error("技能版本冲突: {0}")]
    VersionConflict(String),

    #[error("无效版本号: {0}")]
    InvalidVersion(String),

    #[error("设备不兼容: {0}")]
    DeviceIncompatible(String),

    #[error("内部错误: {0}")]
    Internal(String),

    #[error("YAML 解析错误: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("JSON 解析错误: {0}")]
    JsonParse(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, SkillsError>;
