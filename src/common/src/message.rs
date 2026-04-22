//! FOS 通信层 - 事件通道
//!
//! 提供模块间通信能力
//! Gateway → Validator → Bus 数据流

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 通信消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum FosMessage {
    /// 命令请求
    CommandRequest(CommandRequest),
    /// 命令响应
    CommandResponse(CommandResponse),
    /// 校验请求
    ValidationRequest(ValidationRequest),
    /// 校验响应
    ValidationResponse(ValidationResponse),
    /// 执行请求
    ExecutionRequest(ExecutionRequest),
    /// 执行响应
    ExecutionResponse(ExecutionResponse),
    /// 心跳
    Heartbeat(HeartbeatMessage),
}

/// 命令请求 - Gateway → Validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRequest {
    pub id: String,
    pub event: String,
    pub steps: Vec<String>,
    pub judgment_logic: String,
    pub verification_standard: String,
    pub location: String,
    pub subject: String,
    pub metadata: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

/// 命令响应 - Validator → Gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse {
    pub request_id: String,
    pub success: bool,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// 校验请求 - Validator → Bus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRequest {
    pub id: String,
    pub command_id: String,
    pub event: String,
    pub steps: Vec<String>,
    pub judgment_logic: String,
    pub verification_standard: String,
    pub location: String,
    pub subject: String,
    pub metadata: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

/// 校验响应 - Bus → Validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResponse {
    pub request_id: String,
    pub passed: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

/// 执行请求 - Bus 执行
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub id: String,
    pub validation_id: String,
    pub command_id: String,
    pub event: String,
    pub steps: Vec<String>,
    pub judgment_logic: String,
    pub verification_standard: String,
    pub location: String,
    pub subject: String,
    pub metadata: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

/// 执行响应 - Bus → Gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResponse {
    pub request_id: String,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
}

/// 心跳消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatMessage {
    pub source: String,
    pub timestamp: DateTime<Utc>,
    pub status: String,
}

impl FosMessage {
    /// 创建命令请求
    pub fn new_command_request(req: CommandRequest) -> Self {
        FosMessage::CommandRequest(req)
    }

    /// 创建校验请求
    pub fn new_validation_request(req: ValidationRequest) -> Self {
        FosMessage::ValidationRequest(req)
    }

    /// 创建执行请求
    pub fn new_execution_request(req: ExecutionRequest) -> Self {
        FosMessage::ExecutionRequest(req)
    }
}

/// 消息通道工厂
pub mod channel {
    use super::FosMessage;
    use tokio::sync::mpsc;

    /// 创建消息通道
    pub fn create_channel(
        capacity: usize,
    ) -> (mpsc::Sender<FosMessage>, mpsc::Receiver<FosMessage>) {
        mpsc::channel(capacity)
    }
}

/// 消息路由器
#[derive(Debug)]
pub struct MessageRouter {
    routes: HashMap<String, tokio::sync::mpsc::Sender<FosMessage>>,
}

impl MessageRouter {
    pub fn new() -> Self {
        Self { routes: HashMap::new() }
    }

    /// 注册路由
    pub fn register(&mut self, name: String, sender: tokio::sync::mpsc::Sender<FosMessage>) {
        self.routes.insert(name, sender);
    }

    /// 发送到指定模块
    pub async fn send_to(&self, name: &str, msg: FosMessage) -> Result<(), String> {
        if let Some(sender) = self.routes.get(name) {
            sender.send(msg).await.map_err(|e| e.to_string())
        } else {
            Err(format!("路由 {} 不存在", name))
        }
    }

    /// 广播到所有注册的模块
    pub async fn broadcast(&self, msg: FosMessage) -> usize {
        let mut count = 0;
        for sender in self.routes.values() {
            if sender.send(msg.clone()).await.is_ok() {
                count += 1;
            }
        }
        count
    }
}

impl Default for MessageRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[test]
    fn test_command_request_creation() {
        let req = CommandRequest {
            id: "req-001".to_string(),
            event: "测试事件".to_string(),
            steps: vec!["步骤1".to_string()],
            judgment_logic: "条件1".to_string(),
            verification_standard: "标准1".to_string(),
            location: "位置1".to_string(),
            subject: "主体1".to_string(),
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        };

        assert_eq!(req.event, "测试事件");
    }

    #[test]
    fn test_message_router() {
        let mut router = MessageRouter::new();

        let (tx, _rx) = mpsc::channel::<FosMessage>(10);
        router.register("validator".to_string(), tx);

        assert!(router.routes.contains_key("validator"));
    }

    #[tokio::test]
    async fn test_channel_creation() {
        let (tx, mut rx) = channel::create_channel(10);

        tx.send(FosMessage::Heartbeat(HeartbeatMessage {
            source: "test".to_string(),
            timestamp: Utc::now(),
            status: "active".to_string(),
        }))
        .await
        .unwrap();

        let msg = rx.recv().await.unwrap();
        assert!(matches!(msg, FosMessage::Heartbeat(_)));
    }
}
