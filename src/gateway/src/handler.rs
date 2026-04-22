//! # 命令处理器模块

use crate::error::{GatewayError, Result};
use crate::protocol::ProtocolParser;
use crate::{EventStatus, EventType, FosCommand, FosEvent};
use std::sync::Arc;
use tokio::sync::mpsc;

/// 命令处理器
#[derive(Debug)]
pub struct CommandHandler {
    /// 事件发送器
    event_tx: mpsc::Sender<FosEvent>,
}

impl CommandHandler {
    /// 创建新的命令处理器
    pub fn new() -> Self {
        let (event_tx, _event_rx) = mpsc::channel(1024);
        // 保持接收端存活，防止channel关闭
        std::mem::forget(_event_rx);
        Self { event_tx }
    }

    /// 创建带接收器的命令处理器（用于测试）
    #[cfg(test)]
    pub fn new_with_receiver() -> (Self, mpsc::Receiver<FosEvent>) {
        let (event_tx, event_rx) = mpsc::channel(1024);
        (Self { event_tx }, event_rx)
    }

    /// 处理命令
    pub async fn handle(&self, command: FosCommand) -> Result<FosEvent> {
        // 确定事件类型
        let event_type = self.determine_event_type(&command);

        // 创建事件
        let mut event = FosEvent {
            id: command.id.clone(),
            name: command.anchor.event.clone(),
            event_type,
            status: EventStatus::Pending,
            anchor: command.anchor,
            result: None,
            created_at: chrono::Utc::now().timestamp_millis(),
            updated_at: chrono::Utc::now().timestamp_millis(),
        };

        // 执行预校验
        match self.pre_validate(&event) {
            Ok(_) => {
                event.status = EventStatus::Validating;

                // 发送事件到校验引擎
                self.event_tx
                    .send(event.clone())
                    .await
                    .map_err(|e| GatewayError::Internal(e.to_string()))?;

                Ok(event)
            },
            Err(e) => {
                event.status = EventStatus::Blocked;
                Err(e)
            },
        }
    }

    /// 确定事件类型
    fn determine_event_type(&self, command: &FosCommand) -> EventType {
        let event = &command.anchor.event;

        if event.contains("设备") || event.contains("控制") {
            EventType::DeviceControl
        } else if event.contains("文件") {
            EventType::FileOperation
        } else if event.contains("系统") || event.contains("命令") {
            EventType::SystemCommand
        } else if event.contains("网络") {
            EventType::NetworkOperation
        } else if event.contains("自动化") {
            EventType::AutomationTask
        } else {
            EventType::SkillExecution
        }
    }

    /// 预校验
    fn pre_validate(&self, event: &FosEvent) -> Result<()> {
        // 检查是否包含危险操作
        let dangerous_patterns = ["删除系统", "格式化", "rm -rf /", "del /s", "shutdown", "reboot"];

        for pattern in &dangerous_patterns {
            if event.anchor.event.contains(pattern) {
                return Err(GatewayError::CommandBlocked {
                    reason: format!("检测到危险操作: {}", pattern),
                });
            }
        }

        // 检查步骤是否为空
        if event.anchor.steps.is_empty() {
            return Err(GatewayError::ValidationFailed("步骤不能为空".to_string()));
        }

        Ok(())
    }
}

impl Default for CommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_creation() {
        let handler = CommandHandler::new();
        assert!(handler.event_tx.capacity() > 0);
    }

    #[tokio::test]
    async fn test_handle_command() {
        let handler = CommandHandler::new();
        let command = FosCommand {
            id: "test-001".to_string(),
            anchor: crate::SixAnchor {
                event: "清理桌面文件".to_string(),
                steps: vec!["列出文件".to_string(), "删除临时文件".to_string()],
                judgment_logic: "文件为临时文件".to_string(),
                verification_standard: "桌面干净".to_string(),
                location: "我的电脑".to_string(),
                subject: "用户".to_string(),
            },
            elements: vec![],
            timestamp: 1700000000,
            metadata: std::collections::HashMap::new(),
        };

        let result = handler.handle(command).await;
        if let Err(ref e) = result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());

        let event = result.unwrap();
        assert_eq!(event.status, EventStatus::Validating);
    }

    #[tokio::test]
    async fn test_dangerous_command_blocked() {
        let handler = CommandHandler::new();
        let command = FosCommand {
            id: "test-002".to_string(),
            anchor: crate::SixAnchor {
                event: "删除系统文件".to_string(),
                steps: vec!["步骤1".to_string()],
                judgment_logic: "条件1".to_string(),
                verification_standard: "标准1".to_string(),
                location: "位置1".to_string(),
                subject: "主体1".to_string(),
            },
            elements: vec![],
            timestamp: 1700000000,
            metadata: std::collections::HashMap::new(),
        };

        let result = handler.handle(command).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, GatewayError::CommandBlocked { .. }));
    }
}
