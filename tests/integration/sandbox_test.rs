//! FOS Sandbox 集成测试
//!
//! 测试 Sandbox 模块各组件之间的集成

use fos_sandbox::{
    EnvironmentValidator, FilesystemIsolation, IsolationConfig, IsolationManager, NetworkIsolation,
    PathAccess, ProcessIsolation, Sandbox, SandboxConfig, SandboxSnapshot, SnapshotManager,
    SnapshotType, StatusValidator, ValidationResult,
};
use std::path::PathBuf;
use std::sync::Arc;

/// 测试沙箱完整生命周期
#[tokio::test]
async fn test_sandbox_full_lifecycle() {
    // 创建沙箱
    let config = SandboxConfig::default();
    let sandbox = Sandbox::new(config);

    // 启动
    sandbox.start().await.unwrap();
    assert!(matches!(sandbox.status().await, fos_sandbox::SandboxStatus::Running));

    // 执行命令
    let result = sandbox.execute("test command").await.unwrap();
    assert!(result.contains("成功"));

    // 停止
    sandbox.stop().await.unwrap();
    assert!(matches!(sandbox.status().await, fos_sandbox::SandboxStatus::Success));
}

/// 测试隔离管理器完整流程
#[tokio::test]
async fn test_isolation_manager_full_flow() {
    let mut config = IsolationConfig::default();
    config.readonly_paths.push(PathBuf::from("/readonly"));
    config.readwrite_paths.push(PathBuf::from("/readwrite"));
    config.denied_paths.push(PathBuf::from("/denied"));

    let manager = IsolationManager::new(config);

    // 激活所有隔离
    manager.activate_all().await.unwrap();
    assert!(manager.is_all_active().await);

    // 检查路径访问
    let fs_access =
        manager.filesystem.check_path_access(&PathBuf::from("/readonly/file")).await.unwrap();
    assert_eq!(fs_access, PathAccess::ReadOnly);

    // 销毁所有隔离
    manager.destroy_all().await.unwrap();
    assert!(!manager.is_all_active().await);
}

/// 测试快照管理器完整流程
#[tokio::test]
async fn test_snapshot_manager_full_flow() {
    let manager = Arc::new(SnapshotManager::new());

    // 创建多个快照
    let snapshot1 = SandboxSnapshot::new("sandbox-1".to_string(), SnapshotType::Full)
        .with_filesystem_state(serde_json::json!({"root": "/tmp/sandbox1"}));

    let snapshot2 = SandboxSnapshot::new("sandbox-1".to_string(), SnapshotType::Incremental)
        .with_parent(snapshot1.id.clone())
        .with_filesystem_state(serde_json::json!({"root": "/tmp/sandbox2"}));

    let id1 = manager.create(snapshot1).await.unwrap();
    let id2 = manager.create(snapshot2).await.unwrap();

    // 获取沙箱所有快照
    let snapshots = manager.get_sandbox_snapshots("sandbox-1").await.unwrap();
    assert_eq!(snapshots.len(), 2);

    // 获取最新快照
    let latest = manager.get_latest("sandbox-1").await.unwrap();
    assert_eq!(latest.snapshot_type, SnapshotType::Incremental);

    // 恢复快照
    let restored = manager.restore(&id1).await.unwrap();
    assert!(matches!(restored.status, fos_sandbox::SnapshotStatus::Restored));

    // 删除快照
    manager.delete(&id2).await.unwrap();
    assert_eq!(manager.count().await, 1);
}

/// 测试环境校验完整流程
#[test]
fn test_environment_validator_full_flow() {
    let config = SandboxConfig::default();
    let validator = EnvironmentValidator::new(config);

    let result = validator.validate_all();
    assert!(result.passed);
    assert!(result.errors.is_empty());
}

/// 测试环境校验失败场景
#[test]
fn test_environment_validator_failure() {
    let mut config = SandboxConfig::default();
    config.timeout_secs = 0;
    config.cpu_limit_percent = 150;

    let validator = EnvironmentValidator::new(config);
    let result = validator.validate_all();

    assert!(!result.passed);
    assert!(result.errors.iter().any(|e| e.contains("超时")));
    assert!(result.errors.iter().any(|e| e.contains("CPU")));
}

/// 测试隔离校验器
#[test]
fn test_isolation_validator() {
    let mut config = IsolationConfig::default();
    config.denied_paths.push(PathBuf::from("/denied"));
    config.readonly_paths.push(PathBuf::from("/readonly"));

    let validator = fos_sandbox::IsolationValidator::new(config);
    let result = validator.validate_all();

    assert!(result.passed);
}

/// 测试隔离校验路径冲突
#[test]
fn test_isolation_validator_path_conflict() {
    let mut config = IsolationConfig::default();
    config.denied_paths.push(PathBuf::from("/denied"));
    config.readonly_paths.push(PathBuf::from("/denied/file"));

    let validator = fos_sandbox::IsolationValidator::new(config);
    let result = validator.validate_filesystem();

    assert!(!result.passed);
    assert!(result
        .errors
        .iter()
        .any(|e| e.contains("禁止路径") || e.contains("冲突") || e.contains("路径")));
}

/// 测试状态转换校验
#[test]
fn test_status_validator_transitions() {
    // 有效转换
    let result = StatusValidator::validate_status_transition(
        fos_sandbox::SandboxStatus::Idle,
        fos_sandbox::SandboxStatus::Running,
    );
    assert!(result.passed);

    let result = StatusValidator::validate_status_transition(
        fos_sandbox::SandboxStatus::Running,
        fos_sandbox::SandboxStatus::Success,
    );
    assert!(result.passed);

    // 无效转换
    let result = StatusValidator::validate_status_transition(
        fos_sandbox::SandboxStatus::Success,
        fos_sandbox::SandboxStatus::Running,
    );
    assert!(!result.passed);
}

/// 测试文件系统隔离与快照集成
#[tokio::test]
async fn test_filesystem_snapshot_integration() {
    let mut isolation_config = IsolationConfig::default();
    isolation_config.readwrite_paths.push(PathBuf::from("/sandbox"));

    let filesystem = FilesystemIsolation::new(isolation_config);
    filesystem.activate().await.unwrap();

    // 创建快照记录文件系统状态
    let snapshot = SandboxSnapshot::new("sandbox-1".to_string(), SnapshotType::Full)
        .with_filesystem_state(serde_json::json!({
            "readwrite_paths": ["/sandbox"],
            "status": "active"
        }));

    let manager = SnapshotManager::new();
    let id = manager.create(snapshot).await.unwrap();

    // 验证路径访问
    let access = filesystem.check_path_access(&PathBuf::from("/sandbox/file")).await.unwrap();
    assert_eq!(access, PathAccess::ReadWrite);

    // 清理
    filesystem.destroy().await.unwrap();
    manager.delete(&id).await.unwrap();
}

/// 测试进程隔离与快照集成
#[tokio::test]
async fn test_process_snapshot_integration() {
    let config = IsolationConfig::default();
    let process = ProcessIsolation::new(config);

    process.activate().await.unwrap();

    // 注册子进程
    process.register_child(1234).await.unwrap();
    process.register_child(5678).await.unwrap();

    // 创建快照
    let snapshot = SandboxSnapshot::new("sandbox-1".to_string(), SnapshotType::Full)
        .with_process_state(serde_json::json!({
            "child_pids": [1234, 5678]
        }));

    let manager = SnapshotManager::new();
    let _id = manager.create(snapshot).await.unwrap();

    // 验证进程列表
    let pids = process.get_child_pids().await;
    assert_eq!(pids.len(), 2);

    // 销毁
    process.destroy().await.unwrap();
    let pids = process.get_child_pids().await;
    assert!(pids.is_empty());
}

/// 测试网络隔离与校验集成
#[tokio::test]
async fn test_network_validation_integration() {
    let mut config = IsolationConfig::default();
    config.allowed_networks.push("192.168.1".to_string());
    config.denied_networks.push("10.0.0".to_string());

    let network = NetworkIsolation::new(config.clone());
    network.activate().await.unwrap();

    // 创建校验器检查配置
    let validator = fos_sandbox::IsolationValidator::new(config);
    let result = validator.validate_network();

    assert!(result.passed);

    // 测试网络访问
    let access = network.check_network_access("192.168.1.100").await.unwrap();
    assert!(matches!(access, fos_sandbox::NetworkAccess::Allowed));

    network.destroy().await.unwrap();
}

/// 测试完整沙箱工作流：创建 -> 校验 -> 执行 -> 快照 -> 恢复
#[tokio::test]
async fn test_complete_sandbox_workflow() {
    // 1. 创建配置
    let config = SandboxConfig::default();

    // 2. 校验配置
    let validator = EnvironmentValidator::new(config.clone());
    let validation = validator.validate_all();
    assert!(validation.passed);

    // 3. 创建沙箱
    let sandbox = Sandbox::new(config);
    sandbox.start().await.unwrap();

    // 4. 创建快照
    let snapshot = SandboxSnapshot::new("workflow-test".to_string(), SnapshotType::Checkpoint)
        .with_filesystem_state(serde_json::json!({"state": "before_execution"}));

    let snapshot_manager = Arc::new(SnapshotManager::new());
    let snapshot_id = snapshot_manager.create(snapshot).await.unwrap();

    // 5. 执行操作
    let result = sandbox.execute("important operation").await.unwrap();
    assert!(result.contains("成功"));

    // 6. 恢复快照（模拟回滚）
    let restored = snapshot_manager.restore(&snapshot_id).await.unwrap();
    assert!(matches!(restored.status, fos_sandbox::SnapshotStatus::Restored));

    // 7. 清理
    sandbox.stop().await.unwrap();
    snapshot_manager.delete(&snapshot_id).await.unwrap();
}
