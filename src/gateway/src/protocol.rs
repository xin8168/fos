//! # FOS 协议处理模块
//!
//! 处理 FOS 明文协议的解析和校验

use crate::error::{GatewayError, Result};
use crate::{EventStatus, EventType, FosCommand, FosEvent, FourElement, SixAnchor};
use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 协议版本
pub const PROTOCOL_VERSION: &str = "1.0.0";

/// 协议解析器
#[derive(Debug)]
pub struct ProtocolParser {
    /// 事件名称正则
    event_regex: Regex,

    /// 步骤正则
    step_regex: Regex,
}

impl ProtocolParser {
    /// 创建新的协议解析器
    pub fn new() -> Self {
        Self {
            event_regex: Regex::new(r"^[\u4e00-\u9fa5a-zA-Z0-9_\-]+$").unwrap(),
            step_regex: Regex::new(r"^[\u4e00-\u9fa5a-zA-Z0-9_\-\s\.\,]+$").unwrap(),
        }
    }

    /// 解析明文指令
    ///
    /// # 参数
    /// - `input`: 原始输入字符串
    ///
    /// # 返回
    /// 解析后的 FosCommand 或错误
    pub fn parse(&self, input: &str) -> Result<FosCommand> {
        // 尝试解析 JSON 格式
        if let Ok(command) = serde_json::from_str::<FosCommand>(input) {
            self.validate_command(&command)?;
            return Ok(command);
        }

        // 尝试解析明文格式
        self.parse_plaintext(input)
    }

    /// 解析明文格式指令
    fn parse_plaintext(&self, input: &str) -> Result<FosCommand> {
        // 明文格式: fos run --事件 "xxx" --步骤 "xxx" ...
        let mut anchor = SixAnchor {
            event: String::new(),
            steps: Vec::new(),
            judgment_logic: String::new(),
            verification_standard: String::new(),
            location: String::new(),
            subject: String::new(),
        };

        let mut metadata = HashMap::new();

        // 解析参数
        let parts: Vec<&str> = input.split_whitespace().collect();
        let mut i = 0;

        while i < parts.len() {
            match parts[i] {
                "--事件" | "-e" => {
                    i += 1;
                    if i < parts.len() {
                        anchor.event = self.parse_value(parts[i])?;
                    }
                },
                "--步骤" | "-s" => {
                    i += 1;
                    if i < parts.len() {
                        let steps_str = self.parse_value(parts[i])?;
                        anchor.steps =
                            steps_str.split("，").map(|s| s.trim().to_string()).collect();
                    }
                },
                "--判断逻辑" | "-j" => {
                    i += 1;
                    if i < parts.len() {
                        anchor.judgment_logic = self.parse_value(parts[i])?;
                    }
                },
                "--校验标准" | "-v" => {
                    i += 1;
                    if i < parts.len() {
                        anchor.verification_standard = self.parse_value(parts[i])?;
                    }
                },
                "--地点" | "-l" => {
                    i += 1;
                    if i < parts.len() {
                        anchor.location = self.parse_value(parts[i])?;
                    }
                },
                "--主体" | "-u" => {
                    i += 1;
                    if i < parts.len() {
                        anchor.subject = self.parse_value(parts[i])?;
                    }
                },
                _ => {
                    // 其他参数作为元数据
                    if parts[i].starts_with("--") {
                        let key = parts[i].trim_start_matches("--").to_string();
                        i += 1;
                        if i < parts.len() && !parts[i].starts_with("--") {
                            metadata.insert(key, parts[i].to_string());
                        }
                    }
                },
            }
            i += 1;
        }

        // 校验必填字段
        self.validate_anchor(&anchor)?;

        Ok(FosCommand {
            id: Uuid::new_v4().to_string(),
            anchor,
            elements: Vec::new(),
            timestamp: Utc::now().timestamp_millis(),
            metadata,
        })
    }

    /// 解析值（去掉引号）
    fn parse_value(&self, value: &str) -> Result<String> {
        let value = value.trim();
        if value.starts_with('"') && value.ends_with('"') {
            Ok(value[1..value.len() - 1].to_string())
        } else if value.starts_with('\'') && value.ends_with('\'') {
            Ok(value[1..value.len() - 1].to_string())
        } else {
            Ok(value.to_string())
        }
    }

    /// 校验命令
    pub fn validate_command(&self, command: &FosCommand) -> Result<()> {
        self.validate_anchor(&command.anchor)
    }

    /// 校验 6维锚定
    fn validate_anchor(&self, anchor: &SixAnchor) -> Result<()> {
        // 校验事件
        if anchor.event.is_empty() {
            return Err(GatewayError::MissingField("事件(event)".to_string()));
        }

        if !self.event_regex.is_match(&anchor.event) {
            return Err(GatewayError::InvalidField {
                field: "事件".to_string(),
                value: anchor.event.clone(),
            });
        }

        // 校验步骤
        if anchor.steps.is_empty() {
            return Err(GatewayError::MissingField("步骤(steps)".to_string()));
        }

        for (i, step) in anchor.steps.iter().enumerate() {
            if step.is_empty() {
                return Err(GatewayError::InvalidField {
                    field: format!("步骤[{}]", i),
                    value: "空步骤".to_string(),
                });
            }
        }

        // 校验判断逻辑
        if anchor.judgment_logic.is_empty() {
            return Err(GatewayError::MissingField("判断逻辑(judgment_logic)".to_string()));
        }

        // 校验校验标准
        if anchor.verification_standard.is_empty() {
            return Err(GatewayError::MissingField("校验标准(verification_standard)".to_string()));
        }

        // 校验地点
        if anchor.location.is_empty() {
            return Err(GatewayError::MissingField("地点(location)".to_string()));
        }

        // 校验主体
        if anchor.subject.is_empty() {
            return Err(GatewayError::MissingField("主体(subject)".to_string()));
        }

        Ok(())
    }

    /// 将命令转换为事件
    pub fn command_to_event(&self, command: FosCommand, event_type: EventType) -> FosEvent {
        let now = Utc::now().timestamp_millis();
        FosEvent {
            id: command.id.clone(),
            name: command.anchor.event.clone(),
            event_type,
            status: EventStatus::Pending,
            anchor: command.anchor,
            result: None,
            created_at: now,
            updated_at: now,
        }
    }
}

impl Default for ProtocolParser {
    fn default() -> Self {
        Self::new()
    }
}

/// 协议响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ProtocolResponse {
    /// 响应码
    pub code: u16,

    /// 消息
    pub message: String,

    /// 数据
    pub data: Option<serde_json::Value>,
}

impl ProtocolResponse {
    /// 成功响应
    pub fn success(data: Option<serde_json::Value>) -> Self {
        Self { code: 200, message: "成功".to_string(), data }
    }

    /// 错误响应
    pub fn error(code: u16, message: String) -> Self {
        Self { code, message, data: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = ProtocolParser::new();
        assert!(parser.event_regex.is_match("清理桌面文件"));
        assert!(!parser.event_regex.is_match(""));
    }

    #[test]
    fn test_parse_plaintext_command() {
        let parser = ProtocolParser::new();
        let input = r#"fos run --事件 "清理电脑桌面无用文件" --步骤 "列出桌面文件，筛选7天未修改文件，移动到归档文件夹" --判断逻辑 "文件大小<100MB且7天未修改" --校验标准 "归档文件夹出现对应文件" --地点 "我的Windows电脑" --主体 "我""#;

        let result = parser.parse(input);
        assert!(result.is_ok());

        let command = result.unwrap();
        assert_eq!(command.anchor.event, "清理电脑桌面无用文件");
        assert_eq!(command.anchor.steps.len(), 3);
    }

    #[test]
    fn test_validate_missing_event() {
        let parser = ProtocolParser::new();
        let input = r#"fos run --步骤 "步骤1""#;

        let result = parser.parse(input);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, GatewayError::MissingField(_)));
    }

    #[test]
    fn test_parse_json_command() {
        let parser = ProtocolParser::new();
        let json = r#"{
            "id": "test-001",
            "anchor": {
                "event": "测试事件",
                "steps": ["步骤1", "步骤2"],
                "judgment_logic": "条件1",
                "verification_standard": "标准1",
                "location": "位置1",
                "subject": "主体1"
            },
            "elements": [],
            "timestamp": 1700000000,
            "metadata": {}
        }"#;

        let result = parser.parse(json);
        assert!(result.is_ok());

        let command = result.unwrap();
        assert_eq!(command.id, "test-001");
    }

    #[test]
    fn test_command_to_event() {
        let parser = ProtocolParser::new();
        let command = FosCommand {
            id: "cmd-001".to_string(),
            anchor: SixAnchor {
                event: "测试事件".to_string(),
                steps: vec!["步骤1".to_string()],
                judgment_logic: "条件1".to_string(),
                verification_standard: "标准1".to_string(),
                location: "位置1".to_string(),
                subject: "主体1".to_string(),
            },
            elements: vec![],
            timestamp: 1700000000,
            metadata: HashMap::new(),
        };

        let event = parser.command_to_event(command, EventType::DeviceControl);
        assert_eq!(event.name, "测试事件");
        assert_eq!(event.event_type, EventType::DeviceControl);
        assert_eq!(event.status, EventStatus::Pending);
    }
}
