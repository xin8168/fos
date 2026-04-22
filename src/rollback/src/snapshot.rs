//! # 回滚快照管理模块
//!
//! 负责创建、存储和管理操作快照，用于后续回滚

use crate::error::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// 快照ID
pub type SnapshotId = String;

/// 快照状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotStatus {
    /// 已创建
    Created,
    /// 已使用
    Used,
    /// 已回滚
    RolledBack,
    /// 已过期
    Expired,
}

/// 快照类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotType {
    /// 完整快照
    Full,
    /// 增量快照
    Incremental,
    /// 检查点
    Checkpoint,
}

/// 操作快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// 快照ID
    pub id: SnapshotId,

    /// 关联的操作ID
    pub operation_id: String,

    /// 快照类型
    pub snapshot_type: SnapshotType,

    /// 快照状态
    pub status: SnapshotStatus,

    /// 快照数据
    pub data: serde_json::Value,

    /// 回滚数据（用于回滚操作）
    pub rollback_data: Option<serde_json::Value>,

    /// 父快照ID（用于链式快照）
    pub parent_id: Option<SnapshotId>,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 过期时间
    pub expires_at: Option<DateTime<Utc>>,

    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl Snapshot {
    /// 创建新快照
    pub fn new(operation_id: String, snapshot_type: SnapshotType, data: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            operation_id,
            snapshot_type,
            status: SnapshotStatus::Created,
            data,
            rollback_data: None,
            parent_id: None,
            created_at: Utc::now(),
            expires_at: None,
            metadata: HashMap::new(),
        }
    }

    /// 设置回滚数据
    pub fn with_rollback_data(mut self, rollback_data: serde_json::Value) -> Self {
        self.rollback_data = Some(rollback_data);
        self
    }

    /// 设置父快照
    pub fn with_parent(mut self, parent_id: SnapshotId) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// 设置过期时间
    pub fn with_expiry(mut self, hours: u64) -> Self {
        self.expires_at = Some(Utc::now() + chrono::Duration::hours(hours as i64));
        self
    }

    /// 添加元数据
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }

    /// 检查是否已过期
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            return Utc::now() > expires_at;
        }
        false
    }

    /// 标记为已使用
    pub fn mark_used(&mut self) {
        self.status = SnapshotStatus::Used;
    }

    /// 标记为已回滚
    pub fn mark_rolled_back(&mut self) {
        self.status = SnapshotStatus::RolledBack;
    }
}

/// 快照管理器
pub struct SnapshotManager {
    /// 快照存储
    snapshots: Arc<RwLock<HashMap<SnapshotId, Snapshot>>>,

    /// 操作到快照的映射
    operation_snapshots: Arc<RwLock<HashMap<String, Vec<SnapshotId>>>>,

    /// 配置
    config: crate::config::Config,
}

impl SnapshotManager {
    /// 创建新的快照管理器
    pub fn new() -> Self {
        Self::with_config(crate::config::Config::default())
    }

    /// 使用配置创建
    pub fn with_config(config: crate::config::Config) -> Self {
        Self {
            snapshots: Arc::new(RwLock::new(HashMap::new())),
            operation_snapshots: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// 创建快照
    pub async fn create_snapshot(
        &self,
        operation_id: String,
        snapshot_type: SnapshotType,
        data: serde_json::Value,
    ) -> Result<SnapshotId> {
        let snapshot = Snapshot::new(operation_id.clone(), snapshot_type, data);

        // 检查层级限制
        {
            let op_snapshots = self.operation_snapshots.read().await;
            if let Some(snapshots) = op_snapshots.get(&operation_id) {
                if snapshots.len() >= self.config.max_rollback_levels {
                    return Err(Error::Config(format!(
                        "操作 {} 的快照数量已达上限 {}",
                        operation_id, self.config.max_rollback_levels
                    )));
                }
            }
        }

        let id = snapshot.id.clone();
        let mut snapshots = self.snapshots.write().await;
        let mut operation_snapshots = self.operation_snapshots.write().await;

        snapshots.insert(id.clone(), snapshot);
        operation_snapshots.entry(operation_id).or_insert_with(Vec::new).push(id.clone());

        Ok(id)
    }

    /// 创建带回滚数据的快照
    pub async fn create_snapshot_with_rollback(
        &self,
        operation_id: String,
        snapshot_type: SnapshotType,
        data: serde_json::Value,
        rollback_data: serde_json::Value,
    ) -> Result<SnapshotId> {
        let snapshot = Snapshot::new(operation_id.clone(), snapshot_type, data)
            .with_rollback_data(rollback_data);

        let id = snapshot.id.clone();
        let mut snapshots = self.snapshots.write().await;
        let mut operation_snapshots = self.operation_snapshots.write().await;

        snapshots.insert(id.clone(), snapshot);
        operation_snapshots.entry(operation_id).or_insert_with(Vec::new).push(id.clone());

        Ok(id)
    }

    /// 获取快照
    pub async fn get_snapshot(&self, id: &SnapshotId) -> Result<Snapshot> {
        let snapshots = self.snapshots.read().await;
        snapshots.get(id).cloned().ok_or_else(|| Error::Rollback(format!("快照 {} 不存在", id)))
    }

    /// 获取操作的最新快照
    pub async fn get_latest_snapshot(&self, operation_id: &str) -> Result<Snapshot> {
        let operation_snapshots = self.operation_snapshots.read().await;
        let snapshot_ids = operation_snapshots
            .get(operation_id)
            .ok_or_else(|| Error::Rollback(format!("操作 {} 没有快照", operation_id)))?;

        let snapshots = self.snapshots.read().await;
        let latest_id = snapshot_ids
            .last()
            .ok_or_else(|| Error::Rollback(format!("操作 {} 没有快照", operation_id)))?;

        snapshots
            .get(latest_id)
            .cloned()
            .ok_or_else(|| Error::Rollback(format!("快照 {} 不存在", latest_id)))
    }

    /// 获取操作的所有快照
    pub async fn get_operation_snapshots(&self, operation_id: &str) -> Result<Vec<Snapshot>> {
        let operation_snapshots = self.operation_snapshots.read().await;
        let snapshot_ids = operation_snapshots
            .get(operation_id)
            .ok_or_else(|| Error::Rollback(format!("操作 {} 没有快照", operation_id)))?;

        let snapshots = self.snapshots.read().await;
        let result: Vec<Snapshot> =
            snapshot_ids.iter().filter_map(|id| snapshots.get(id).cloned()).collect();

        Ok(result)
    }

    /// 标记快照为已使用
    pub async fn mark_used(&self, id: &SnapshotId) -> Result<()> {
        let mut snapshots = self.snapshots.write().await;
        let snapshot =
            snapshots.get_mut(id).ok_or_else(|| Error::Rollback(format!("快照 {} 不存在", id)))?;
        snapshot.mark_used();
        Ok(())
    }

    /// 标记快照为已回滚
    pub async fn mark_rolled_back(&self, id: &SnapshotId) -> Result<()> {
        let mut snapshots = self.snapshots.write().await;
        let snapshot =
            snapshots.get_mut(id).ok_or_else(|| Error::Rollback(format!("快照 {} 不存在", id)))?;
        snapshot.mark_rolled_back();
        Ok(())
    }

    /// 删除快照
    pub async fn delete_snapshot(&self, id: &SnapshotId) -> Result<()> {
        let mut snapshots = self.snapshots.write().await;
        let snapshot =
            snapshots.remove(id).ok_or_else(|| Error::Rollback(format!("快照 {} 不存在", id)))?;

        // 从操作映射中移除
        let mut operation_snapshots = self.operation_snapshots.write().await;
        if let Some(ids) = operation_snapshots.get_mut(&snapshot.operation_id) {
            ids.retain(|i| i != id);
        }

        Ok(())
    }

    /// 清理过期快照
    pub async fn cleanup_expired(&self) -> Result<usize> {
        let mut snapshots = self.snapshots.write().await;
        let mut operation_snapshots = self.operation_snapshots.write().await;

        let expired_ids: Vec<SnapshotId> =
            snapshots.iter().filter(|(_, s)| s.is_expired()).map(|(id, _)| id.clone()).collect();

        let count = expired_ids.len();

        for id in expired_ids {
            if let Some(snapshot) = snapshots.remove(&id) {
                if let Some(ids) = operation_snapshots.get_mut(&snapshot.operation_id) {
                    ids.retain(|i| i != &id);
                }
            }
        }

        Ok(count)
    }

    /// 统计快照数量
    pub async fn count(&self) -> usize {
        let snapshots = self.snapshots.read().await;
        snapshots.len()
    }

    /// 统计操作的快照数量
    pub async fn count_for_operation(&self, operation_id: &str) -> usize {
        let operation_snapshots = self.operation_snapshots.read().await;
        operation_snapshots.get(operation_id).map(|v| v.len()).unwrap_or(0)
    }

    /// 清空所有快照
    pub async fn clear(&self) -> Result<()> {
        let mut snapshots = self.snapshots.write().await;
        let mut operation_snapshots = self.operation_snapshots.write().await;
        snapshots.clear();
        operation_snapshots.clear();
        Ok(())
    }
}

impl Default for SnapshotManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_snapshot() {
        let manager = SnapshotManager::new();
        let data = serde_json::json!({"key": "value"});

        let id =
            manager.create_snapshot("op-001".to_string(), SnapshotType::Full, data).await.unwrap();

        assert!(!id.is_empty());
        let snapshot = manager.get_snapshot(&id).await.unwrap();
        assert_eq!(snapshot.operation_id, "op-001");
    }

    #[tokio::test]
    async fn test_create_snapshot_with_rollback() {
        let manager = SnapshotManager::new();
        let data = serde_json::json!({"key": "value"});
        let rollback_data = serde_json::json!({"rollback": "data"});

        let id = manager
            .create_snapshot_with_rollback(
                "op-001".to_string(),
                SnapshotType::Full,
                data,
                rollback_data,
            )
            .await
            .unwrap();

        let snapshot = manager.get_snapshot(&id).await.unwrap();
        assert!(snapshot.rollback_data.is_some());
    }

    #[tokio::test]
    async fn test_get_latest_snapshot() {
        let manager = SnapshotManager::new();

        manager
            .create_snapshot("op-001".to_string(), SnapshotType::Full, serde_json::json!({}))
            .await
            .unwrap();
        manager
            .create_snapshot("op-001".to_string(), SnapshotType::Incremental, serde_json::json!({}))
            .await
            .unwrap();

        let latest = manager.get_latest_snapshot("op-001").await.unwrap();
        assert_eq!(latest.snapshot_type, SnapshotType::Incremental);
    }

    #[tokio::test]
    async fn test_mark_used() {
        let manager = SnapshotManager::new();
        let id = manager
            .create_snapshot("op-001".to_string(), SnapshotType::Full, serde_json::json!({}))
            .await
            .unwrap();

        manager.mark_used(&id).await.unwrap();
        let snapshot = manager.get_snapshot(&id).await.unwrap();
        assert_eq!(snapshot.status, SnapshotStatus::Used);
    }

    #[tokio::test]
    async fn test_mark_rolled_back() {
        let manager = SnapshotManager::new();
        let id = manager
            .create_snapshot("op-001".to_string(), SnapshotType::Full, serde_json::json!({}))
            .await
            .unwrap();

        manager.mark_rolled_back(&id).await.unwrap();
        let snapshot = manager.get_snapshot(&id).await.unwrap();
        assert_eq!(snapshot.status, SnapshotStatus::RolledBack);
    }

    #[tokio::test]
    async fn test_delete_snapshot() {
        let manager = SnapshotManager::new();
        let id = manager
            .create_snapshot("op-001".to_string(), SnapshotType::Full, serde_json::json!({}))
            .await
            .unwrap();

        manager.delete_snapshot(&id).await.unwrap();
        let result = manager.get_snapshot(&id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_max_levels_limit() {
        let config = crate::config::Config { max_rollback_levels: 2, auto_rollback: false };
        let manager = SnapshotManager::with_config(config);

        // 创建两条快照
        manager
            .create_snapshot("op-001".to_string(), SnapshotType::Full, serde_json::json!({}))
            .await
            .unwrap();
        manager
            .create_snapshot("op-001".to_string(), SnapshotType::Full, serde_json::json!({}))
            .await
            .unwrap();

        // 第三条应该失败
        let result = manager
            .create_snapshot("op-001".to_string(), SnapshotType::Full, serde_json::json!({}))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_operation_snapshots() {
        let manager = SnapshotManager::new();

        manager
            .create_snapshot("op-001".to_string(), SnapshotType::Full, serde_json::json!({}))
            .await
            .unwrap();
        manager
            .create_snapshot("op-001".to_string(), SnapshotType::Incremental, serde_json::json!({}))
            .await
            .unwrap();

        let snapshots = manager.get_operation_snapshots("op-001").await.unwrap();
        assert_eq!(snapshots.len(), 2);
    }

    #[tokio::test]
    async fn test_count() {
        let manager = SnapshotManager::new();

        manager
            .create_snapshot("op-001".to_string(), SnapshotType::Full, serde_json::json!({}))
            .await
            .unwrap();
        manager
            .create_snapshot("op-002".to_string(), SnapshotType::Full, serde_json::json!({}))
            .await
            .unwrap();

        assert_eq!(manager.count().await, 2);
        assert_eq!(manager.count_for_operation("op-001").await, 1);
    }

    #[tokio::test]
    async fn test_clear() {
        let manager = SnapshotManager::new();

        manager
            .create_snapshot("op-001".to_string(), SnapshotType::Full, serde_json::json!({}))
            .await
            .unwrap();
        manager.clear().await.unwrap();

        assert_eq!(manager.count().await, 0);
    }

    #[test]
    fn test_snapshot_expiry() {
        let snapshot =
            Snapshot::new("op-001".to_string(), SnapshotType::Full, serde_json::json!({}))
                .with_expiry(0); // 立即过期

        // 由于时间精度问题，可能需要短暂等待
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(snapshot.is_expired());
    }
}
