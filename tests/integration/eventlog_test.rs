//! FOS EventLog 集成测试
//!
//! 测试 EventLog 模块各组件之间的集成

use fos_eventlog::{AggregationQuery, EventTracer, EventType};

/// 测试追踪器与聚合器集成
#[tokio::test]
async fn test_tracer_aggregator_integration() {
    let tracer = EventTracer::new();
    let aggregator = tracer.create_aggregator();

    // 创建追踪和事件
    let mut ctx = tracer.start_trace("test-service".to_string()).await;

    let mut event1 = ctx.create_event("event1".to_string(), EventType::CommandExecution);
    event1.mark_success();
    tracer.record(event1).await.unwrap();

    let mut event2 = ctx.create_event("event2".to_string(), EventType::DataOperation);
    event2.mark_failed("测试错误".to_string());
    tracer.record(event2).await.unwrap();

    // 聚合查询
    let events = aggregator.aggregate(AggregationQuery::default()).await.unwrap();
    assert_eq!(events.len(), 2);
}

/// 测试完整链路追踪流程
#[tokio::test]
async fn test_full_trace_flow() {
    let tracer = EventTracer::new();

    // 开始追踪
    let mut ctx = tracer.start_trace("api-gateway".to_string()).await;
    let trace_id = ctx.trace_id.clone();

    // 创建多个事件
    let mut event1 = ctx.create_event("request-received".to_string(), EventType::CommandExecution);
    event1.add_tag("http_method", "POST");
    event1.add_tag("path", "/api/execute");
    event1.mark_success();
    tracer.record(event1).await.unwrap();

    let mut event2 = ctx.create_event("permission-check".to_string(), EventType::PermissionCheck);
    event2.add_tag("role", "admin");
    event2.mark_success();
    tracer.record(event2).await.unwrap();

    let mut event3 = ctx.create_event("execute-command".to_string(), EventType::CommandExecution);
    event3.add_tag("command", "cleanup");
    event3.mark_success();
    tracer.record(event3).await.unwrap();

    // 获取追踪树
    let tree = tracer.get_trace_tree(&trace_id).await.unwrap();
    assert_eq!(tree.len(), 3);

    // 验证事件关系
    assert!(tree[0].is_root());
    assert!(tree[1].parent_span_id.is_some());
    assert!(tree[2].parent_span_id.is_some());
}

/// 测试统计聚合
#[tokio::test]
async fn test_stats_aggregation() {
    let tracer = EventTracer::new();
    let aggregator = tracer.create_aggregator();

    // 创建多个事件
    let mut ctx = tracer.start_trace("test".to_string()).await;

    for i in 0..10 {
        let mut event = ctx.create_event(format!("event-{}", i), EventType::CommandExecution);
        if i < 7 {
            event.mark_success();
        } else {
            event.mark_failed(format!("error-{}", i));
        }
        tracer.record(event).await.unwrap();
    }

    // 计算统计
    let stats = aggregator.compute_stats(AggregationQuery::default()).await.unwrap();

    assert_eq!(stats.total, 10);
    assert_eq!(stats.success_count, 7);
    assert_eq!(stats.failed_count, 3);
}

/// 测试分组统计
#[tokio::test]
async fn test_grouped_stats() {
    let tracer = EventTracer::new();
    let aggregator = tracer.create_aggregator();

    // 创建不同状态的事件
    let mut ctx = tracer.start_trace("test".to_string()).await;

    let mut event1 = ctx.create_event("e1".to_string(), EventType::CommandExecution);
    event1.mark_success();
    tracer.record(event1).await.unwrap();

    let mut event2 = ctx.create_event("e2".to_string(), EventType::CommandExecution);
    event2.mark_failed("error".to_string());
    tracer.record(event2).await.unwrap();

    let mut event3 = ctx.create_event("e3".to_string(), EventType::CommandExecution);
    event3.mark_cancelled();
    tracer.record(event3).await.unwrap();

    // 分组统计
    let grouped = aggregator.group_stats(AggregationQuery::default(), "status").await.unwrap();

    assert_eq!(grouped.len(), 3);
}

/// 测试时间范围查询
#[tokio::test]
async fn test_time_range_query() {
    let tracer = EventTracer::new();
    let aggregator = tracer.create_aggregator();

    // 创建事件
    let mut ctx = tracer.start_trace("test".to_string()).await;

    for i in 0..5 {
        let mut event = ctx.create_event(format!("event-{}", i), EventType::CommandExecution);
        event.mark_success();
        tracer.record(event).await.unwrap();
    }

    // 查询全部
    let all = aggregator.aggregate(AggregationQuery::default()).await.unwrap();
    assert_eq!(all.len(), 5);
}

/// 测试事件类型过滤
#[tokio::test]
async fn test_event_type_filter() {
    let tracer = EventTracer::new();
    let aggregator = tracer.create_aggregator();

    let mut ctx = tracer.start_trace("test".to_string()).await;

    // 创建不同类型事件
    let mut e1 = ctx.create_event("cmd".to_string(), EventType::CommandExecution);
    e1.mark_success();
    tracer.record(e1).await.unwrap();

    let mut e2 = ctx.create_event("perm".to_string(), EventType::PermissionCheck);
    e2.mark_success();
    tracer.record(e2).await.unwrap();

    let mut e3 = ctx.create_event("data".to_string(), EventType::DataOperation);
    e3.mark_success();
    tracer.record(e3).await.unwrap();

    // 按类型过滤
    let query =
        AggregationQuery { event_type: Some(EventType::CommandExecution), ..Default::default() };
    let filtered = aggregator.aggregate(query).await.unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "cmd");
}

/// 测试错误率和成功率计算
#[tokio::test]
async fn test_rate_calculation() {
    let tracer = EventTracer::new();
    let aggregator = tracer.create_aggregator();

    let mut ctx = tracer.start_trace("test".to_string()).await;

    // 8成功，2失败
    for i in 0..10 {
        let mut event = ctx.create_event(format!("event-{}", i), EventType::CommandExecution);
        if i < 8 {
            event.mark_success();
        } else {
            event.mark_failed("error".to_string());
        }
        tracer.record(event).await.unwrap();
    }

    let error_rate = aggregator.error_rate(AggregationQuery::default()).await.unwrap();
    let success_rate = aggregator.success_rate(AggregationQuery::default()).await.unwrap();

    assert!((error_rate - 20.0).abs() < 0.01);
    assert!((success_rate - 80.0).abs() < 0.01);
}

/// 测试追踪上下文创建事件
#[tokio::test]
async fn test_trace_context_event_creation() {
    let tracer = EventTracer::new();
    let mut ctx = tracer.start_trace("service".to_string()).await;

    // 创建第一个事件（根事件）
    let event1 = ctx.create_event("root".to_string(), EventType::CommandExecution);
    assert!(event1.is_root());

    // 创建第二个事件（子事件）
    let event2 = ctx.create_event("child".to_string(), EventType::CommandExecution);
    assert!(!event2.is_root());
    assert!(event2.parent_span_id.is_some());
}
