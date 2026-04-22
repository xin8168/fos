//! 迁移管理器

use crate::error::{MigrationError, Result as MigrationResult};
use crate::version::{MigrationDirection, MigrationRecord, MigrationStatus, MigrationVersion};

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 迁移执行器 trait
#[async_trait::async_trait]
pub trait MigrationExecutor: Send + Sync {
    /// 执行向上迁移
    async fn up(&self, version: &str) -> MigrationResult<()>;

    /// 执行向下迁移
    async fn down(&self, version: &str) -> MigrationResult<()>;
}

/// 迁移管理器
pub struct MigrationManager<E: MigrationExecutor> {
    /// 迁移注册表
    registry: Arc<RwLock<HashMap<String, MigrationVersion>>>,
    /// 已执行的迁移记录
    records: Arc<RwLock<HashMap<String, Vec<MigrationRecord>>>>,
    /// 执行器
    executor: Arc<E>,
    /// 当前版本
    current_version: Arc<RwLock<Option<String>>>,
    /// 自动应用最新版本
    #[allow(dead_code)]
    auto_migrate: bool,
}

impl<E: MigrationExecutor> MigrationManager<E> {
    /// 创建新的迁移管理器
    pub fn new(executor: E) -> Self {
        Self {
            registry: Arc::new(RwLock::new(HashMap::new())),
            records: Arc::new(RwLock::new(HashMap::new())),
            executor: Arc::new(executor),
            current_version: Arc::new(RwLock::new(None)),
            auto_migrate: false,
        }
    }

    /// 创建配置好的迁移管理器
    pub fn with_config(_executor: E, _auto_migrate: bool) -> Self {
        Self {
            registry: Arc::new(RwLock::new(HashMap::new())),
            records: Arc::new(RwLock::new(HashMap::new())),
            executor: Arc::new(_executor),
            current_version: Arc::new(RwLock::new(None)),
            auto_migrate: _auto_migrate,
        }
    }

    /// 注册迁移
    pub async fn register(&self, migration: MigrationVersion) -> MigrationResult<()> {
        let mut registry = self.registry.write().await;

        // 检查重复
        if registry.contains_key(&migration.version) {
            return Err(MigrationError::AlreadyRegistered(migration.version));
        }

        // 验证依赖
        for dep in &migration.dependencies {
            if !registry.contains_key(dep) {
                return Err(MigrationError::DependencyNotFound(dep.clone()));
            }
        }

        registry.insert(migration.version.clone(), migration);
        Ok(())
    }

    /// 批量注册迁移
    pub async fn register_batch(&self, migrations: Vec<MigrationVersion>) -> MigrationResult<()> {
        for migration in migrations {
            self.register(migration).await?;
        }
        Ok(())
    }

    /// 获取所有已注册的迁移
    pub async fn get_all_migrations(&self) -> Vec<MigrationVersion> {
        let registry = self.registry.read().await;
        let mut versions: Vec<MigrationVersion> = registry.values().cloned().collect();
        versions.sort();
        versions
    }

    /// 获取待执行的迁移
    pub async fn get_pending_migrations(&self) -> Vec<MigrationVersion> {
        let registry = self.registry.read().await;
        let records = self.records.read().await;
        let current_version = self.current_version.read().await;

        let current = current_version.as_deref().unwrap_or("");

        let pending: Vec<MigrationVersion> = registry
            .values()
            .filter(|v| v.version.as_str() > current)
            .filter(|v| {
                // 检查是否已执行
                !records
                    .get(&v.version)
                    .map(|recs| recs.iter().any(|r| r.status == MigrationStatus::Completed))
                    .unwrap_or(false)
            })
            .cloned()
            .collect();

        pending
    }

    /// 执行迁移
    pub async fn migrate(&self) -> MigrationResult<Vec<MigrationRecord>> {
        let pending = self.get_pending_migrations().await;

        if pending.is_empty() {
            return Ok(vec![]);
        }

        let mut results = Vec::new();

        for version in pending {
            let record = self.migrate_to(&version.version).await?;
            results.push(record);
        }

        Ok(results)
    }

    /// 迁移到特定版本
    pub async fn migrate_to(&self, target_version: &str) -> MigrationResult<MigrationRecord> {
        // 检查目标版本是否已注册
        let registry = self.registry.read().await;
        if !registry.contains_key(target_version) {
            return Err(MigrationError::VersionNotFound(target_version.to_string()));
        }
        drop(registry);

        // 检查是否已执行
        let records = self.records.read().await;
        if let Some(recs) = records.get(target_version) {
            if recs.iter().any(|r| r.status == MigrationStatus::Completed) {
                return Err(MigrationError::AlreadyMigrated(target_version.to_string()));
            }
        }
        drop(records);

        // 执行迁移
        let mut record = MigrationRecord::new(target_version.to_string(), MigrationDirection::Up);
        record.mark_running();

        self.executor.up(target_version).await?;

        record.mark_completed();

        // 更新记录
        {
            let mut records = self.records.write().await;
            records.entry(target_version.to_string()).or_insert_with(Vec::new).push(record.clone());

            // 更新当前版本
            let mut current_version = self.current_version.write().await;
            *current_version = Some(target_version.to_string());
        }

        Ok(record)
    }

    /// 回滚迁移
    pub async fn rollback(&self, version: &str) -> MigrationResult<MigrationRecord> {
        // 检查版本是否已执行
        let records = self.records.read().await;
        let version_record = records.get(version).and_then(|recs| {
            recs.iter().find(|r| {
                r.direction == MigrationDirection::Up && r.status == MigrationStatus::Completed
            })
        });

        if version_record.is_none() {
            return Err(MigrationError::CannotRollback(version.to_string()));
        }
        drop(records);

        // 执行回滚
        let mut record = MigrationRecord::new(version.to_string(), MigrationDirection::Down);
        record.mark_running();

        self.executor.down(version).await?;

        record.mark_rolled_back();

        // 更新记录
        {
            let mut records = self.records.write().await;
            records.entry(version.to_string()).or_insert_with(Vec::new).push(record.clone());
        }

        Ok(record)
    }

    /// 回滚到特定版本
    pub async fn rollback_to(&self, target_version: &str) -> MigrationResult<Vec<MigrationRecord>> {
        let current_version = self.current_version.read().await;
        let current = current_version.as_deref().ok_or(MigrationError::NoMigration)?.to_string();
        drop(current_version);

        if target_version >= current.as_str() {
            return Err(MigrationError::InvalidVersionDirection);
        }

        // 获取需要回滚的版本（按降序）
        let registry = self.registry.read().await;
        let target = target_version.to_string();
        let current_string = current.to_string();

        // 收集所有需要回滚的版本（不过滤 rollback，因为 migrate 时已检查过）
        let mut versions: Vec<MigrationVersion> = registry
            .values()
            .filter(|v| v.version > target && v.version <= current_string)
            .cloned()
            .collect();
        versions.sort_by(|a, b| b.version.cmp(&a.version));
        drop(registry);

        let mut results = Vec::new();

        for version in &versions {
            let record = self.rollback(&version.version).await?;
            results.push(record);
        }

        // 更新当前版本为目标版本
        let mut current_version = self.current_version.write().await;
        *current_version = Some(target_version.to_string());

        Ok(results)
    }

    /// 获取当前版本
    pub async fn current_version(&self) -> Option<String> {
        self.current_version.read().await.clone()
    }

    /// 检查版本状态
    pub async fn check_version(&self, version: &str) -> MigrationVersionStatus {
        let registry = self.registry.read().await;
        let records = self.records.read().await;

        if !registry.contains_key(version) {
            return MigrationVersionStatus::NotFound;
        }

        if let Some(recs) = records.get(version) {
            if recs.iter().any(|r| r.status == MigrationStatus::Completed) {
                return MigrationVersionStatus::Migrated;
            }
        }

        MigrationVersionStatus::Pending
    }

    /// 获取版本信息
    pub async fn get_version(&self, version: &str) -> Option<MigrationVersion> {
        self.registry.read().await.get(version).cloned()
    }

    /// 获取所有执行记录
    pub async fn get_records(&self) -> Vec<MigrationRecord> {
        let records = self.records.read().await;
        records.values().flatten().cloned().collect()
    }

    /// 清理已回滚的记录
    pub async fn cleanup_rollback_records(&self) -> MigrationResult<usize> {
        let mut total = 0;

        {
            let mut records = self.records.write().await;
            for (_, recs) in records.iter_mut() {
                let original_len = recs.len();
                recs.retain(|r| r.status != MigrationStatus::RolledBack);
                total += original_len - recs.len();
            }
        }

        Ok(total)
    }
}

/// 迁移版本状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationVersionStatus {
    /// 版本未找到
    NotFound,
    /// 等待迁移
    Pending,
    /// 已迁移
    Migrated,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // 测试用的迁移执行器
    struct TestExecutor {
        up_count: AtomicUsize,
        down_count: AtomicUsize,
    }

    impl TestExecutor {
        fn new() -> Self {
            Self { up_count: AtomicUsize::new(0), down_count: AtomicUsize::new(0) }
        }
    }

    #[async_trait::async_trait]
    impl MigrationExecutor for TestExecutor {
        async fn up(&self, _version: &str) -> MigrationResult<()> {
            self.up_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        async fn down(&self, _version: &str) -> MigrationResult<()> {
            self.down_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_register_migration() {
        let executor = TestExecutor::new();
        let manager = MigrationManager::new(executor);

        let migration = MigrationVersion::new(
            "20240313120000".to_string(),
            "Test migration".to_string(),
            "Test".to_string(),
        );

        assert!(manager.register(migration).await.is_ok());
    }

    #[tokio::test]
    async fn test_register_duplicate() {
        let executor = TestExecutor::new();
        let manager = MigrationManager::new(executor);

        let migration = MigrationVersion::new(
            "20240313120000".to_string(),
            "Test migration".to_string(),
            "Test".to_string(),
        );

        assert!(manager.register(migration.clone()).await.is_ok());
        assert!(matches!(
            manager.register(migration).await,
            Err(MigrationError::AlreadyRegistered(_))
        ));
    }

    #[tokio::test]
    async fn test_register_with_dependency() {
        let executor = TestExecutor::new();
        let manager = MigrationManager::new(executor);

        let v1 = MigrationVersion::new(
            "20240313000000".to_string(),
            "Base migration".to_string(),
            "Test".to_string(),
        );

        let v2 = MigrationVersion::new(
            "20240313120000".to_string(),
            "Dependent migration".to_string(),
            "Test".to_string(),
        )
        .with_dependency("20240313000000".to_string());

        assert!(manager.register(v1).await.is_ok());
        assert!(manager.register(v2).await.is_ok());
    }

    #[tokio::test]
    async fn test_register_missing_dependency() {
        let executor = TestExecutor::new();
        let manager = MigrationManager::new(executor);

        let migration = MigrationVersion::new(
            "20240313120000".to_string(),
            "Dependent migration".to_string(),
            "Test".to_string(),
        )
        .with_dependency("nonexistent".to_string());

        assert!(matches!(
            manager.register(migration).await,
            Err(MigrationError::DependencyNotFound(_))
        ));
    }

    #[tokio::test]
    async fn test_migrate_to_version() {
        let executor = TestExecutor::new();
        let manager = MigrationManager::new(executor);

        let migration = MigrationVersion::new(
            "20240313120000".to_string(),
            "Test migration".to_string(),
            "Test".to_string(),
        );

        manager.register(migration).await.unwrap();

        let record = manager.migrate_to("20240313120000").await.unwrap();

        assert_eq!(record.version, "20240313120000");
        assert_eq!(record.status, MigrationStatus::Completed);
        assert_eq!(manager.current_version().await, Some("20240313120000".to_string()));
    }

    #[tokio::test]
    async fn test_migrate_auto() {
        let executor = TestExecutor::new();
        let manager = MigrationManager::new(executor);

        let v1 = MigrationVersion::new("00001".to_string(), "V1".to_string(), "Test".to_string());
        let v2 = MigrationVersion::new("00002".to_string(), "V2".to_string(), "Test".to_string());

        manager.register_batch(vec![v1, v2]).await.unwrap();

        let results = manager.migrate().await.unwrap();

        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_rollback() {
        let executor = TestExecutor::new();
        let manager = MigrationManager::new(executor);

        let migration = MigrationVersion::new(
            "20240313120000".to_string(),
            "Test migration".to_string(),
            "Test".to_string(),
        );

        manager.register(migration).await.unwrap();
        manager.migrate_to("20240313120000").await.unwrap();

        let record = manager.rollback("20240313120000").await.unwrap();

        assert_eq!(record.status, MigrationStatus::RolledBack);
    }

    #[tokio::test]
    async fn test_rollback_to_version() {
        let executor = TestExecutor::new();
        let manager = MigrationManager::new(executor);

        let v1 = MigrationVersion::new("00001".to_string(), "V1".to_string(), "Test".to_string());
        let v2 = MigrationVersion::new("00002".to_string(), "V2".to_string(), "Test".to_string());
        let v3 = MigrationVersion::new("00003".to_string(), "V3".to_string(), "Test".to_string());

        manager.register_batch(vec![v1, v2, v3]).await.unwrap();
        manager.migrate().await.unwrap();

        let results = manager.rollback_to("00001").await.unwrap();

        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_check_version_status() {
        let executor = TestExecutor::new();
        let manager = MigrationManager::new(executor);

        let migration = MigrationVersion::new(
            "20240313120000".to_string(),
            "Test migration".to_string(),
            "Test".to_string(),
        );

        manager.register(migration).await.unwrap();

        assert_eq!(manager.check_version("20240313120000").await, MigrationVersionStatus::Pending);

        manager.migrate_to("20240313120000").await.unwrap();

        assert_eq!(manager.check_version("20240313120000").await, MigrationVersionStatus::Migrated);
    }
}
