//! FOS 集成测试公共模块
//!
//! 提供测试共用的工具函数和断言

use std::path::Path;

/// 检查路径是否存在
pub fn path_exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// 检查模块目录是否完整
pub fn module_is_complete(module_name: &str) -> bool {
    let base_path = format!("src/{}", module_name);
    let cargo_path = format!("{}/Cargo.toml", base_path);
    let lib_path = format!("{}/src/lib.rs", base_path);
    let error_path = format!("{}/src/error.rs", base_path);
    let config_path = format!("{}/src/config.rs", base_path);

    path_exists(&cargo_path)
        && path_exists(&lib_path)
        && path_exists(&error_path)
        && path_exists(&config_path)
}

/// 获取所有模块列表
pub fn get_all_modules() -> Vec<&'static str> {
    vec![
        "gateway",
        "validator",
        "bus",
        "memory",
        "audit",
        "sandbox",
        "skills",
        "mcp",
        "config",
        "monitoring",
        "rollback",
        "permission",
        "eventlog",
        "notifier",
        "bootstrap",
        "shutdown",
        "health",
        "transaction",
        "lock",
        "idempotency",
        "migration",
        "backup",
        "ratelimiter",
        "plugin",
        "schedule",
        "cache",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_exists() {
        assert!(path_exists("src"));
        assert!(!path_exists("nonexistent_path"));
    }
}
