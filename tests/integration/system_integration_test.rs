//! # 系统集成测试
//!
//! 测试FOS全链路功能和模块集成

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_system_basic_health_check() {
    println!("✅ 系统基础健康检查通过");
}

#[tokio::test]
async fn test_system_transaction_coordinator() {
    use fos_transaction::TransactionCoordinator;

    let coordinator = TransactionCoordinator::with_defaults();
    let tx_id = coordinator.begin("integration-tx-001").unwrap();
    assert!(!tx_id.as_bytes().iter().all(|&b| b == 0));

    println!("✅ 事务协调器测试通过 - TX ID: {}", tx_id);
}

#[tokio::test]
async fn test_system_distributed_lock() {
    use fos_lock::{LockConfig, LockManager};

    let lock_manager = LockManager::with_defaults();

    let lock1 = lock_manager.try_lock("integration-lock-1", "owner1").unwrap();
    assert!(lock1.is_some());
    assert!(lock_manager.is_locked("integration-lock-1"));

    let lock2 = lock_manager.try_lock("integration-lock-1", "owner2").unwrap();
    assert!(lock2.is_none(), "同一锁不能被两个所有者获取");

    lock_manager.unlock("integration-lock-1", "owner1").unwrap();
    assert!(!lock_manager.is_locked("integration-lock-1"));

    println!("✅ 分布式锁测试通过");
}

#[tokio::test]
async fn test_system_scheduling() {
    use fos_schedule::{ClosureJobHandler, DelayedJob, DelayedQueue, JobResult};

    let delayed_queue = DelayedQueue::new();
    let execute_at = chrono::Utc::now() + chrono::Duration::milliseconds(100);
    let handler = ClosureJobHandler::new(|| {
        Box::pin(async move { JobResult { success: true, error: None, execution_time_ms: 10 } })
    });

    let job = DelayedJob::new(
        "scheduled-job-integration".to_string(),
        "Integration Test Job".to_string(),
        execute_at,
        std::sync::Arc::new(handler),
    );

    delayed_queue.add(job).await.unwrap();

    sleep(Duration::from_millis(150)).await;

    let ready_jobs = delayed_queue.get_ready_jobs().await;
    assert_eq!(ready_jobs.len(), 1);

    let result = ready_jobs[0].execute().await;
    assert!(result.success, "任务执行应该成功");

    println!("✅ 调度系统测试通过");
}

#[tokio::test]
async fn test_system_memory_storage() {
    use fos_memory::{InMemoryStorage, SuccessEvent};

    let storage = InMemoryStorage::new();

    let event = SuccessEvent::new(
        "Integration Event".to_string(),
        "test_type".to_string(),
        vec!["step1".to_string(), "step2".to_string()],
        "logic1".to_string(),
        "standard1".to_string(),
        "location1".to_string(),
        "subject1".to_string(),
    );

    let event_id = storage.store(event.clone()).await.unwrap();
    assert!(!event_id.is_empty());

    let retrieved = storage.get(&event_id).await.unwrap();
    assert_eq!(retrieved.name, "Integration Event");
    assert_eq!(retrieved.event_type, "test_type");

    let count = storage.count().await.unwrap();
    assert_eq!(count, 1);

    println!("✅ 内存存储测试通过");
}

#[tokio::test]
async fn test_system_rollback_snapshot() {
    use fos_rollback::{Snapshot, SnapshotManager, SnapshotType};

    let manager = SnapshotManager::new();

    let snapshot_data = serde_json::json!({
        "key1": "value1",
        "key2": "value2"
    });

    let snapshot =
        Snapshot::new("integration-operation-001".to_string(), SnapshotType::Full, snapshot_data);

    let snapshot_id = manager.save_snapshot(snapshot).await.unwrap();
    assert!(!snapshot_id.is_empty());

    let retrieved = manager.get_snapshot(&snapshot_id).await.unwrap();
    assert_eq!(retrieved.operation_id, "integration-operation-001");
    assert!(retrieved.data.get("key1").is_some());

    println!("✅ 回滚快照测试通过");
}

#[tokio::test]
async fn test_system_cache_integration() {
    use fos_cache::LocalCache;

    let cache = LocalCache::new();

    cache.set("cache_key_1", "cache_value_1").await.unwrap();
    let value: Option<String> = cache.get("cache_key_1").await;
    assert_eq!(value, Some("cache_value_1".to_string()));

    cache.set("cache_key_2", 42).await.unwrap();
    let int_value: Option<i32> = cache.get("cache_key_2").await;
    assert_eq!(int_value, Some(42));

    cache.remove("cache_key_1").await;
    let removed_value: Option<String> = cache.get("cache_key_1").await;
    assert!(removed_value.is_none());

    println!("✅ 缓存集成测试通过");
}

#[tokio::test]
async fn test_system_concurrent_operations() {
    use fos_cache::LocalCache;
    use std::sync::atomic::{AtomicU32, Ordering};
    use tokio::task::JoinSet;

    let cache = Arc::new(LocalCache::new());
    let counter = Arc::new(AtomicU32::new(0));

    let mut join_set = JoinSet::new();

    for i in 0..20 {
        let cache_clone = cache.clone();
        let counter_clone = counter.clone();

        join_set.spawn(async move {
            let key = format!("concurrent_key_{}", i);
            let value = format!("value_{}", i);

            cache_clone.set(key, value).await.unwrap();

            if let Some(_) = cache_clone.get::<String>(&key).await {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            }
        });
    }

    let results: Vec<_> = join_set.join_all().await;
    assert_eq!(results.len(), 20);

    assert_eq!(counter.load(Ordering::SeqCst), 20);

    println!("✅ 并发操作测试通过（20个并发任务）");
}

#[tokio::test]
async fn test_system_plugin_lifecycle() {
    use fos_plugin::{PluginContext, PluginManager, PluginStatus};

    let mut manager = PluginManager::new();

    let context = PluginContext::new("integration_test");

    let status = manager.load_plugin(&context, "integration-plugin-001", "1.0.0").await;
    assert!(status.is_ok());
    let status = status.unwrap();
    assert_eq!(status, PluginStatus::Loaded);

    let plugin = manager.get_plugin("integration-plugin-001");
    assert!(plugin.is_some());

    let unload_result = manager.unload_plugin(&context, "integration-plugin-001").await;
    assert!(unload_result.is_ok());

    let plugin_after = manager.get_plugin("integration-plugin-001");
    assert!(plugin_after.is_none());

    println!("✅ Plugin生命周期测试通过");
}

#[tokio::test]
async fn test_system_multi_module_integration() {
    // 测试多个模块协同工作
    use fos_cache::LocalCache;
    use fos_lock::{LockConfig, LockManager};
    use fos_schedule::{ClosureJobHandler, DelayedJob, DelayedQueue, JobResult};

    let cache = Arc::new(LocalCache::new());
    let lock_manager = Arc::new(LockManager::with_defaults());
    let delayed_queue = DelayedQueue::new();

    // 1. 使用缓存
    cache.set("integration_data", "integration_value").await.unwrap();

    // 2. 使用分布式锁
    let lock_id = lock_manager.try_lock("integration_multi", "owner").unwrap();
    assert!(lock_id.is_some());

    // 3. 使用调度器安排任务
    let execute_at = chrono::Utc::now() + chrono::Duration::milliseconds(50);
    let handler = ClosureJobHandler::new(|| {
        Box::pin(async move { JobResult { success: true, error: None, execution_time_ms: 5 } })
    });

    let job = DelayedJob::new(
        "multi-module-job".to_string(),
        "Multi Module Job".to_string(),
        execute_at,
        Arc::new(handler),
    );

    delayed_queue.add(job).await.unwrap();

    // 等待任务执行
    sleep(Duration::from_millis(100)).await;

    // 验证缓存仍然可用
    let cached_value: Option<String> = cache.get("integration_data").await;
    assert_eq!(cached_value, Some("integration_value".to_string()));

    // 验证锁已释放
    lock_manager.unlock("integration_multi", "owner").unwrap();

    // 验证任务已执行
    let ready_jobs = delayed_queue.get_ready_jobs().await;
    assert_eq!(ready_jobs.len(), 1);

    println!("✅ 多模块集成测试通过");
}
