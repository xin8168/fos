//! FOS 核心模块集成测试
//!
//! 测试各核心模块之间的跨模块协作

use fos_audit::{AuditLogType, AuditLogger};
use fos_eventlog::{EventTracer, EventType};
use fos_notifier::{
    ChannelType, EmailChannel, Notification, NotificationChannel, NotificationPriority,
};

/// 测试 EventLog + Audit 集成
/// 验证事件追踪与审计日志的协作
#[tokio::test]
async fn test_eventlog_audit_integration() {
    // 创建事件追踪器
    let tracer = EventTracer::new();
    let mut ctx = tracer.start_trace("audit-service".to_string()).await;

    // 创建审计日志器
    let audit_logger = AuditLogger::new();

    // 记录事件
    let mut event = ctx.create_event("user-action".to_string(), EventType::CommandExecution);
    event.add_tag("action", "login");
    event.mark_success();
    let event_id = tracer.record(event).await.unwrap();

    // 记录审计日志（格式拦截场景）
    let audit_id = audit_logger
        .log_format_blocked("login attempt".to_string(), "invalid format".to_string())
        .await
        .unwrap();

    // 验证事件和审计日志
    assert!(!event_id.is_empty());
    assert!(!audit_id.is_empty());

    // 获取追踪事件
    let events = tracer.get_trace_events(&ctx.trace_id).await.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].tags.get("action"), Some(&"login".to_string()));

    // 验证审计日志数量
    let count = audit_logger.count().await;
    assert_eq!(count, 1);
}

/// 测试 Notifier + EventLog 集成
/// 验证通知发送事件追踪
#[tokio::test]
async fn test_notifier_eventlog_integration() {
    let tracer = EventTracer::new();
    let aggregator = tracer.create_aggregator();

    // 创建通知渠道
    let email_channel =
        EmailChannel::new("smtp.example.com".to_string(), 587, "noreply@example.com".to_string());

    // 开始追踪
    let mut ctx = tracer.start_trace("notification-service".to_string()).await;

    // 创建并发送通知
    let notification = Notification::new(
        ChannelType::Email,
        vec!["user@example.com".to_string()],
        "系统通知".to_string(),
        "您的操作已完成".to_string(),
    )
    .with_priority(NotificationPriority::High);

    // 记录通知创建事件
    let mut event1 =
        ctx.create_event("notification-created".to_string(), EventType::CommandExecution);
    event1.mark_success();
    tracer.record(event1).await.unwrap();

    // 发送通知
    let _result = email_channel.send(&notification).await;

    // 记录发送事件
    let mut event2 = ctx.create_event("notification-sent".to_string(), EventType::CommandExecution);
    event2.mark_success();
    tracer.record(event2).await.unwrap();

    // 验证事件链
    let events = tracer.get_trace_events(&ctx.trace_id).await.unwrap();
    assert_eq!(events.len(), 2);

    // 验证事件统计
    let stats = aggregator.compute_stats(fos_eventlog::AggregationQuery::default()).await.unwrap();
    assert_eq!(stats.total, 2);
    assert_eq!(stats.success_count, 2);
}

/// 测试 Audit + Notifier 集成
/// 验证审计事件通知
#[tokio::test]
async fn test_audit_notifier_integration() {
    let audit_logger = AuditLogger::new();
    let email_channel =
        EmailChannel::new("smtp.example.com".to_string(), 587, "security@example.com".to_string());

    // 创建安全审计日志（执行失败）
    let audit_id = audit_logger
        .log_execution_failed("sensitive_operation".to_string(), "permission denied".to_string())
        .await
        .unwrap();

    // 执行失败触发通知
    let notification = Notification::new(
        ChannelType::Email,
        vec!["security-team@example.com".to_string()],
        "安全告警".to_string(),
        "检测到执行失败操作".to_string(),
    )
    .with_priority(NotificationPriority::Urgent);

    // 发送通知
    let result = email_channel.send(&notification).await;
    assert!(result.is_ok());

    // 验证审计日志已记录
    let audit_log = audit_logger.get(&audit_id).await.unwrap();
    assert_eq!(audit_log.log_type, AuditLogType::ExecutionFailed);

    // 验证统计
    let stats = audit_logger.stats().await;
    assert_eq!(stats.execution_failed, 1);
}

/// 测试 EventLog + Audit + Notifier 完整流程
#[tokio::test]
async fn test_eventlog_audit_notifier_flow() {
    // 初始化各模块
    let tracer = EventTracer::new();
    let audit_logger = AuditLogger::new();
    let email_channel =
        EmailChannel::new("smtp.example.com".to_string(), 587, "noreply@example.com".to_string());

    // 开始追踪
    let mut ctx = tracer.start_trace("full-flow".to_string()).await;

    // 1. 记录操作开始事件
    let mut start_event =
        ctx.create_event("operation-start".to_string(), EventType::CommandExecution);
    start_event.add_tag("operation", "sensitive-action");
    tracer.record(start_event).await.unwrap();

    // 2. 记录审计日志（格式拦截）
    let audit_id = audit_logger
        .log_rule_blocked("sensitive-action".to_string(), "违反安全规则".to_string())
        .await
        .unwrap();

    // 3. 记录拦截事件
    let mut blocked_event =
        ctx.create_event("operation-blocked".to_string(), EventType::CommandExecution);
    blocked_event.mark_failed("违反安全规则".to_string());
    tracer.record(blocked_event).await.unwrap();

    // 4. 发送通知
    let notification = Notification::new(
        ChannelType::Email,
        vec!["admin@example.com".to_string()],
        "安全拦截告警".to_string(),
        "检测到违反安全规则的操作".to_string(),
    )
    .with_priority(NotificationPriority::Urgent);
    email_channel.send(&notification).await.unwrap();

    // 验证完整流程
    let events = tracer.get_trace_events(&ctx.trace_id).await.unwrap();
    assert_eq!(events.len(), 2);

    let audit_log = audit_logger.get(&audit_id).await.unwrap();
    assert_eq!(audit_log.log_type, AuditLogType::RuleBlocked);

    let stats = audit_logger.stats().await;
    assert_eq!(stats.rule_blocked, 1);
}

/// 测试多模块状态一致性
#[tokio::test]
async fn test_multi_module_state_consistency() {
    // 初始化模块
    let tracer = EventTracer::new();
    let audit_logger = AuditLogger::new();

    // 创建操作ID作为唯一标识
    let operation_id = uuid::Uuid::new_v4().to_string();

    // 开始追踪
    let mut ctx = tracer.start_trace("consistency-check".to_string()).await;

    // 在所有模块中使用相同的操作ID
    let mut event = ctx.create_event("multi-module-op".to_string(), EventType::CommandExecution);
    event.add_metadata("operation_id", serde_json::json!(operation_id));
    event.mark_success();
    tracer.record(event).await.unwrap();

    // 创建关联的审计日志
    let audit_id = audit_logger
        .log_system_error("multi-module-operation".to_string(), format!("op_id: {}", operation_id))
        .await
        .unwrap();

    // 验证数据一致性
    let events = tracer.get_trace_events(&ctx.trace_id).await.unwrap();
    let audit_count = audit_logger.count().await;

    // 所有模块应能通过operation_id关联
    assert!(!events.is_empty());
    assert!(audit_count >= 1);
    assert!(!audit_id.is_empty());
}

/// 测试通知批量发送与事件追踪
#[tokio::test]
async fn test_batch_notification_with_eventlog() {
    let tracer = EventTracer::new();
    let email_channel =
        EmailChannel::new("smtp.example.com".to_string(), 587, "noreply@example.com".to_string());

    // 开始追踪
    let mut ctx = tracer.start_trace("batch-notification".to_string()).await;

    // 创建多个通知
    let notifications = vec![
        Notification::new(
            ChannelType::Email,
            vec!["user1@example.com".to_string()],
            "通知1".to_string(),
            "内容1".to_string(),
        ),
        Notification::new(
            ChannelType::Email,
            vec!["user2@example.com".to_string()],
            "通知2".to_string(),
            "内容2".to_string(),
        ),
        Notification::new(
            ChannelType::Email,
            vec!["user3@example.com".to_string()],
            "通知3".to_string(),
            "内容3".to_string(),
        ),
    ];

    // 记录批量发送事件
    let mut batch_event =
        ctx.create_event("batch-send-start".to_string(), EventType::CommandExecution);
    batch_event.add_tag("count", "3");
    batch_event.mark_success();
    tracer.record(batch_event).await.unwrap();

    // 批量发送
    let results = email_channel.send_batch(&notifications).await.unwrap();

    // 记录完成事件
    let mut complete_event =
        ctx.create_event("batch-send-complete".to_string(), EventType::CommandExecution);
    complete_event.mark_success();
    tracer.record(complete_event).await.unwrap();

    // 验证
    assert_eq!(results.len(), 3);
    for result in results {
        assert!(result.is_ok());
    }

    let events = tracer.get_trace_events(&ctx.trace_id).await.unwrap();
    assert_eq!(events.len(), 2);
}

/// 测试审计日志状态更新与通知
#[tokio::test]
async fn test_audit_status_update_with_notification() {
    let audit_logger = AuditLogger::new();
    let email_channel =
        EmailChannel::new("smtp.example.com".to_string(), 587, "noreply@example.com".to_string());

    // 创建审计日志
    let audit_id = audit_logger
        .log_format_blocked("test-command".to_string(), "格式错误".to_string())
        .await
        .unwrap();

    // 标记为已分析
    audit_logger.mark_analyzed(&audit_id).await.unwrap();

    // 发送分析完成通知
    let notification = Notification::new(
        ChannelType::Email,
        vec!["analyst@example.com".to_string()],
        "审计分析完成".to_string(),
        format!("审计日志 {} 已分析完成", audit_id),
    );
    email_channel.send(&notification).await.unwrap();

    // 验证状态
    let audit_log = audit_logger.get(&audit_id).await.unwrap();
    assert_eq!(audit_log.status, fos_audit::AuditLogStatus::Analyzed);

    // 验证统计
    let stats = audit_logger.stats().await;
    assert_eq!(stats.analyzed, 1);
}
