//! 幂等键定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 幂等键
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdempotencyKey {
    /// 键值
    pub key: String,
    /// 资源标识
    pub resource: String,
    /// 操作类型
    pub operation: String,
    /// 状态
    pub status: KeyStatus,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 过期时间
    pub expires_at: DateTime<Utc>,
    /// 请求哈希（用于验证）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_hash: Option<String>,
}

impl IdempotencyKey {
    /// 创建新幂等键
    pub fn new(key: &str, resource: &str, operation: &str, ttl_secs: u64) -> Self {
        let now = Utc::now();
        Self {
            key: key.to_string(),
            resource: resource.to_string(),
            operation: operation.to_string(),
            status: KeyStatus::Pending,
            created_at: now,
            expires_at: now + chrono::Duration::seconds(ttl_secs as i64),
            request_hash: None,
        }
    }

    /// 设置请求哈希
    pub fn with_hash(mut self, hash: &str) -> Self {
        self.request_hash = Some(hash.to_string());
        self
    }

    /// 标记为处理中
    pub fn mark_processing(&mut self) {
        self.status = KeyStatus::Processing;
    }

    /// 标记为已完成
    pub fn mark_completed(&mut self) {
        self.status = KeyStatus::Completed;
    }

    /// 标记为失败
    pub fn mark_failed(&mut self) {
        self.status = KeyStatus::Failed;
    }

    /// 检查是否已过期
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// 检查是否可以重试
    pub fn can_retry(&self) -> bool {
        matches!(self.status, KeyStatus::Failed) && !self.is_expired()
    }

    /// 检查是否已完成
    pub fn is_completed(&self) -> bool {
        matches!(self.status, KeyStatus::Completed)
    }

    /// 检查是否正在处理
    pub fn is_processing(&self) -> bool {
        matches!(self.status, KeyStatus::Processing)
    }

    /// 验证请求哈希
    pub fn verify_hash(&self, hash: &str) -> bool {
        self.request_hash.as_ref().map_or(false, |h| h == hash)
    }

    /// 获取剩余有效期（秒）
    pub fn remaining_ttl(&self) -> i64 {
        let remaining = (self.expires_at - Utc::now()).num_seconds();
        if remaining > 0 {
            remaining
        } else {
            0
        }
    }
}

/// 幂等键状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyStatus {
    /// 待处理
    Pending,
    /// 处理中
    Processing,
    /// 已完成
    Completed,
    /// 已失败
    Failed,
}

impl std::fmt::Display for KeyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyStatus::Pending => write!(f, "待处理"),
            KeyStatus::Processing => write!(f, "处理中"),
            KeyStatus::Completed => write!(f, "已完成"),
            KeyStatus::Failed => write!(f, "已失败"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_creation() {
        let key = IdempotencyKey::new("test-key", "order", "create", 3600);
        assert_eq!(key.key, "test-key");
        assert_eq!(key.resource, "order");
        assert_eq!(key.status, KeyStatus::Pending);
    }

    #[test]
    fn test_key_status_transitions() {
        let mut key = IdempotencyKey::new("test", "res", "op", 60);

        key.mark_processing();
        assert!(key.is_processing());

        key.mark_completed();
        assert!(key.is_completed());
    }

    #[test]
    fn test_key_expiry() {
        let key = IdempotencyKey::new("test", "res", "op", 0);
        // 立即过期
        assert!(key.is_expired());
    }

    #[test]
    fn test_key_hash() {
        let key = IdempotencyKey::new("test", "res", "op", 60).with_hash("abc123");

        assert!(key.verify_hash("abc123"));
        assert!(!key.verify_hash("wrong"));
    }

    #[test]
    fn test_can_retry() {
        let mut key = IdempotencyKey::new("test", "res", "op", 60);
        key.mark_failed();

        assert!(key.can_retry());
    }

    #[test]
    fn test_remaining_ttl() {
        let key = IdempotencyKey::new("test", "res", "op", 60);
        assert!(key.remaining_ttl() > 0);
        assert!(key.remaining_ttl() <= 60);
    }
}
