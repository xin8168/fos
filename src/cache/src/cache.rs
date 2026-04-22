//! # 本地缓存
//!
//! 线程安全的本地缓存实现

use crate::entry::CacheEntry;
use crate::stats::CacheStats;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock as AsyncRwLock;
use tracing::{debug, info};

/// 缓存键类型
pub type CacheKey = String;

/// 本地缓存
pub struct LocalCache<T>
where
    T: Clone + Send + Sync,
{
    /// 缓存条目
    entries: Arc<AsyncRwLock<HashMap<CacheKey, CacheEntry<T>>>>,
    /// 统计信息
    stats: Arc<AsyncRwLock<CacheStats>>,
}

impl<T> Default for LocalCache<T>
where
    T: Clone + Send + Sync,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> LocalCache<T>
where
    T: Clone + Send + Sync,
{
    /// 创建新的本地缓存
    pub fn new() -> Self {
        Self {
            entries: Arc::new(AsyncRwLock::new(HashMap::new())),
            stats: Arc::new(AsyncRwLock::new(CacheStats::new())),
        }
    }

    /// 设置缓存值
    ///
    /// # 参数
    /// - `key`: 缓存键
    /// - `value`: 缓存值
    /// - `ttl_seconds`: 生存时间（秒），None表示永不过期
    pub async fn set(&self, key: CacheKey, value: T, ttl_seconds: Option<u64>) {
        let entry = CacheEntry::new(value, ttl_seconds);
        let mut entries = self.entries.write().await;

        let is_new = !entries.contains_key(&key);
        entries.insert(key.clone(), entry);

        // 更新大小统计
        {
            let mut stats = self.stats.write().await;
            stats.set_size(entries.len());
            if is_new {
                debug!("New cache entry added: {}", key);
            } else {
                debug!("Cache entry updated: {}", key);
            }
        }
    }

    /// 获取缓存值
    ///
    /// # 返回
    /// - `Some(T)`: 如果缓存命中且未过期
    /// - `None`: 如果缓存未命中或已过期
    pub async fn get(&self, key: &str) -> Option<T> {
        let mut entries = self.entries.write().await;

        if let Some(entry) = entries.get_mut(key) {
            // 检查是否过期
            if entry.is_expired() {
                debug!("Cache entry expired: {}", key);
                entries.remove(key);

                let mut stats = self.stats.write().await;
                stats.record_miss();
                stats.set_size(entries.len());

                return None;
            }

            // 记录访问并返回值
            let value = entry.get().clone();

            let mut stats = self.stats.write().await;
            stats.record_hit();
            stats.set_size(entries.len());

            debug!("Cache hit: {}", key);
            Some(value)
        } else {
            debug!("Cache miss: {}", key);

            let mut stats = self.stats.write().await;
            stats.record_miss();

            None
        }
    }

    /// 删除缓存条目
    ///
    /// # 返回
    /// - `true`: 如果条目存在并被删除
    /// - `false`: 如果条目不存在
    pub async fn del(&self, key: &str) -> bool {
        let mut entries = self.entries.write().await;
        let removed = entries.remove(key).is_some();

        if removed {
            info!("Cache entry deleted: {}", key);

            let mut stats = self.stats.write().await;
            stats.set_size(entries.len());
        }

        removed
    }

    /// 检查缓存条目是否存在且未过期
    ///
    /// # 返回
    /// - `true`: 如果条目存在且未过期
    /// - `false`: 如果条目不存在或已过期
    pub async fn exists(&self, key: &str) -> bool {
        let entries = self.entries.read().await;

        if let Some(entry) = entries.get(key) {
            !entry.is_expired()
        } else {
            false
        }
    }

    /// 清空所有缓存条目
    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        let count = entries.len();
        entries.clear();

        let mut stats = self.stats.write().await;
        stats.set_size(0);

        info!("Cleared {} cache entries", count);
    }

    /// 清理所有过期的缓存条目
    ///
    /// # 返回
    /// 清理的条目数量
    pub async fn cleanup_expired(&self) -> usize {
        let mut entries = self.entries.write().await;

        let mut expired_keys: Vec<String> = Vec::new();
        for (key, entry) in entries.iter() {
            if entry.is_expired() {
                expired_keys.push(key.clone());
            }
        }

        let count = expired_keys.len();
        for key in expired_keys {
            entries.remove(&key);
        }

        if count > 0 {
            info!("Cleaned up {} expired cache entries", count);
        }

        let mut stats = self.stats.write().await;
        for _ in 0..count {
            stats.record_eviction();
        }
        stats.set_size(entries.len());

        count
    }

    /// 获取缓存大小
    pub async fn size(&self) -> usize {
        let entries = self.entries.read().await;
        entries.len()
    }

    /// 检查缓存是否为空
    pub async fn is_empty(&self) -> bool {
        self.size().await == 0
    }

    /// 获取所有缓存键
    pub async fn keys(&self) -> Vec<CacheKey> {
        let entries = self.entries.read().await;
        entries.keys().cloned().collect()
    }

    /// 获取缓存统计信息
    pub async fn stats(&self) -> CacheStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// 重置统计信息
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        stats.reset();
        info!("Cache stats reset");
    }

    /// 获取缓存条目的元数据
    ///
    /// # 返回
    /// - `Some(CacheEntry)`: 如果条目存在
    /// - `None`: 如果条目不存在
    pub async fn get_entry(&self, key: &str) -> Option<CacheEntry<T>> {
        let entries = self.entries.read().await;
        entries.get(key).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_cache_creation() {
        let cache: LocalCache<String> = LocalCache::new();
        assert!(cache.is_empty().await);
        assert_eq!(cache.size().await, 0);
    }

    #[tokio::test]
    async fn test_set_and_get() {
        let cache: LocalCache<String> = LocalCache::new();

        cache.set("key1".to_string(), "value1".to_string(), None).await;
        cache.set("key2".to_string(), "value2".to_string(), None).await;

        assert_eq!(cache.size().await, 2);

        let value1 = cache.get("key1").await;
        assert_eq!(value1, Some("value1".to_string()));

        let value2 = cache.get("key2").await;
        assert_eq!(value2, Some("value2".to_string()));
    }

    #[tokio::test]
    async fn test_get_nonexistent() {
        let cache: LocalCache<String> = LocalCache::new();

        let value = cache.get("nonexistent").await;
        assert!(value.is_none());
    }

    #[tokio::test]
    async fn test_delete() {
        let cache: LocalCache<String> = LocalCache::new();

        cache.set("key1".to_string(), "value1".to_string(), None).await;
        assert_eq!(cache.size().await, 1);

        let removed = cache.del("key1").await;
        assert!(removed);
        assert!(cache.is_empty().await);

        let removed = cache.del("key1").await;
        assert!(!removed);
    }

    #[tokio::test]
    async fn test_exists() {
        let cache: LocalCache<String> = LocalCache::new();

        assert!(!cache.exists("key1").await);

        cache.set("key1".to_string(), "value1".to_string(), None).await;
        assert!(cache.exists("key1").await);
    }

    #[tokio::test]
    async fn test_clear() {
        let cache: LocalCache<String> = LocalCache::new();

        cache.set("key1".to_string(), "value1".to_string(), None).await;
        cache.set("key2".to_string(), "value2".to_string(), None).await;
        cache.set("key3".to_string(), "value3".to_string(), None).await;

        assert_eq!(cache.size().await, 3);

        cache.clear().await;
        assert!(cache.is_empty().await);
    }

    #[tokio::test]
    async fn test_keys() {
        let cache: LocalCache<String> = LocalCache::new();

        cache.set("key1".to_string(), "value1".to_string(), None).await;
        cache.set("key2".to_string(), "value2".to_string(), None).await;
        cache.set("key3".to_string(), "value3".to_string(), None).await;

        let keys = cache.keys().await;
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
        assert!(keys.contains(&"key3".to_string()));
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        let cache: LocalCache<String> = LocalCache::new();

        // 设置一个1秒后过期的缓存
        cache.set("key1".to_string(), "value1".to_string(), Some(1)).await;

        // 立即获取应该成功
        let value = cache.get("key1").await;
        assert_eq!(value, Some("value1".to_string()));

        // 等待2秒后获取应该失败
        sleep(Duration::from_secs(2)).await;
        let value = cache.get("key1").await;
        assert!(value.is_none());
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let cache: LocalCache<String> = LocalCache::new();

        cache.set("key1".to_string(), "value1".to_string(), Some(1)).await;
        cache.set("key2".to_string(), "value2".to_string(), Some(1)).await;
        cache.set("key3".to_string(), "value3".to_string(), None).await;

        assert_eq!(cache.size().await, 3);

        // 等待2秒
        sleep(Duration::from_secs(2)).await;

        // 清理过期条目
        let count = cache.cleanup_expired().await;
        assert_eq!(count, 2);
        assert_eq!(cache.size().await, 1);

        // key3应该还存在
        let value = cache.get("key3").await;
        assert_eq!(value, Some("value3".to_string()));
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache: LocalCache<String> = LocalCache::new();

        // 初始统计
        let stats = cache.stats().await;
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.size, 0);

        // 添加缓存
        cache.set("key1".to_string(), "value1".to_string(), None).await;

        // 命中
        cache.get("key1").await;
        cache.get("key1").await;

        // 未命中
        cache.get("key2").await;
        cache.get("key3").await;

        let stats = cache.stats().await;
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 2);
        assert_eq!(stats.size, 1);
        assert_eq!(stats.hit_rate(), 0.5);
    }

    #[tokio::test]
    async fn test_reset_stats() {
        let cache: LocalCache<String> = LocalCache::new();

        cache.set("key1".to_string(), "value1".to_string(), None).await;
        cache.get("key1").await;

        let stats = cache.stats().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.size, 1);

        cache.reset_stats().await;

        let stats = cache.stats().await;
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.evictions, 0);
        // reset会将size也重置为0
        assert_eq!(stats.size, 0);
    }

    #[tokio::test]
    async fn test_update_existing() {
        let cache: LocalCache<String> = LocalCache::new();

        cache.set("key1".to_string(), "value1".to_string(), None).await;
        assert_eq!(cache.size().await, 1);

        // 更新已存在的键
        cache.set("key1".to_string(), "updated_value".to_string(), None).await;

        assert_eq!(cache.size().await, 1);
        let value = cache.get("key1").await;
        assert_eq!(value, Some("updated_value".to_string()));
    }

    #[tokio::test]
    async fn test_get_entry() {
        let cache: LocalCache<String> = LocalCache::new();

        cache.set("key1".to_string(), "value1".to_string(), Some(60)).await;

        let entry = cache.get_entry("key1").await;
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.get_value(), &"value1".to_string());
        assert_eq!(entry.access_count, 0);
    }

    #[tokio::test]
    async fn test_access_count() {
        let cache: LocalCache<String> = LocalCache::new();

        cache.set("key1".to_string(), "value1".to_string(), None).await;

        cache.get("key1").await;
        cache.get("key1").await;
        cache.get("key1").await;

        let entry = cache.get_entry("key1").await;
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().access_count, 3);
    }
}
