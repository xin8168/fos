//! 事务参与者和动作定义

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 事务参与者
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    /// 参与者ID
    pub id: Uuid,
    /// 参与者名称
    pub name: String,
    /// 参与者类型
    pub participant_type: ParticipantType,
    /// 执行顺序
    pub order: u32,
    /// 执行动作
    pub action: ParticipantAction,
    /// 补偿动作
    pub compensate_action: ParticipantAction,
    /// 当前状态
    pub status: ParticipantStatus,
    /// 重试次数
    pub retry_count: u32,
    /// 最大重试次数
    pub max_retries: u32,
}

impl Participant {
    /// 创建新参与者
    pub fn new(name: &str, order: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            participant_type: ParticipantType::Service,
            order,
            action: ParticipantAction::default(),
            compensate_action: ParticipantAction::default(),
            status: ParticipantStatus::Pending,
            retry_count: 0,
            max_retries: 3,
        }
    }

    /// 设置执行动作
    pub fn with_action(mut self, action: ParticipantAction) -> Self {
        self.action = action;
        self
    }

    /// 设置补偿动作
    pub fn with_compensate_action(mut self, action: ParticipantAction) -> Self {
        self.compensate_action = action;
        self
    }

    /// 设置参与者类型
    pub fn with_type(mut self, participant_type: ParticipantType) -> Self {
        self.participant_type = participant_type;
        self
    }

    /// 设置最大重试次数
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// 开始执行
    pub fn start(&mut self) {
        self.status = ParticipantStatus::Executing;
    }

    /// 执行完成
    pub fn complete(&mut self) {
        self.status = ParticipantStatus::Completed;
    }

    /// 执行失败
    pub fn fail(&mut self) {
        self.status = ParticipantStatus::Failed;
    }

    /// 补偿完成
    pub fn compensate_done(&mut self) {
        self.status = ParticipantStatus::Compensated;
    }

    /// 增加重试次数
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }

    /// 检查是否可以重试
    pub fn can_retry(&self) -> bool {
        matches!(self.status, ParticipantStatus::Failed | ParticipantStatus::Pending)
            && self.retry_count < self.max_retries
    }

    /// 检查是否需要补偿
    pub fn needs_compensation(&self) -> bool {
        matches!(self.status, ParticipantStatus::Completed | ParticipantStatus::Failed)
    }
}

/// 参与者类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantType {
    /// 服务
    Service,
    /// 数据库
    Database,
    /// 消息队列
    MessageQueue,
    /// 外部系统
    External,
    /// 自定义
    Custom,
}

/// 参与者动作定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantAction {
    /// 动作类型
    pub action_type: ActionType,
    /// 目标地址
    pub target: String,
    /// 请求载荷
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
    /// 超时时间（毫秒）
    pub timeout_ms: u64,
}

impl ParticipantAction {
    /// 创建新动作
    pub fn new(action_type: ActionType, target: &str) -> Self {
        Self { action_type, target: target.to_string(), payload: None, timeout_ms: 5000 }
    }

    /// 设置载荷
    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = Some(payload);
        self
    }

    /// 设置超时
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// 创建HTTP动作
    pub fn http(method: &str, url: &str) -> Self {
        Self::new(ActionType::Http(method.to_string()), url)
    }

    /// 创建数据库动作
    pub fn database(query: &str) -> Self {
        Self::new(ActionType::Database, query)
    }

    /// 创建消息队列动作
    pub fn message_queue(topic: &str) -> Self {
        Self::new(ActionType::MessageQueue, topic)
    }
}

impl Default for ParticipantAction {
    fn default() -> Self {
        Self::new(ActionType::None, "")
    }
}

/// 动作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActionType {
    /// 无动作
    None,
    /// HTTP请求
    Http(String),
    /// 数据库操作
    Database,
    /// 消息队列
    MessageQueue,
    /// 自定义动作
    Custom(String),
}

/// 参与者状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantStatus {
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
    /// 超时
    TimedOut,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_participant_creation() {
        let p = Participant::new("test_service", 1);
        assert_eq!(p.name, "test_service");
        assert_eq!(p.order, 1);
        assert_eq!(p.status, ParticipantStatus::Pending);
    }

    #[test]
    fn test_participant_lifecycle() {
        let mut p = Participant::new("test", 1);

        p.start();
        assert_eq!(p.status, ParticipantStatus::Executing);

        p.complete();
        assert_eq!(p.status, ParticipantStatus::Completed);
        assert!(p.needs_compensation());
    }

    #[test]
    fn test_participant_retry() {
        let mut p = Participant::new("test", 1).with_max_retries(2);

        p.fail();
        assert!(p.can_retry());

        p.increment_retry();
        assert!(p.can_retry());

        p.increment_retry();
        assert!(!p.can_retry());
    }

    #[test]
    fn test_action_creation() {
        let action = ParticipantAction::http("POST", "http://example.com/api")
            .with_payload(serde_json::json!({"key": "value"}))
            .with_timeout(10000);

        assert!(matches!(action.action_type, ActionType::Http(_)));
        assert!(action.payload.is_some());
        assert_eq!(action.timeout_ms, 10000);
    }

    #[test]
    fn test_database_action() {
        let action = ParticipantAction::database("INSERT INTO table VALUES (?)");
        assert!(matches!(action.action_type, ActionType::Database));
    }
}
