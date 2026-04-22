//! Migration 模块集成测试

use fos_migration::{
    MigrationExecutor, MigrationManager, MigrationResult, MigrationStatus, MigrationVersion,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[tokio::test]
async fn test_migration_integration_basic() {
    struct SimpleExecutor {
        counter: Arc<AtomicUsize>,
    }

    impl SimpleExecutor {
        fn new() -> Self {
            Self { counter: Arc::new(AtomicUsize::new(0)) }
        }
    }

    #[async_trait::async_trait]
    impl MigrationExecutor for SimpleExecutor {
        async fn up(&self, _version: &str) -> MigrationResult<()> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        async fn down(&self, _version: &str) -> MigrationResult<()> {
            Ok(())
        }
    }

    let executor = SimpleExecutor::new();
    let manager = MigrationManager::new(executor);

    let v1 = MigrationVersion::new("00001".to_string(), "V1".to_string(), "Dev".to_string());
    manager.register(v1).await.unwrap();

    let record = manager.migrate_to("00001").await.unwrap();
    assert_eq!(record.status, MigrationStatus::Completed);
}

#[tokio::test]
async fn test_migration_dependency_chain() {
    struct SimpleExecutor {
        counter: Arc<AtomicUsize>,
    }

    impl SimpleExecutor {
        fn new() -> Self {
            Self { counter: Arc::new(AtomicUsize::new(0)) }
        }
    }

    #[async_trait::async_trait]
    impl MigrationExecutor for SimpleExecutor {
        async fn up(&self, _version: &str) -> MigrationResult<()> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        async fn down(&self, _version: &str) -> MigrationResult<()> {
            Ok(())
        }
    }

    let executor = SimpleExecutor::new();
    let manager = MigrationManager::new(executor);

    let v1 = MigrationVersion::new("00001".to_string(), "Base".to_string(), "Dev".to_string());
    let v2 = MigrationVersion::new("00002".to_string(), "Dep".to_string(), "Dev".to_string())
        .with_dependency("00001".to_string());

    manager.register_batch(vec![v1, v2]).await.unwrap();
    let records = manager.migrate().await.unwrap();

    assert_eq!(records.len(), 2);
}

#[tokio::test]
async fn test_rollback_capabilities() {
    struct SimpleExecutor {
        counter: Arc<AtomicUsize>,
    }

    impl SimpleExecutor {
        fn new() -> Self {
            Self { counter: Arc::new(AtomicUsize::new(0)) }
        }
    }

    #[async_trait::async_trait]
    impl MigrationExecutor for SimpleExecutor {
        async fn up(&self, _version: &str) -> MigrationResult<()> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        async fn down(&self, _version: &str) -> MigrationResult<()> {
            Ok(())
        }
    }

    let executor = SimpleExecutor::new();
    let manager = MigrationManager::new(executor);

    let migration = MigrationVersion::new("00001".to_string(), "V1".to_string(), "Dev".to_string())
        .with_rollback(true);

    manager.register(migration).await.unwrap();
    manager.migrate_to("00001").await.unwrap();

    let rollback_record = manager.rollback("00001").await.unwrap();
    assert!(format!("{:?}", rollback_record.status).contains("RolledBack"));
}
