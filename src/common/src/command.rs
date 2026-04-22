//! FOS 完整命令结构

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::anchor::SixAnchor;
use crate::element::FourElement;

/// FOS 完整命令结构
///
/// 包含完整的 FOS 执行指令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FosCommand {
    /// 命令唯一ID
    pub id: String,

    /// 6维锚定
    pub anchor: SixAnchor,

    /// 4要素执行（每个步骤的详细定义）
    pub elements: Vec<FourElement>,

    /// 时间戳
    pub timestamp: i64,

    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl FosCommand {
    /// 创建新的 FOS 命令
    pub fn new(id: impl Into<String>, anchor: SixAnchor, elements: Vec<FourElement>) -> Self {
        Self {
            id: id.into(),
            anchor,
            elements,
            timestamp: chrono::Utc::now().timestamp(),
            metadata: HashMap::new(),
        }
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// 验证命令是否有效
    pub fn validate(&self) -> bool {
        !self.id.is_empty() && self.anchor.validate() && self.elements.iter().all(|e| e.validate())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fos_command_creation() {
        let anchor =
            SixAnchor::new("测试事件", vec!["步骤1".into()], "条件1", "标准1", "位置1", "主体1");
        let command = FosCommand::new("cmd-001", anchor, vec![]);

        assert_eq!(command.id, "cmd-001");
        assert!(!command.metadata.contains_key("test"));
    }

    #[test]
    fn test_fos_command_with_metadata() {
        let anchor =
            SixAnchor::new("测试事件", vec!["步骤1".into()], "条件1", "标准1", "位置1", "主体1");
        let command = FosCommand::new("cmd-001", anchor, vec![]).with_metadata("key", "value");

        assert_eq!(command.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_fos_command_serialization() {
        let anchor =
            SixAnchor::new("测试事件", vec!["步骤1".into()], "条件1", "标准1", "位置1", "主体1");
        let command = FosCommand::new("cmd-001", anchor, vec![]);

        let json = serde_json::to_string(&command).unwrap();
        assert!(json.contains("测试事件"));

        let deserialized: FosCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "cmd-001");
    }
}
