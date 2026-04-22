//! FOS Notifier 集成测试
//!
//! 测试 Notifier 模块各组件之间的集成

use fos_notifier::{
    ChannelConfig, ChannelType, EmailChannel, Notification, NotificationChannel,
    NotificationPriority, NotificationStatus, WebhookChannel,
};

/// 测试邮件渠道完整流程
#[tokio::test]
async fn test_email_channel_full_flow() {
    let channel =
        EmailChannel::new("smtp.example.com".to_string(), 587, "noreply@example.com".to_string());

    // 创建通知
    let mut notification = Notification::new(
        ChannelType::Email,
        vec!["user@example.com".to_string()],
        "测试邮件".to_string(),
        "这是一封测试邮件".to_string(),
    );
    notification.add_metadata("trace_id", "test-trace-001");

    // 验证接收者
    assert!(channel.validate_recipient("user@example.com").is_ok());
    assert!(channel.validate_recipient("invalid-email").is_err());

    // 健康检查
    let healthy = channel.health_check().await.unwrap();
    assert!(healthy);

    // 发送通知
    channel.validate_recipient(&notification.recipients[0]).unwrap();
    notification.mark_sending();
    let result = channel.send(&notification).await.unwrap();
    assert!(result.starts_with("email-sent-"));

    notification.mark_sent();
    assert_eq!(notification.status, NotificationStatus::Sent);
}

/// 测试Webhook渠道完整流程
#[tokio::test]
async fn test_webhook_channel_full_flow() {
    let channel = WebhookChannel::new("https://hooks.example.com/notify".to_string())
        .with_header("Authorization", "Bearer test-token")
        .with_timeout(60);

    // 创建通知
    let notification = Notification::new(
        ChannelType::Webhook,
        vec!["https://example.com/webhook".to_string()],
        "Webhook通知".to_string(),
        "{\"event\": \"test\"}".to_string(),
    )
    .with_priority(NotificationPriority::High);

    // 验证接收者
    assert!(channel.validate_recipient("https://example.com").is_ok());
    assert!(channel.validate_recipient("invalid-url").is_err());

    // 健康检查
    let healthy = channel.health_check().await.unwrap();
    assert!(healthy);

    // 发送通知
    let result = channel.send(&notification).await.unwrap();
    assert!(result.starts_with("webhook-sent-"));
}

/// 测试通知重试机制
#[tokio::test]
async fn test_notification_retry_mechanism() {
    let channel =
        EmailChannel::new("smtp.example.com".to_string(), 587, "noreply@example.com".to_string());

    let mut notification = Notification::new(
        ChannelType::Email,
        vec!["user@example.com".to_string()],
        "测试重试".to_string(),
        "重试测试内容".to_string(),
    );
    notification.max_retries = 3;

    // 模拟第一次失败
    notification.mark_failed("SMTP连接超时".to_string());
    assert_eq!(notification.status, NotificationStatus::Failed);
    assert!(notification.can_retry());

    // 重试
    notification.increment_retry();
    assert_eq!(notification.retry_count, 1);
    assert_eq!(notification.status, NotificationStatus::Pending);

    // 成功发送
    let _ = channel.send(&notification).await;
    notification.mark_sent();
    assert_eq!(notification.status, NotificationStatus::Sent);
}

/// 测试批量发送
#[tokio::test]
async fn test_batch_notification() {
    let channel =
        EmailChannel::new("smtp.example.com".to_string(), 587, "noreply@example.com".to_string());

    let notifications = vec![
        Notification::new(
            ChannelType::Email,
            vec!["user1@example.com".to_string()],
            "批量测试1".to_string(),
            "内容1".to_string(),
        ),
        Notification::new(
            ChannelType::Email,
            vec!["user2@example.com".to_string()],
            "批量测试2".to_string(),
            "内容2".to_string(),
        ),
        Notification::new(
            ChannelType::Email,
            vec!["user3@example.com".to_string()],
            "批量测试3".to_string(),
            "内容3".to_string(),
        ),
    ];

    let results = channel.send_batch(&notifications).await.unwrap();
    assert_eq!(results.len(), 3);
    for result in results {
        assert!(result.is_ok());
    }
}

/// 测试通知优先级处理
#[tokio::test]
async fn test_priority_handling() {
    let urgent_notification = Notification::new(
        ChannelType::Email,
        vec!["admin@example.com".to_string()],
        "紧急通知".to_string(),
        "系统告警".to_string(),
    )
    .with_priority(NotificationPriority::Urgent);

    let normal_notification = Notification::new(
        ChannelType::Email,
        vec!["user@example.com".to_string()],
        "普通通知".to_string(),
        "日常信息".to_string(),
    )
    .with_priority(NotificationPriority::Normal);

    assert_eq!(urgent_notification.priority, NotificationPriority::Urgent);
    assert_eq!(normal_notification.priority, NotificationPriority::Normal);
}

/// 测试渠道配置
#[test]
fn test_channel_configuration() {
    // Email配置
    let email_config = ChannelConfig::new(ChannelType::Email)
        .with_config("smtp_server", "smtp.example.com")
        .with_config("smtp_port", "587")
        .with_config("username", "noreply@example.com");

    assert!(email_config.enabled);
    assert_eq!(email_config.config.get("smtp_server"), Some(&"smtp.example.com".to_string()));

    // Webhook配置
    let webhook_config = ChannelConfig::new(ChannelType::Webhook)
        .with_config("url", "https://hooks.example.com/notify")
        .with_config("timeout", "30");

    assert!(webhook_config.enabled);

    // 禁用配置
    let disabled_config = ChannelConfig::new(ChannelType::Sms).disable();
    assert!(!disabled_config.enabled);
}

/// 测试通知状态流转
#[test]
fn test_notification_status_flow() {
    let mut notification = Notification::new(
        ChannelType::Email,
        vec!["test@example.com".to_string()],
        "状态测试".to_string(),
        "内容".to_string(),
    );

    // 初始状态
    assert_eq!(notification.status, NotificationStatus::Pending);

    // 发送中
    notification.mark_sending();
    assert_eq!(notification.status, NotificationStatus::Sending);

    // 已发送
    notification.mark_sent();
    assert_eq!(notification.status, NotificationStatus::Sent);

    // 已送达
    notification.mark_delivered();
    assert_eq!(notification.status, NotificationStatus::Delivered);
}

/// 测试多渠道类型
#[test]
fn test_multiple_channel_types() {
    let channels = vec![
        ChannelType::Email,
        ChannelType::Sms,
        ChannelType::Webhook,
        ChannelType::WechatWork,
        ChannelType::DingTalk,
        ChannelType::Slack,
        ChannelType::Custom("custom_channel".to_string()),
    ];

    assert_eq!(channels.len(), 7);

    // 验证Display trait
    assert_eq!(format!("{}", ChannelType::Email), "email");
    assert_eq!(format!("{}", ChannelType::WechatWork), "wechat_work");
    assert_eq!(format!("{}", ChannelType::Custom("my_channel".to_string())), "custom:my_channel");
}
