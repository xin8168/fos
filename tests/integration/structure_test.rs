//! FOS 项目结构验证测试
//!
//! 验证所有模块目录和配置文件是否正确创建

use std::path::Path;

/// 获取项目根目录
fn project_root() -> std::path::PathBuf {
    std::env::current_dir().unwrap().parent().unwrap().to_path_buf()
}

/// 测试：验证核心模块目录存在
#[test]
fn test_core_modules_exist() {
    let root = project_root();
    let modules = ["src/gateway", "src/validator", "src/bus", "src/memory", "src/audit"];

    for module in &modules {
        let path = root.join(module);
        assert!(path.exists(), "核心模块目录不存在: {:?}", path);
    }
}

/// 测试：验证扩展模块目录存在
#[test]
fn test_extension_modules_exist() {
    let root = project_root();
    let modules = ["src/sandbox", "src/skills", "src/mcp", "src/config", "src/monitoring"];

    for module in &modules {
        let path = root.join(module);
        assert!(path.exists(), "扩展模块目录不存在: {:?}", path);
    }
}

/// 测试：验证新增核心模块目录存在
#[test]
fn test_new_core_modules_exist() {
    let root = project_root();
    let modules = ["src/rollback", "src/permission", "src/eventlog", "src/notifier"];

    for module in &modules {
        let path = root.join(module);
        assert!(path.exists(), "新增核心模块目录不存在: {:?}", path);
    }
}

/// 测试：验证基础设施模块目录存在
#[test]
fn test_infrastructure_modules_exist() {
    let root = project_root();
    let modules = ["src/bootstrap", "src/shutdown", "src/health"];

    for module in &modules {
        let path = root.join(module);
        assert!(path.exists(), "基础设施模块目录不存在: {:?}", path);
    }
}

/// 测试：验证数据一致性模块目录存在
#[test]
fn test_data_consistency_modules_exist() {
    let root = project_root();
    let modules = ["src/transaction", "src/lock", "src/idempotency"];

    for module in &modules {
        let path = root.join(module);
        assert!(path.exists(), "数据一致性模块目录不存在: {:?}", path);
    }
}

/// 测试：验证运维支持模块目录存在
#[test]
fn test_ops_modules_exist() {
    let root = project_root();
    let modules = ["src/migration", "src/backup", "src/ratelimiter"];

    for module in &modules {
        let path = root.join(module);
        assert!(path.exists(), "运维支持模块目录不存在: {:?}", path);
    }
}

/// 测试：验证扩展能力模块目录存在
#[test]
fn test_extension_capability_modules_exist() {
    let root = project_root();
    let modules = ["src/plugin", "src/schedule", "src/cache"];

    for module in &modules {
        let path = root.join(module);
        assert!(path.exists(), "扩展能力模块目录不存在: {:?}", path);
    }
}

/// 测试：验证所有模块Cargo.toml存在
#[test]
fn test_all_module_cargo_toml_exist() {
    let root = project_root();
    let modules = [
        "gateway",
        "validator",
        "bus",
        "memory",
        "audit",
        "sandbox",
        "skills",
        "mcp",
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
        "config",
        "monitoring",
    ];

    for module in &modules {
        let cargo_path = root.join(format!("src/{}/Cargo.toml", module));
        assert!(cargo_path.exists(), "模块Cargo.toml不存在: {:?}", cargo_path);
    }
}

/// 测试：验证所有模块lib.rs存在
#[test]
fn test_all_module_lib_rs_exist() {
    let root = project_root();
    let modules = [
        "gateway",
        "validator",
        "bus",
        "memory",
        "audit",
        "sandbox",
        "skills",
        "mcp",
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
        "config",
        "monitoring",
    ];

    for module in &modules {
        let lib_path = root.join(format!("src/{}/src/lib.rs", module));
        assert!(lib_path.exists(), "模块lib.rs不存在: {:?}", lib_path);
    }
}

/// 测试：验证文档目录结构
#[test]
fn test_docs_structure() {
    let root = project_root();
    let paths = ["docs/tasks", "docs/reports", "docs/architecture", "docs/api"];

    for path in &paths {
        let full_path = root.join(path);
        assert!(full_path.exists(), "文档目录不存在: {:?}", full_path);
    }
}

/// 测试：验证测试目录结构
#[test]
fn test_tests_structure() {
    let root = project_root();
    let paths = ["tests/integration", "tests/e2e", "tests/common", "benches", "examples"];

    for path in &paths {
        let full_path = root.join(path);
        assert!(full_path.exists(), "测试目录不存在: {:?}", full_path);
    }
}

/// 测试：验证脚本目录结构
#[test]
fn test_scripts_structure() {
    let root = project_root();
    let paths = ["scripts/build", "scripts/test", "scripts/deploy"];

    for path in &paths {
        let full_path = root.join(path);
        assert!(full_path.exists(), "脚本目录不存在: {:?}", full_path);
    }
}

/// 测试：验证配置文件存在
#[test]
fn test_config_files() {
    let root = project_root();
    let files = [".rustfmt.toml", "clippy.toml", ".gitignore"];

    for file in &files {
        let full_path = root.join(file);
        assert!(full_path.exists(), "配置文件不存在: {:?}", full_path);
    }
}

/// 测试：验证Cargo工作空间文件存在
#[test]
fn test_cargo_workspace() {
    let root = project_root();
    let cargo_path = root.join("src/Cargo.toml");
    assert!(cargo_path.exists(), "工作空间Cargo.toml不存在: {:?}", cargo_path);
}

/// 测试：验证主控文档存在
#[test]
fn test_master_control_doc() {
    let root = project_root();
    let doc_path = root.join("docs/FOS-MASTER-CONTROL.md");
    assert!(doc_path.exists(), "主控文档不存在: {:?}", doc_path);
}

/// 测试：验证进度跟踪文档存在
#[test]
fn test_progress_doc() {
    let root = project_root();
    let doc_path = root.join("docs/PROGRESS.md");
    assert!(doc_path.exists(), "进度跟踪文档不存在: {:?}", doc_path);
}

/// 测试：统计总模块数
#[test]
fn test_total_module_count() {
    let root = project_root();
    let modules = [
        "gateway",
        "validator",
        "bus",
        "memory",
        "audit",
        "sandbox",
        "skills",
        "mcp",
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
        "config",
        "monitoring",
    ];

    let mut count = 0;
    for module in &modules {
        let path = root.join(format!("src/{}", module));
        if path.exists() {
            count += 1;
        }
    }

    assert_eq!(count, 26, "总模块数应为26，实际为{}", count);
}
