//! FOS 红线规则模块
//!
//! 定义不可逾越的安全边界，任何触犯红线的请求都将被直接拒绝

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// 红线规则定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedLine {
    /// 红线ID
    pub id: String,
    /// 红线名称
    pub name: String,
    /// 红线描述
    pub description: String,
    /// 红线类型
    pub red_line_type: RedLineType,
    /// 严重程度
    pub severity: RedLineSeverity,
    /// 触发条件
    pub triggers: Vec<String>,
    /// 是否启用
    pub enabled: bool,
}

/// 红线类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RedLineType {
    /// 系统安全红线
    SystemSecurity,
    /// 数据安全红线
    DataSecurity,
    /// 操作安全红线
    OperationSecurity,
    /// 网络安全红线
    NetworkSecurity,
    /// 资源安全红线
    ResourceSecurity,
}

/// 红线严重程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum RedLineSeverity {
    /// 严重 - 立即终止
    Critical,
    /// 高危 - 需要审批
    High,
    /// 中危 - 需要确认
    Medium,
    /// 低危 - 记录警告
    Low,
}

impl RedLine {
    /// 创建新红线
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        red_line_type: RedLineType,
        severity: RedLineSeverity,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            red_line_type,
            severity,
            triggers: Vec::new(),
            enabled: true,
        }
    }

    /// 添加触发条件
    pub fn with_trigger(mut self, trigger: impl Into<String>) -> Self {
        self.triggers.push(trigger.into());
        self
    }

    /// 设置描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// 检查是否触发红线
    pub fn is_triggered(&self, content: &str) -> bool {
        for trigger in &self.triggers {
            if content.contains(trigger) {
                return true;
            }
        }
        false
    }
}

/// 红线检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedLineResult {
    /// 是否触发红线
    pub triggered: bool,
    /// 触发的红线列表
    pub triggered_red_lines: Vec<RedLineViolation>,
    /// 检查时间（毫秒）
    pub check_time_ms: u64,
}

/// 红线违规记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedLineViolation {
    /// 红线ID
    pub red_line_id: String,
    /// 红线名称
    pub red_line_name: String,
    /// 严重程度
    pub severity: RedLineSeverity,
    /// 触发内容
    pub trigger_content: String,
    /// 违规时间
    pub violation_time: i64,
}

/// 内置红线规则
pub struct BuiltinRedLines;

impl BuiltinRedLines {
    /// 获取所有内置红线
    pub fn all() -> Vec<RedLine> {
        vec![
            // 系统安全红线
            RedLine::new(
                "sys-001",
                "系统文件操作",
                RedLineType::SystemSecurity,
                RedLineSeverity::Critical,
            )
            .with_description("禁止操作系统关键文件")
            .with_trigger("/etc/passwd")
            .with_trigger("/etc/shadow")
            .with_trigger("C:\\Windows\\System32")
            .with_trigger("registry")
            .with_trigger("HKEY_"),
            RedLine::new(
                "sys-002",
                "系统命令执行",
                RedLineType::SystemSecurity,
                RedLineSeverity::Critical,
            )
            .with_description("禁止执行危险系统命令")
            .with_trigger("rm -rf /")
            .with_trigger("del /s /q")
            .with_trigger("format ")
            .with_trigger("shutdown -h")
            .with_trigger("reboot")
            .with_trigger("init 0")
            .with_trigger("mkfs"),
            // 数据安全红线
            RedLine::new(
                "data-001",
                "数据库危险操作",
                RedLineType::DataSecurity,
                RedLineSeverity::Critical,
            )
            .with_description("禁止执行危险数据库操作")
            .with_trigger("DROP DATABASE")
            .with_trigger("DROP TABLE")
            .with_trigger("TRUNCATE TABLE")
            .with_trigger("DELETE FROM")
            .with_trigger("GRANT ALL"),
            RedLine::new(
                "data-002",
                "敏感数据访问",
                RedLineType::DataSecurity,
                RedLineSeverity::High,
            )
            .with_description("禁止访问敏感数据")
            .with_trigger("password")
            .with_trigger("secret")
            .with_trigger("token")
            .with_trigger("api_key")
            .with_trigger("private_key"),
            // 操作安全红线
            RedLine::new(
                "op-001",
                "远程执行",
                RedLineType::OperationSecurity,
                RedLineSeverity::High,
            )
            .with_description("禁止未经授权的远程执行")
            .with_trigger("ssh ")
            .with_trigger("telnet ")
            .with_trigger("rsh ")
            .with_trigger("远程桌面")
            .with_trigger("Remote Desktop"),
            RedLine::new(
                "op-002",
                "进程操作",
                RedLineType::OperationSecurity,
                RedLineSeverity::Medium,
            )
            .with_description("限制进程操作")
            .with_trigger("kill -9")
            .with_trigger("taskkill /F")
            .with_trigger("pkill"),
            // 网络安全红线
            RedLine::new(
                "net-001",
                "网络攻击",
                RedLineType::NetworkSecurity,
                RedLineSeverity::Critical,
            )
            .with_description("禁止网络攻击行为")
            .with_trigger("nmap")
            .with_trigger("sqlmap")
            .with_trigger("metasploit")
            .with_trigger("nmap ")
            .with_trigger("nikto"),
            RedLine::new(
                "net-002",
                "端口扫描",
                RedLineType::NetworkSecurity,
                RedLineSeverity::High,
            )
            .with_description("禁止端口扫描")
            .with_trigger("port scan")
            .with_trigger("端口扫描"),
            // 资源安全红线
            RedLine::new(
                "res-001",
                "资源耗尽",
                RedLineType::ResourceSecurity,
                RedLineSeverity::High,
            )
            .with_description("禁止资源耗尽攻击")
            .with_trigger("fork bomb")
            .with_trigger(":(){ :|:& };:")
            .with_trigger("while true"),
        ]
    }
}

/// 红线检查器
pub struct RedLineChecker {
    /// 红线规则集合
    red_lines: Vec<RedLine>,
    /// 禁用的触发词（快速检查）
    blocked_patterns: HashSet<String>,
}

impl RedLineChecker {
    /// 创建新检查器
    pub fn new() -> Self {
        let red_lines = BuiltinRedLines::all();
        let mut blocked_patterns = HashSet::new();

        for red_line in &red_lines {
            for trigger in &red_line.triggers {
                blocked_patterns.insert(trigger.to_lowercase());
            }
        }

        Self { red_lines, blocked_patterns }
    }

    /// 检查内容是否触犯红线
    pub fn check(&self, content: &str) -> RedLineResult {
        let start = std::time::Instant::now();
        let content_lower = content.to_lowercase();
        let mut triggered_red_lines = Vec::new();

        for red_line in &self.red_lines {
            if !red_line.enabled {
                continue;
            }

            if red_line.is_triggered(&content_lower) {
                triggered_red_lines.push(RedLineViolation {
                    red_line_id: red_line.id.clone(),
                    red_line_name: red_line.name.clone(),
                    severity: red_line.severity.clone(),
                    trigger_content: content.to_string(),
                    violation_time: chrono::Utc::now().timestamp_millis(),
                });
            }
        }

        let triggered = !triggered_red_lines.is_empty();
        let check_time_ms = start.elapsed().as_millis() as u64;

        RedLineResult { triggered, triggered_red_lines, check_time_ms }
    }

    /// 快速检查（仅检查关键词）
    pub fn quick_check(&self, content: &str) -> bool {
        let content_lower = content.to_lowercase();

        for pattern in &self.blocked_patterns {
            if content_lower.contains(pattern) {
                return true;
            }
        }
        false
    }

    /// 获取所有红线
    pub fn get_red_lines(&self) -> &[RedLine] {
        &self.red_lines
    }

    /// 获取指定严重程度的红线
    pub fn get_by_severity(&self, severity: RedLineSeverity) -> Vec<&RedLine> {
        self.red_lines.iter().filter(|r| r.severity == severity).collect()
    }
}

impl Default for RedLineChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_red_line_creation() {
        let red_line = RedLine::new(
            "test-001",
            "测试红线",
            RedLineType::SystemSecurity,
            RedLineSeverity::Critical,
        );

        assert_eq!(red_line.id, "test-001");
        assert_eq!(red_line.severity, RedLineSeverity::Critical);
        assert!(red_line.enabled);
    }

    #[test]
    fn test_red_line_trigger() {
        let red_line = RedLine::new(
            "test-001",
            "测试红线",
            RedLineType::SystemSecurity,
            RedLineSeverity::Critical,
        )
        .with_trigger("dangerous");

        assert!(red_line.is_triggered("this is dangerous content"));
        assert!(!red_line.is_triggered("safe content"));
    }

    #[test]
    fn test_checker_creation() {
        let checker = RedLineChecker::new();
        assert!(!checker.get_red_lines().is_empty());
    }

    #[test]
    fn test_check_safe_content() {
        let checker = RedLineChecker::new();
        let result = checker.check("hello world, this is a normal operation");

        assert!(!result.triggered);
        assert!(result.triggered_red_lines.is_empty());
    }

    #[test]
    fn test_check_dangerous_content() {
        let checker = RedLineChecker::new();
        let result = checker.check("rm -rf /");

        assert!(result.triggered, "Should detect dangerous content");
        assert!(!result.triggered_red_lines.is_empty());
    }

    #[test]
    fn test_quick_check() {
        let checker = RedLineChecker::new();

        assert!(checker.quick_check("rm -rf /"));
        assert!(!checker.quick_check("safe operation"));
    }

    #[test]
    fn test_builtin_red_lines() {
        let red_lines = BuiltinRedLines::all();
        assert!(!red_lines.is_empty());

        // 验证有不同严重程度的红线
        let has_critical = red_lines.iter().any(|r| r.severity == RedLineSeverity::Critical);
        assert!(has_critical);
    }

    #[test]
    fn test_get_by_severity() {
        let checker = RedLineChecker::new();
        let critical = checker.get_by_severity(RedLineSeverity::Critical);

        assert!(!critical.is_empty());
        for red_line in critical {
            assert_eq!(red_line.severity, RedLineSeverity::Critical);
        }
    }
}
