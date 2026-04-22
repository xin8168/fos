//! FOS Rollback 集成测试
//!
//! 测试 Rollback 模块与其他模块的集成

use fos_rollback::{
    RollbackActionStatus, RollbackActionType, RollbackExecutor, RollbackResult, RollbackVerifier,
    SnapshotManager, SnapshotStatus, SnapshotType, VerificationStatus,
};
use std::sync::Arc;

/// 测试快照管理器与执行器集成
#[tokio::test]
async fn test_snapshot_executor_integration() {
    let snapshot_manager = Arc::new(SnapshotManager::new());
    let executor = RollbackExecutor::new(snapshot_manager.clone());

    // 创建快照
    let id = snapshot_manager
        .create_snapshot_with_rollback(
            "op-001".to_string(),
            SnapshotType::Full,
            serde_json::json!({"data": "value"}),
            serde_json::json!({"rollback": "data"}),
        )
        .await
        .unwrap();

    // 执行回滚
    let result = executor.execute(&id).await.unwrap();
    assert!(result.success);

    // 验证快照状态
    let snapshot = snapshot_manager.get_snapshot(&id).await.unwrap();
    assert_eq!(snapshot.status, SnapshotStatus::RolledBack);
}

/// 测试多级回滚集成
#[tokio::test]
async fn test_multi_level_integration() {
    let snapshot_manager = Arc::new(SnapshotManager::new());
    let executor = RollbackExecutor::new(snapshot_manager.clone());

    // 创建多级快照
    for i in 0..3 {
        snapshot_manager
            .create_snapshot_with_rollback(
                "op-multi".to_string(),
                SnapshotType::Incremental,
                serde_json::json!({"version": i}),
                serde_json::json!({"rollback": i}),
            )
            .await
            .unwrap();
    }

    // 执行两级回滚
    let results = executor.execute_multi_level("op-multi", 2).await.unwrap();
    assert_eq!(results.len(), 2);

    // 验证所有回滚都成功
    for result in &results {
        assert!(result.success);
    }
}

/// 测试执行器与验证器集成
#[tokio::test]
async fn test_executor_verifier_integration() {
    let snapshot_manager = Arc::new(SnapshotManager::new());
    let executor = RollbackExecutor::new(snapshot_manager.clone());
    let verifier = RollbackVerifier::new(snapshot_manager.clone());

    // 创建快照并执行回滚
    let id = snapshot_manager
        .create_snapshot_with_rollback(
            "op-verify".to_string(),
            SnapshotType::Full,
            serde_json::json!({}),
            serde_json::json!({}),
        )
        .await
        .unwrap();

    let rollback_result = executor.execute(&id).await.unwrap();

    // 验证回滚结果
    let verification = verifier.verify(&rollback_result).await.unwrap();
    assert!(verification.is_passed());
    assert_eq!(verification.status, VerificationStatus::Passed);
}

/// 测试快照统计与执行器集成
#[tokio::test]
async fn test_snapshot_stats_integration() {
    let snapshot_manager = Arc::new(SnapshotManager::new());

    // 创建多个快照
    for i in 0..5 {
        snapshot_manager
            .create_snapshot(
                format!("op-{}", i % 2),
                SnapshotType::Incremental,
                serde_json::json!({"index": i}),
            )
            .await
            .unwrap();
    }

    // 统计
    assert_eq!(snapshot_manager.count().await, 5);
    assert_eq!(snapshot_manager.count_for_operation("op-0").await, 3);
    assert_eq!(snapshot_manager.count_for_operation("op-1").await, 2);
}

/// 测试快照生命周期集成
#[tokio::test]
async fn test_snapshot_lifecycle() {
    let snapshot_manager = Arc::new(SnapshotManager::new());

    // 创建快照
    let id = snapshot_manager
        .create_snapshot_with_rollback(
            "op-lifecycle".to_string(),
            SnapshotType::Full,
            serde_json::json!({}),
            serde_json::json!({}),
        )
        .await
        .unwrap();

    // 验证初始状态
    let snapshot = snapshot_manager.get_snapshot(&id).await.unwrap();
    assert_eq!(snapshot.status, SnapshotStatus::Created);

    // 标记为已使用
    snapshot_manager.mark_used(&id).await.unwrap();
    let snapshot = snapshot_manager.get_snapshot(&id).await.unwrap();
    assert_eq!(snapshot.status, SnapshotStatus::Used);

    // 标记为已回滚
    snapshot_manager.mark_rolled_back(&id).await.unwrap();
    let snapshot = snapshot_manager.get_snapshot(&id).await.unwrap();
    assert_eq!(snapshot.status, SnapshotStatus::RolledBack);
}

/// 测试批量验证集成
#[tokio::test]
async fn test_batch_verification() {
    let snapshot_manager = Arc::new(SnapshotManager::new());
    let executor = RollbackExecutor::new(snapshot_manager.clone());
    let verifier = RollbackVerifier::new(snapshot_manager.clone());

    let mut rollback_results = Vec::new();

    // 创建并执行多个回滚
    for i in 0..3 {
        let id = snapshot_manager
            .create_snapshot_with_rollback(
                format!("op-batch-{}", i),
                SnapshotType::Full,
                serde_json::json!({"index": i}),
                serde_json::json!({"rollback": i}),
            )
            .await
            .unwrap();

        let result = executor.execute(&id).await.unwrap();
        rollback_results.push(result);
    }

    // 批量验证
    let verifications = verifier.verify_batch(&rollback_results).await.unwrap();
    assert_eq!(verifications.len(), 3);

    for v in &verifications {
        assert!(v.is_passed());
    }
}

/// 测试执行历史集成
#[tokio::test]
async fn test_execution_history() {
    let snapshot_manager = Arc::new(SnapshotManager::new());
    let executor = RollbackExecutor::new(snapshot_manager.clone());

    // 执行多次回滚
    for i in 0..3 {
        let id = snapshot_manager
            .create_snapshot_with_rollback(
                format!("op-history-{}", i),
                SnapshotType::Full,
                serde_json::json!({}),
                serde_json::json!({}),
            )
            .await
            .unwrap();

        executor.execute(&id).await.unwrap();
    }

    // 检查执行历史
    assert_eq!(executor.execution_count().await, 3);
    let history = executor.get_history().await;
    assert_eq!(history.len(), 3);

    // 清空历史
    executor.clear_history().await;
    assert_eq!(executor.execution_count().await, 0);
}

/// 测试快速验证集成
#[tokio::test]
async fn test_quick_verification() {
    let snapshot_manager = Arc::new(SnapshotManager::new());
    let executor = RollbackExecutor::new(snapshot_manager.clone());
    let verifier = RollbackVerifier::new(snapshot_manager.clone());

    let id = snapshot_manager
        .create_snapshot_with_rollback(
            "op-quick".to_string(),
            SnapshotType::Full,
            serde_json::json!({}),
            serde_json::json!({}),
        )
        .await
        .unwrap();

    let rollback_result = executor.execute(&id).await.unwrap();

    // 快速验证
    let passed = verifier.quick_verify(&rollback_result).await.unwrap();
    assert!(passed);
}

/// 测试操作快照管理集成
#[tokio::test]
async fn test_operation_snapshot_management() {
    let snapshot_manager = Arc::new(SnapshotManager::new());

    // 为同一操作创建多个快照
    for i in 0..5 {
        snapshot_manager
            .create_snapshot(
                "op-same".to_string(),
                SnapshotType::Incremental,
                serde_json::json!({"step": i}),
            )
            .await
            .unwrap();
    }

    // 获取操作所有快照
    let snapshots = snapshot_manager.get_operation_snapshots("op-same").await.unwrap();
    assert_eq!(snapshots.len(), 5);

    // 获取最新快照
    let latest = snapshot_manager.get_latest_snapshot("op-same").await.unwrap();
    assert_eq!(latest.snapshot_type, SnapshotType::Incremental);
}

/// 测试回滚动作类型集成
#[test]
fn test_rollback_action_types() {
    use fos_rollback::RollbackAction;

    // 测试各种动作类型
    let data_restore =
        RollbackAction::new("snap-001".to_string(), RollbackActionType::DataRestore, 1);
    assert_eq!(data_restore.action_type, RollbackActionType::DataRestore);
    assert_eq!(data_restore.status, RollbackActionStatus::Pending);

    let state_reset =
        RollbackAction::new("snap-001".to_string(), RollbackActionType::StateReset, 2);
    assert_eq!(state_reset.action_type, RollbackActionType::StateReset);

    let cleanup =
        RollbackAction::new("snap-001".to_string(), RollbackActionType::ResourceCleanup, 3);
    assert_eq!(cleanup.action_type, RollbackActionType::ResourceCleanup);

    let notification =
        RollbackAction::new("snap-001".to_string(), RollbackActionType::NotificationSend, 4);
    assert_eq!(notification.action_type, RollbackActionType::NotificationSend);

    let custom = RollbackAction::new(
        "snap-001".to_string(),
        RollbackActionType::Custom("custom".to_string()),
        5,
    );
    assert_eq!(custom.action_type, RollbackActionType::Custom("custom".to_string()));
}
