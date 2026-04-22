//! # 分布式缓存实现
//!
//! 基于本地缓存的分布式缓存实现

use crate::cache::LocalCache;
use crate::distributed::{CacheConfig, DistributedCache};
use crate::error::Result;
use tracing::debug;

/// 本地分布式缓存
///
/// 基于LocalCache实现的分布式缓存接口
pub struct LocalDistributedCache {
    inner: LocalCache<Vec<u8>>,
}

impl LocalDistributedCache {
    /// 创建新的本地分布式缓存
    pub fn new() -> Self {
        Self { inner: LocalCache::new() }
    }

    /// 从配置创建
    pub fn from_config(_config: CacheConfig) -> Self {
        // 本地缓存实现忽略config中的连接字符串等参数
        Self::new()
    }
}

impl Default for LocalDistributedCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl DistributedCache for LocalDistributedCache {
    async fn set(&self, key: &str, value: Vec<u8>, ttl_seconds: Option<u64>) -> Result<()> {
        debug!("Setting cache key: {}", key);
        self.inner.set(key.to_string(), value, ttl_seconds).await;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        debug!("Getting cache key: {}", key);
        Ok(self.inner.get(key).await)
    }

    async fn del(&self, key: &str) -> Result<bool> {
        debug!("Deleting cache key: {}", key);
        Ok(self.inner.del(key).await)
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        debug!("Checking cache key exists: {}", key);
        Ok(self.inner.exists(key).await)
    }

    async fn expire(&self, key: &str, ttl_seconds: u64) -> Result<()> {
        debug!("Setting expire for key: {}, ttl: {}s", key, ttl_seconds);
        // 本地缓存实现需要重新设置来模拟expire
        if let Some(value) = self.inner.get(key).await {
            self.inner.set(key.to_string(), value, Some(ttl_seconds)).await;
        } else {
            return Err(crate::error::Error::Cache(format!("Key not found: {}", key)));
        }
        Ok(())
    }

    async fn ttl(&self, key: &str) -> Result<Option<u64>> {
        if let Some(entry) = self.inner.get_entry(key).await {
            Ok(entry.remaining_ttl().map(|ttl| if ttl > 0 { ttl as u64 } else { 0 }))
        } else {
            Ok(None)
        }
    }

    async fn clear(&self) -> Result<()> {
        debug!("Clearing all cache");
        self.inner.clear().await;
        Ok(())
    }

    async fn size(&self) -> Result<Option<usize>> {
        Ok(Some(self.inner.size().await))
    }

    async fn mset(&self, items: Vec<(String, Vec<u8>)>, ttl_seconds: Option<u64>) -> Result<()> {
        debug!("Setting {} cache items", items.len());
        for (key, value) in items {
            self.inner.set(key, value, ttl_seconds).await;
        }
        Ok(())
    }

    async fn mget(&self, keys: Vec<String>) -> Result<Vec<Option<Vec<u8>>>> {
        debug!("Getting {} cache items", keys.len());
        let mut results = Vec::new();
        for key in keys {
            results.push(self.inner.get(&key).await);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_distributed_cache_set_and_get() {
        let cache = LocalDistributedCache::new();

        cache.set("key1", b"value1".to_vec(), None).await.unwrap();

        let value = cache.get("key1").await.unwrap();
        assert_eq!(value, Some(b"value1".to_vec()));
    }

    #[tokio::test]
    async fn test_distributed_cache_get_nonexistent() {
        let cache = LocalDistributedCache::new();

        let value = cache.get("nonexistent").await.unwrap();
        assert!(value.is_none());
    }

    #[tokio::test]
    async fn test_distributed_cache_del() {
        let cache = LocalDistributedCache::new();

        cache.set("key1", b"value1".to_vec(), None).await.unwrap();

        let deleted = cache.del("key1").await.unwrap();
        assert!(deleted);

        let value = cache.get("key1").await.unwrap();
        assert!(value.is_none());

        let deleted = cache.del("key1").await.unwrap();
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_distributed_cache_exists() {
        let cache = LocalDistributedCache::new();

        assert!(!cache.exists("key1").await.unwrap());

        cache.set("key1", b"value1".to_vec(), None).await.unwrap();

        assert!(cache.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_distributed_cache_clear() {
        let cache = LocalDistributedCache::new();

        cache.set("key1", b"value1".to_vec(), None).await.unwrap();
        cache.set("key2", b"value2".to_vec(), None).await.unwrap();

        let size = cache.size().await.unwrap();
        assert_eq!(size, Some(2));

        cache.clear().await.unwrap();

        let size = cache.size().await.unwrap();
        assert_eq!(size, Some(0));
    }

    #[tokio::test]
    async fn test_distributed_cache_mset_mget() {
        let cache = LocalDistributedCache::new();

        let items = vec![
            ("key1".to_string(), b"value1".to_vec()),
            ("key2".to_string(), b"value2".to_vec()),
            ("key3".to_string(), b"value3".to_vec()),
        ];

        cache.mset(items.clone(), None).await.unwrap();

        let keys: Vec<String> = items.into_iter().map(|(k, _)| k).collect();
        let values = cache.mget(keys).await.unwrap();

        assert_eq!(values.len(), 3);
        assert_eq!(values[0], Some(b"value1".to_vec()));
        assert_eq!(values[1], Some(b"value2".to_vec()));
        assert_eq!(values[2], Some(b"value3".to_vec()));
    }

    #[tokio::test]
    async fn test_distributed_cache_expire_on_nonexistent() {
        let cache = LocalDistributedCache::new();

        let result = cache.expire("key1", 60).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_distributed_cache_ttl() {
        let cache = LocalDistributedCache::new();

        // 无TTL的键
        cache.set("key1", b"value1".to_vec(), None).await.unwrap();
        let ttl = cache.ttl("key1").await.unwrap();
        assert!(ttl.is_none());

        // 有TTL的键
        cache.set("key2", b"value2".to_vec(), Some(60)).await.unwrap();
        let ttl = cache.ttl("key2").await.unwrap();
        assert!(ttl.is_some());
        // TTL应该在59-60秒之间
        let ttl_value = ttl.unwrap();
        assert!(ttl_value >= 59);
        assert!(ttl_value <= 60);
    }

    #[tokio::test]
    async fn test_distributed_cache_from_config() {
        let config = CacheConfig::default();
        let _cache = LocalDistributedCache::from_config(config);
        // 验证创建成功
    }
}
