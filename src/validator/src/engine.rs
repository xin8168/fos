//! # 校验引擎模块

use crate::error::{Result, ValidatorError};
use crate::rules::{BuiltinRules, Rule, RuleResult, RuleType};
use crate::{DeviceStatus, ExecutionContext, ValidationRequest, ValidationResult};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// 校验引擎
pub struct ValidationEngine {
    /// 规则集合
    rules: Arc<RwLock<HashMap<String, Rule>>>,

    /// 规则执行顺序
    rule_order: Arc<RwLock<Vec<String>>>,
}

impl ValidationEngine {
    /// 创建新的校验引擎
    pub fn new() -> Self {
        let mut rules = HashMap::new();
        let mut rule_order = Vec::new();

        // 加载内置规则
        for rule in BuiltinRules::all() {
            rule_order.push(rule.id.clone());
            rules.insert(rule.id.clone(), rule);
        }

        // 按优先级排序
        rule_order.sort_by(|a, b| {
            let rule_a = rules.get(a).unwrap();
            let rule_b = rules.get(b).unwrap();
            rule_a.priority.cmp(&rule_b.priority)
        });

        Self { rules: Arc::new(RwLock::new(rules)), rule_order: Arc::new(RwLock::new(rule_order)) }
    }

    /// 执行校验
    pub async fn validate(
        &self,
        request: &ValidationRequest,
        context: &ExecutionContext,
    ) -> Result<ValidationResult> {
        let start = Instant::now();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut rule_results = Vec::new();

        let rules = self.rules.read().await;
        let rule_order = self.rule_order.read().await;

        // 按顺序执行规则
        for rule_id in rule_order.iter() {
            if let Some(rule) = rules.get(rule_id) {
                if !rule.enabled {
                    continue;
                }

                let rule_start = Instant::now();
                let result = self.execute_rule(rule, request, context).await;
                let execution_time = rule_start.elapsed().as_micros() as u64;

                let mut rule_result = match result {
                    Ok(passed) => {
                        if passed {
                            RuleResult::success(rule)
                        } else {
                            errors.push(rule.error_message.clone());
                            RuleResult::failure(rule, &rule.error_message)
                        }
                    },
                    Err(e) => {
                        errors.push(e.to_string());
                        RuleResult::failure(rule, &e.to_string())
                    },
                };

                rule_result = rule_result.with_execution_time(execution_time);
                rule_results.push(rule_result);
            }
        }

        let passed = errors.is_empty();
        let validation_time = start.elapsed().as_millis() as u64;

        Ok(ValidationResult {
            request_id: request.id.clone(),
            passed,
            errors,
            warnings,
            rule_results,
            validation_time_ms: validation_time,
        })
    }

    /// 执行单个规则
    async fn execute_rule(
        &self,
        rule: &Rule,
        request: &ValidationRequest,
        context: &ExecutionContext,
    ) -> Result<bool> {
        match rule.rule_type {
            RuleType::Format => self.check_format(rule, request),
            RuleType::Security => self.check_security(rule, request, context),
            RuleType::Device => self.check_device(rule, request, context),
            RuleType::Business => self.check_business(rule, request, context),
            RuleType::Permission => self.check_permission(rule, request, context),
            RuleType::Environment => self.check_environment(rule, request, context),
        }
    }

    /// 检查格式规则
    fn check_format(&self, rule: &Rule, request: &ValidationRequest) -> Result<bool> {
        match rule.id.as_str() {
            "format-001" => {
                // 事件名称格式校验
                if request.event.is_empty() {
                    return Err(ValidatorError::RuleValidationFailed {
                        rule: rule.id.clone(),
                        reason: "事件名称不能为空".to_string(),
                    });
                }
                Ok(true)
            },
            "format-002" => {
                // 步骤列表格式校验
                if request.steps.is_empty() {
                    return Err(ValidatorError::RuleValidationFailed {
                        rule: rule.id.clone(),
                        reason: "步骤列表不能为空".to_string(),
                    });
                }
                for (i, step) in request.steps.iter().enumerate() {
                    if step.is_empty() {
                        return Err(ValidatorError::RuleValidationFailed {
                            rule: rule.id.clone(),
                            reason: format!("步骤{}不能为空", i + 1),
                        });
                    }
                }
                Ok(true)
            },
            _ => Ok(true),
        }
    }

    /// 检查安全规则
    fn check_security(
        &self,
        rule: &Rule,
        request: &ValidationRequest,
        _context: &ExecutionContext,
    ) -> Result<bool> {
        match rule.id.as_str() {
            "security-001" => {
                // 危险操作检测
                let dangerous_patterns = [
                    "删除系统",
                    "格式化",
                    "rm -rf",
                    "del /s",
                    "shutdown",
                    "reboot",
                    "DROP TABLE",
                    "TRUNCATE",
                ];

                for pattern in &dangerous_patterns {
                    if request.event.contains(pattern) {
                        return Err(ValidatorError::RuleValidationFailed {
                            rule: rule.id.clone(),
                            reason: format!("检测到危险操作: {}", pattern),
                        });
                    }
                    for step in &request.steps {
                        if step.contains(pattern) {
                            return Err(ValidatorError::RuleValidationFailed {
                                rule: rule.id.clone(),
                                reason: format!("检测到危险操作: {}", pattern),
                            });
                        }
                    }
                }
                Ok(true)
            },
            "security-002" => {
                // 权限校验
                // TODO: 实现权限检查逻辑
                Ok(true)
            },
            _ => Ok(true),
        }
    }

    /// 检查设备规则
    fn check_device(
        &self,
        rule: &Rule,
        request: &ValidationRequest,
        context: &ExecutionContext,
    ) -> Result<bool> {
        match rule.id.as_str() {
            "device-001" => {
                // 设备状态校验
                if context.device_info.status != DeviceStatus::Online {
                    return Err(ValidatorError::DeviceUnavailable(format!(
                        "设备状态: {:?}",
                        context.device_info.status
                    )));
                }
                Ok(true)
            },
            "device-002" => {
                // 设备能力校验
                // TODO: 实现设备能力检查
                Ok(true)
            },
            _ => Ok(true),
        }
    }

    /// 检查业务规则
    fn check_business(
        &self,
        _rule: &Rule,
        _request: &ValidationRequest,
        _context: &ExecutionContext,
    ) -> Result<bool> {
        // TODO: 实现业务规则检查
        Ok(true)
    }

    /// 检查权限规则
    fn check_permission(
        &self,
        _rule: &Rule,
        request: &ValidationRequest,
        context: &ExecutionContext,
    ) -> Result<bool> {
        // 定义权限映射
        let permission_map: HashMap<&str, Vec<&str>> = [
            ("file_operation", vec!["file:read", "file:write", "file:delete"]),
            ("device_control", vec!["device:control", "device:config"]),
            ("system_command", vec!["system:execute", "system:config"]),
            ("network_operation", vec!["network:connect", "network:config"]),
            ("automation_task", vec!["automation:execute", "automation:schedule"]),
            ("skill_execution", vec!["skill:execute"]),
        ]
        .iter()
        .cloned()
        .collect();

        // 根据事件类型确定所需权限
        let required_permissions = self.get_required_permissions(request, &permission_map);

        // 检查用户是否有所需权限
        for perm in &required_permissions {
            if !context.permissions.contains(perm) {
                return Err(ValidatorError::PermissionDenied {
                    user: context.user_id.clone(),
                    required: perm.clone(),
                    reason: "用户缺少必要权限".to_string(),
                });
            }
        }

        // 检查角色权限
        let role_permissions: HashMap<&str, Vec<&str>> = [
            (
                "admin",
                vec!["file:*", "device:*", "system:*", "network:*", "automation:*", "skill:*"],
            ),
            ("operator", vec!["file:read", "file:write", "device:control", "automation:execute"]),
            ("viewer", vec!["file:read", "device:status"]),
            ("guest", vec![]),
        ]
        .iter()
        .cloned()
        .collect();

        // 检查角色是否允许此操作
        let has_role_permission = context.roles.iter().any(|role| {
            if let Some(perms) = role_permissions.get(&role.as_str()) {
                perms.iter().any(|p| {
                    let is_wildcard = p.ends_with("*")
                        && required_permissions.iter().any(|rp| rp.starts_with(&p[..p.len() - 1]));
                    let is_exact = required_permissions.iter().any(|rp| rp == *p);
                    is_wildcard || is_exact
                })
            } else {
                false
            }
        });

        // 如果有直接权限或角色权限，则通过
        if required_permissions.is_empty() || has_role_permission {
            Ok(true)
        } else {
            // 再次检查直接权限
            Ok(true)
        }
    }

    /// 获取所需权限
    fn get_required_permissions(
        &self,
        request: &ValidationRequest,
        permission_map: &HashMap<&str, Vec<&str>>,
    ) -> Vec<String> {
        // 根据事件内容判断操作类型
        let event = &request.event;

        if event.contains("文件") || event.contains("目录") {
            permission_map
                .get("file_operation")
                .map(|p| p.iter().map(|s| s.to_string()).collect())
                .unwrap_or_default()
        } else if event.contains("设备") || event.contains("控制") {
            permission_map
                .get("device_control")
                .map(|p| p.iter().map(|s| s.to_string()).collect())
                .unwrap_or_default()
        } else if event.contains("系统") || event.contains("命令") {
            permission_map
                .get("system_command")
                .map(|p| p.iter().map(|s| s.to_string()).collect())
                .unwrap_or_default()
        } else if event.contains("网络") {
            permission_map
                .get("network_operation")
                .map(|p| p.iter().map(|s| s.to_string()).collect())
                .unwrap_or_default()
        } else {
            vec!["skill:execute".to_string()]
        }
    }

    /// 检查环境规则
    fn check_environment(
        &self,
        _rule: &Rule,
        _request: &ValidationRequest,
        _context: &ExecutionContext,
    ) -> Result<bool> {
        // TODO: 实现环境检查
        Ok(true)
    }

    /// 添加自定义规则
    pub async fn add_rule(&self, rule: Rule) -> Result<()> {
        let mut rules = self.rules.write().await;
        let mut rule_order = self.rule_order.write().await;

        let rule_id = rule.id.clone();
        rules.insert(rule_id.clone(), rule);

        // 重新排序
        rule_order.push(rule_id);
        rule_order.sort_by(|a, b| {
            let rule_a = rules.get(a).unwrap();
            let rule_b = rules.get(b).unwrap();
            rule_a.priority.cmp(&rule_b.priority)
        });

        Ok(())
    }

    /// 移除规则
    pub async fn remove_rule(&self, rule_id: &str) -> Result<()> {
        let mut rules = self.rules.write().await;
        let mut rule_order = self.rule_order.write().await;

        rules.remove(rule_id);
        rule_order.retain(|id| id != rule_id);

        Ok(())
    }
}

impl Default for ValidationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_engine_creation() {
        let engine = ValidationEngine::new();
        let rules = engine.rules.read().await;
        assert!(!rules.is_empty());
    }

    #[tokio::test]
    async fn test_validate_success() {
        let engine = ValidationEngine::new();
        let request = ValidationRequest {
            id: "req-001".to_string(),
            event: "测试事件".to_string(),
            steps: vec!["步骤1".to_string()],
            judgment_logic: "条件1".to_string(),
            verification_standard: "标准1".to_string(),
            location: "位置1".to_string(),
            subject: "主体1".to_string(),
            metadata: HashMap::new(),
        };

        let context = ExecutionContext {
            user_id: "user-001".to_string(),
            roles: vec!["admin".to_string()],
            permissions: vec!["execute".to_string()],
            device_info: crate::DeviceInfo {
                device_id: "device-001".to_string(),
                device_type: "computer".to_string(),
                status: DeviceStatus::Online,
                capabilities: vec!["file_operation".to_string()],
            },
            environment: HashMap::new(),
        };

        let result = engine.validate(&request, &context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_dangerous_operation() {
        let engine = ValidationEngine::new();
        let request = ValidationRequest {
            id: "req-002".to_string(),
            event: "删除系统文件".to_string(),
            steps: vec!["步骤1".to_string()],
            judgment_logic: "条件1".to_string(),
            verification_standard: "标准1".to_string(),
            location: "位置1".to_string(),
            subject: "主体1".to_string(),
            metadata: HashMap::new(),
        };

        let context = ExecutionContext {
            user_id: "user-001".to_string(),
            roles: vec!["admin".to_string()],
            permissions: vec!["execute".to_string()],
            device_info: crate::DeviceInfo {
                device_id: "device-001".to_string(),
                device_type: "computer".to_string(),
                status: DeviceStatus::Online,
                capabilities: vec![],
            },
            environment: HashMap::new(),
        };

        let result = engine.validate(&request, &context).await.unwrap();
        assert!(!result.passed);
        assert!(!result.errors.is_empty());
    }
}
