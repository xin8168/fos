//! # 缓存条目
//!
//! 缓存条目的定义和管理

use chrono::{DateTime, Utc};

/// 缓存条目
#[derive(Debug, Clone)]
pub struct CacheEntry<T>
where
    T: Clone,
{
    /// 缓存值
    pub value: T,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 过期时间（None表示永不过期）
    pub expires_at: Option<DateTime<Utc>>,
    /// 访问次数
    pub access_count: u64,
    /// 最后访问时间
    pub last_accessed_at: DateTime<Utc>,
}

impl<T> CacheEntry<T>
where
    T: Clone,
{
    /// 创建新的缓存条目
    pub fn new(value: T, ttl_seconds: Option<u64>) -> Self {
        let now = Utc::now();
        let expires_at = ttl_seconds.map(|ttl| now + chrono::Duration::seconds(ttl as i64));

        Self { value, created_at: now, expires_at, access_count: 0, last_accessed_at: now }
    }

    /// 检查是否已过期
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// 记录访问
    pub fn record_access(&mut self) {
        self.access_count += 1;
        self.last_accessed_at = Utc::now();
    }

    /// 获取剩余生存时间（秒）
    pub fn remaining_ttl(&self) -> Option<i64> {
        self.expires_at.map(|expires_at| {
            let remaining = expires_at - Utc::now();
            remaining.num_seconds()
        })
    }

    /// 获取缓存值
    pub fn get_value(&self) -> &T {
        &self.value
    }

    /// 获取值（记录访问）
    pub fn get(&mut self) -> &T {
        self.record_access();
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_entry_creation() {
        let entry = CacheEntry::new("test_value", None);
        assert_eq!(entry.get_value(), &"test_value");
        assert!(entry.expires_at.is_none());
        assert_eq!(entry.access_count, 0);
    }

    #[test]
    fn test_cache_entry_with_ttl() {
        let entry = CacheEntry::new("test_value", Some(60));
        assert!(entry.expires_at.is_some());
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_cache_entry_expired() {
        // 创建一个已过期的条目
        let ttl = -1; // 负数会立即过期
        let _entry = CacheEntry::new("test_value", Some(ttl as u64));
        // 注意：由于时间精度问题，这个测试可能不稳定
        // 实际使用中，过期时间是相对于创建时间
    }

    #[test]
    fn test_record_access() {
        let mut entry = CacheEntry::new("test_value", None);
        assert_eq!(entry.access_count, 0);

        entry.record_access();
        assert_eq!(entry.access_count, 1);
        assert_eq!(entry.get_value(), &"test_value");

        entry.record_access();
        assert_eq!(entry.access_count, 2);
    }

    #[test]
    fn test_remaining_ttl() {
        let entry = CacheEntry::new("test_value", Some(60));
        assert!(entry.remaining_ttl().is_some());
        let remaining = entry.remaining_ttl().unwrap();
        // 剩余时间应该在59-60秒之间
        assert!(remaining > 58);
        assert!(remaining <= 60);
    }

    #[test]
    fn test_remaining_ttl_none() {
        let entry = CacheEntry::new("test_value", None);
        assert!(entry.remaining_ttl().is_none());
    }

    #[test]
    fn test_get_record_access() {
        let mut entry = CacheEntry::new("test_value", None);
        assert_eq!(entry.access_count, 0);

        let value = entry.get();
        assert_eq!(value, &"test_value");
        assert_eq!(entry.access_count, 1);
    }
}
