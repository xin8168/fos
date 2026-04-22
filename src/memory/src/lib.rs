//! # FOS Memory - 硬记忆库模块
//!
//! FOS 硬记忆库，负责存储成功执行的事件
//!
//! ## 核心职责
//! - 存储所有成功执行的事件
//! - 提供事件查询和复用功能
//! - 保证数据的持久化和可靠性
//!
//! ## 安全铁律
//! - 只存储成功执行的事件
//! - 拒绝存储失败或被拦截的事件
//! - 所有数据变更必须有完整日志

pub mod error;
pub mod repository;
pub mod storage;
pub mod version;

pub use error::{MemoryError, Result};
pub use repository::EventRepository;
pub use storage::InMemoryStorage;
pub use version::{ChangeType, EventVersion, Version, VersionHistory, VersionManager};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 事件状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventStatus {
    /// 成功
    Success,

    /// 失败
    Failed,

    /// 已拦截
    Blocked,

    /// 已回滚
    RolledBack,
}

/// 成功事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessEvent {
    /// 事件ID
    pub id: String,

    /// 事件名称
    pub name: String,

    /// 事件类型
    pub event_type: String,

    /// 执行步骤
    pub steps: Vec<String>,

    /// 判断逻辑
    pub judgment_logic: String,

    /// 校验标准
    pub verification_standard: String,

    /// 执行地点
    pub location: String,

    /// 执行主体
    pub subject: String,

    /// 执行结果
    pub result: ExecutionResultData,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,

    /// 元数据
    pub metadata: serde_json::Value,
}

/// 执行结果数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResultData {
    /// 是否成功
    pub success: bool,

    /// 输出内容
    pub output: String,

    /// 执行时间（毫秒）
    pub duration_ms: u64,

    /// 步骤结果
    pub step_results: Vec<StepResultData>,
}

/// 步骤执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResultData {
    /// 步骤索引
    pub step_index: usize,

    /// 是否成功
    pub success: bool,

    /// 输出
    pub output: String,
}

/// 事件查询条件
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventQuery {
    /// 事件名称（模糊匹配）
    pub name: Option<String>,

    /// 事件类型
    pub event_type: Option<String>,

    /// 执行地点
    pub location: Option<String>,

    /// 执行主体
    pub subject: Option<String>,

    /// 开始时间
    pub start_time: Option<DateTime<Utc>>,

    /// 结束时间
    pub end_time: Option<DateTime<Utc>>,

    /// 分页偏移
    pub offset: Option<usize>,

    /// 分页限制
    pub limit: Option<usize>,
}

impl SuccessEvent {
    /// 创建新的成功事件
    pub fn new(
        name: String,
        event_type: String,
        steps: Vec<String>,
        judgment_logic: String,
        verification_standard: String,
        location: String,
        subject: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            event_type,
            steps,
            judgment_logic,
            verification_standard,
            location,
            subject,
            result: ExecutionResultData {
                success: true,
                output: String::new(),
                duration_ms: 0,
                step_results: Vec::new(),
            },
            created_at: now,
            updated_at: now,
            metadata: serde_json::json!({}),
        }
    }

    /// 设置执行结果
    pub fn with_result(mut self, result: ExecutionResultData) -> Self {
        self.result = result;
        self
    }

    /// 设置元数据
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_event_creation() {
        let event = SuccessEvent::new(
            "测试事件".to_string(),
            "device_control".to_string(),
            vec!["步骤1".to_string()],
            "条件1".to_string(),
            "标准1".to_string(),
            "位置1".to_string(),
            "主体1".to_string(),
        );

        assert!(!event.id.is_empty());
        assert_eq!(event.name, "测试事件");
        assert_eq!(event.steps.len(), 1);
    }

    #[test]
    fn test_event_query_default() {
        let query = EventQuery::default();
        assert!(query.name.is_none());
        assert!(query.limit.is_none());
    }
}
