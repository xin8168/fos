//! 健康检查器

use crate::checks::{CheckResult, HealthCheck};
use crate::error::Result;
use crate::status::{HealthLevel, HealthStatus};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// 健康检查器
pub struct HealthChecker {
    /// 已注册的检查项
    checks: std::sync::Mutex<HashMap<String, Box<dyn HealthCheck>>>,
    /// 检查计数
    check_count: AtomicU64,
    /// 失败计数
    failure_count: AtomicU64,
}

impl HealthChecker {
    /// 创建新的健康检查器
    pub fn new() -> Self {
        Self {
            checks: std::sync::Mutex::new(HashMap::new()),
            check_count: AtomicU64::new(0),
            failure_count: AtomicU64::new(0),
        }
    }

    /// 注册检查项
    pub fn register<C: HealthCheck + 'static>(&self, name: &str, check: C) {
        self.checks.lock().unwrap().insert(name.to_string(), Box::new(check));
    }

    /// 执行单个检查
    pub fn check(&self, name: &str) -> Result<CheckResult> {
        self.check_count.fetch_add(1, Ordering::SeqCst);

        let checks = self.checks.lock().unwrap();
        if let Some(check) = checks.get(name) {
            let result = check.execute();
            if !result.is_healthy() {
                self.failure_count.fetch_add(1, Ordering::SeqCst);
            }
            Ok(result)
        } else {
            Ok(CheckResult::unknown(name, "检查项不存在"))
        }
    }

    /// 执行所有检查
    pub fn check_all(&self) -> Result<crate::HealthReport> {
        let mut report = crate::HealthReport::new();

        let checks = self.checks.lock().unwrap();
        for (name, check) in checks.iter() {
            let result = check.execute();
            report.add_check(name, result.clone());
        }

        Ok(report)
    }

    /// 获取整体状态
    pub fn overall_status(&self) -> HealthStatus {
        let checks = self.checks.lock().unwrap();
        let mut has_unhealthy = false;
        let mut has_degraded = false;

        for check in checks.values() {
            match check.execute().level {
                HealthLevel::Unhealthy => has_unhealthy = true,
                HealthLevel::Degraded => has_degraded = true,
                HealthLevel::Healthy => {},
            }
        }

        if has_unhealthy {
            HealthStatus::Unhealthy
        } else if has_degraded {
            HealthStatus::Degraded
        } else if !checks.is_empty() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        }
    }

    /// 获取检查次数
    pub fn check_count(&self) -> u64 {
        self.check_count.load(Ordering::SeqCst)
    }

    /// 获取失败次数
    pub fn failure_count(&self) -> u64 {
        self.failure_count.load(Ordering::SeqCst)
    }

    /// 清除所有检查项
    pub fn clear(&self) {
        self.checks.lock().unwrap().clear();
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checker_creation() {
        let checker = HealthChecker::new();
        assert_eq!(checker.check_count(), 0);
    }

    #[test]
    fn test_overall_status_empty() {
        let checker = HealthChecker::new();
        assert_eq!(checker.overall_status(), HealthStatus::Unknown);
    }
}
