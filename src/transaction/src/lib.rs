//! # FOS Transaction - 事务管理模块
//!
//! 提供分布式事务协调能力，确保数据一致性
//!
//! ## 核心职责
//! - 分布式事务协调（Saga模式）
//! - 事务状态管理
//! - 补偿事务执行
//! - 事务恢复机制
//!
//! ## 安全铁律
//! - 不做规则判断
//! - 不执行业务逻辑
//! - 只负责事务协调和状态管理

pub mod config;
pub mod coordinator;
pub mod error;
pub mod log;
pub mod participant;
pub mod state;

pub use config::TransactionConfig;
pub use coordinator::TransactionCoordinator;
pub use error::{Error, Result};
pub use log::{LogEntry, TransactionLog};
pub use participant::{Participant, ParticipantAction};
pub use state::{TransactionState, TransactionStatus};

/// 模块版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 模块名称
pub const NAME: &str = env!("CARGO_PKG_NAME");

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 事务ID类型
pub type TransactionId = Uuid;

/// 步骤ID类型
pub type StepId = Uuid;

/// 事务定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// 事务ID
    pub id: TransactionId,
    /// 事务名称
    pub name: String,
    /// 当前状态
    pub status: TransactionStatus,
    /// 参与者列表
    pub participants: Vec<Participant>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 超时时间
    pub timeout_secs: u64,
    /// 元数据
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Transaction {
    /// 创建新事务
    pub fn new(name: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            status: TransactionStatus::Created,
            participants: Vec::new(),
            created_at: now,
            updated_at: now,
            timeout_secs: 60,
            metadata: HashMap::new(),
        }
    }

    /// 添加参与者
    pub fn add_participant(&mut self, participant: Participant) {
        self.participants.push(participant);
        self.updated_at = Utc::now();
    }

    /// 获取参与者数量
    pub fn participant_count(&self) -> usize {
        self.participants.len()
    }

    /// 检查是否可以提交
    pub fn can_commit(&self) -> bool {
        matches!(self.status, TransactionStatus::Created | TransactionStatus::Pending)
            && !self.participants.is_empty()
    }

    /// 检查是否可以回滚
    pub fn can_rollback(&self) -> bool {
        matches!(
            self.status,
            TransactionStatus::Created
                | TransactionStatus::Pending
                | TransactionStatus::PartiallyCommitted
                | TransactionStatus::Failed
        )
    }

    /// 检查是否已完成
    pub fn is_completed(&self) -> bool {
        matches!(
            self.status,
            TransactionStatus::Committed
                | TransactionStatus::RolledBack
                | TransactionStatus::TimedOut
        )
    }

    /// 设置超时时间
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }
}

/// 事务步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStep {
    /// 步骤ID
    pub id: StepId,
    /// 步骤名称
    pub name: String,
    /// 步骤顺序
    pub order: u32,
    /// 执行动作
    pub action: String,
    /// 补偿动作
    pub compensate_action: String,
    /// 步骤状态
    pub status: StepStatus,
    /// 执行结果
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// 重试次数
    pub retry_count: u32,
}

impl TransactionStep {
    /// 创建新步骤
    pub fn new(name: &str, order: u32, action: &str, compensate_action: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            order,
            action: action.to_string(),
            compensate_action: compensate_action.to_string(),
            status: StepStatus::Pending,
            result: None,
            retry_count: 0,
        }
    }

    /// 标记为执行中
    pub fn start(&mut self) {
        self.status = StepStatus::Executing;
    }

    /// 标记为完成
    pub fn complete(&mut self, result: Option<serde_json::Value>) {
        self.status = StepStatus::Completed;
        self.result = result;
    }

    /// 标记为失败
    pub fn fail(&mut self) {
        self.status = StepStatus::Failed;
    }

    /// 标记为已补偿
    pub fn compensate(&mut self) {
        self.status = StepStatus::Compensated;
    }

    /// 增加重试次数
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }

    /// 检查是否可重试
    pub fn can_retry(&self, max_retries: u32) -> bool {
        matches!(self.status, StepStatus::Failed | StepStatus::Pending)
            && self.retry_count < max_retries
    }
}

/// 步骤状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    /// 待执行
    Pending,
    /// 执行中
    Executing,
    /// 已完成
    Completed,
    /// 已失败
    Failed,
    /// 已补偿
    Compensated,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let tx = Transaction::new("test_transaction");
        assert_eq!(tx.name, "test_transaction");
        assert_eq!(tx.status, TransactionStatus::Created);
        assert!(tx.participants.is_empty());
    }

    #[test]
    fn test_transaction_with_timeout() {
        let tx = Transaction::new("test").with_timeout(120);
        assert_eq!(tx.timeout_secs, 120);
    }

    #[test]
    fn test_transaction_can_commit() {
        let mut tx = Transaction::new("test");
        // 添加参与者后才能提交
        tx.add_participant(crate::participant::Participant::new("service1", 1));
        assert!(tx.can_commit());
    }

    #[test]
    fn test_step_creation() {
        let step = TransactionStep::new("step1", 1, "execute", "compensate");
        assert_eq!(step.name, "step1");
        assert_eq!(step.order, 1);
        assert_eq!(step.status, StepStatus::Pending);
    }

    #[test]
    fn test_step_lifecycle() {
        let mut step = TransactionStep::new("step1", 1, "execute", "compensate");

        step.start();
        assert_eq!(step.status, StepStatus::Executing);

        step.complete(Some(serde_json::json!({"result": "ok"})));
        assert_eq!(step.status, StepStatus::Completed);
        assert!(step.result.is_some());
    }

    #[test]
    fn test_step_retry() {
        let mut step = TransactionStep::new("step1", 1, "execute", "compensate");
        step.fail();

        assert!(step.can_retry(3));

        step.increment_retry();
        step.increment_retry();
        step.increment_retry();

        assert!(!step.can_retry(3));
    }
}
