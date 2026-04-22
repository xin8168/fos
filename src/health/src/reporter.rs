//! 健康报告生成器

use crate::checks::CheckResult;
use crate::status::{HealthLevel, HealthStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 报告格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReportFormat {
    /// JSON格式
    #[default]
    Json,
    /// 文本格式
    Text,
    /// Prometheus格式
    Prometheus,
}

/// 健康报告生成器
pub struct HealthReporter {
    /// 报告格式
    format: ReportFormat,
    /// 是否包含详细信息
    include_details: bool,
}

impl HealthReporter {
    /// 创建新的报告生成器
    pub fn new() -> Self {
        Self { format: ReportFormat::Json, include_details: true }
    }

    /// 设置报告格式
    pub fn with_format(mut self, format: ReportFormat) -> Self {
        self.format = format;
        self
    }

    /// 设置是否包含详细信息
    pub fn with_details(mut self, include: bool) -> Self {
        self.include_details = include;
        self
    }

    /// 生成报告
    pub fn generate(&self, checks: &HashMap<String, CheckResult>) -> GeneratedReport {
        let timestamp = Utc::now();
        let overall_status = Self::calculate_overall_status(checks);

        GeneratedReport {
            status: overall_status,
            timestamp,
            checks: checks.clone(),
            format: self.format,
        }
    }

    /// 生成JSON格式报告
    pub fn to_json(&self, checks: &HashMap<String, CheckResult>) -> String {
        let report = self.generate(checks);
        serde_json::to_string_pretty(&report).unwrap_or_default()
    }

    /// 生成文本格式报告
    pub fn to_text(&self, checks: &HashMap<String, CheckResult>) -> String {
        let mut output = String::new();
        let overall = Self::calculate_overall_status(checks);

        output.push_str(&format!("=== Health Report ===\n"));
        output.push_str(&format!("Status: {}\n", overall));
        output.push_str(&format!("Time: {}\n", Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        output.push_str("\n");

        for (name, result) in checks {
            let status_icon = match result.level {
                HealthLevel::Healthy => "✓",
                HealthLevel::Degraded => "⚠",
                HealthLevel::Unhealthy => "✗",
            };

            output.push_str(&format!("{} {} - {}\n", status_icon, name, result.message));

            if self.include_details {
                if let Some(ref details) = result.details {
                    output.push_str(&format!("  Details: {}\n", details));
                }
                output.push_str(&format!("  Duration: {}ms\n", result.duration_ms));
            }
        }

        output
    }

    /// 生成Prometheus格式报告
    pub fn to_prometheus(&self, checks: &HashMap<String, CheckResult>) -> String {
        let mut output = String::new();
        let timestamp = Utc::now().timestamp_millis();

        for (name, result) in checks {
            let status_value: f64 = match result.level {
                HealthLevel::Healthy => 1.0,
                HealthLevel::Degraded => 0.5,
                HealthLevel::Unhealthy => 0.0,
            };

            output.push_str(&format!(
                "fos_health_check_status{{check=\"{}\"}} {} {}\n",
                name, status_value, timestamp
            ));

            output.push_str(&format!(
                "fos_health_check_duration_ms{{check=\"{}\"}} {} {}\n",
                name, result.duration_ms, timestamp
            ));
        }

        output
    }

    /// 计算整体状态
    fn calculate_overall_status(checks: &HashMap<String, CheckResult>) -> HealthStatus {
        let mut has_unhealthy = false;
        let mut has_degraded = false;

        for result in checks.values() {
            match result.level {
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
}

impl Default for HealthReporter {
    fn default() -> Self {
        Self::new()
    }
}

/// 生成的报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedReport {
    /// 整体状态
    pub status: HealthStatus,
    /// 生成时间
    pub timestamp: DateTime<Utc>,
    /// 检查结果
    pub checks: HashMap<String, CheckResult>,
    /// 报告格式
    #[serde(skip)]
    pub format: ReportFormat,
}

impl GeneratedReport {
    /// 检查是否健康
    pub fn is_healthy(&self) -> bool {
        matches!(self.status, HealthStatus::Healthy)
    }

    /// 获取不健康的检查项
    pub fn unhealthy_checks(&self) -> Vec<&CheckResult> {
        self.checks.values().filter(|c| matches!(c.level, HealthLevel::Unhealthy)).collect()
    }

    /// 获取降级的检查项
    pub fn degraded_checks(&self) -> Vec<&CheckResult> {
        self.checks.values().filter(|c| matches!(c.level, HealthLevel::Degraded)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reporter_creation() {
        let reporter = HealthReporter::new();
        let checks = HashMap::new();
        let report = reporter.generate(&checks);
        assert_eq!(report.status, HealthStatus::Unknown);
    }

    #[test]
    fn test_json_report() {
        let reporter = HealthReporter::new();
        let mut checks = HashMap::new();
        checks.insert("test".to_string(), CheckResult::healthy("test", "OK"));

        let json = reporter.to_json(&checks);
        assert!(json.contains("test"));
    }

    #[test]
    fn test_text_report() {
        let reporter = HealthReporter::new();
        let mut checks = HashMap::new();
        checks.insert("test".to_string(), CheckResult::healthy("test", "OK"));

        let text = reporter.to_text(&checks);
        assert!(text.contains("Health Report"));
        assert!(text.contains("✓"));
    }

    #[test]
    fn test_prometheus_report() {
        let reporter = HealthReporter::new();
        let mut checks = HashMap::new();
        checks.insert("test".to_string(), CheckResult::healthy("test", "OK"));

        let prom = reporter.to_prometheus(&checks);
        assert!(prom.contains("fos_health_check_status"));
    }

    #[test]
    fn test_unhealthy_checks() {
        let reporter = HealthReporter::new();
        let mut checks = HashMap::new();
        checks.insert("healthy".to_string(), CheckResult::healthy("healthy", "OK"));
        checks.insert("unhealthy".to_string(), CheckResult::unhealthy("unhealthy", "FAIL"));

        let report = reporter.generate(&checks);
        let unhealthy = report.unhealthy_checks();
        assert_eq!(unhealthy.len(), 1);
    }

    #[test]
    fn test_calculate_overall_status() {
        let mut checks = HashMap::new();
        checks.insert("test".to_string(), CheckResult::healthy("test", "OK"));

        let status = HealthReporter::calculate_overall_status(&checks);
        assert_eq!(status, HealthStatus::Healthy);
    }
}
