//! FOS 数据一致性集成测试

mod transaction_lock {
    use fos_lock::LockManager;
    use fos_transaction::{Participant, TransactionCoordinator};

    #[test]
    fn test_transaction_with_lock() {
        let coordinator = TransactionCoordinator::with_defaults();
        let lock_manager = LockManager::with_defaults();

        let tx_id = coordinator.begin("transfer_order").unwrap();
        let lock_id = lock_manager.try_lock("order:123", "tx_handler").unwrap();
        assert!(lock_id.is_some());

        coordinator.add_participant(tx_id, Participant::new("inventory", 1)).unwrap();
        coordinator.commit(tx_id).unwrap();
        lock_manager.unlock("order:123", "tx_handler").unwrap();
    }

    #[test]
    fn test_concurrent_transaction_lock_conflict() {
        let lock_manager = LockManager::with_defaults();

        let lock1 = lock_manager.try_lock("resource:1", "tx1").unwrap();
        assert!(lock1.is_some());

        let lock2 = lock_manager.try_lock("resource:1", "tx2").unwrap();
        assert!(lock2.is_none());

        lock_manager.unlock("resource:1", "tx1").unwrap();
        let lock3 = lock_manager.try_lock("resource:1", "tx2").unwrap();
        assert!(lock3.is_some());
    }

    #[test]
    fn test_rollback_releases_lock() {
        let coordinator = TransactionCoordinator::with_defaults();
        let lock_manager = LockManager::with_defaults();

        lock_manager.try_lock("resource:1", "owner1").unwrap();
        assert!(lock_manager.is_locked("resource:1"));

        let tx_id = coordinator.begin("test_tx").unwrap();
        coordinator.rollback(tx_id).unwrap();

        // 锁仍然存在（需要显式释放）
        assert!(lock_manager.is_locked("resource:1"));
        lock_manager.unlock("resource:1", "owner1").unwrap();
        assert!(!lock_manager.is_locked("resource:1"));
    }
}

mod transaction_idempotency {
    use fos_idempotency::IdempotencyManager;
    use fos_transaction::TransactionCoordinator;

    #[test]
    fn test_idempotent_transaction() {
        let _coordinator = TransactionCoordinator::with_defaults();
        let idempotency = IdempotencyManager::with_defaults();

        let result: i32 = idempotency.execute("tx-key-1", "order", "create", || 42).unwrap();
        assert_eq!(result, 42);

        let result2: i32 = idempotency.execute("tx-key-1", "order", "create", || 100).unwrap();
        assert_eq!(result2, 42);
    }

    #[test]
    fn test_transaction_idempotency_status() {
        let idempotency = IdempotencyManager::with_defaults();

        let check = idempotency.check("tx-123", "order", "create").unwrap();
        assert!(check.is_first_time());

        idempotency.mark_processing("tx-123").unwrap();
        let check2 = idempotency.check("tx-123", "order", "create").unwrap();
        assert!(check2.is_processing());

        idempotency.mark_completed("tx-123").unwrap();
        let check3 = idempotency.check("tx-123", "order", "create").unwrap();
        assert!(check3.is_duplicate());
    }

    #[test]
    fn test_failed_transaction_retry() {
        let idempotency = IdempotencyManager::with_defaults();

        idempotency.check("retry-tx", "order", "create").unwrap();
        idempotency.mark_processing("retry-tx").unwrap();
        idempotency.mark_failed("retry-tx").unwrap();

        let check2 = idempotency.check("retry-tx", "order", "create").unwrap();
        assert!(check2.is_retry());
    }
}

mod lock_idempotency {
    use fos_idempotency::IdempotencyManager;
    use fos_lock::LockManager;

    #[test]
    fn test_lock_protected_idempotency() {
        let lock_manager = LockManager::with_defaults();
        let idempotency = IdempotencyManager::with_defaults();

        let lock_id = lock_manager.try_lock("resource:1", "worker1").unwrap();
        assert!(lock_id.is_some());

        let result: i32 = idempotency.execute("op-1", "resource", "process", || 100).unwrap();
        assert_eq!(result, 100);

        lock_manager.unlock("resource:1", "worker1").unwrap();
    }

    #[test]
    fn test_idempotency_lock_independence() {
        let lock_manager = LockManager::with_defaults();
        let idempotency = IdempotencyManager::with_defaults();

        let check = idempotency.check("key-1", "res", "op").unwrap();
        assert!(check.is_first_time());

        lock_manager.try_lock("resource:1", "owner1").unwrap();
        let check2 = idempotency.check("key-2", "res", "op").unwrap();
        assert!(check2.is_first_time());

        lock_manager.unlock("resource:1", "owner1").unwrap();
    }

    #[test]
    fn test_concurrent_idempotent_operations() {
        let lock_manager = LockManager::with_defaults();

        let lock1 = lock_manager.try_lock("shared-resource", "worker1").unwrap();
        assert!(lock1.is_some());

        let lock2 = lock_manager.try_lock("shared-resource", "worker2").unwrap();
        assert!(lock2.is_none());

        let stats = lock_manager.get_stats();
        assert_eq!(stats.acquire_count, 2);
    }
}

mod full_consistency {
    use fos_idempotency::IdempotencyManager;
    use fos_lock::LockManager;
    use fos_transaction::{Participant, TransactionCoordinator};

    #[test]
    fn test_full_transaction_flow() {
        let coordinator = TransactionCoordinator::with_defaults();
        let lock_manager = LockManager::with_defaults();
        let idempotency = IdempotencyManager::with_defaults();

        let check = idempotency.check("full-tx-1", "order", "transfer").unwrap();
        assert!(check.is_first_time());

        let lock = lock_manager.try_lock("order:full-tx-1", "transaction_manager").unwrap();
        assert!(lock.is_some());

        let tx_id = coordinator.begin("full_transaction").unwrap();
        coordinator.add_participant(tx_id, Participant::new("inventory", 1)).unwrap();
        coordinator.add_participant(tx_id, Participant::new("payment", 2)).unwrap();
        coordinator.commit(tx_id).unwrap();

        idempotency.store_result("full-tx-1", serde_json::json!({"status": "completed"}));
        idempotency.mark_completed("full-tx-1").unwrap();
        lock_manager.unlock("order:full-tx-1", "transaction_manager").unwrap();

        let check2 = idempotency.check("full-tx-1", "order", "transfer").unwrap();
        assert!(check2.is_duplicate());
    }

    #[test]
    fn test_full_rollback_flow() {
        let coordinator = TransactionCoordinator::with_defaults();
        let lock_manager = LockManager::with_defaults();
        let idempotency = IdempotencyManager::with_defaults();

        idempotency.check("rollback-tx", "order", "cancel").unwrap();
        idempotency.mark_processing("rollback-tx").unwrap();

        lock_manager.try_lock("order:rollback-tx", "manager").unwrap();

        let tx_id = coordinator.begin("rollback_test").unwrap();
        coordinator.add_participant(tx_id, Participant::new("service1", 1)).unwrap();
        coordinator.rollback(tx_id).unwrap();

        idempotency.mark_failed("rollback-tx").unwrap();
        lock_manager.unlock("order:rollback-tx", "manager").unwrap();

        let check = idempotency.check("rollback-tx", "order", "cancel").unwrap();
        assert!(check.is_retry());
    }

    #[test]
    fn test_resource_cleanup() {
        let coordinator = TransactionCoordinator::with_defaults();
        let lock_manager = LockManager::with_defaults();
        let idempotency = IdempotencyManager::with_defaults();

        let tx_id = coordinator.begin("cleanup_test").unwrap();
        coordinator.rollback(tx_id).unwrap();

        lock_manager.try_lock("temp:1", "cleanup").unwrap();
        lock_manager.unlock("temp:1", "cleanup").unwrap();

        idempotency.check("temp-key", "temp", "op").unwrap();

        let _ = coordinator.cleanup_completed().unwrap();
        let _ = lock_manager.cleanup_expired().unwrap();
        let _ = idempotency.cleanup();
    }

    #[test]
    fn test_aggregated_stats() {
        let coordinator = TransactionCoordinator::with_defaults();
        let lock_manager = LockManager::with_defaults();
        let idempotency = IdempotencyManager::with_defaults();

        let tx_id = coordinator.begin("stats_test").unwrap();
        coordinator.rollback(tx_id).unwrap();

        lock_manager.try_lock("stats:1", "test").unwrap();
        lock_manager.unlock("stats:1", "test").unwrap();

        idempotency.check("stats-key", "stats", "test").unwrap();

        let lock_stats = lock_manager.get_stats();
        let idemp_stats = idempotency.get_stats();

        assert!(lock_stats.acquire_count > 0);
        assert!(idemp_stats.checker.total_checks > 0);
    }
}

mod performance {
    use fos_idempotency::IdempotencyManager;
    use fos_lock::LockManager;
    use fos_transaction::TransactionCoordinator;
    use std::time::Instant;

    #[test]
    fn test_transaction_performance() {
        let coordinator = TransactionCoordinator::with_defaults();
        let start = Instant::now();

        for i in 0..10 {
            let tx_id = coordinator.begin(&format!("perf_tx_{}", i)).unwrap();
            coordinator.rollback(tx_id).unwrap();
        }

        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() < 100);
    }

    #[test]
    fn test_lock_performance() {
        let lock_manager = LockManager::with_defaults();
        let start = Instant::now();

        for i in 0..10 {
            let key = format!("perf_lock_{}", i);
            lock_manager.try_lock(&key, "perf_test").unwrap();
            lock_manager.unlock(&key, "perf_test").unwrap();
        }

        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() < 50);
    }

    #[test]
    fn test_idempotency_performance() {
        let idempotency = IdempotencyManager::with_defaults();
        let start = Instant::now();

        for i in 0..10 {
            idempotency.check(&format!("perf_key_{}", i), "resource", "op").unwrap();
        }

        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() < 50);
    }
}
