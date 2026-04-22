//! FOS Skills - 标准化执行单元模块
//!
//! 存储和管理成功执行的操作技能

pub mod adapter;
pub mod definition;
pub mod error;
pub mod loader;
pub mod registry;
pub mod skill;
pub mod version;

pub use adapter::{
    AdaptationRule, DeviceAdapter, DeviceCapability, DeviceConfig, DeviceId, DeviceRegistry,
    DeviceType,
};
pub use definition::{
    DeviceConstraint, EnhancedSkillDefinition, ExecutionStep, OutputDefinition, ParamDefinition,
    ParamType, SkillDependency, SkillId, SkillVersion, ValidationRules,
};
pub use error::{Result, SkillsError};
pub use registry::SkillRegistry;
pub use skill::Skill;
pub use version::{VersionLock, VersionManager};

use serde::{Deserialize, Serialize};

/// 技能状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SkillStatus {
    /// 草稿
    Draft,
    /// 已验证
    Verified,
    /// 已发布
    Published,
    /// 已弃用
    Deprecated,
}

/// 技能定义（兼容旧版）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    /// 技能ID
    pub id: String,

    /// 技能名称
    pub name: String,

    /// 技能描述
    pub description: String,

    /// 版本
    pub version: String,

    /// 执行步骤
    pub steps: Vec<String>,

    /// 判断逻辑
    pub judgment_logic: String,

    /// 校验标准
    pub verification_standard: String,

    /// 适配设备
    pub supported_devices: Vec<String>,

    /// 状态
    pub status: SkillStatus,

    /// 元数据
    pub metadata: serde_json::Value,
}

impl Default for SkillDefinition {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            description: String::new(),
            version: "1.0.0".to_string(),
            steps: vec![],
            judgment_logic: String::new(),
            verification_standard: String::new(),
            supported_devices: vec!["*".to_string()],
            status: SkillStatus::Draft,
            metadata: serde_json::json!({}),
        }
    }
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_definition_default() {
        let skill = SkillDefinition::default();
        assert_eq!(skill.version, "1.0.0");
        assert_eq!(skill.status, SkillStatus::Draft);
    }
}
