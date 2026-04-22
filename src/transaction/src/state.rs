//! 事务状态定义

use serde::{Deserialize, Serialize};

/// 事务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    /// 已创建
    Created,
    /// 待处理
    Pending,
    /// 部分提交
    PartiallyCommitted,
    /// 已提交
    Committed,
    /// 已回滚
    RolledBack,
    /// 已超时
    TimedOut,
    /// 已失败
    Failed,
}

impl TransactionStatus {
    /// 检查是否为终态
    pub fn is_final(&self) -> bool {
        matches!(
            self,
            TransactionStatus::Committed
                | TransactionStatus::RolledBack
                | TransactionStatus::TimedOut
                | TransactionStatus::Failed
        )
    }

    /// 检查是否可以推进
    pub fn can_progress(&self) -> bool {
        !self.is_final()
    }

    /// 获取状态描述
    pub fn description(&self) -> &str {
        match self {
            TransactionStatus::Created => "事务已创建",
            TransactionStatus::Pending => "事务待处理",
            TransactionStatus::PartiallyCommitted => "事务部分提交",
            TransactionStatus::Committed => "事务已提交",
            TransactionStatus::RolledBack => "事务已回滚",
            TransactionStatus::TimedOut => "事务已超时",
            TransactionStatus::Failed => "事务已失败",
        }
    }
}

impl std::fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap_or_default())
    }
}

/// 事务状态机
#[derive(Debug, Clone)]
pub struct TransactionState {
    /// 当前状态
    status: TransactionStatus,
    /// 状态变更历史
    history: Vec<(TransactionStatus, chrono::DateTime<chrono::Utc>)>,
}

impl TransactionState {
    /// 创建新状态
    pub fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            status: TransactionStatus::Created,
            history: vec![(TransactionStatus::Created, now)],
        }
    }

    /// 获取当前状态
    pub fn current(&self) -> TransactionStatus {
        self.status
    }

    /// 推进到新状态
    pub fn transition(&mut self, new_status: TransactionStatus) -> bool {
        if self.can_transition_to(new_status) {
            let now = chrono::Utc::now();
            self.status = new_status;
            self.history.push((new_status, now));
            true
        } else {
            false
        }
    }

    /// 检查是否可以转换到目标状态
    pub fn can_transition_to(&self, target: TransactionStatus) -> bool {
        match self.status {
            TransactionStatus::Created => matches!(
                target,
                TransactionStatus::Pending
                    | TransactionStatus::RolledBack
                    | TransactionStatus::TimedOut
                    | TransactionStatus::Failed
            ),
            TransactionStatus::Pending => matches!(
                target,
                TransactionStatus::PartiallyCommitted
                    | TransactionStatus::Committed
                    | TransactionStatus::RolledBack
                    | TransactionStatus::TimedOut
                    | TransactionStatus::Failed
            ),
            TransactionStatus::PartiallyCommitted => matches!(
                target,
                TransactionStatus::Committed
                    | TransactionStatus::RolledBack
                    | TransactionStatus::Failed
            ),
            TransactionStatus::Committed
            | TransactionStatus::RolledBack
            | TransactionStatus::TimedOut
            | TransactionStatus::Failed => false,
        }
    }

    /// 获取状态历史
    pub fn history(&self) -> &[(TransactionStatus, chrono::DateTime<chrono::Utc>)] {
        &self.history
    }

    /// 获取状态变更次数
    pub fn transition_count(&self) -> usize {
        self.history.len().saturating_sub(1)
    }
}

impl Default for TransactionState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_is_final() {
        assert!(!TransactionStatus::Created.is_final());
        assert!(!TransactionStatus::Pending.is_final());
        assert!(TransactionStatus::Committed.is_final());
        assert!(TransactionStatus::RolledBack.is_final());
        assert!(TransactionStatus::Failed.is_final());
    }

    #[test]
    fn test_state_creation() {
        let state = TransactionState::new();
        assert_eq!(state.current(), TransactionStatus::Created);
    }

    #[test]
    fn test_state_transition() {
        let mut state = TransactionState::new();

        assert!(state.transition(TransactionStatus::Pending));
        assert_eq!(state.current(), TransactionStatus::Pending);

        assert!(state.transition(TransactionStatus::Committed));
        assert_eq!(state.current(), TransactionStatus::Committed);
    }

    #[test]
    fn test_invalid_transition() {
        let mut state = TransactionState::new();

        // Created -> Committed 是非法转换
        assert!(!state.transition(TransactionStatus::Committed));
        assert_eq!(state.current(), TransactionStatus::Created);
    }

    #[test]
    fn test_state_history() {
        let mut state = TransactionState::new();
        state.transition(TransactionStatus::Pending);
        state.transition(TransactionStatus::Committed);

        assert_eq!(state.transition_count(), 2);
        assert_eq!(state.history().len(), 3);
    }

    #[test]
    fn test_status_description() {
        assert!(!TransactionStatus::Created.description().is_empty());
    }
}
