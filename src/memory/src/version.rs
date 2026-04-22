//! # 版本管理模块
//!
//! 负责成功事件的版本控制和历史追踪

use crate::error::{MemoryError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 版本号
pub type Version = u64;

/// 事件版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventVersion {
    /// 版本号
    pub version: Version,

    /// 事件ID
    pub event_id: String,

    /// 变更描述
    pub change_description: String,

    /// 变更类型
    pub change_type: ChangeType,

    /// 变更时间
    pub changed_at: DateTime<Utc>,

    /// 变更者
    pub changed_by: String,
}

/// 变更类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    /// 创建
    Created,

    /// 内容更新
    ContentUpdated,

    /// 元数据更新
    MetadataUpdated,

    /// 步骤更新
    StepsUpdated,

    /// 状态变更
    StatusChanged,

    /// 回滚
    RolledBack,
}

/// 版本历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionHistory {
    /// 事件ID
    pub event_id: String,

    /// 当前版本
    pub current_version: Version,

    /// 版本列表
    pub versions: Vec<EventVersion>,
}

impl VersionHistory {
    /// 创建新的版本历史
    pub fn new(event_id: String) -> Self {
        Self { event_id, current_version: 0, versions: Vec::new() }
    }

    /// 添加新版本
    pub fn add_version(&mut self, version: EventVersion) {
        self.current_version = version.version;
        self.versions.push(version);
    }

    /// 获取指定版本
    pub fn get_version(&self, version: Version) -> Option<&EventVersion> {
        self.versions.iter().find(|v| v.version == version)
    }

    /// 获取最新版本
    pub fn latest(&self) -> Option<&EventVersion> {
        self.versions.last()
    }

    /// 获取版本数量
    pub fn version_count(&self) -> usize {
        self.versions.len()
    }
}

/// 版本管理器
pub struct VersionManager {
    /// 版本历史存储
    histories: Arc<RwLock<HashMap<String, VersionHistory>>>,

    /// 版本计数器
    version_counter: Arc<RwLock<u64>>,
}

impl VersionManager {
    /// 创建新的版本管理器
    pub fn new() -> Self {
        Self {
            histories: Arc::new(RwLock::new(HashMap::new())),
            version_counter: Arc::new(RwLock::new(1)),
        }
    }

    /// 创建初始版本
    pub async fn create_initial_version(
        &self,
        event_id: &str,
        changed_by: String,
    ) -> Result<Version> {
        let version = self.next_version().await;

        let event_version = EventVersion {
            version,
            event_id: event_id.to_string(),
            change_description: "初始创建".to_string(),
            change_type: ChangeType::Created,
            changed_at: Utc::now(),
            changed_by,
        };

        let mut histories = self.histories.write().await;
        let history = histories
            .entry(event_id.to_string())
            .or_insert_with(|| VersionHistory::new(event_id.to_string()));
        history.add_version(event_version);

        Ok(version)
    }

    /// 创建内容更新版本
    pub async fn create_content_update(
        &self,
        event_id: &str,
        description: String,
        changed_by: String,
    ) -> Result<Version> {
        self.create_version(event_id, description, ChangeType::ContentUpdated, changed_by).await
    }

    /// 创建元数据更新版本
    pub async fn create_metadata_update(
        &self,
        event_id: &str,
        description: String,
        changed_by: String,
    ) -> Result<Version> {
        self.create_version(event_id, description, ChangeType::MetadataUpdated, changed_by).await
    }

    /// 创建步骤更新版本
    pub async fn create_steps_update(
        &self,
        event_id: &str,
        description: String,
        changed_by: String,
    ) -> Result<Version> {
        self.create_version(event_id, description, ChangeType::StepsUpdated, changed_by).await
    }

    /// 创建回滚版本
    pub async fn create_rollback_version(
        &self,
        event_id: &str,
        target_version: Version,
        changed_by: String,
    ) -> Result<Version> {
        let description = format!("回滚到版本 {}", target_version);
        self.create_version(event_id, description, ChangeType::RolledBack, changed_by).await
    }

    /// 通用版本创建
    async fn create_version(
        &self,
        event_id: &str,
        description: String,
        change_type: ChangeType,
        changed_by: String,
    ) -> Result<Version> {
        let version = self.next_version().await;

        let event_version = EventVersion {
            version,
            event_id: event_id.to_string(),
            change_description: description,
            change_type,
            changed_at: Utc::now(),
            changed_by,
        };

        let mut histories = self.histories.write().await;
        let history = histories
            .get_mut(event_id)
            .ok_or_else(|| MemoryError::EventNotFound(format!("事件 {} 无版本历史", event_id)))?;
        history.add_version(event_version);

        Ok(version)
    }

    /// 获取下一个版本号
    async fn next_version(&self) -> Version {
        let mut counter = self.version_counter.write().await;
        let version = *counter;
        *counter += 1;
        version
    }

    /// 获取版本历史
    pub async fn get_history(&self, event_id: &str) -> Result<VersionHistory> {
        let histories = self.histories.read().await;
        histories
            .get(event_id)
            .cloned()
            .ok_or_else(|| MemoryError::EventNotFound(format!("事件 {} 无版本历史", event_id)))
    }

    /// 获取指定版本
    pub async fn get_version(&self, event_id: &str, version: Version) -> Result<EventVersion> {
        let histories = self.histories.read().await;
        let history = histories
            .get(event_id)
            .ok_or_else(|| MemoryError::EventNotFound(format!("事件 {} 无版本历史", event_id)))?;
        history
            .get_version(version)
            .cloned()
            .ok_or_else(|| MemoryError::QueryError(format!("版本 {} 不存在", version)))
    }

    /// 获取当前版本号
    pub async fn get_current_version(&self, event_id: &str) -> Result<Version> {
        let histories = self.histories.read().await;
        let history = histories
            .get(event_id)
            .ok_or_else(|| MemoryError::EventNotFound(format!("事件 {} 无版本历史", event_id)))?;
        Ok(history.current_version)
    }

    /// 获取版本数量
    pub async fn get_version_count(&self, event_id: &str) -> Result<usize> {
        let histories = self.histories.read().await;
        let history = histories
            .get(event_id)
            .ok_or_else(|| MemoryError::EventNotFound(format!("事件 {} 无版本历史", event_id)))?;
        Ok(history.version_count())
    }

    /// 删除事件版本历史
    pub async fn delete_history(&self, event_id: &str) -> Result<()> {
        let mut histories = self.histories.write().await;
        histories
            .remove(event_id)
            .ok_or_else(|| MemoryError::EventNotFound(format!("事件 {} 无版本历史", event_id)))?;
        Ok(())
    }

    /// 统计版本历史数量
    pub async fn count_histories(&self) -> Result<usize> {
        let histories = self.histories.read().await;
        Ok(histories.len())
    }
}

impl Default for VersionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_initial_version() {
        let manager = VersionManager::new();
        let version = manager.create_initial_version("event-001", "user-1".to_string()).await;

        assert!(version.is_ok());
        assert_eq!(version.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_create_content_update() {
        let manager = VersionManager::new();
        manager.create_initial_version("event-001", "user-1".to_string()).await.unwrap();

        let version = manager
            .create_content_update("event-001", "更新内容".to_string(), "user-2".to_string())
            .await;

        assert!(version.is_ok());
        assert_eq!(version.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_get_history() {
        let manager = VersionManager::new();
        manager.create_initial_version("event-001", "user-1".to_string()).await.unwrap();
        manager
            .create_content_update("event-001", "更新".to_string(), "user-2".to_string())
            .await
            .unwrap();

        let history = manager.get_history("event-001").await.unwrap();

        assert_eq!(history.version_count(), 2);
        assert_eq!(history.current_version, 2);
    }

    #[tokio::test]
    async fn test_get_version() {
        let manager = VersionManager::new();
        manager.create_initial_version("event-001", "user-1".to_string()).await.unwrap();

        let version = manager.get_version("event-001", 1).await.unwrap();

        assert_eq!(version.version, 1);
        assert_eq!(version.change_type, ChangeType::Created);
    }

    #[tokio::test]
    async fn test_get_current_version() {
        let manager = VersionManager::new();
        manager.create_initial_version("event-001", "user-1".to_string()).await.unwrap();
        manager
            .create_content_update("event-001", "更新".to_string(), "user-2".to_string())
            .await
            .unwrap();

        let current = manager.get_current_version("event-001").await.unwrap();

        assert_eq!(current, 2);
    }

    #[tokio::test]
    async fn test_create_rollback_version() {
        let manager = VersionManager::new();
        manager.create_initial_version("event-001", "user-1".to_string()).await.unwrap();
        manager
            .create_content_update("event-001", "更新".to_string(), "user-2".to_string())
            .await
            .unwrap();

        let version = manager.create_rollback_version("event-001", 1, "user-3".to_string()).await;

        assert!(version.is_ok());
        let version = version.unwrap();
        let v = manager.get_version("event-001", version).await.unwrap();
        assert_eq!(v.change_type, ChangeType::RolledBack);
    }

    #[tokio::test]
    async fn test_delete_history() {
        let manager = VersionManager::new();
        manager.create_initial_version("event-001", "user-1".to_string()).await.unwrap();

        manager.delete_history("event-001").await.unwrap();

        let result = manager.get_history("event-001").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_count_histories() {
        let manager = VersionManager::new();
        manager.create_initial_version("event-001", "user-1".to_string()).await.unwrap();
        manager.create_initial_version("event-002", "user-1".to_string()).await.unwrap();

        let count = manager.count_histories().await.unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_version_history() {
        let mut history = VersionHistory::new("event-001".to_string());

        history.add_version(EventVersion {
            version: 1,
            event_id: "event-001".to_string(),
            change_description: "创建".to_string(),
            change_type: ChangeType::Created,
            changed_at: Utc::now(),
            changed_by: "user-1".to_string(),
        });

        history.add_version(EventVersion {
            version: 2,
            event_id: "event-001".to_string(),
            change_description: "更新".to_string(),
            change_type: ChangeType::ContentUpdated,
            changed_at: Utc::now(),
            changed_by: "user-2".to_string(),
        });

        assert_eq!(history.current_version, 2);
        assert_eq!(history.version_count(), 2);
        assert!(history.get_version(1).is_some());
        assert!(history.latest().is_some());
    }
}
