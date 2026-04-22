//! 沙箱环境校验模块
//!
//! 提供沙箱环境的验证和检查功能

use crate::error::{Result, SandboxError};
use crate::{IsolationConfig, SandboxConfig, SandboxStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// 校验结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// 是否通过
    pub passed: bool,
    /// 错误列表
    pub errors: Vec<String>,
    /// 警告列表
    pub warnings: Vec<String>,
    /// 检查详情
    pub details: HashMap<String, CheckResult>,
}

impl ValidationResult {
    /// 创建新的校验结果
    pub fn new() -> Self {
        Self { passed: true, errors: Vec::new(), warnings: Vec::new(), details: HashMap::new() }
    }

    /// 添加错误
    pub fn add_error(&mut self, check_name: &str, error: String) {
        self.passed = false;
        self.errors.push(error.clone());
        self.details.insert(
            check_name.to_string(),
            CheckResult { name: check_name.to_string(), passed: false, message: Some(error) },
        );
    }

    /// 添加警告
    pub fn add_warning(&mut self, check_name: &str, warning: String) {
        self.warnings.push(warning.clone());
        self.details.insert(
            check_name.to_string(),
            CheckResult { name: check_name.to_string(), passed: true, message: Some(warning) },
        );
    }

    /// 添加通过的检查
    pub fn add_pass(&mut self, check_name: &str) {
        self.details.insert(
            check_name.to_string(),
            CheckResult { name: check_name.to_string(), passed: true, message: None },
        );
    }

    /// 合并其他校验结果
    pub fn merge(&mut self, other: ValidationResult) {
        if !other.passed {
            self.passed = false;
        }
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self.details.extend(other.details);
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// 单个检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// 检查名称
    pub name: String,
    /// 是否通过
    pub passed: bool,
    /// 消息
    pub message: Option<String>,
}

/// 环境校验器
pub struct EnvironmentValidator {
    /// 沙箱配置
    config: SandboxConfig,
}

impl EnvironmentValidator {
    /// 创建新的环境校验器
    pub fn new(config: SandboxConfig) -> Self {
        Self { config }
    }

    /// 执行所有校验
    pub fn validate_all(&self) -> ValidationResult {
        let mut result = ValidationResult::new();

        result.merge(self.validate_config());
        result.merge(self.validate_resources());
        result.merge(self.validate_isolation());
        result.merge(self.validate_paths());

        result
    }

    /// 校验配置
    pub fn validate_config(&self) -> ValidationResult {
        let mut result = ValidationResult::new();

        // 检查超时配置
        if self.config.timeout_secs == 0 {
            result.add_error("timeout_check", "超时时间不能为0".to_string());
        } else if self.config.timeout_secs > 3600 {
            result.add_warning(
                "timeout_check",
                "超时时间超过1小时，可能导致资源占用过长".to_string(),
            );
        } else {
            result.add_pass("timeout_check");
        }

        // 检查内存限制
        if self.config.memory_limit_mb == 0 {
            result.add_error("memory_check", "内存限制不能为0".to_string());
        } else if self.config.memory_limit_mb > 4096 {
            result.add_warning("memory_check", "内存限制超过4GB，请确认系统资源充足".to_string());
        } else {
            result.add_pass("memory_check");
        }

        // 检查CPU限制
        if self.config.cpu_limit_percent > 100 {
            result.add_error("cpu_check", "CPU限制不能超过100%".to_string());
        } else if self.config.cpu_limit_percent == 0 {
            result.add_warning("cpu_check", "CPU限制为0%，可能导致无法执行".to_string());
        } else {
            result.add_pass("cpu_check");
        }

        result
    }

    /// 校验资源
    pub fn validate_resources(&self) -> ValidationResult {
        let mut result = ValidationResult::new();

        // 检查系统内存是否足够
        // 实际实现中会检查真实系统资源
        let available_memory_mb = 8192u64; // 模拟值
        if self.config.memory_limit_mb > available_memory_mb {
            result.add_error(
                "resource_memory",
                format!(
                    "请求的内存 {}MB 超过可用内存 {}MB",
                    self.config.memory_limit_mb, available_memory_mb
                ),
            );
        } else {
            result.add_pass("resource_memory");
        }

        result
    }

    /// 校验隔离配置
    pub fn validate_isolation(&self) -> ValidationResult {
        let mut result = ValidationResult::new();

        // 检查网络配置
        if self.config.network_allowed && self.config.allowed_paths.is_empty() {
            result.add_warning(
                "isolation_network",
                "允许网络访问但未限制路径，可能存在安全风险".to_string(),
            );
        } else {
            result.add_pass("isolation_network");
        }

        result
    }

    /// 校验路径
    pub fn validate_paths(&self) -> ValidationResult {
        let mut result = ValidationResult::new();

        for path in &self.config.allowed_paths {
            let path_buf = PathBuf::from(path);
            if !path_buf.is_absolute() {
                result.add_error("path_absolute", format!("路径 {} 不是绝对路径", path));
            } else {
                result.add_pass("path_absolute");
            }
        }

        if result.details.is_empty() {
            result.add_pass("path_check");
        }

        result
    }
}

/// 隔离校验器
pub struct IsolationValidator {
    /// 隔离配置
    config: IsolationConfig,
}

impl IsolationValidator {
    /// 创建新的隔离校验器
    pub fn new(config: IsolationConfig) -> Self {
        Self { config }
    }

    /// 执行所有校验
    pub fn validate_all(&self) -> ValidationResult {
        let mut result = ValidationResult::new();

        result.merge(self.validate_filesystem());
        result.merge(self.validate_network());

        result
    }

    /// 校验文件系统隔离
    pub fn validate_filesystem(&self) -> ValidationResult {
        let mut result = ValidationResult::new();

        // 检查路径冲突
        for denied in &self.config.denied_paths {
            for readonly in &self.config.readonly_paths {
                if readonly.starts_with(denied) {
                    result.add_error(
                        "path_conflict",
                        format!("只读路径 {:?} 在禁止路径 {:?} 下", readonly, denied),
                    );
                }
            }
            for readwrite in &self.config.readwrite_paths {
                if readwrite.starts_with(denied) {
                    result.add_error(
                        "path_conflict",
                        format!("读写路径 {:?} 在禁止路径 {:?} 下", readwrite, denied),
                    );
                }
            }
        }

        // 检查重复路径
        let all_paths: Vec<&PathBuf> = self
            .config
            .readonly_paths
            .iter()
            .chain(self.config.readwrite_paths.iter())
            .chain(self.config.denied_paths.iter())
            .collect();

        for (i, path1) in all_paths.iter().enumerate() {
            for path2 in all_paths.iter().skip(i + 1) {
                if path1 == path2 {
                    result.add_warning("duplicate_path", format!("路径 {:?} 重复定义", path1));
                }
            }
        }

        if result.errors.is_empty() {
            result.add_pass("filesystem_isolation");
        }

        result
    }

    /// 校验网络隔离
    pub fn validate_network(&self) -> ValidationResult {
        let mut result = ValidationResult::new();

        // 检查网络地址冲突
        for denied in &self.config.denied_networks {
            for allowed in &self.config.allowed_networks {
                if allowed.starts_with(denied) || denied.starts_with(allowed) {
                    result.add_warning(
                        "network_conflict",
                        format!("允许网络 {} 和禁止网络 {} 可能冲突", allowed, denied),
                    );
                }
            }
        }

        if result.errors.is_empty() {
            result.add_pass("network_isolation");
        }

        result
    }
}

/// 状态校验器
pub struct StatusValidator;

impl StatusValidator {
    /// 创建新的状态校验器
    pub fn new() -> Self {
        Self
    }

    /// 校验沙箱状态转换
    pub fn validate_status_transition(from: SandboxStatus, to: SandboxStatus) -> ValidationResult {
        let mut result = ValidationResult::new();

        let valid = match (&from, &to) {
            // 从 Idle 可以到 Running
            (SandboxStatus::Idle, SandboxStatus::Running) => true,
            // 从 Running 可以到 Success 或 Failed
            (SandboxStatus::Running, SandboxStatus::Success) => true,
            (SandboxStatus::Running, SandboxStatus::Failed) => true,
            // 从任何状态可以到 Destroyed
            (_, SandboxStatus::Destroyed) => true,
            // 相同状态
            (a, b) if a == b => true,
            // 其他转换无效
            _ => false,
        };

        if valid {
            result.add_pass("status_transition");
        } else {
            result
                .add_error("status_transition", format!("无效的状态转换: {:?} -> {:?}", from, to));
        }

        result
    }
}

impl Default for StatusValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_new() {
        let result = ValidationResult::new();
        assert!(result.passed);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_validation_result_add_error() {
        let mut result = ValidationResult::new();
        result.add_error("test", "error message".to_string());

        assert!(!result.passed);
        assert_eq!(result.errors.len(), 1);
        assert!(result.details.contains_key("test"));
    }

    #[test]
    fn test_validation_result_add_warning() {
        let mut result = ValidationResult::new();
        result.add_warning("test", "warning message".to_string());

        assert!(result.passed); // 警告不影响通过状态
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_environment_validator_config() {
        let config = SandboxConfig::default();
        let validator = EnvironmentValidator::new(config);

        let result = validator.validate_config();
        assert!(result.passed);
    }

    #[test]
    fn test_environment_validator_invalid_timeout() {
        let mut config = SandboxConfig::default();
        config.timeout_secs = 0;

        let validator = EnvironmentValidator::new(config);
        let result = validator.validate_config();

        assert!(!result.passed);
        assert!(result.errors.iter().any(|e| e.contains("超时")));
    }

    #[test]
    fn test_environment_validator_invalid_cpu() {
        let mut config = SandboxConfig::default();
        config.cpu_limit_percent = 150;

        let validator = EnvironmentValidator::new(config);
        let result = validator.validate_config();

        assert!(!result.passed);
        assert!(result.errors.iter().any(|e| e.contains("CPU")));
    }

    #[test]
    fn test_isolation_validator() {
        let config = IsolationConfig::default();
        let validator = IsolationValidator::new(config);

        let result = validator.validate_all();
        assert!(result.passed);
    }

    #[test]
    fn test_isolation_validator_path_conflict() {
        let mut config = IsolationConfig::default();
        config.denied_paths.push(PathBuf::from("/denied"));
        config.readonly_paths.push(PathBuf::from("/denied/file"));

        let validator = IsolationValidator::new(config);
        let result = validator.validate_filesystem();

        assert!(!result.passed);
    }

    #[test]
    fn test_status_validator_valid_transition() {
        let result = StatusValidator::validate_status_transition(
            SandboxStatus::Idle,
            SandboxStatus::Running,
        );
        assert!(result.passed);
    }

    #[test]
    fn test_status_validator_invalid_transition() {
        let result = StatusValidator::validate_status_transition(
            SandboxStatus::Success,
            SandboxStatus::Running,
        );
        assert!(!result.passed);
    }

    #[test]
    fn test_validate_all() {
        let config = SandboxConfig::default();
        let validator = EnvironmentValidator::new(config);

        let result = validator.validate_all();
        assert!(result.passed);
    }
}
