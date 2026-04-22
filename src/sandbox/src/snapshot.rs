//! 沙箱快照管理模块
//!
//! 提供沙箱状态的快照和恢复能力

use crate::error::{Result, SandboxError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 快照ID
pub type SnapshotId = String;

/// 快照状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotStatus {
    /// 已创建
    Created,
    /// 已恢复
    Restored,
    /// 已过期
    Expired,
    /// 已删除
    Deleted,
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

/// 沙箱快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxSnapshot {
    /// 快照ID
    pub id: SnapshotId,
    /// 沙箱ID
    pub sandbox_id: String,
    /// 快照类型
    pub snapshot_type: SnapshotType,
    /// 快照状态
    pub status: SnapshotStatus,
    /// 文件系统状态
    pub filesystem_state: serde_json::Value,
    /// 网络状态
    pub network_state: serde_json::Value,
    /// 进程状态
    pub process_state: serde_json::Value,
    /// 环境变量
    pub environment: HashMap<String, String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 过期时间
    pub expires_at: Option<DateTime<Utc>>,
    /// 父快照ID（用于增量快照）
    pub parent_id: Option<SnapshotId>,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl SandboxSnapshot {
    /// 创建新快照
    pub fn new(sandbox_id: String, snapshot_type: SnapshotType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            sandbox_id,
            snapshot_type,
            status: SnapshotStatus::Created,
            filesystem_state: serde_json::json!({}),
            network_state: serde_json::json!({}),
            process_state: serde_json::json!({}),
            environment: HashMap::new(),
            created_at: Utc::now(),
            expires_at: None,
            parent_id: None,
            metadata: HashMap::new(),
        }
    }

    /// 设置文件系统状态
    pub fn with_filesystem_state(mut self, state: serde_json::Value) -> Self {
        self.filesystem_state = state;
        self
    }

    /// 设置网络状态
    pub fn with_network_state(mut self, state: serde_json::Value) -> Self {
        self.network_state = state;
        self
    }

    /// 设置进程状态
    pub fn with_process_state(mut self, state: serde_json::Value) -> Self {
        self.process_state = state;
        self
    }

    /// 设置环境变量
    pub fn with_environment(mut self, env: HashMap<String, String>) -> Self {
        self.environment = env;
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

    /// 标记为已恢复
    pub fn mark_restored(&mut self) {
        self.status = SnapshotStatus::Restored;
    }

    /// 标记为已删除
    pub fn mark_deleted(&mut self) {
        self.status = SnapshotStatus::Deleted;
    }
}

/// 快照管理器
pub struct SnapshotManager {
    /// 快照存储
    snapshots: Arc<RwLock<HashMap<SnapshotId, SandboxSnapshot>>>,
    /// 沙箱到快照的映射
    sandbox_snapshots: Arc<RwLock<HashMap<String, Vec<SnapshotId>>>>,
    /// 最大快照数量
    max_snapshots: usize,
}

impl SnapshotManager {
    /// 创建新的快照管理器
    pub fn new() -> Self {
        Self {
            snapshots: Arc::new(RwLock::new(HashMap::new())),
            sandbox_snapshots: Arc::new(RwLock::new(HashMap::new())),
            max_snapshots: 100,
        }
    }

    /// 设置最大快照数量
    pub fn with_max_snapshots(mut self, max: usize) -> Self {
        self.max_snapshots = max;
        self
    }

    /// 创建快照
    pub async fn create(&self, snapshot: SandboxSnapshot) -> Result<SnapshotId> {
        // 检查数量限制
        {
            let snapshots = self.snapshots.read().await;
            if snapshots.len() >= self.max_snapshots {
                return Err(SandboxError::Snapshot("快照数量已达上限".to_string()));
            }
        }

        let id = snapshot.id.clone();
        let sandbox_id = snapshot.sandbox_id.clone();

        let mut snapshots = self.snapshots.write().await;
        let mut sandbox_snapshots = self.sandbox_snapshots.write().await;

        snapshots.insert(id.clone(), snapshot);
        sandbox_snapshots.entry(sandbox_id).or_insert_with(Vec::new).push(id.clone());

        Ok(id)
    }

    /// 获取快照
    pub async fn get(&self, id: &SnapshotId) -> Result<SandboxSnapshot> {
        let snapshots = self.snapshots.read().await;
        snapshots
            .get(id)
            .cloned()
            .ok_or_else(|| SandboxError::Snapshot(format!("快照 {} 不存在", id)))
    }

    /// 获取沙箱的所有快照
    pub async fn get_sandbox_snapshots(&self, sandbox_id: &str) -> Result<Vec<SandboxSnapshot>> {
        let sandbox_snapshots = self.sandbox_snapshots.read().await;
        let snapshot_ids = sandbox_snapshots.get(sandbox_id).cloned().unwrap_or_default();

        let snapshots = self.snapshots.read().await;
        let result: Vec<SandboxSnapshot> =
            snapshot_ids.iter().filter_map(|id| snapshots.get(id).cloned()).collect();

        Ok(result)
    }

    /// 获取沙箱的最新快照
    pub async fn get_latest(&self, sandbox_id: &str) -> Result<SandboxSnapshot> {
        let snapshots = self.get_sandbox_snapshots(sandbox_id).await?;
        snapshots
            .into_iter()
            .max_by_key(|s| s.created_at)
            .ok_or_else(|| SandboxError::Snapshot(format!("沙箱 {} 没有快照", sandbox_id)))
    }

    /// 恢复快照
    pub async fn restore(&self, id: &SnapshotId) -> Result<SandboxSnapshot> {
        let mut snapshots = self.snapshots.write().await;
        let snapshot = snapshots
            .get_mut(id)
            .ok_or_else(|| SandboxError::Snapshot(format!("快照 {} 不存在", id)))?;

        if snapshot.is_expired() {
            return Err(SandboxError::Snapshot(format!("快照 {} 已过期", id)));
        }

        snapshot.mark_restored();
        Ok(snapshot.clone())
    }

    /// 删除快照
    pub async fn delete(&self, id: &SnapshotId) -> Result<()> {
        let mut snapshots = self.snapshots.write().await;
        let snapshot = snapshots
            .remove(id)
            .ok_or_else(|| SandboxError::Snapshot(format!("快照 {} 不存在", id)))?;

        // 从沙箱映射中移除
        let mut sandbox_snapshots = self.sandbox_snapshots.write().await;
        if let Some(ids) = sandbox_snapshots.get_mut(&snapshot.sandbox_id) {
            ids.retain(|i| i != id);
        }

        Ok(())
    }

    /// 清理过期快照
    pub async fn cleanup_expired(&self) -> Result<usize> {
        let mut snapshots = self.snapshots.write().await;
        let mut sandbox_snapshots = self.sandbox_snapshots.write().await;

        let expired_ids: Vec<SnapshotId> =
            snapshots.iter().filter(|(_, s)| s.is_expired()).map(|(id, _)| id.clone()).collect();

        let count = expired_ids.len();

        for id in expired_ids {
            if let Some(snapshot) = snapshots.remove(&id) {
                if let Some(ids) = sandbox_snapshots.get_mut(&snapshot.sandbox_id) {
                    ids.retain(|i| i != &id);
                }
            }
        }

        Ok(count)
    }

    /// 获取快照数量
    pub async fn count(&self) -> usize {
        self.snapshots.read().await.len()
    }

    /// 获取沙箱的快照数量
    pub async fn count_for_sandbox(&self, sandbox_id: &str) -> usize {
        self.get_sandbox_snapshots(sandbox_id).await.unwrap_or_default().len()
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
        let snapshot = SandboxSnapshot::new("sandbox-1".to_string(), SnapshotType::Full);

        let id = manager.create(snapshot).await.unwrap();
        assert!(!id.is_empty());

        let count = manager.count().await;
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_get_snapshot() {
        let manager = SnapshotManager::new();
        let snapshot = SandboxSnapshot::new("sandbox-1".to_string(), SnapshotType::Full);

        let id = manager.create(snapshot).await.unwrap();
        let retrieved = manager.get(&id).await.unwrap();

        assert_eq!(retrieved.id, id);
        assert_eq!(retrieved.sandbox_id, "sandbox-1");
    }

    #[tokio::test]
    async fn test_get_sandbox_snapshots() {
        let manager = SnapshotManager::new();

        let snapshot1 = SandboxSnapshot::new("sandbox-1".to_string(), SnapshotType::Full);
        let snapshot2 = SandboxSnapshot::new("sandbox-1".to_string(), SnapshotType::Incremental);

        manager.create(snapshot1).await.unwrap();
        manager.create(snapshot2).await.unwrap();

        let snapshots = manager.get_sandbox_snapshots("sandbox-1").await.unwrap();
        assert_eq!(snapshots.len(), 2);
    }

    #[tokio::test]
    async fn test_get_latest_snapshot() {
        let manager = SnapshotManager::new();

        let snapshot1 = SandboxSnapshot::new("sandbox-1".to_string(), SnapshotType::Full);
        let snapshot2 = SandboxSnapshot::new("sandbox-1".to_string(), SnapshotType::Checkpoint);

        manager.create(snapshot1).await.unwrap();
        // 稍等片刻确保时间不同
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        manager.create(snapshot2).await.unwrap();

        let latest = manager.get_latest("sandbox-1").await.unwrap();
        assert_eq!(latest.snapshot_type, SnapshotType::Checkpoint);
    }

    #[tokio::test]
    async fn test_restore_snapshot() {
        let manager = SnapshotManager::new();
        let snapshot = SandboxSnapshot::new("sandbox-1".to_string(), SnapshotType::Full);

        let id = manager.create(snapshot).await.unwrap();
        let restored = manager.restore(&id).await.unwrap();

        assert_eq!(restored.status, SnapshotStatus::Restored);
    }

    #[tokio::test]
    async fn test_delete_snapshot() {
        let manager = SnapshotManager::new();
        let snapshot = SandboxSnapshot::new("sandbox-1".to_string(), SnapshotType::Full);

        let id = manager.create(snapshot).await.unwrap();
        assert_eq!(manager.count().await, 1);

        manager.delete(&id).await.unwrap();
        assert_eq!(manager.count().await, 0);
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let manager = SnapshotManager::new();

        // 创建一个已过期的快照
        let mut snapshot = SandboxSnapshot::new("sandbox-1".to_string(), SnapshotType::Full);
        snapshot.expires_at = Some(Utc::now() - chrono::Duration::hours(1));

        manager.create(snapshot).await.unwrap();
        assert_eq!(manager.count().await, 1);

        let cleaned = manager.cleanup_expired().await.unwrap();
        assert_eq!(cleaned, 1);
        assert_eq!(manager.count().await, 0);
    }

    #[tokio::test]
    async fn test_snapshot_with_state() {
        let snapshot = SandboxSnapshot::new("sandbox-1".to_string(), SnapshotType::Full)
            .with_filesystem_state(serde_json::json!({"root": "/tmp/sandbox"}))
            .with_network_state(serde_json::json!({"namespace": "ns1"}))
            .with_process_state(serde_json::json!({"pids": [1, 2, 3]}));

        assert_eq!(snapshot.filesystem_state["root"], "/tmp/sandbox");
        assert_eq!(snapshot.network_state["namespace"], "ns1");
        assert_eq!(snapshot.process_state["pids"].as_array().unwrap().len(), 3);
    }

    #[tokio::test]
    async fn test_max_snapshots_limit() {
        let manager = SnapshotManager::new().with_max_snapshots(2);

        let snapshot1 = SandboxSnapshot::new("sandbox-1".to_string(), SnapshotType::Full);
        let snapshot2 = SandboxSnapshot::new("sandbox-2".to_string(), SnapshotType::Full);
        let snapshot3 = SandboxSnapshot::new("sandbox-3".to_string(), SnapshotType::Full);

        manager.create(snapshot1).await.unwrap();
        manager.create(snapshot2).await.unwrap();

        // 第三个应该失败
        let result = manager.create(snapshot3).await;
        assert!(result.is_err());
    }
}
