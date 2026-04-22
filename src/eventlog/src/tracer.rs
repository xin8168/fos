//! # 事件追踪模块
//!
//! 负责事件的链路追踪和记录

use crate::error::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 事件ID
pub type EventId = String;

/// 追踪ID（用于关联多个事件）
pub type TraceId = String;

/// 跨度ID
pub type SpanId = String;

/// 事件级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventLevel {
    /// 信息
    Info,
    /// 警告
    Warning,
    /// 错误
    Error,
    /// 调试
    Debug,
}

/// 事件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// 命令执行
    CommandExecution,
    /// 状态变更
    StateChange,
    /// 权限检查
    PermissionCheck,
    /// 数据操作
    DataOperation,
    /// 系统事件
    SystemEvent,
    /// 自定义
    Custom(String),
}

/// 事件状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventStatus {
    /// 成功
    Success,
    /// 失败
    Failed,
    /// 进行中
    InProgress,
    /// 已取消
    Cancelled,
}

/// 事件日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLog {
    /// 事件ID
    pub id: EventId,

    /// 追踪ID
    pub trace_id: TraceId,

    /// 父跨度ID
    pub parent_span_id: Option<SpanId>,

    /// 当前跨度ID
    pub span_id: SpanId,

    /// 事件名称
    pub name: String,

    /// 事件类型
    pub event_type: EventType,

    /// 事件级别
    pub level: EventLevel,

    /// 事件状态
    pub status: EventStatus,

    /// 事件数据
    pub data: serde_json::Value,

    /// 开始时间
    pub started_at: DateTime<Utc>,

    /// 结束时间
    pub ended_at: Option<DateTime<Utc>>,

    /// 持续时间（毫秒）
    pub duration_ms: Option<u64>,

    /// 错误信息
    pub error: Option<String>,

    /// 来源
    pub source: String,

    /// 标签
    pub tags: HashMap<String, String>,

    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

impl EventLog {
    /// 创建新事件
    pub fn new(trace_id: TraceId, name: String, event_type: EventType, source: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            trace_id,
            parent_span_id: None,
            span_id: uuid::Uuid::new_v4().to_string(),
            name,
            event_type,
            level: EventLevel::Info,
            status: EventStatus::InProgress,
            data: serde_json::json!({}),
            started_at: Utc::now(),
            ended_at: None,
            duration_ms: None,
            error: None,
            source,
            tags: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// 设置父跨度
    pub fn with_parent(mut self, parent_span_id: SpanId) -> Self {
        self.parent_span_id = Some(parent_span_id);
        self
    }

    /// 设置级别
    pub fn with_level(mut self, level: EventLevel) -> Self {
        self.level = level;
        self
    }

    /// 设置数据
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }

    /// 添加标签
    pub fn add_tag(&mut self, key: &str, value: &str) {
        self.tags.insert(key.to_string(), value.to_string());
    }

    /// 添加元数据
    pub fn add_metadata(&mut self, key: &str, value: serde_json::Value) {
        self.metadata.insert(key.to_string(), value);
    }

    /// 标记成功
    pub fn mark_success(&mut self) {
        self.status = EventStatus::Success;
        self.end();
    }

    /// 标记失败
    pub fn mark_failed(&mut self, error: String) {
        self.status = EventStatus::Failed;
        self.error = Some(error);
        self.end();
    }

    /// 标记取消
    pub fn mark_cancelled(&mut self) {
        self.status = EventStatus::Cancelled;
        self.end();
    }

    /// 结束事件
    fn end(&mut self) {
        let now = Utc::now();
        self.ended_at = Some(now);
        self.duration_ms = Some((now - self.started_at).num_milliseconds() as u64);
    }

    /// 检查是否为根事件
    pub fn is_root(&self) -> bool {
        self.parent_span_id.is_none()
    }
}

/// 追踪上下文
#[derive(Debug, Clone)]
pub struct TraceContext {
    /// 追踪ID
    pub trace_id: TraceId,

    /// 当前跨度ID
    pub current_span_id: Option<SpanId>,

    /// 来源
    pub source: String,
}

impl TraceContext {
    /// 创建新追踪上下文
    pub fn new(source: String) -> Self {
        Self { trace_id: uuid::Uuid::new_v4().to_string(), current_span_id: None, source }
    }

    /// 创建子事件
    pub fn create_event(&mut self, name: String, event_type: EventType) -> EventLog {
        let parent_span_id = self.current_span_id.clone();
        let mut event = EventLog::new(self.trace_id.clone(), name, event_type, self.source.clone());

        if let Some(parent) = parent_span_id {
            event = event.with_parent(parent);
        }

        self.current_span_id = Some(event.span_id.clone());
        event
    }

    /// 完成当前跨度
    pub fn finish_span(&mut self) {
        // 返回到父跨度（如果有）
        // 实际实现中需要维护跨度栈
    }
}

/// 事件追踪器
pub struct EventTracer {
    /// 事件存储
    events: Arc<RwLock<Vec<EventLog>>>,

    /// 追踪索引
    trace_index: Arc<RwLock<HashMap<TraceId, Vec<EventId>>>>,

    /// 配置
    #[allow(dead_code)]
    config: crate::config::Config,
}

impl EventTracer {
    /// 创建新的追踪器
    pub fn new() -> Self {
        Self::with_config(crate::config::Config::default())
    }

    /// 使用配置创建
    pub fn with_config(config: crate::config::Config) -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            trace_index: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// 开始新追踪
    pub async fn start_trace(&self, source: String) -> TraceContext {
        TraceContext::new(source)
    }

    /// 记录事件
    pub async fn record(&self, event: EventLog) -> Result<EventId> {
        let id = event.id.clone();
        let trace_id = event.trace_id.clone();

        let mut events = self.events.write().await;
        let mut trace_index = self.trace_index.write().await;

        events.push(event);
        trace_index.entry(trace_id).or_insert_with(Vec::new).push(id.clone());

        Ok(id)
    }

    /// 获取事件
    pub async fn get_event(&self, id: &EventId) -> Result<EventLog> {
        let events = self.events.read().await;
        events
            .iter()
            .find(|e| e.id == *id)
            .cloned()
            .ok_or_else(|| Error::Event(format!("事件 {} 不存在", id)))
    }

    /// 获取追踪的所有事件
    pub async fn get_trace_events(&self, trace_id: &TraceId) -> Result<Vec<EventLog>> {
        let trace_index = self.trace_index.read().await;
        let event_ids = trace_index
            .get(trace_id)
            .cloned()
            .ok_or_else(|| Error::Event(format!("追踪 {} 不存在", trace_id)))?;

        let events = self.events.read().await;
        let result: Vec<EventLog> = event_ids
            .iter()
            .filter_map(|id| events.iter().find(|e| e.id == *id).cloned())
            .collect();

        Ok(result)
    }

    /// 获取追踪树
    pub async fn get_trace_tree(&self, trace_id: &TraceId) -> Result<Vec<EventLog>> {
        let mut events = self.get_trace_events(trace_id).await?;

        // 按开始时间排序
        events.sort_by(|a, b| a.started_at.cmp(&b.started_at));

        Ok(events)
    }

    /// 统计事件数量
    pub async fn count(&self) -> usize {
        let events = self.events.read().await;
        events.len()
    }

    /// 统计追踪数量
    pub async fn trace_count(&self) -> usize {
        let trace_index = self.trace_index.read().await;
        trace_index.len()
    }

    /// 清空所有事件
    pub async fn clear(&self) -> Result<()> {
        let mut events = self.events.write().await;
        let mut trace_index = self.trace_index.write().await;
        events.clear();
        trace_index.clear();
        Ok(())
    }

    /// 创建聚合器
    pub fn create_aggregator(&self) -> crate::aggregator::LogAggregator {
        crate::aggregator::LogAggregator::new(self.events.clone())
    }
}

impl Default for EventTracer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_start_trace() {
        let tracer = EventTracer::new();
        let ctx = tracer.start_trace("test-service".to_string()).await;

        assert!(!ctx.trace_id.is_empty());
    }

    #[tokio::test]
    async fn test_create_event() {
        let tracer = EventTracer::new();
        let mut ctx = tracer.start_trace("test".to_string()).await;

        let event = ctx.create_event("test-event".to_string(), EventType::CommandExecution);
        assert!(!event.id.is_empty());
        assert_eq!(event.trace_id, ctx.trace_id);
    }

    #[tokio::test]
    async fn test_record_event() {
        let tracer = EventTracer::new();
        let mut ctx = tracer.start_trace("test".to_string()).await;

        let mut event = ctx.create_event("test".to_string(), EventType::CommandExecution);
        event.mark_success();

        let id = tracer.record(event).await.unwrap();
        assert!(!id.is_empty());

        let retrieved = tracer.get_event(&id).await.unwrap();
        assert_eq!(retrieved.status, EventStatus::Success);
    }

    #[tokio::test]
    async fn test_get_trace_events() {
        let tracer = EventTracer::new();
        let mut ctx = tracer.start_trace("test".to_string()).await;

        let mut event1 = ctx.create_event("e1".to_string(), EventType::CommandExecution);
        event1.mark_success();
        tracer.record(event1).await.unwrap();

        let mut event2 = ctx.create_event("e2".to_string(), EventType::CommandExecution);
        event2.mark_success();
        tracer.record(event2).await.unwrap();

        let events = tracer.get_trace_events(&ctx.trace_id).await.unwrap();
        assert_eq!(events.len(), 2);
    }

    #[tokio::test]
    async fn test_event_success() {
        let mut event = EventLog::new(
            "trace-1".to_string(),
            "test".to_string(),
            EventType::CommandExecution,
            "test".to_string(),
        );
        event.mark_success();

        assert_eq!(event.status, EventStatus::Success);
        assert!(event.ended_at.is_some());
        assert!(event.duration_ms.is_some());
    }

    #[tokio::test]
    async fn test_event_failure() {
        let mut event = EventLog::new(
            "trace-1".to_string(),
            "test".to_string(),
            EventType::CommandExecution,
            "test".to_string(),
        );
        event.mark_failed("测试错误".to_string());

        assert_eq!(event.status, EventStatus::Failed);
        assert!(event.error.is_some());
    }

    #[test]
    fn test_event_with_parent() {
        let event = EventLog::new(
            "trace-1".to_string(),
            "test".to_string(),
            EventType::CommandExecution,
            "test".to_string(),
        )
        .with_parent("parent-span".to_string());

        assert!(event.parent_span_id.is_some());
        assert!(!event.is_root());
    }

    #[test]
    fn test_event_add_tags() {
        let mut event = EventLog::new(
            "trace-1".to_string(),
            "test".to_string(),
            EventType::CommandExecution,
            "test".to_string(),
        );
        event.add_tag("key1", "value1");
        event.add_tag("key2", "value2");

        assert_eq!(event.tags.len(), 2);
        assert_eq!(event.tags.get("key1"), Some(&"value1".to_string()));
    }
}
