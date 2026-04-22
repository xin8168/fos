//! 结果缓存

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 结果缓存
pub struct ResultCache {
    /// 缓存存储
    cache: std::sync::RwLock<HashMap<String, CachedResult>>,
    /// 最大缓存条目数
    max_entries: usize,
}

/// 缓存的结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedResult {
    /// 幂等键
    pub key: String,
    /// 结果数据
    pub data: serde_json::Value,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 过期时间
    pub expires_at: DateTime<Utc>,
    /// 访问次数
    pub access_count: u64,
}

impl CachedResult {
    /// 创建新的缓存结果
    pub fn new(key: &str, data: serde_json::Value, ttl_secs: u64) -> Self {
        let now = Utc::now();
        Self {
            key: key.to_string(),
            data,
            created_at: now,
            expires_at: now + chrono::Duration::seconds(ttl_secs as i64),
            access_count: 0,
        }
    }

    /// 检查是否已过期
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// 增加访问计数
    pub fn increment_access(&mut self) {
        self.access_count += 1;
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

impl ResultCache {
    /// 创建新的结果缓存
    pub fn new() -> Self {
        Self { cache: std::sync::RwLock::new(HashMap::new()), max_entries: 10000 }
    }

    /// 设置最大条目数
    pub fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    /// 存储结果
    pub fn store(&self, key: &str, data: serde_json::Value, ttl_secs: u64) -> bool {
        let mut cache = self.cache.write().unwrap();

        // 如果超过最大条目数，清理过期条目
        if cache.len() >= self.max_entries {
            cache.retain(|_, v| !v.is_expired());
        }

        // 如果仍然超过，移除最旧的
        if cache.len() >= self.max_entries {
            if let Some(oldest_key) =
                cache.iter().min_by_key(|(_, v)| v.created_at).map(|(k, _)| k.clone())
            {
                cache.remove(&oldest_key);
            }
        }

        cache.insert(key.to_string(), CachedResult::new(key, data, ttl_secs));
        true
    }

    /// 获取结果
    pub fn get(&self, key: &str) -> Option<CachedResult> {
        let mut cache = self.cache.write().unwrap();

        if let Some(result) = cache.get_mut(key) {
            if result.is_expired() {
                cache.remove(key);
                return None;
            }
            result.increment_access();
            return Some(result.clone());
        }
        None
    }

    /// 检查是否存在
    pub fn contains(&self, key: &str) -> bool {
        let cache = self.cache.read().unwrap();
        cache.get(key).map_or(false, |r| !r.is_expired())
    }

    /// 移除结果
    pub fn remove(&self, key: &str) -> Option<CachedResult> {
        let mut cache = self.cache.write().unwrap();
        cache.remove(key)
    }

    /// 清理过期条目
    pub fn cleanup_expired(&self) -> usize {
        let mut cache = self.cache.write().unwrap();
        let initial_len = cache.len();
        cache.retain(|_, v| !v.is_expired());
        initial_len - cache.len()
    }

    /// 清空缓存
    pub fn clear(&self) {
        self.cache.write().unwrap().clear();
    }

    /// 获取缓存条目数
    pub fn len(&self) -> usize {
        self.cache.read().unwrap().len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> CacheStats {
        let cache = self.cache.read().unwrap();
        let total_access: u64 = cache.values().map(|v| v.access_count).sum();

        CacheStats { total_entries: cache.len(), total_access, max_entries: self.max_entries }
    }
}

impl Default for ResultCache {
    fn default() -> Self {
        Self::new()
    }
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_access: u64,
    pub max_entries: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let cache = ResultCache::new();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_store_and_get() {
        let cache = ResultCache::new();

        cache.store("key1", serde_json::json!({"result": "ok"}), 60);

        let result = cache.get("key1");
        assert!(result.is_some());
        assert_eq!(result.unwrap().data["result"], "ok");
    }

    #[test]
    fn test_expiry() {
        let cache = ResultCache::new();

        cache.store("key1", serde_json::json!({}), 0);

        // 立即过期
        let result = cache.get("key1");
        assert!(result.is_none());
    }

    #[test]
    fn test_remove() {
        let cache = ResultCache::new();

        cache.store("key1", serde_json::json!({}), 60);
        cache.remove("key1");

        assert!(!cache.contains("key1"));
    }

    #[test]
    fn test_cleanup() {
        let cache = ResultCache::new();

        cache.store("key1", serde_json::json!({}), 0);
        cache.store("key2", serde_json::json!({}), 60);

        let removed = cache.cleanup_expired();
        assert_eq!(removed, 1);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_access_count() {
        let cache = ResultCache::new();

        cache.store("key1", serde_json::json!({}), 60);

        cache.get("key1");
        cache.get("key1");
        cache.get("key1");

        let result = cache.get("key1").unwrap();
        assert_eq!(result.access_count, 4); // 3 gets + 1 for final get
    }

    #[test]
    fn test_max_entries() {
        let cache = ResultCache::new().with_max_entries(2);

        cache.store("key1", serde_json::json!({}), 60);
        cache.store("key2", serde_json::json!({}), 60);
        cache.store("key3", serde_json::json!({}), 60);

        // 应该清理过期或最旧的
        assert!(cache.len() <= 2);
    }

    #[test]
    fn test_get_stats() {
        let cache = ResultCache::new();

        cache.store("key1", serde_json::json!({}), 60);
        cache.store("key2", serde_json::json!({}), 60);

        let stats = cache.get_stats();
        assert_eq!(stats.total_entries, 2);
    }
}
