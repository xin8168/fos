//! 事务协调器

use crate::config::TransactionConfig;
use crate::error::{Error, Result};
use crate::log::{LogEntry, TransactionLog};
use crate::participant::{Participant, ParticipantStatus};
use crate::state::{TransactionState, TransactionStatus};
use crate::{Transaction, TransactionId};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// 事务协调器
pub struct TransactionCoordinator {
    /// 配置
    #[allow(dead_code)]
    config: TransactionConfig,
    /// 活跃事务
    active_transactions: std::sync::RwLock<HashMap<TransactionId, Transaction>>,
    /// 事务状态
    transaction_states: std::sync::RwLock<HashMap<TransactionId, TransactionState>>,
    /// 事务日志
    log: Arc<TransactionLog>,
    /// 步骤执行器
    step_executor: StepExecutor,
}

impl TransactionCoordinator {
    /// 创建新的事务协调器
    pub fn new(config: TransactionConfig) -> Self {
        Self {
            config,
            active_transactions: std::sync::RwLock::new(HashMap::new()),
            transaction_states: std::sync::RwLock::new(HashMap::new()),
            log: Arc::new(TransactionLog::new()),
            step_executor: StepExecutor::new(),
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(TransactionConfig::default())
    }

    /// 开始新事务
    pub fn begin(&self, name: &str) -> Result<TransactionId> {
        let mut transaction = Transaction::new(name);
        transaction.status = TransactionStatus::Pending;

        let tx_id = transaction.id;
        let state = TransactionState::new();

        // 记录日志
        self.log.append(LogEntry::transaction_started(tx_id, name))?;

        // 存储事务
        self.active_transactions.write().unwrap().insert(tx_id, transaction);
        self.transaction_states.write().unwrap().insert(tx_id, state);

        tracing::info!("事务开始: {} ({})", name, tx_id);
        Ok(tx_id)
    }

    /// 添加参与者
    pub fn add_participant(&self, tx_id: TransactionId, participant: Participant) -> Result<()> {
        let mut transactions = self.active_transactions.write().unwrap();

        if let Some(tx) = transactions.get_mut(&tx_id) {
            tx.add_participant(participant.clone());
            self.log.append(LogEntry::participant_added(tx_id, participant.id))?;
            tracing::debug!("参与者添加: {} -> {}", tx_id, participant.name);
            Ok(())
        } else {
            Err(Error::Transaction(format!("事务不存在: {}", tx_id)))
        }
    }

    /// 提交事务
    pub fn commit(&self, tx_id: TransactionId) -> Result<()> {
        let transactions = self.active_transactions.read().unwrap();

        if let Some(tx) = transactions.get(&tx_id) {
            if !tx.can_commit() {
                return Err(Error::Transaction(format!("事务状态不允许提交: {:?}", tx.status)));
            }
        } else {
            return Err(Error::Transaction(format!("事务不存在: {}", tx_id)));
        }
        drop(transactions);

        // 执行两阶段提交
        self.execute_commit(tx_id)
    }

    /// 执行提交流程
    fn execute_commit(&self, tx_id: TransactionId) -> Result<()> {
        self.log.append(LogEntry::commit_started(tx_id))?;

        // 获取参与者列表
        let participants: Vec<Participant> = {
            let transactions = self.active_transactions.read().unwrap();
            transactions.get(&tx_id).map(|tx| tx.participants.clone()).unwrap_or_default()
        };

        // 按顺序执行参与者动作
        let mut sorted_participants = participants;
        sorted_participants.sort_by_key(|p| p.order);

        for participant in sorted_participants {
            match self.execute_participant(tx_id, &participant) {
                Ok(_) => {
                    self.mark_participant_completed(tx_id, participant.id)?;
                },
                Err(e) => {
                    tracing::error!("参与者执行失败: {} - {}", participant.name, e);
                    self.rollback(tx_id)?;
                    return Err(Error::Transaction(format!("提交失败，已回滚: {}", e)));
                },
            }
        }

        // 更新状态为已提交
        self.update_status(tx_id, TransactionStatus::Committed)?;
        self.log.append(LogEntry::transaction_committed(tx_id))?;

        tracing::info!("事务提交成功: {}", tx_id);
        Ok(())
    }

    /// 执行参与者动作
    fn execute_participant(&self, tx_id: TransactionId, participant: &Participant) -> Result<()> {
        tracing::debug!("执行参与者: {} (顺序: {})", participant.name, participant.order);

        // 更新参与者状态为执行中
        self.mark_participant_executing(tx_id, participant.id)?;

        // 记录执行日志
        self.log.append(LogEntry::step_started(tx_id, participant.id))?;

        // 执行动作（模拟）
        self.step_executor.execute(&participant.action)?;

        Ok(())
    }

    /// 回滚事务
    pub fn rollback(&self, tx_id: TransactionId) -> Result<()> {
        self.log.append(LogEntry::rollback_started(tx_id))?;

        // 获取已完成参与者列表（逆序）
        let participants: Vec<Participant> = {
            let transactions = self.active_transactions.read().unwrap();
            transactions
                .get(&tx_id)
                .map(|tx| {
                    let mut completed: Vec<_> = tx
                        .participants
                        .iter()
                        .filter(|p| p.status == ParticipantStatus::Completed)
                        .cloned()
                        .collect();
                    completed.sort_by(|a, b| b.order.cmp(&a.order)); // 逆序
                    completed
                })
                .unwrap_or_default()
        };

        // 按逆序执行补偿
        for participant in participants {
            if participant.status == ParticipantStatus::Completed {
                match self.compensate_participant(tx_id, &participant) {
                    Ok(_) => {
                        self.mark_participant_compensated(tx_id, participant.id)?;
                    },
                    Err(e) => {
                        tracing::error!("补偿失败: {} - {}", participant.name, e);
                        // 继续执行其他补偿
                    },
                }
            }
        }

        // 更新状态为已回滚
        self.update_status(tx_id, TransactionStatus::RolledBack)?;
        self.log.append(LogEntry::transaction_rolled_back(tx_id))?;

        tracing::info!("事务回滚完成: {}", tx_id);
        Ok(())
    }

    /// 执行参与者补偿
    fn compensate_participant(
        &self,
        _tx_id: TransactionId,
        participant: &Participant,
    ) -> Result<()> {
        tracing::debug!("补偿参与者: {}", participant.name);
        self.step_executor.execute(&participant.compensate_action)?;
        Ok(())
    }

    /// 更新事务状态
    fn update_status(&self, tx_id: TransactionId, status: TransactionStatus) -> Result<()> {
        let mut states = self.transaction_states.write().unwrap();
        let mut transactions = self.active_transactions.write().unwrap();

        if let Some(state) = states.get_mut(&tx_id) {
            state.transition(status);
        }

        if let Some(tx) = transactions.get_mut(&tx_id) {
            tx.status = status;
            tx.updated_at = chrono::Utc::now();
        }

        Ok(())
    }

    /// 标记参与者执行中
    fn mark_participant_executing(
        &self,
        tx_id: TransactionId,
        participant_id: uuid::Uuid,
    ) -> Result<()> {
        let mut transactions = self.active_transactions.write().unwrap();
        if let Some(tx) = transactions.get_mut(&tx_id) {
            if let Some(p) = tx.participants.iter_mut().find(|p| p.id == participant_id) {
                p.start();
            }
        }
        Ok(())
    }

    /// 标记参与者完成
    fn mark_participant_completed(
        &self,
        tx_id: TransactionId,
        participant_id: uuid::Uuid,
    ) -> Result<()> {
        let mut transactions = self.active_transactions.write().unwrap();
        if let Some(tx) = transactions.get_mut(&tx_id) {
            if let Some(p) = tx.participants.iter_mut().find(|p| p.id == participant_id) {
                p.complete();
            }
        }
        Ok(())
    }

    /// 标记参与者已补偿
    fn mark_participant_compensated(
        &self,
        tx_id: TransactionId,
        participant_id: uuid::Uuid,
    ) -> Result<()> {
        let mut transactions = self.active_transactions.write().unwrap();
        if let Some(tx) = transactions.get_mut(&tx_id) {
            if let Some(p) = tx.participants.iter_mut().find(|p| p.id == participant_id) {
                p.compensate_done();
            }
        }
        Ok(())
    }

    /// 获取事务状态
    pub fn get_status(&self, tx_id: TransactionId) -> Option<TransactionStatus> {
        let transactions = self.active_transactions.read().unwrap();
        transactions.get(&tx_id).map(|tx| tx.status)
    }

    /// 获取事务信息
    pub fn get_transaction(&self, tx_id: TransactionId) -> Option<Transaction> {
        let transactions = self.active_transactions.read().unwrap();
        transactions.get(&tx_id).cloned()
    }

    /// 检查事务超时
    pub fn check_timeouts(&self) -> Result<Vec<TransactionId>> {
        let now = chrono::Utc::now();
        let mut timed_out = Vec::new();

        let transactions = self.active_transactions.read().unwrap();
        for (tx_id, tx) in transactions.iter() {
            if tx.status.can_progress() {
                let elapsed = now.signed_duration_since(tx.created_at);
                if elapsed.num_seconds() as u64 > tx.timeout_secs {
                    timed_out.push(*tx_id);
                }
            }
        }
        drop(transactions);

        // 处理超时事务
        for tx_id in timed_out.iter() {
            self.update_status(*tx_id, TransactionStatus::TimedOut)?;
            self.log.append(LogEntry::transaction_timed_out(*tx_id))?;
        }

        Ok(timed_out)
    }

    /// 获取活跃事务数量
    pub fn active_count(&self) -> usize {
        self.active_transactions.read().unwrap().len()
    }

    /// 清理已完成事务
    pub fn cleanup_completed(&self) -> Result<usize> {
        let mut transactions = self.active_transactions.write().unwrap();
        let mut states = self.transaction_states.write().unwrap();

        let initial_count = transactions.len();

        transactions.retain(|_, tx| !tx.is_completed());
        states.retain(|tx_id, _| transactions.contains_key(tx_id));

        let removed = initial_count - transactions.len();
        if removed > 0 {
            tracing::debug!("清理了 {} 个已完成事务", removed);
        }

        Ok(removed)
    }
}

impl Default for TransactionCoordinator {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// 步骤执行器（模拟实现）
struct StepExecutor {
    // 实际实现中会包含HTTP客户端、数据库连接等
}

impl StepExecutor {
    fn new() -> Self {
        Self {}
    }

    fn execute(&self, action: &crate::participant::ParticipantAction) -> Result<()> {
        // 模拟执行
        tracing::trace!("执行动作: {:?}", action.action_type);

        // 实际实现中会根据action_type执行相应操作
        // 这里只是模拟成功执行
        std::thread::sleep(Duration::from_millis(1));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinator_creation() {
        let coordinator = TransactionCoordinator::with_defaults();
        assert_eq!(coordinator.active_count(), 0);
    }

    #[test]
    fn test_begin_transaction() {
        let coordinator = TransactionCoordinator::with_defaults();
        let tx_id = coordinator.begin("test_transaction").unwrap();

        assert_eq!(coordinator.active_count(), 1);
        assert!(coordinator.get_transaction(tx_id).is_some());
    }

    #[test]
    fn test_add_participant() {
        let coordinator = TransactionCoordinator::with_defaults();
        let tx_id = coordinator.begin("test").unwrap();

        let participant = Participant::new("service1", 1);
        coordinator.add_participant(tx_id, participant).unwrap();

        let tx = coordinator.get_transaction(tx_id).unwrap();
        assert_eq!(tx.participants.len(), 1);
    }

    #[test]
    fn test_commit_transaction() {
        let coordinator = TransactionCoordinator::with_defaults();
        let tx_id = coordinator.begin("test").unwrap();

        let participant = Participant::new("service1", 1);
        coordinator.add_participant(tx_id, participant).unwrap();

        coordinator.commit(tx_id).unwrap();

        let status = coordinator.get_status(tx_id).unwrap();
        assert_eq!(status, TransactionStatus::Committed);
    }

    #[test]
    fn test_rollback_transaction() {
        let coordinator = TransactionCoordinator::with_defaults();
        let tx_id = coordinator.begin("test").unwrap();

        coordinator.rollback(tx_id).unwrap();

        let status = coordinator.get_status(tx_id).unwrap();
        assert_eq!(status, TransactionStatus::RolledBack);
    }

    #[test]
    fn test_cleanup_completed() {
        let coordinator = TransactionCoordinator::with_defaults();
        let tx_id = coordinator.begin("test").unwrap();

        // 直接回滚以完成事务
        coordinator.rollback(tx_id).unwrap();

        let removed = coordinator.cleanup_completed().unwrap();
        assert_eq!(removed, 1);
        assert_eq!(coordinator.active_count(), 0);
    }
}
