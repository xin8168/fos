//! # 日志聚合模块
//!
//! 负责日志的聚合、统计和分析

use crate::error::Result;
use crate::tracer::{EventLevel, EventLog, EventStatus, EventType, TraceId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 聚合查询参数
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AggregationQuery {
    /// 开始时间
    pub start_time: Option<DateTime<Utc>>,

    /// 结束时间
    pub end_time: Option<DateTime<Utc>>,

    /// 事件类型过滤
    pub event_type: Option<EventType>,

    /// 事件级别过滤
    pub level: Option<EventLevel>,

    /// 状态过滤
    pub status: Option<EventStatus>,

    /// 来源过滤
    pub source: Option<String>,

    /// 分组字段
    pub group_by: Option<String>,

    /// 分页偏移
    pub offset: Option<usize>,

    /// 分页限制
    pub limit: Option<usize>,
}

/// 事件统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStats {
    /// 总数
    pub total: usize,

    /// 成功数
    pub success_count: usize,

    /// 失败数
    pub failed_count: usize,

    /// 进行中数
    pub in_progress_count: usize,

    /// 取消数
    pub cancelled_count: usize,

    /// 平均持续时间（毫秒）
    pub avg_duration_ms: f64,

    /// 最大持续时间（毫秒）
    pub max_duration_ms: u64,

    /// 最小持续时间（毫秒）
    pub min_duration_ms: u64,
}

impl Default for EventStats {
    fn default() -> Self {
        Self {
            total: 0,
            success_count: 0,
            failed_count: 0,
            in_progress_count: 0,
            cancelled_count: 0,
            avg_duration_ms: 0.0,
            max_duration_ms: 0,
            min_duration_ms: 0,
        }
    }
}

impl EventStats {
    /// 从事件列表计算统计
    pub fn from_events(events: &[EventLog]) -> Self {
        if events.is_empty() {
            return Self::default();
        }

        let mut stats = Self::default();
        stats.total = events.len();

        let mut total_duration = 0u64;
        let mut durations: Vec<u64> = Vec::new();

        for event in events {
            match event.status {
                EventStatus::Success => stats.success_count += 1,
                EventStatus::Failed => stats.failed_count += 1,
                EventStatus::InProgress => stats.in_progress_count += 1,
                EventStatus::Cancelled => stats.cancelled_count += 1,
            }

            if let Some(duration) = event.duration_ms {
                total_duration += duration;
                durations.push(duration);
            }
        }

        if !durations.is_empty() {
            stats.avg_duration_ms = total_duration as f64 / durations.len() as f64;
            stats.max_duration_ms = *durations.iter().max().unwrap_or(&0);
            stats.min_duration_ms = *durations.iter().min().unwrap_or(&0);
        }

        stats
    }
}

/// 分组统计结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupedStats {
    /// 分组键
    pub key: String,

    /// 统计信息
    pub stats: EventStats,
}

/// 日志聚合器
pub struct LogAggregator {
    /// 事件存储
    events: Arc<RwLock<Vec<EventLog>>>,
}

impl LogAggregator {
    /// 创建新的聚合器
    pub fn new(events: Arc<RwLock<Vec<EventLog>>>) -> Self {
        Self { events }
    }

    /// 聚合查询
    pub async fn aggregate(&self, query: AggregationQuery) -> Result<Vec<EventLog>> {
        let events = self.events.read().await;

        let mut filtered: Vec<EventLog> = events
            .iter()
            .filter(|event| {
                // 时间范围过滤
                if let Some(start) = query.start_time {
                    if event.started_at < start {
                        return false;
                    }
                }

                if let Some(end) = query.end_time {
                    if event.started_at > end {
                        return false;
                    }
                }

                // 类型过滤
                if let Some(ref event_type) = query.event_type {
                    if event.event_type != *event_type {
                        return false;
                    }
                }

                // 级别过滤
                if let Some(ref level) = query.level {
                    if event.level != *level {
                        return false;
                    }
                }

                // 状态过滤
                if let Some(ref status) = query.status {
                    if event.status != *status {
                        return false;
                    }
                }

                // 来源过滤
                if let Some(ref source) = query.source {
                    if event.source != *source {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        // 按时间排序
        filtered.sort_by(|a, b| b.started_at.cmp(&a.started_at));

        // 分页
        if let Some(offset) = query.offset {
            filtered = filtered.into_iter().skip(offset).collect();
        }

        if let Some(limit) = query.limit {
            filtered = filtered.into_iter().take(limit).collect();
        }

        Ok(filtered)
    }

    /// 计算统计信息
    pub async fn compute_stats(&self, query: AggregationQuery) -> Result<EventStats> {
        let events = self.aggregate(query).await?;
        Ok(EventStats::from_events(&events))
    }

    /// 分组统计
    pub async fn group_stats(
        &self,
        query: AggregationQuery,
        group_by: &str,
    ) -> Result<Vec<GroupedStats>> {
        let events = self.aggregate(query).await?;

        let mut groups: HashMap<String, Vec<EventLog>> = HashMap::new();

        for event in events {
            let key = match group_by {
                "status" => format!("{:?}", event.status),
                "level" => format!("{:?}", event.level),
                "type" => format!("{:?}", event.event_type),
                "source" => event.source.clone(),
                _ => "unknown".to_string(),
            };

            groups.entry(key).or_insert_with(Vec::new).push(event);
        }

        let result: Vec<GroupedStats> = groups
            .into_iter()
            .map(|(key, events)| GroupedStats { key, stats: EventStats::from_events(&events) })
            .collect();

        Ok(result)
    }

    /// 按追踪ID聚合
    pub async fn aggregate_by_trace(&self, trace_id: &TraceId) -> Result<Vec<EventLog>> {
        let events = self.events.read().await;
        let filtered: Vec<EventLog> =
            events.iter().filter(|e| e.trace_id == *trace_id).cloned().collect();
        Ok(filtered)
    }

    /// 按来源聚合
    pub async fn aggregate_by_source(&self, source: &str) -> Result<Vec<EventLog>> {
        let events = self.events.read().await;
        let filtered: Vec<EventLog> =
            events.iter().filter(|e| e.source == source).cloned().collect();
        Ok(filtered)
    }

    /// 按时间范围聚合
    pub async fn aggregate_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<EventLog>> {
        let events = self.events.read().await;
        let filtered: Vec<EventLog> = events
            .iter()
            .filter(|e| e.started_at >= start && e.started_at <= end)
            .cloned()
            .collect();
        Ok(filtered)
    }

    /// 计算错误率
    pub async fn error_rate(&self, query: AggregationQuery) -> Result<f64> {
        let stats = self.compute_stats(query).await?;
        if stats.total == 0 {
            return Ok(0.0);
        }
        Ok(stats.failed_count as f64 / stats.total as f64 * 100.0)
    }

    /// 计算成功率
    pub async fn success_rate(&self, query: AggregationQuery) -> Result<f64> {
        let stats = self.compute_stats(query).await?;
        if stats.total == 0 {
            return Ok(0.0);
        }
        Ok(stats.success_count as f64 / stats.total as f64 * 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_event(status: EventStatus, duration_ms: Option<u64>) -> EventLog {
        let mut event = EventLog::new(
            "trace-1".to_string(),
            "test".to_string(),
            EventType::CommandExecution,
            "test-source".to_string(),
        );
        event.status = status;
        event.duration_ms = duration_ms;
        event
    }

    #[tokio::test]
    async fn test_aggregate() {
        let events = Arc::new(RwLock::new(vec![
            create_test_event(EventStatus::Success, Some(100)),
            create_test_event(EventStatus::Failed, Some(200)),
        ]));

        let aggregator = LogAggregator::new(events);
        let result = aggregator.aggregate(AggregationQuery::default()).await.unwrap();

        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_compute_stats() {
        let events = Arc::new(RwLock::new(vec![
            create_test_event(EventStatus::Success, Some(100)),
            create_test_event(EventStatus::Success, Some(200)),
            create_test_event(EventStatus::Failed, Some(300)),
        ]));

        let aggregator = LogAggregator::new(events);
        let stats = aggregator.compute_stats(AggregationQuery::default()).await.unwrap();

        assert_eq!(stats.total, 3);
        assert_eq!(stats.success_count, 2);
        assert_eq!(stats.failed_count, 1);
        assert_eq!(stats.avg_duration_ms, 200.0);
        assert_eq!(stats.max_duration_ms, 300);
        assert_eq!(stats.min_duration_ms, 100);
    }

    #[tokio::test]
    async fn test_group_stats() {
        let events = Arc::new(RwLock::new(vec![
            create_test_event(EventStatus::Success, Some(100)),
            create_test_event(EventStatus::Failed, Some(200)),
        ]));

        let aggregator = LogAggregator::new(events);
        let grouped = aggregator.group_stats(AggregationQuery::default(), "status").await.unwrap();

        assert_eq!(grouped.len(), 2);
    }

    #[tokio::test]
    async fn test_error_rate() {
        let events = Arc::new(RwLock::new(vec![
            create_test_event(EventStatus::Success, None),
            create_test_event(EventStatus::Success, None),
            create_test_event(EventStatus::Failed, None),
        ]));

        let aggregator = LogAggregator::new(events);
        let rate = aggregator.error_rate(AggregationQuery::default()).await.unwrap();

        assert!((rate - 33.333333333333336).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_success_rate() {
        let events = Arc::new(RwLock::new(vec![
            create_test_event(EventStatus::Success, None),
            create_test_event(EventStatus::Success, None),
            create_test_event(EventStatus::Failed, None),
        ]));

        let aggregator = LogAggregator::new(events);
        let rate = aggregator.success_rate(AggregationQuery::default()).await.unwrap();

        assert!((rate - 66.66666666666666).abs() < 0.01);
    }

    #[test]
    fn test_event_stats_from_events() {
        let events = vec![
            create_test_event(EventStatus::Success, Some(100)),
            create_test_event(EventStatus::Failed, Some(200)),
        ];

        let stats = EventStats::from_events(&events);

        assert_eq!(stats.total, 2);
        assert_eq!(stats.success_count, 1);
        assert_eq!(stats.failed_count, 1);
        assert_eq!(stats.avg_duration_ms, 150.0);
    }

    #[test]
    fn test_event_stats_empty() {
        let events: Vec<EventLog> = vec![];
        let stats = EventStats::from_events(&events);

        assert_eq!(stats.total, 0);
        assert_eq!(stats.avg_duration_ms, 0.0);
    }
}
