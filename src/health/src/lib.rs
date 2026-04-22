//! # FOS Health - 本体感觉神经层
//!
//! FOS神经元控制器的本体感觉：健康状态监控与自愈
//!
//! 类比于人体神经系统的本体感觉，感受自身状态并反馈
//!
//! ## 核心职责
//! - 神经通路健康检查（感觉→脊髓→运动连接状态）
//! - 神经末梢健康检查（设备连接状态）
//! - 依赖服务可用性检查（数据库、缓存等）
//! - 健康报告生成
//! - 自愈机制（自动恢复）
//!
//! ## 检查项
//! - 神经通路: Gateway/Validator/Bus连接状态
//! - 神经末梢: 设备连接状态
//! - 反馈回路: 执行结果回传状态
//!
//! ## 安全铁律
//! - 不做规则判断
//! - 不执行业务操作
//! - 不修改系统状态（只报告）

pub mod checker;
pub mod checks;
pub mod config;
pub mod error;
pub mod healing;
pub mod reporter;
pub mod status;

pub use checker::HealthChecker;
pub use checks::{CheckResult, HealthCheck};
pub use config::Config as HealthConfig;
pub use error::{Error, Result};
pub use healing::SelfHealing;
pub use reporter::{GeneratedReport, HealthReporter, ReportFormat};
pub use status::{HealthLevel, HealthStatus};

/// 模块版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 模块名称
pub const NAME: &str = env!("CARGO_PKG_NAME");

use std::collections::HashMap;

/// 健康检查管理器
pub struct HealthManager {
    #[allow(dead_code)]
    config: HealthConfig,
    checker: HealthChecker,
    healing: SelfHealing,
}

impl HealthManager {
    /// 创建新的健康管理器
    pub fn new(config: HealthConfig) -> Self {
        Self { config, checker: HealthChecker::new(), healing: SelfHealing::new() }
    }

    /// 执行全面健康检查
    pub fn check_all(&self) -> Result<HealthReport> {
        self.checker.check_all()
    }

    /// 检查单个模块
    pub fn check_module(&self, name: &str) -> Result<CheckResult> {
        self.checker.check(name)
    }

    /// 尝试自愈
    pub fn heal(&self, issue: &str) -> Result<bool> {
        self.healing.attempt(issue)
    }

    /// 获取当前状态
    pub fn get_status(&self) -> HealthStatus {
        self.checker.overall_status()
    }
}

impl Default for HealthManager {
    fn default() -> Self {
        Self::new(HealthConfig::default())
    }
}

/// 健康报告
#[derive(Debug, Clone)]
pub struct HealthReport {
    /// 整体状态
    pub status: HealthStatus,
    /// 检查结果列表
    pub checks: HashMap<String, CheckResult>,
    /// 生成时间
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl HealthReport {
    /// 创建新报告
    pub fn new() -> Self {
        Self {
            status: HealthStatus::Unknown,
            checks: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// 添加检查结果
    pub fn add_check(&mut self, name: &str, result: CheckResult) {
        self.checks.insert(name.to_string(), result);
        self.update_overall_status();
    }

    /// 更新整体状态
    fn update_overall_status(&mut self) {
        let mut has_unhealthy = false;
        let mut has_degraded = false;

        for result in self.checks.values() {
            match result.level {
                HealthLevel::Unhealthy => has_unhealthy = true,
                HealthLevel::Degraded => has_degraded = true,
                HealthLevel::Healthy => {},
            }
        }

        self.status = if has_unhealthy {
            HealthStatus::Unhealthy
        } else if has_degraded {
            HealthStatus::Degraded
        } else if !self.checks.is_empty() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        };
    }

    /// 检查是否健康
    pub fn is_healthy(&self) -> bool {
        matches!(self.status, HealthStatus::Healthy)
    }
}

impl Default for HealthReport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_manager_creation() {
        let manager = HealthManager::default();
        assert!(manager.check_all().is_ok());
    }

    #[test]
    fn test_health_report() {
        let mut report = HealthReport::new();
        assert!(report.checks.is_empty());

        report.add_check("test", CheckResult::healthy("test", "OK"));
        assert_eq!(report.checks.len(), 1);
    }

    #[test]
    fn test_health_report_status_update() {
        let mut report = HealthReport::new();
        assert_eq!(report.status, HealthStatus::Unknown);

        report.add_check("module1", CheckResult::healthy("module1", "OK"));
        assert_eq!(report.status, HealthStatus::Healthy);

        report.add_check("module2", CheckResult::degraded("module2", "Slow"));
        assert_eq!(report.status, HealthStatus::Degraded);

        report.add_check("module3", CheckResult::unhealthy("module3", "Down"));
        assert_eq!(report.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_health_report_is_healthy() {
        let mut report = HealthReport::new();
        assert!(!report.is_healthy());

        report.add_check("test", CheckResult::healthy("test", "OK"));
        assert!(report.is_healthy());
    }

    #[test]
    fn test_health_manager_check_module() {
        let manager = HealthManager::default();
        let result = manager.check_module("nonexistent");
        assert!(result.is_ok());
        let check_result = result.unwrap();
        assert_eq!(check_result.level, HealthLevel::Unhealthy);
    }

    #[test]
    fn test_health_manager_heal() {
        let manager = HealthManager::default();
        // 没有注册策略时应该返回错误
        let result = manager.heal("test_issue");
        assert!(result.is_err());
    }

    #[test]
    fn test_health_manager_get_status() {
        let manager = HealthManager::default();
        let status = manager.get_status();
        assert_eq!(status, HealthStatus::Unknown);
    }
}
