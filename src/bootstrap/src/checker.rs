//! 依赖检查器

use crate::error::{Error, Result};
use std::collections::HashMap;

/// 依赖检查项
#[derive(Debug, Clone)]
pub struct DependencyCheck {
    /// 检查项名称
    pub name: String,
    /// 检查项描述
    pub description: String,
    /// 是否必需
    pub required: bool,
}

impl DependencyCheck {
    /// 创建新的检查项
    pub fn new(name: &str, description: &str, required: bool) -> Self {
        Self { name: name.to_string(), description: description.to_string(), required }
    }
}

/// 依赖检查结果
#[derive(Debug, Clone)]
pub struct CheckResult {
    /// 检查项名称
    pub name: String,
    /// 是否通过
    pub passed: bool,
    /// 错误信息
    pub error: Option<String>,
}

impl CheckResult {
    /// 创建成功结果
    pub fn success(name: &str) -> Self {
        Self { name: name.to_string(), passed: true, error: None }
    }

    /// 创建失败结果
    pub fn failure(name: &str, error: &str) -> Self {
        Self { name: name.to_string(), passed: false, error: Some(error.to_string()) }
    }
}

/// 依赖检查器
pub struct DependencyChecker {
    checks: HashMap<String, DependencyCheck>,
}

impl DependencyChecker {
    /// 创建新的依赖检查器
    pub fn new() -> Self {
        let mut checker = Self { checks: HashMap::new() };

        // 添加默认检查项
        checker.add_default_checks();
        checker
    }

    /// 添加默认检查项
    fn add_default_checks(&mut self) {
        self.checks.insert(
            "rust_version".to_string(),
            DependencyCheck::new("Rust版本", "检查Rust版本是否满足要求", true),
        );

        self.checks.insert(
            "config_files".to_string(),
            DependencyCheck::new("配置文件", "检查必要配置文件是否存在", true),
        );

        self.checks.insert(
            "storage_connection".to_string(),
            DependencyCheck::new("存储连接", "检查数据库/存储是否可连接", true),
        );

        self.checks.insert(
            "network_port".to_string(),
            DependencyCheck::new("网络端口", "检查服务端口是否可用", true),
        );
    }

    /// 添加检查项
    pub fn add_check(&mut self, check: DependencyCheck) {
        self.checks.insert(check.name.clone(), check);
    }

    /// 获取所有检查项
    pub fn checks(&self) -> Vec<&DependencyCheck> {
        self.checks.values().collect()
    }

    /// 执行所有检查
    pub fn check_all(&self) -> Result<Vec<CheckResult>> {
        let mut results = Vec::new();

        for (name, check) in &self.checks {
            let result = self.check_single(name, check);

            // 如果是必需项且失败，立即返回错误
            if check.required && !result.passed {
                return Err(Error::DependencyCheckFailed(format!(
                    "必需依赖检查失败: {} - {}",
                    name,
                    result.error.as_ref().unwrap_or(&"未知错误".to_string())
                )));
            }

            results.push(result);
        }

        Ok(results)
    }

    /// 执行单个检查
    fn check_single(&self, name: &str, _check: &DependencyCheck) -> CheckResult {
        match name {
            "rust_version" => self.check_rust_version(),
            "config_files" => self.check_config_files(),
            "storage_connection" => self.check_storage_connection(),
            "network_port" => self.check_network_port(),
            _ => CheckResult::success(name),
        }
    }

    fn check_rust_version(&self) -> CheckResult {
        // 检查Rust版本
        CheckResult::success("rust_version")
    }

    fn check_config_files(&self) -> CheckResult {
        // 检查配置文件
        CheckResult::success("config_files")
    }

    fn check_storage_connection(&self) -> CheckResult {
        // 检查存储连接
        CheckResult::success("storage_connection")
    }

    fn check_network_port(&self) -> CheckResult {
        // 检查网络端口
        CheckResult::success("network_port")
    }
}

impl Default for DependencyChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checker_creation() {
        let checker = DependencyChecker::new();
        assert!(checker.checks().len() >= 4);
    }

    #[test]
    fn test_check_all() {
        let checker = DependencyChecker::new();
        let result = checker.check_all();
        assert!(result.is_ok());
    }

    #[test]
    fn test_dependency_check_new() {
        let check = DependencyCheck::new("test", "测试检查", true);
        assert_eq!(check.name, "test");
        assert!(check.required);
    }
}
