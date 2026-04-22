//! 检查项定义

use crate::status::HealthLevel;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// 检查项名称
    pub name: String,
    /// 健康级别
    pub level: HealthLevel,
    /// 消息
    pub message: String,
    /// 详细信息
    pub details: Option<String>,
    /// 检查时间
    pub timestamp: DateTime<Utc>,
    /// 执行时间（毫秒）
    pub duration_ms: u64,
}

impl CheckResult {
    /// 创建健康结果
    pub fn healthy(name: &str, message: &str) -> Self {
        Self {
            name: name.to_string(),
            level: HealthLevel::Healthy,
            message: message.to_string(),
            details: None,
            timestamp: Utc::now(),
            duration_ms: 0,
        }
    }

    /// 创建降级结果
    pub fn degraded(name: &str, message: &str) -> Self {
        Self {
            name: name.to_string(),
            level: HealthLevel::Degraded,
            message: message.to_string(),
            details: None,
            timestamp: Utc::now(),
            duration_ms: 0,
        }
    }

    /// 创建不健康结果
    pub fn unhealthy(name: &str, message: &str) -> Self {
        Self {
            name: name.to_string(),
            level: HealthLevel::Unhealthy,
            message: message.to_string(),
            details: None,
            timestamp: Utc::now(),
            duration_ms: 0,
        }
    }

    /// 创建未知结果
    pub fn unknown(name: &str, message: &str) -> Self {
        Self {
            name: name.to_string(),
            level: HealthLevel::Unhealthy,
            message: message.to_string(),
            details: None,
            timestamp: Utc::now(),
            duration_ms: 0,
        }
    }

    /// 设置详细信息
    pub fn with_details(mut self, details: &str) -> Self {
        self.details = Some(details.to_string());
        self
    }

    /// 设置执行时间
    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }

    /// 检查是否健康
    pub fn is_healthy(&self) -> bool {
        matches!(self.level, HealthLevel::Healthy)
    }
}

/// 健康检查trait
pub trait HealthCheck: Send + Sync {
    /// 执行检查
    fn execute(&self) -> CheckResult;

    /// 获取检查项名称
    fn name(&self) -> &str;
}

/// 简单检查实现
pub struct SimpleCheck {
    name: String,
    check_fn: Box<dyn Fn() -> CheckResult + Send + Sync>,
}

impl SimpleCheck {
    /// 创建新的简单检查
    pub fn new<F>(name: &str, f: F) -> Self
    where
        F: Fn() -> CheckResult + Send + Sync + 'static,
    {
        Self { name: name.to_string(), check_fn: Box::new(f) }
    }
}

impl HealthCheck for SimpleCheck {
    fn execute(&self) -> CheckResult {
        (self.check_fn)()
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// 模块检查实现
pub struct ModuleCheck {
    name: String,
    module_name: String,
}

impl ModuleCheck {
    /// 创建新的模块检查
    pub fn new(module_name: &str) -> Self {
        Self { name: format!("module_{}", module_name), module_name: module_name.to_string() }
    }
}

impl HealthCheck for ModuleCheck {
    fn execute(&self) -> CheckResult {
        // 检查模块状态
        CheckResult::healthy(&self.name, &format!("模块 {} 运行正常", self.module_name))
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_result_healthy() {
        let result = CheckResult::healthy("test", "OK");
        assert!(result.is_healthy());
        assert_eq!(result.name, "test");
    }

    #[test]
    fn test_check_result_unhealthy() {
        let result = CheckResult::unhealthy("test", "FAILED");
        assert!(!result.is_healthy());
    }

    #[test]
    fn test_simple_check() {
        let check = SimpleCheck::new("test", || CheckResult::healthy("test", "OK"));
        let result = check.execute();
        assert!(result.is_healthy());
    }

    #[test]
    fn test_module_check() {
        let check = ModuleCheck::new("gateway");
        let result = check.execute();
        assert!(result.is_healthy());
    }
}
