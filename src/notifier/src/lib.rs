//! # FOS Notifier - 通知服务模块
//!
//! ## 核心职责
//! - 多渠道通知发送
//! - 通知模板管理
//! - 通知状态追踪
//!
//! ## 安全铁律
//! - 不做规则判断

pub mod channel;
pub mod config;
pub mod error;

pub use channel::{
    ChannelConfig, ChannelType, EmailChannel, Notification, NotificationChannel,
    NotificationPriority, NotificationStatus, WebhookChannel,
};
pub use config::Config;
pub use error::{Error, Result};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
