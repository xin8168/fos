//! FOS 事件结构

use serde::{Deserialize, Serialize};

use crate::anchor::SixAnchor;
use crate::result::ExecutionResult;
use crate::status::{EventStatus, EventType};

/// 事件元数据
pub type EventMetadata = std::collections::HashMap<String, String>;

/// FOS 事件结构
///
/// 代表一个完整的 FOS 执行事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FosEvent {
    /// 事件ID
    pub id: String,

    /// 事件名称
    pub name: String,

    /// 事件类型
    pub event_type: EventType,

    /// 事件状态
    pub status: EventStatus,

    /// 6维锚定
    pub anchor: SixAnchor,

    /// 执行结果
    pub result: Option<ExecutionResult>,

    /// 创建时间戳
    pub created_at: i64,

    /// 更新时间戳
    pub updated_at: i64,
}

impl FosEvent {
    /// 创建新的事件
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        event_type: EventType,
        anchor: SixAnchor,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: id.into(),
            name: name.into(),
            event_type,
            status: EventStatus::Pending,
            anchor,
            result: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// 更新事件状态
    pub fn with_status(mut self, status: EventStatus) -> Self {
        self.status = status;
        self.updated_at = chrono::Utc::now().timestamp();
        self
    }

    /// 设置执行结果
    pub fn with_result(mut self, result: ExecutionResult) -> Self {
        self.result = Some(result);
        self.updated_at = chrono::Utc::now().timestamp();
        self
    }

    /// 判断事件是否处于终态
    pub fn is_terminal(&self) -> bool {
        self.status.is_terminal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fos_event_creation() {
        let anchor =
            SixAnchor::new("测试事件", vec!["步骤1".into()], "条件1", "标准1", "位置1", "主体1");
        let event = FosEvent::new("evt-001", "测试事件", EventType::DeviceControl, anchor);

        assert_eq!(event.id, "evt-001");
        assert_eq!(event.status, EventStatus::Pending);
        assert!(!event.is_terminal());
    }

    #[test]
    fn test_fos_event_status_transition() {
        let anchor =
            SixAnchor::new("测试事件", vec!["步骤1".into()], "条件1", "标准1", "位置1", "主体1");
        let event = FosEvent::new("evt-001", "测试事件", EventType::DeviceControl, anchor)
            .with_status(EventStatus::Executing)
            .with_status(EventStatus::Success);

        assert_eq!(event.status, EventStatus::Success);
        assert!(event.is_terminal());
    }
}
