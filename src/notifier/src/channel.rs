//! # 通知渠道模块
//!
//! 定义通知渠道类型和基础实现

use crate::error::{Error, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 通知渠道类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    /// 邮件
    Email,
    /// 短信
    Sms,
    /// Webhook
    Webhook,
    /// 企业微信
    WechatWork,
    /// 钉钉
    DingTalk,
    /// Slack
    Slack,
    /// 自定义
    Custom(String),
}

impl std::fmt::Display for ChannelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelType::Email => write!(f, "email"),
            ChannelType::Sms => write!(f, "sms"),
            ChannelType::Webhook => write!(f, "webhook"),
            ChannelType::WechatWork => write!(f, "wechat_work"),
            ChannelType::DingTalk => write!(f, "ding_talk"),
            ChannelType::Slack => write!(f, "slack"),
            ChannelType::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}

/// 通知优先级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationPriority {
    /// 低优先级
    Low,
    /// 普通优先级
    Normal,
    /// 高优先级
    High,
    /// 紧急
    Urgent,
}

impl Default for NotificationPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// 通知状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationStatus {
    /// 待发送
    Pending,
    /// 发送中
    Sending,
    /// 已发送
    Sent,
    /// 已送达
    Delivered,
    /// 发送失败
    Failed,
    /// 已取消
    Cancelled,
}

/// 通知消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// 通知ID
    pub id: String,
    /// 渠道类型
    pub channel: ChannelType,
    /// 接收者
    pub recipients: Vec<String>,
    /// 标题
    pub title: String,
    /// 内容
    pub content: String,
    /// 优先级
    pub priority: NotificationPriority,
    /// 状态
    pub status: NotificationStatus,
    /// 模板ID（可选）
    pub template_id: Option<String>,
    /// 模板参数（可选）
    pub template_params: HashMap<String, String>,
    /// 元数据
    pub metadata: HashMap<String, String>,
    /// 重试次数
    pub retry_count: u32,
    /// 最大重试次数
    pub max_retries: u32,
    /// 错误信息
    pub error: Option<String>,
}

impl Notification {
    /// 创建新通知
    pub fn new(
        channel: ChannelType,
        recipients: Vec<String>,
        title: String,
        content: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            channel,
            recipients,
            title,
            content,
            priority: NotificationPriority::default(),
            status: NotificationStatus::Pending,
            template_id: None,
            template_params: HashMap::new(),
            metadata: HashMap::new(),
            retry_count: 0,
            max_retries: 3,
            error: None,
        }
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: NotificationPriority) -> Self {
        self.priority = priority;
        self
    }

    /// 设置模板
    pub fn with_template(mut self, template_id: String, params: HashMap<String, String>) -> Self {
        self.template_id = Some(template_id);
        self.template_params = params;
        self
    }

    /// 添加元数据
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }

    /// 标记发送中
    pub fn mark_sending(&mut self) {
        self.status = NotificationStatus::Sending;
    }

    /// 标记已发送
    pub fn mark_sent(&mut self) {
        self.status = NotificationStatus::Sent;
    }

    /// 标记已送达
    pub fn mark_delivered(&mut self) {
        self.status = NotificationStatus::Delivered;
    }

    /// 标记失败
    pub fn mark_failed(&mut self, error: String) {
        self.status = NotificationStatus::Failed;
        self.error = Some(error);
    }

    /// 标记取消
    pub fn mark_cancelled(&mut self) {
        self.status = NotificationStatus::Cancelled;
    }

    /// 是否可重试
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries && self.status == NotificationStatus::Failed
    }

    /// 增加重试计数
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
        if self.can_retry() {
            self.status = NotificationStatus::Pending;
            self.error = None;
        }
    }
}

/// 通知渠道 trait
#[async_trait]
pub trait NotificationChannel: Send + Sync {
    /// 获取渠道类型
    fn channel_type(&self) -> ChannelType;

    /// 发送通知
    async fn send(&self, notification: &Notification) -> Result<String>;

    /// 批量发送
    async fn send_batch(&self, notifications: &[Notification]) -> Result<Vec<Result<String>>> {
        let mut results = Vec::new();
        for notification in notifications {
            let result = self.send(notification).await;
            results.push(result);
        }
        Ok(results)
    }

    /// 验证接收者
    fn validate_recipient(&self, recipient: &str) -> Result<()>;

    /// 健康检查
    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}

/// 渠道配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    /// 渠道类型
    pub channel_type: ChannelType,
    /// 是否启用
    pub enabled: bool,
    /// 配置参数
    pub config: HashMap<String, String>,
}

impl ChannelConfig {
    /// 创建新配置
    pub fn new(channel_type: ChannelType) -> Self {
        Self { channel_type, enabled: true, config: HashMap::new() }
    }

    /// 设置参数
    pub fn with_config(mut self, key: &str, value: &str) -> Self {
        self.config.insert(key.to_string(), value.to_string());
        self
    }

    /// 禁用
    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// 邮件渠道实现
#[allow(dead_code)]
pub struct EmailChannel {
    /// SMTP服务器
    smtp_server: String,
    /// SMTP端口
    smtp_port: u16,
    /// 发件人
    from_address: String,
    /// 是否使用TLS
    use_tls: bool,
}

impl EmailChannel {
    /// 创建邮件渠道
    pub fn new(smtp_server: String, smtp_port: u16, from_address: String) -> Self {
        Self { smtp_server, smtp_port, from_address, use_tls: true }
    }

    /// 设置TLS
    pub fn with_tls(mut self, use_tls: bool) -> Self {
        self.use_tls = use_tls;
        self
    }
}

#[async_trait]
impl NotificationChannel for EmailChannel {
    fn channel_type(&self) -> ChannelType {
        ChannelType::Email
    }

    async fn send(&self, notification: &Notification) -> Result<String> {
        // 实际实现中会连接SMTP服务器发送邮件
        // 这里只做模拟
        Ok(format!("email-sent-{}", notification.id))
    }

    fn validate_recipient(&self, recipient: &str) -> Result<()> {
        if !recipient.contains('@') || !recipient.contains('.') {
            return Err(Error::Notify(format!("无效的邮箱地址: {}", recipient)));
        }
        Ok(())
    }

    async fn health_check(&self) -> Result<bool> {
        // 检查SMTP服务器连接
        Ok(!self.smtp_server.is_empty())
    }
}

/// Webhook渠道实现
pub struct WebhookChannel {
    /// Webhook URL
    url: String,
    /// 请求头
    headers: HashMap<String, String>,
    /// 超时（秒）
    timeout_secs: u64,
}

impl WebhookChannel {
    /// 创建Webhook渠道
    pub fn new(url: String) -> Self {
        Self { url, headers: HashMap::new(), timeout_secs: 30 }
    }

    /// 添加请求头
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    /// 设置超时
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }
}

#[async_trait]
impl NotificationChannel for WebhookChannel {
    fn channel_type(&self) -> ChannelType {
        ChannelType::Webhook
    }

    async fn send(&self, notification: &Notification) -> Result<String> {
        // 实际实现中会发送HTTP请求
        // 这里只做模拟
        if self.url.is_empty() {
            return Err(Error::Notify("Webhook URL不能为空".to_string()));
        }
        Ok(format!("webhook-sent-{}", notification.id))
    }

    fn validate_recipient(&self, recipient: &str) -> Result<()> {
        if !recipient.starts_with("http://") && !recipient.starts_with("https://") {
            return Err(Error::Notify(format!("无效的URL: {}", recipient)));
        }
        Ok(())
    }

    async fn health_check(&self) -> Result<bool> {
        // 检查URL有效性
        Ok(!self.url.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_creation() {
        let notification = Notification::new(
            ChannelType::Email,
            vec!["test@example.com".to_string()],
            "Test Subject".to_string(),
            "Test Content".to_string(),
        );

        assert!(!notification.id.is_empty());
        assert_eq!(notification.channel, ChannelType::Email);
        assert_eq!(notification.recipients.len(), 1);
        assert_eq!(notification.status, NotificationStatus::Pending);
    }

    #[test]
    fn test_notification_priority() {
        let notification = Notification::new(
            ChannelType::Email,
            vec!["test@example.com".to_string()],
            "Test".to_string(),
            "Content".to_string(),
        )
        .with_priority(NotificationPriority::Urgent);

        assert_eq!(notification.priority, NotificationPriority::Urgent);
    }

    #[test]
    fn test_notification_status_transitions() {
        let mut notification = Notification::new(
            ChannelType::Email,
            vec!["test@example.com".to_string()],
            "Test".to_string(),
            "Content".to_string(),
        );

        notification.mark_sending();
        assert_eq!(notification.status, NotificationStatus::Sending);

        notification.mark_sent();
        assert_eq!(notification.status, NotificationStatus::Sent);

        notification.mark_delivered();
        assert_eq!(notification.status, NotificationStatus::Delivered);
    }

    #[test]
    fn test_notification_retry() {
        let mut notification = Notification::new(
            ChannelType::Email,
            vec!["test@example.com".to_string()],
            "Test".to_string(),
            "Content".to_string(),
        );
        notification.max_retries = 3;

        notification.mark_failed("Error".to_string());
        assert_eq!(notification.status, NotificationStatus::Failed);
        assert!(notification.can_retry());

        notification.increment_retry();
        assert_eq!(notification.retry_count, 1);
        assert_eq!(notification.status, NotificationStatus::Pending);
    }

    #[test]
    fn test_email_channel() {
        let channel = EmailChannel::new(
            "smtp.example.com".to_string(),
            587,
            "noreply@example.com".to_string(),
        );

        assert_eq!(channel.channel_type(), ChannelType::Email);
        assert!(channel.validate_recipient("test@example.com").is_ok());
        assert!(channel.validate_recipient("invalid-email").is_err());
    }

    #[test]
    fn test_webhook_channel() {
        let channel = WebhookChannel::new("https://hooks.example.com/notify".to_string());

        assert_eq!(channel.channel_type(), ChannelType::Webhook);
        assert!(channel.validate_recipient("https://example.com").is_ok());
        assert!(channel.validate_recipient("invalid-url").is_err());
    }

    #[tokio::test]
    async fn test_email_channel_send() {
        let channel = EmailChannel::new(
            "smtp.example.com".to_string(),
            587,
            "noreply@example.com".to_string(),
        );

        let notification = Notification::new(
            ChannelType::Email,
            vec!["test@example.com".to_string()],
            "Test".to_string(),
            "Content".to_string(),
        );

        let result = channel.send(&notification).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_webhook_channel_send() {
        let channel = WebhookChannel::new("https://hooks.example.com/notify".to_string());

        let notification = Notification::new(
            ChannelType::Webhook,
            vec!["https://example.com/webhook".to_string()],
            "Test".to_string(),
            "Content".to_string(),
        );

        let result = channel.send(&notification).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_channel_type_display() {
        assert_eq!(format!("{}", ChannelType::Email), "email");
        assert_eq!(format!("{}", ChannelType::Sms), "sms");
        assert_eq!(format!("{}", ChannelType::WechatWork), "wechat_work");
        assert_eq!(
            format!("{}", ChannelType::Custom("custom_channel".to_string())),
            "custom:custom_channel"
        );
    }

    #[test]
    fn test_channel_config() {
        let config = ChannelConfig::new(ChannelType::Email)
            .with_config("smtp_server", "smtp.example.com")
            .with_config("smtp_port", "587");

        assert_eq!(config.channel_type, ChannelType::Email);
        assert!(config.enabled);
        assert_eq!(config.config.get("smtp_server"), Some(&"smtp.example.com".to_string()));
    }

    #[test]
    fn test_channel_config_disable() {
        let config = ChannelConfig::new(ChannelType::Sms).disable();

        assert!(!config.enabled);
    }

    #[test]
    fn test_notification_with_template() {
        let mut params = HashMap::new();
        params.insert("name".to_string(), "John".to_string());
        params.insert("code".to_string(), "123456".to_string());

        let notification = Notification::new(
            ChannelType::Sms,
            vec!["+8613800138000".to_string()],
            "验证码".to_string(),
            "".to_string(),
        )
        .with_template("sms_verification_code".to_string(), params.clone());

        assert_eq!(notification.template_id, Some("sms_verification_code".to_string()));
        assert_eq!(notification.template_params.get("name"), Some(&"John".to_string()));
        assert_eq!(notification.template_params.get("code"), Some(&"123456".to_string()));
    }

    #[test]
    fn test_notification_metadata() {
        let mut notification = Notification::new(
            ChannelType::Email,
            vec!["test@example.com".to_string()],
            "Test".to_string(),
            "Content".to_string(),
        );

        notification.add_metadata("trace_id", "trace-123");
        notification.add_metadata("user_id", "user-456");

        assert_eq!(notification.metadata.get("trace_id"), Some(&"trace-123".to_string()));
        assert_eq!(notification.metadata.get("user_id"), Some(&"user-456".to_string()));
    }

    #[test]
    fn test_notification_max_retries_exceeded() {
        let mut notification = Notification::new(
            ChannelType::Email,
            vec!["test@example.com".to_string()],
            "Test".to_string(),
            "Content".to_string(),
        );
        notification.max_retries = 2;

        // 第一次失败
        notification.mark_failed("Error 1".to_string());
        assert!(notification.can_retry());
        notification.increment_retry();
        assert_eq!(notification.retry_count, 1);

        // 第二次失败
        notification.mark_failed("Error 2".to_string());
        assert!(notification.can_retry());
        notification.increment_retry();
        assert_eq!(notification.retry_count, 2);

        // 第三次失败，达到最大重试次数
        notification.mark_failed("Error 3".to_string());
        assert!(!notification.can_retry()); // 不能再重试了
    }

    #[test]
    fn test_notification_cancel() {
        let mut notification = Notification::new(
            ChannelType::Email,
            vec!["test@example.com".to_string()],
            "Test".to_string(),
            "Content".to_string(),
        );

        notification.mark_cancelled();
        assert_eq!(notification.status, NotificationStatus::Cancelled);
        assert!(!notification.can_retry());
    }

    #[tokio::test]
    async fn test_batch_send() {
        let channel = EmailChannel::new(
            "smtp.example.com".to_string(),
            587,
            "noreply@example.com".to_string(),
        );

        let notifications = vec![
            Notification::new(
                ChannelType::Email,
                vec!["user1@example.com".to_string()],
                "Test 1".to_string(),
                "Content 1".to_string(),
            ),
            Notification::new(
                ChannelType::Email,
                vec!["user2@example.com".to_string()],
                "Test 2".to_string(),
                "Content 2".to_string(),
            ),
        ];

        let results = channel.send_batch(&notifications).await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
    }

    #[test]
    fn test_notification_priority_order() {
        let low = Notification::new(
            ChannelType::Email,
            vec!["test@example.com".to_string()],
            "Test".to_string(),
            "Content".to_string(),
        )
        .with_priority(NotificationPriority::Low);

        let urgent = Notification::new(
            ChannelType::Email,
            vec!["test@example.com".to_string()],
            "Test".to_string(),
            "Content".to_string(),
        )
        .with_priority(NotificationPriority::Urgent);

        assert_eq!(low.priority, NotificationPriority::Low);
        assert_eq!(urgent.priority, NotificationPriority::Urgent);
    }

    #[test]
    fn test_webhook_channel_headers() {
        let channel = WebhookChannel::new("https://hooks.example.com/notify".to_string())
            .with_header("Authorization", "Bearer token123")
            .with_header("X-Custom-Header", "custom-value")
            .with_timeout(60);

        assert_eq!(channel.headers.get("Authorization"), Some(&"Bearer token123".to_string()));
        assert_eq!(channel.headers.get("X-Custom-Header"), Some(&"custom-value".to_string()));
        assert_eq!(channel.timeout_secs, 60);
    }

    #[tokio::test]
    async fn test_webhook_health_check() {
        let channel = WebhookChannel::new("https://hooks.example.com/notify".to_string());
        let healthy = channel.health_check().await.unwrap();
        assert!(healthy);

        let empty_channel = WebhookChannel::new("".to_string());
        let healthy = empty_channel.health_check().await.unwrap();
        assert!(!healthy);
    }

    #[tokio::test]
    async fn test_email_health_check() {
        let channel = EmailChannel::new(
            "smtp.example.com".to_string(),
            587,
            "noreply@example.com".to_string(),
        );
        let healthy = channel.health_check().await.unwrap();
        assert!(healthy);

        let empty_channel =
            EmailChannel::new("".to_string(), 587, "noreply@example.com".to_string());
        let healthy = empty_channel.health_check().await.unwrap();
        assert!(!healthy);
    }
}
