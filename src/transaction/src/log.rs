//! 事务日志

use crate::error::Result;
use crate::{StepId, TransactionId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// 事务日志
#[derive(Debug)]
pub struct TransactionLog {
    /// 日志条目
    entries: std::sync::Mutex<VecDeque<LogEntry>>,
    /// 最大条目数
    max_entries: usize,
}

impl TransactionLog {
    /// 创建新的事务日志
    pub fn new() -> Self {
        Self { entries: std::sync::Mutex::new(VecDeque::new()), max_entries: 10000 }
    }

    /// 设置最大条目数
    pub fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    /// 追加日志条目
    pub fn append(&self, entry: LogEntry) -> Result<()> {
        let mut entries = self.entries.lock().unwrap();

        // 如果超过最大条目数，移除最旧的条目
        while entries.len() >= self.max_entries {
            entries.pop_front();
        }

        entries.push_back(entry);
        Ok(())
    }

    /// 获取指定事务的所有日志
    pub fn get_by_transaction(&self, tx_id: TransactionId) -> Vec<LogEntry> {
        let entries = self.entries.lock().unwrap();
        entries.iter().filter(|e| e.transaction_id == tx_id).cloned().collect()
    }

    /// 获取最近N条日志
    pub fn get_recent(&self, count: usize) -> Vec<LogEntry> {
        let entries = self.entries.lock().unwrap();
        entries.iter().rev().take(count).cloned().collect()
    }

    /// 获取日志条目总数
    pub fn len(&self) -> usize {
        self.entries.lock().unwrap().len()
    }

    /// 检查日志是否为空
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 清空日志
    pub fn clear(&self) {
        self.entries.lock().unwrap().clear();
    }
}

impl Default for TransactionLog {
    fn default() -> Self {
        Self::new()
    }
}

/// 日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// 条目ID
    pub id: uuid::Uuid,
    /// 事务ID
    pub transaction_id: TransactionId,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 事件类型
    pub event_type: TransactionEvent,
    /// 步骤ID（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step_id: Option<StepId>,
    /// 参与者ID（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant_id: Option<uuid::Uuid>,
    /// 详细信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl LogEntry {
    /// 创建新条目
    fn new(tx_id: TransactionId, event: TransactionEvent) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            transaction_id: tx_id,
            timestamp: Utc::now(),
            event_type: event,
            step_id: None,
            participant_id: None,
            details: None,
        }
    }

    /// 事务开始
    pub fn transaction_started(tx_id: TransactionId, name: &str) -> Self {
        let mut entry = Self::new(tx_id, TransactionEvent::Started);
        entry.details = Some(format!("事务名称: {}", name));
        entry
    }

    /// 事务提交开始
    pub fn commit_started(tx_id: TransactionId) -> Self {
        Self::new(tx_id, TransactionEvent::CommitStarted)
    }

    /// 事务提交完成
    pub fn transaction_committed(tx_id: TransactionId) -> Self {
        Self::new(tx_id, TransactionEvent::Committed)
    }

    /// 回滚开始
    pub fn rollback_started(tx_id: TransactionId) -> Self {
        Self::new(tx_id, TransactionEvent::RollbackStarted)
    }

    /// 回滚完成
    pub fn transaction_rolled_back(tx_id: TransactionId) -> Self {
        Self::new(tx_id, TransactionEvent::RolledBack)
    }

    /// 事务超时
    pub fn transaction_timed_out(tx_id: TransactionId) -> Self {
        Self::new(tx_id, TransactionEvent::TimedOut)
    }

    /// 参与者添加
    pub fn participant_added(tx_id: TransactionId, participant_id: uuid::Uuid) -> Self {
        let mut entry = Self::new(tx_id, TransactionEvent::ParticipantAdded);
        entry.participant_id = Some(participant_id);
        entry
    }

    /// 步骤开始
    pub fn step_started(tx_id: TransactionId, participant_id: uuid::Uuid) -> Self {
        let mut entry = Self::new(tx_id, TransactionEvent::StepStarted);
        entry.participant_id = Some(participant_id);
        entry
    }

    /// 步骤完成
    pub fn step_completed(tx_id: TransactionId, participant_id: uuid::Uuid) -> Self {
        let mut entry = Self::new(tx_id, TransactionEvent::StepCompleted);
        entry.participant_id = Some(participant_id);
        entry
    }

    /// 步骤失败
    pub fn step_failed(tx_id: TransactionId, participant_id: uuid::Uuid, error: &str) -> Self {
        let mut entry = Self::new(tx_id, TransactionEvent::StepFailed);
        entry.participant_id = Some(participant_id);
        entry.details = Some(error.to_string());
        entry
    }

    /// 补偿开始
    pub fn compensate_started(tx_id: TransactionId, participant_id: uuid::Uuid) -> Self {
        let mut entry = Self::new(tx_id, TransactionEvent::CompensateStarted);
        entry.participant_id = Some(participant_id);
        entry
    }

    /// 补偿完成
    pub fn compensate_completed(tx_id: TransactionId, participant_id: uuid::Uuid) -> Self {
        let mut entry = Self::new(tx_id, TransactionEvent::CompensateCompleted);
        entry.participant_id = Some(participant_id);
        entry
    }
}

/// 事务事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionEvent {
    /// 事务开始
    Started,
    /// 提交开始
    CommitStarted,
    /// 已提交
    Committed,
    /// 回滚开始
    RollbackStarted,
    /// 已回滚
    RolledBack,
    /// 超时
    TimedOut,
    /// 参与者添加
    ParticipantAdded,
    /// 步骤开始
    StepStarted,
    /// 步骤完成
    StepCompleted,
    /// 步骤失败
    StepFailed,
    /// 补偿开始
    CompensateStarted,
    /// 补偿完成
    CompensateCompleted,
}

impl std::fmt::Display for TransactionEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionEvent::Started => write!(f, "事务开始"),
            TransactionEvent::CommitStarted => write!(f, "提交开始"),
            TransactionEvent::Committed => write!(f, "已提交"),
            TransactionEvent::RollbackStarted => write!(f, "回滚开始"),
            TransactionEvent::RolledBack => write!(f, "已回滚"),
            TransactionEvent::TimedOut => write!(f, "超时"),
            TransactionEvent::ParticipantAdded => write!(f, "参与者添加"),
            TransactionEvent::StepStarted => write!(f, "步骤开始"),
            TransactionEvent::StepCompleted => write!(f, "步骤完成"),
            TransactionEvent::StepFailed => write!(f, "步骤失败"),
            TransactionEvent::CompensateStarted => write!(f, "补偿开始"),
            TransactionEvent::CompensateCompleted => write!(f, "补偿完成"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_creation() {
        let log = TransactionLog::new();
        assert!(log.is_empty());
    }

    #[test]
    fn test_append_entry() {
        let log = TransactionLog::new();
        let tx_id = uuid::Uuid::new_v4();

        log.append(LogEntry::transaction_started(tx_id, "test")).unwrap();

        assert_eq!(log.len(), 1);
    }

    #[test]
    fn test_get_by_transaction() {
        let log = TransactionLog::new();
        let tx_id = uuid::Uuid::new_v4();

        log.append(LogEntry::transaction_started(tx_id, "test")).unwrap();
        log.append(LogEntry::commit_started(tx_id)).unwrap();

        let entries = log.get_by_transaction(tx_id);
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_get_recent() {
        let log = TransactionLog::new();
        let tx_id = uuid::Uuid::new_v4();

        log.append(LogEntry::transaction_started(tx_id, "test")).unwrap();
        log.append(LogEntry::commit_started(tx_id)).unwrap();
        log.append(LogEntry::transaction_committed(tx_id)).unwrap();

        let recent = log.get_recent(2);
        assert_eq!(recent.len(), 2);
    }

    #[test]
    fn test_max_entries() {
        let log = TransactionLog::new().with_max_entries(3);
        let tx_id = uuid::Uuid::new_v4();

        for i in 0..5 {
            log.append(LogEntry::transaction_started(tx_id, &format!("test{}", i))).unwrap();
        }

        assert_eq!(log.len(), 3);
    }

    #[test]
    fn test_clear() {
        let log = TransactionLog::new();
        let tx_id = uuid::Uuid::new_v4();

        log.append(LogEntry::transaction_started(tx_id, "test")).unwrap();
        assert!(!log.is_empty());

        log.clear();
        assert!(log.is_empty());
    }
}
