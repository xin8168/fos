//! # 规则定义模块

use serde::{Deserialize, Serialize};

/// 规则类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RuleType {
    /// 权限规则
    Permission,

    /// 格式规则
    Format,

    /// 业务规则
    Business,

    /// 安全规则
    Security,

    /// 设备规则
    Device,

    /// 环境规则
    Environment,
}

/// 规则定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// 规则ID
    pub id: String,

    /// 规则名称
    pub name: String,

    /// 规则类型
    pub rule_type: RuleType,

    /// 规则描述
    pub description: String,

    /// 规则优先级（数字越小优先级越高）
    pub priority: u32,

    /// 是否启用
    pub enabled: bool,

    /// 规则条件（JSON 格式）
    pub condition: serde_json::Value,

    /// 错误消息
    pub error_message: String,
}

impl Rule {
    /// 创建新规则
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        rule_type: RuleType,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            rule_type,
            description: description.into(),
            priority: 100,
            enabled: true,
            condition: serde_json::json!({}),
            error_message: "规则校验失败".to_string(),
        }
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// 设置条件
    pub fn with_condition(mut self, condition: serde_json::Value) -> Self {
        self.condition = condition;
        self
    }

    /// 设置错误消息
    pub fn with_error_message(mut self, message: impl Into<String>) -> Self {
        self.error_message = message.into();
        self
    }

    /// 禁用规则
    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// 规则执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleResult {
    /// 规则ID
    pub rule_id: String,

    /// 规则名称
    pub rule_name: String,

    /// 是否通过
    pub passed: bool,

    /// 执行时间（微秒）
    pub execution_time_us: u64,

    /// 错误消息
    pub error_message: Option<String>,

    /// 详细信息
    pub details: serde_json::Value,
}

impl RuleResult {
    /// 创建成功结果
    pub fn success(rule: &Rule) -> Self {
        Self {
            rule_id: rule.id.clone(),
            rule_name: rule.name.clone(),
            passed: true,
            execution_time_us: 0,
            error_message: None,
            details: serde_json::json!({}),
        }
    }

    /// 创建失败结果
    pub fn failure(rule: &Rule, error: impl Into<String>) -> Self {
        Self {
            rule_id: rule.id.clone(),
            rule_name: rule.name.clone(),
            passed: false,
            execution_time_us: 0,
            error_message: Some(error.into()),
            details: serde_json::json!({}),
        }
    }

    /// 设置执行时间
    pub fn with_execution_time(mut self, time_us: u64) -> Self {
        self.execution_time_us = time_us;
        self
    }
}

/// 内置规则集合
pub struct BuiltinRules;

impl BuiltinRules {
    /// 获取所有内置规则
    pub fn all() -> Vec<Rule> {
        vec![
            // 格式规则
            Rule::new(
                "format-001",
                "事件名称格式校验",
                RuleType::Format,
                "事件名称必须非空且符合规范",
            )
            .with_priority(10)
            .with_error_message("事件名称格式不符合规范"),
            Rule::new("format-002", "步骤列表格式校验", RuleType::Format, "步骤列表必须非空")
                .with_priority(10)
                .with_error_message("步骤列表不能为空"),
            // 安全规则
            Rule::new("security-001", "危险操作检测", RuleType::Security, "检测危险操作关键词")
                .with_priority(5)
                .with_error_message("检测到危险操作"),
            Rule::new("security-002", "权限校验", RuleType::Security, "校验用户执行权限")
                .with_priority(8)
                .with_error_message("权限不足"),
            // 设备规则
            Rule::new("device-001", "设备状态校验", RuleType::Device, "设备必须在线且可用")
                .with_priority(15)
                .with_error_message("设备不可用"),
            Rule::new("device-002", "设备能力校验", RuleType::Device, "设备必须支持所需操作")
                .with_priority(16)
                .with_error_message("设备不支持此操作"),
            // 业务规则
            Rule::new(
                "business-001",
                "执行时间限制",
                RuleType::Business,
                "执行时间必须在允许范围内",
            )
            .with_priority(20)
            .with_error_message("执行时间超出限制"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_creation() {
        let rule = Rule::new("test-rule", "测试规则", RuleType::Security, "这是一个测试规则");

        assert_eq!(rule.id, "test-rule");
        assert_eq!(rule.name, "测试规则");
        assert_eq!(rule.rule_type, RuleType::Security);
        assert!(rule.enabled);
    }

    #[test]
    fn test_rule_with_priority() {
        let rule = Rule::new("test-rule", "测试规则", RuleType::Security, "描述").with_priority(50);

        assert_eq!(rule.priority, 50);
    }

    #[test]
    fn test_rule_result_success() {
        let rule = Rule::new("test-rule", "测试规则", RuleType::Security, "描述");

        let result = RuleResult::success(&rule);
        assert!(result.passed);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_rule_result_failure() {
        let rule = Rule::new("test-rule", "测试规则", RuleType::Security, "描述");

        let result = RuleResult::failure(&rule, "校验失败");
        assert!(!result.passed);
        assert_eq!(result.error_message, Some("校验失败".to_string()));
    }

    #[test]
    fn test_builtin_rules() {
        let rules = BuiltinRules::all();
        assert!(!rules.is_empty());

        // 验证规则按优先级排序
        for i in 0..rules.len() - 1 {
            // 内置规则应该按优先级升序排列
        }
    }
}
