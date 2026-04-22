//! # 回滚执行模块
//!
//! 负责执行回滚操作和管理回滚动作链

use crate::error::{Error, Result};
use crate::snapshot::{SnapshotId, SnapshotManager, SnapshotStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 回滚动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackAction {
    /// 动作ID
    pub id: String,

    /// 关联的快照ID
    pub snapshot_id: SnapshotId,

    /// 动作类型
    pub action_type: RollbackActionType,

    /// 动作状态
    pub status: RollbackActionStatus,

    /// 执行顺序
    pub order: usize,

    /// 执行时间
    pub executed_at: Option<DateTime<Utc>>,

    /// 执行结果
    pub result: Option<String>,

    /// 错误信息
    pub error: Option<String>,
}

/// 回滚动作类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RollbackActionType {
    /// 数据恢复
    DataRestore,
    /// 状态重置
    StateReset,
    /// 资源清理
    ResourceCleanup,
    /// 通知发送
    NotificationSend,
    /// 自定义动作
    Custom(String),
}

/// 回滚动作状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RollbackActionStatus {
    /// 待执行
    Pending,
    /// 执行中
    Executing,
    /// 已完成
    Completed,
    /// 已失败
    Failed,
    /// 已跳过
    Skipped,
}

impl RollbackAction {
    /// 创建新的回滚动作
    pub fn new(snapshot_id: SnapshotId, action_type: RollbackActionType, order: usize) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            snapshot_id,
            action_type,
            status: RollbackActionStatus::Pending,
            order,
            executed_at: None,
            result: None,
            error: None,
        }
    }

    /// 标记为执行中
    pub fn mark_executing(&mut self) {
        self.status = RollbackActionStatus::Executing;
    }

    /// 标记为已完成
    pub fn mark_completed(&mut self, result: String) {
        self.status = RollbackActionStatus::Completed;
        self.executed_at = Some(Utc::now());
        self.result = Some(result);
    }

    /// 标记为已失败
    pub fn mark_failed(&mut self, error: String) {
        self.status = RollbackActionStatus::Failed;
        self.executed_at = Some(Utc::now());
        self.error = Some(error);
    }

    /// 标记为已跳过
    pub fn mark_skipped(&mut self, reason: String) {
        self.status = RollbackActionStatus::Skipped;
        self.result = Some(reason);
    }
}

/// 回滚结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackResult {
    /// 是否成功
    pub success: bool,

    /// 快照ID
    pub snapshot_id: SnapshotId,

    /// 执行的动作数量
    pub actions_executed: usize,

    /// 失败的动作数量
    pub actions_failed: usize,

    /// 执行时间（毫秒）
    pub duration_ms: u64,

    /// 错误信息
    pub error: Option<String>,

    /// 完成时间
    pub completed_at: DateTime<Utc>,
}

/// 回滚执行器
pub struct RollbackExecutor {
    /// 快照管理器
    snapshot_manager: Arc<SnapshotManager>,

    /// 执行历史
    execution_history: Arc<RwLock<Vec<RollbackResult>>>,
}

impl RollbackExecutor {
    /// 创建新的回滚执行器
    pub fn new(snapshot_manager: Arc<SnapshotManager>) -> Self {
        Self { snapshot_manager, execution_history: Arc::new(RwLock::new(Vec::new())) }
    }

    /// 执行回滚
    pub async fn execute(&self, snapshot_id: &SnapshotId) -> Result<RollbackResult> {
        let start = std::time::Instant::now();

        // 获取快照
        let snapshot = self.snapshot_manager.get_snapshot(snapshot_id).await?;

        // 检查快照状态
        if snapshot.status == SnapshotStatus::RolledBack {
            return Err(Error::Rollback(format!("快照 {} 已回滚", snapshot_id)));
        }

        if snapshot.is_expired() {
            return Err(Error::Rollback(format!("快照 {} 已过期", snapshot_id)));
        }

        // 检查回滚数据
        let _rollback_data = snapshot
            .rollback_data
            .clone()
            .ok_or_else(|| Error::Rollback(format!("快照 {} 没有回滚数据", snapshot_id)))?;

        // 执行回滚动作
        let actions = self.create_rollback_actions(snapshot_id).await?;
        let (executed, failed) = self.execute_actions(actions).await?;

        // 标记快照为已回滚
        self.snapshot_manager.mark_rolled_back(snapshot_id).await?;

        let duration_ms = start.elapsed().as_millis() as u64;
        let success = failed == 0;

        let result = RollbackResult {
            success,
            snapshot_id: snapshot_id.clone(),
            actions_executed: executed,
            actions_failed: failed,
            duration_ms,
            error: if failed > 0 { Some(format!("{} 个动作失败", failed)) } else { None },
            completed_at: Utc::now(),
        };

        // 记录执行历史
        let mut history = self.execution_history.write().await;
        history.push(result.clone());

        Ok(result)
    }

    /// 执行回滚到指定操作
    pub async fn rollback_operation(&self, operation_id: &str) -> Result<RollbackResult> {
        let snapshot = self.snapshot_manager.get_latest_snapshot(operation_id).await?;
        self.execute(&snapshot.id).await
    }

    /// 执行多级回滚
    pub async fn execute_multi_level(
        &self,
        operation_id: &str,
        levels: usize,
    ) -> Result<Vec<RollbackResult>> {
        let snapshots = self.snapshot_manager.get_operation_snapshots(operation_id).await?;

        if snapshots.len() < levels {
            return Err(Error::Rollback(format!(
                "操作 {} 只有 {} 个快照，无法回滚 {} 级",
                operation_id,
                snapshots.len(),
                levels
            )));
        }

        let mut results = Vec::new();
        let start_index = snapshots.len().saturating_sub(levels);

        // 从最新到最旧执行回滚
        for i in (start_index..snapshots.len()).rev() {
            let snapshot = &snapshots[i];
            let result = self.execute(&snapshot.id).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// 创建回滚动作
    async fn create_rollback_actions(
        &self,
        snapshot_id: &SnapshotId,
    ) -> Result<Vec<RollbackAction>> {
        let mut actions = Vec::new();

        // 数据恢复动作
        actions.push(RollbackAction::new(snapshot_id.clone(), RollbackActionType::DataRestore, 1));

        // 状态重置动作
        actions.push(RollbackAction::new(snapshot_id.clone(), RollbackActionType::StateReset, 2));

        // 资源清理动作
        actions.push(RollbackAction::new(
            snapshot_id.clone(),
            RollbackActionType::ResourceCleanup,
            3,
        ));

        // 通知发送动作
        actions.push(RollbackAction::new(
            snapshot_id.clone(),
            RollbackActionType::NotificationSend,
            4,
        ));

        Ok(actions)
    }

    /// 执行动作列表
    async fn execute_actions(&self, actions: Vec<RollbackAction>) -> Result<(usize, usize)> {
        let mut executed = 0;
        let mut failed = 0;

        for mut action in actions {
            action.mark_executing();

            // 模拟执行（实际应用中调用具体执行逻辑）
            match self.execute_single_action(&mut action).await {
                Ok(_) => {
                    executed += 1;
                },
                Err(e) => {
                    action.mark_failed(e.to_string());
                    failed += 1;
                },
            }
        }

        Ok((executed, failed))
    }

    /// 执行单个动作
    async fn execute_single_action(&self, action: &mut RollbackAction) -> Result<()> {
        // 模拟执行延迟
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;

        // 标记完成
        action.mark_completed(format!("{:?} 执行成功", action.action_type));

        Ok(())
    }

    /// 获取执行历史
    pub async fn get_history(&self) -> Vec<RollbackResult> {
        let history = self.execution_history.read().await;
        history.clone()
    }

    /// 统计执行次数
    pub async fn execution_count(&self) -> usize {
        let history = self.execution_history.read().await;
        history.len()
    }

    /// 清空执行历史
    pub async fn clear_history(&self) {
        let mut history = self.execution_history.write().await;
        history.clear();
    }
}

impl Default for RollbackExecutor {
    fn default() -> Self {
        Self::new(Arc::new(SnapshotManager::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_rollback() {
        let snapshot_manager = Arc::new(SnapshotManager::new());
        let executor = RollbackExecutor::new(snapshot_manager.clone());

        // 创建快照
        let id = snapshot_manager
            .create_snapshot_with_rollback(
                "op-001".to_string(),
                crate::snapshot::SnapshotType::Full,
                serde_json::json!({"data": "value"}),
                serde_json::json!({"rollback": "data"}),
            )
            .await
            .unwrap();

        let result = executor.execute(&id).await.unwrap();
        assert!(result.success);
        assert_eq!(result.actions_executed, 4);
    }

    #[tokio::test]
    async fn test_rollback_operation() {
        let snapshot_manager = Arc::new(SnapshotManager::new());
        let executor = RollbackExecutor::new(snapshot_manager.clone());

        snapshot_manager
            .create_snapshot_with_rollback(
                "op-001".to_string(),
                crate::snapshot::SnapshotType::Full,
                serde_json::json!({}),
                serde_json::json!({}),
            )
            .await
            .unwrap();

        let result = executor.rollback_operation("op-001").await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_multi_level_rollback() {
        let snapshot_manager = Arc::new(SnapshotManager::new());
        let executor = RollbackExecutor::new(snapshot_manager.clone());

        // 创建多个快照
        for i in 0..3 {
            snapshot_manager
                .create_snapshot_with_rollback(
                    "op-001".to_string(),
                    crate::snapshot::SnapshotType::Incremental,
                    serde_json::json!({"version": i}),
                    serde_json::json!({"rollback": i}),
                )
                .await
                .unwrap();
        }

        let results = executor.execute_multi_level("op-001", 2).await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_already_rolled_back() {
        let snapshot_manager = Arc::new(SnapshotManager::new());
        let executor = RollbackExecutor::new(snapshot_manager.clone());

        let id = snapshot_manager
            .create_snapshot_with_rollback(
                "op-001".to_string(),
                crate::snapshot::SnapshotType::Full,
                serde_json::json!({}),
                serde_json::json!({}),
            )
            .await
            .unwrap();

        // 第一次回滚
        executor.execute(&id).await.unwrap();

        // 第二次应该失败
        let result = executor.execute(&id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_no_rollback_data() {
        let snapshot_manager = Arc::new(SnapshotManager::new());
        let executor = RollbackExecutor::new(snapshot_manager.clone());

        let id = snapshot_manager
            .create_snapshot(
                "op-001".to_string(),
                crate::snapshot::SnapshotType::Full,
                serde_json::json!({}),
            )
            .await
            .unwrap();

        let result = executor.execute(&id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execution_history() {
        let snapshot_manager = Arc::new(SnapshotManager::new());
        let executor = RollbackExecutor::new(snapshot_manager.clone());

        let id = snapshot_manager
            .create_snapshot_with_rollback(
                "op-001".to_string(),
                crate::snapshot::SnapshotType::Full,
                serde_json::json!({}),
                serde_json::json!({}),
            )
            .await
            .unwrap();

        executor.execute(&id).await.unwrap();

        assert_eq!(executor.execution_count().await, 1);
        let history = executor.get_history().await;
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_rollback_action() {
        let mut action =
            RollbackAction::new("snap-001".to_string(), RollbackActionType::DataRestore, 1);

        assert_eq!(action.status, RollbackActionStatus::Pending);

        action.mark_executing();
        assert_eq!(action.status, RollbackActionStatus::Executing);

        action.mark_completed("成功".to_string());
        assert_eq!(action.status, RollbackActionStatus::Completed);
        assert!(action.executed_at.is_some());
    }

    #[test]
    fn test_rollback_action_failed() {
        let mut action =
            RollbackAction::new("snap-001".to_string(), RollbackActionType::DataRestore, 1);

        action.mark_failed("执行失败".to_string());
        assert_eq!(action.status, RollbackActionStatus::Failed);
        assert!(action.error.is_some());
    }

    #[test]
    fn test_rollback_action_skipped() {
        let mut action =
            RollbackAction::new("snap-001".to_string(), RollbackActionType::DataRestore, 1);

        action.mark_skipped("条件不满足".to_string());
        assert_eq!(action.status, RollbackActionStatus::Skipped);
        assert!(action.result.is_some());
    }
}
