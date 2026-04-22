//! 技能定义增强模块
//!
//! 提供完整的技能定义、参数、输入输出和执行约束

use crate::error::{Result, SkillsError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 技能ID
pub type SkillId = String;

/// 技能版本
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SkillVersion {
    /// 主版本号
    pub major: u32,
    /// 次版本号
    pub minor: u32,
    /// 修订号
    pub patch: u32,
}

impl SkillVersion {
    /// 创建新版本
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    /// 解析版本字符串
    pub fn parse(version: &str) -> Result<Self> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return Err(SkillsError::InvalidVersion(version.to_string()));
        }

        let major =
            parts[0].parse().map_err(|_| SkillsError::InvalidVersion(version.to_string()))?;
        let minor =
            parts[1].parse().map_err(|_| SkillsError::InvalidVersion(version.to_string()))?;
        let patch =
            parts[2].parse().map_err(|_| SkillsError::InvalidVersion(version.to_string()))?;

        Ok(Self { major, minor, patch })
    }

    /// 转换为字符串
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }

    /// 检查是否兼容
    pub fn is_compatible(&self, other: &SkillVersion) -> bool {
        self.major == other.major && self.minor >= other.minor
    }
}

impl Default for SkillVersion {
    fn default() -> Self {
        Self::new(1, 0, 0)
    }
}

impl std::fmt::Display for SkillVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// 参数类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ParamType {
    /// 字符串
    String,
    /// 整数
    Integer,
    /// 浮点数
    Float,
    /// 布尔值
    Boolean,
    /// 数组
    Array,
    /// 对象
    Object,
    /// 枚举
    Enum(Vec<String>),
}

/// 参数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDefinition {
    /// 参数名称
    pub name: String,
    /// 参数类型
    pub param_type: ParamType,
    /// 是否必需
    pub required: bool,
    /// 默认值
    pub default_value: Option<serde_json::Value>,
    /// 描述
    pub description: String,
    /// 验证规则
    pub validation: Option<ValidationRules>,
}

impl ParamDefinition {
    /// 创建新参数定义
    pub fn new(name: String, param_type: ParamType, required: bool) -> Self {
        Self {
            name,
            param_type,
            required,
            default_value: None,
            description: String::new(),
            validation: None,
        }
    }

    /// 设置默认值
    pub fn with_default(mut self, value: serde_json::Value) -> Self {
        self.default_value = Some(value);
        self
    }

    /// 设置描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    /// 设置验证规则
    pub fn with_validation(mut self, rules: ValidationRules) -> Self {
        self.validation = Some(rules);
        self
    }

    /// 验证参数值
    pub fn validate(&self, value: &serde_json::Value) -> Result<()> {
        if let Some(ref rules) = self.validation {
            rules.validate(value)?;
        }
        Ok(())
    }
}

/// 验证规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    /// 最小值（数字）
    pub min: Option<f64>,
    /// 最大值（数字）
    pub max: Option<f64>,
    /// 最小长度（字符串/数组）
    pub min_length: Option<usize>,
    /// 最大长度（字符串/数组）
    pub max_length: Option<usize>,
    /// 正则表达式（字符串）
    pub pattern: Option<String>,
    /// 枚举值列表
    pub enum_values: Option<Vec<String>>,
}

impl ValidationRules {
    /// 创建新验证规则
    pub fn new() -> Self {
        Self {
            min: None,
            max: None,
            min_length: None,
            max_length: None,
            pattern: None,
            enum_values: None,
        }
    }

    /// 设置范围
    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.min = Some(min);
        self.max = Some(max);
        self
    }

    /// 设置长度范围
    pub fn with_length(mut self, min: usize, max: usize) -> Self {
        self.min_length = Some(min);
        self.max_length = Some(max);
        self
    }

    /// 设置正则表达式
    pub fn with_pattern(mut self, pattern: String) -> Self {
        self.pattern = Some(pattern);
        self
    }

    /// 验证值
    pub fn validate(&self, value: &serde_json::Value) -> Result<()> {
        // 检查数值范围
        if let (Some(min), Some(max)) = (self.min, self.max) {
            if let Some(num) = value.as_f64() {
                if num < min || num > max {
                    return Err(SkillsError::ValidationFailed(format!(
                        "值 {} 不在范围 [{}, {}] 内",
                        num, min, max
                    )));
                }
            }
        }

        // 检查字符串长度
        if let (Some(min_len), Some(max_len)) = (self.min_length, self.max_length) {
            if let Some(s) = value.as_str() {
                let len = s.len();
                if len < min_len || len > max_len {
                    return Err(SkillsError::ValidationFailed(format!(
                        "字符串长度 {} 不在范围 [{}, {}] 内",
                        len, min_len, max_len
                    )));
                }
            }
        }

        // 检查枚举值
        if let Some(ref enum_values) = self.enum_values {
            if let Some(s) = value.as_str() {
                if !enum_values.contains(&s.to_string()) {
                    return Err(SkillsError::ValidationFailed(format!(
                        "值 '{}' 不在允许的枚举值列表中",
                        s
                    )));
                }
            }
        }

        Ok(())
    }
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self::new()
    }
}

/// 执行步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    /// 步骤ID
    pub id: String,
    /// 步骤名称
    pub name: String,
    /// 步骤描述
    pub description: String,
    /// 执行命令
    pub command: String,
    /// 是否必需
    pub required: bool,
    /// 超时时间（秒）
    pub timeout_secs: Option<u64>,
    /// 重试次数
    pub retry_count: Option<u32>,
    /// 回滚命令
    pub rollback_command: Option<String>,
}

impl ExecutionStep {
    /// 创建新步骤
    pub fn new(id: String, name: String, command: String) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            command,
            required: true,
            timeout_secs: None,
            retry_count: None,
            rollback_command: None,
        }
    }

    /// 设置超时
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }

    /// 设置重试次数
    pub fn with_retry(mut self, count: u32) -> Self {
        self.retry_count = Some(count);
        self
    }

    /// 设置回滚命令
    pub fn with_rollback(mut self, command: String) -> Self {
        self.rollback_command = Some(command);
        self
    }
}

/// 输出定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputDefinition {
    /// 输出名称
    pub name: String,
    /// 输出类型
    pub output_type: ParamType,
    /// 描述
    pub description: String,
}

impl OutputDefinition {
    /// 创建新输出定义
    pub fn new(name: String, output_type: ParamType) -> Self {
        Self { name, output_type, description: String::new() }
    }
}

/// 设备约束
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConstraint {
    /// 设备类型
    pub device_type: String,
    /// 最低版本
    pub min_version: Option<String>,
    /// 最高版本
    pub max_version: Option<String>,
    /// 必需功能
    pub required_features: Vec<String>,
    /// 参数覆盖
    pub param_overrides: HashMap<String, serde_json::Value>,
}

impl DeviceConstraint {
    /// 创建新设备约束
    pub fn new(device_type: String) -> Self {
        Self {
            device_type,
            min_version: None,
            max_version: None,
            required_features: Vec::new(),
            param_overrides: HashMap::new(),
        }
    }

    /// 检查设备是否满足约束
    pub fn is_satisfied(&self, device_type: &str, version: &str, features: &[String]) -> bool {
        if self.device_type != "*" && self.device_type != device_type {
            return false;
        }

        if let Some(ref min_ver) = self.min_version {
            if version < min_ver.as_str() {
                return false;
            }
        }

        if let Some(ref max_ver) = self.max_version {
            if version > max_ver.as_str() {
                return false;
            }
        }

        for feature in &self.required_features {
            if !features.contains(feature) {
                return false;
            }
        }

        true
    }
}

/// 技能依赖
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDependency {
    /// 依赖技能ID
    pub skill_id: String,
    /// 最低版本
    pub min_version: String,
    /// 是否必需
    pub required: bool,
}

impl SkillDependency {
    /// 创建新依赖
    pub fn new(skill_id: String, min_version: String) -> Self {
        Self { skill_id, min_version, required: true }
    }
}

/// 增强的技能定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSkillDefinition {
    /// 技能ID
    pub id: SkillId,
    /// 技能名称
    pub name: String,
    /// 技能描述
    pub description: String,
    /// 版本
    pub version: SkillVersion,
    /// 分类
    pub category: String,
    /// 标签
    pub tags: Vec<String>,
    /// 输入参数
    pub inputs: Vec<ParamDefinition>,
    /// 输出定义
    pub outputs: Vec<OutputDefinition>,
    /// 执行步骤
    pub steps: Vec<ExecutionStep>,
    /// 设备约束
    pub device_constraints: Vec<DeviceConstraint>,
    /// 技能依赖
    pub dependencies: Vec<SkillDependency>,
    /// 状态
    pub status: crate::SkillStatus,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 作者
    pub author: String,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl EnhancedSkillDefinition {
    /// 创建新技能定义
    pub fn new(name: String, category: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: String::new(),
            version: SkillVersion::default(),
            category,
            tags: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            steps: Vec::new(),
            device_constraints: vec![DeviceConstraint::new("*".to_string())],
            dependencies: Vec::new(),
            status: crate::SkillStatus::Draft,
            created_at: now,
            updated_at: now,
            author: String::new(),
            metadata: HashMap::new(),
        }
    }

    /// 添加输入参数
    pub fn add_input(&mut self, input: ParamDefinition) {
        self.inputs.push(input);
        self.updated_at = Utc::now();
    }

    /// 添加输出定义
    pub fn add_output(&mut self, output: OutputDefinition) {
        self.outputs.push(output);
        self.updated_at = Utc::now();
    }

    /// 添加执行步骤
    pub fn add_step(&mut self, step: ExecutionStep) {
        self.steps.push(step);
        self.updated_at = Utc::now();
    }

    /// 添加设备约束
    pub fn add_device_constraint(&mut self, constraint: DeviceConstraint) {
        self.device_constraints.push(constraint);
        self.updated_at = Utc::now();
    }

    /// 检查设备兼容性
    pub fn is_compatible_with(
        &self,
        device_type: &str,
        version: &str,
        features: &[String],
    ) -> bool {
        self.device_constraints.iter().any(|c| c.is_satisfied(device_type, version, features))
    }

    /// 验证输入参数
    pub fn validate_inputs(&self, params: &HashMap<String, serde_json::Value>) -> Result<()> {
        for input in &self.inputs {
            if input.required {
                if !params.contains_key(&input.name) && input.default_value.is_none() {
                    return Err(SkillsError::ValidationFailed(format!(
                        "缺少必需参数: {}",
                        input.name
                    )));
                }
            }

            if let Some(value) = params.get(&input.name) {
                input.validate(value)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_version() {
        let v1 = SkillVersion::new(1, 2, 3);
        assert_eq!(v1.to_string(), "1.2.3");

        let v2 = SkillVersion::parse("2.0.0").unwrap();
        assert_eq!(v2.major, 2);

        assert!(v1.is_compatible(&SkillVersion::new(1, 2, 3)));
        assert!(!v1.is_compatible(&SkillVersion::new(2, 0, 0)));
    }

    #[test]
    fn test_param_definition() {
        let param = ParamDefinition::new("count".to_string(), ParamType::Integer, true)
            .with_description("计数".to_string());

        assert_eq!(param.name, "count");
        assert!(param.required);
    }

    #[test]
    fn test_validation_rules() {
        let rules = ValidationRules::new().with_range(0.0, 100.0);

        assert!(rules.validate(&serde_json::json!(50)).is_ok());
        assert!(rules.validate(&serde_json::json!(150)).is_err());
    }

    #[test]
    fn test_execution_step() {
        let step = ExecutionStep::new(
            "step1".to_string(),
            "检查状态".to_string(),
            "check_status".to_string(),
        )
        .with_timeout(30)
        .with_retry(3);

        assert_eq!(step.id, "step1");
        assert_eq!(step.timeout_secs, Some(30));
        assert_eq!(step.retry_count, Some(3));
    }

    #[test]
    fn test_device_constraint() {
        let constraint = DeviceConstraint::new("sensor".to_string());
        assert!(constraint.is_satisfied("sensor", "1.0", &[]));
        assert!(!constraint.is_satisfied("actuator", "1.0", &[]));
    }

    #[test]
    fn test_enhanced_skill_definition() {
        let mut skill = EnhancedSkillDefinition::new("温度读取".to_string(), "传感器".to_string());
        skill.add_input(
            ParamDefinition::new("interval".to_string(), ParamType::Integer, false)
                .with_default(serde_json::json!(1000)),
        );
        skill.add_step(ExecutionStep::new(
            "read".to_string(),
            "读取温度".to_string(),
            "read_temperature".to_string(),
        ));

        assert!(!skill.id.is_empty());
        assert_eq!(skill.inputs.len(), 1);
        assert_eq!(skill.steps.len(), 1);
    }

    #[test]
    fn test_validate_inputs() {
        let mut skill = EnhancedSkillDefinition::new("测试技能".to_string(), "测试".to_string());
        skill.add_input(ParamDefinition::new(
            "required_param".to_string(),
            ParamType::String,
            true,
        ));

        let mut params = HashMap::new();
        params.insert("required_param".to_string(), serde_json::json!("value"));

        assert!(skill.validate_inputs(&params).is_ok());

        let empty_params = HashMap::new();
        assert!(skill.validate_inputs(&empty_params).is_err());
    }
}
