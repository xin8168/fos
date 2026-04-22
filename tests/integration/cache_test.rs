//! # Cache 集成测试
//!
//! 测试缓存系统的完整功能，包括本地缓存、分布式缓存和序列化

use fos_cache::{
    CacheCodec, CacheConfig, DistributedCache, JsonCodec, LocalCache, LocalDistributedCache,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// 测试用的数据结构
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestData {
    id: String,
    name: String,
    value: i32,
    tags: Vec<String>,
}

#[tokio::test]
async fn test_local_cache_basic_operations() {
    let cache: LocalCache<String> = LocalCache::new();

    // 基础设置和获取
    cache.set("key1".to_string(), "value1".to_string(), None).await;
    let value = cache.get("key1").await;
    assert_eq!(value, Some("value1".to_string()));

    // 获取不存在的键
    let value = cache.get("nonexistent").await;
    assert!(value.is_none());

    // 删除操作
    let removed = cache.del("key1").await;
    assert!(removed);
    assert!(cache.get("key1").await.is_none());

    // 再次删除
    let removed = cache.del("key1").await;
    assert!(!removed);
}

#[tokio::test]
async fn test_local_cache_ttl() {
    let cache: LocalCache<String> = LocalCache::new();

    // 设置带TTL的缓存
    cache.set("key1".to_string(), "value1".to_string(), Some(1)).await;

    // 立即获取应该成功
    let value = cache.get("key1").await;
    assert_eq!(value, Some("value1".to_string()));

    // 等待过期
    sleep(Duration::from_secs(2)).await;

    // 获取应该失败
    let value = cache.get("key1").await;
    assert!(value.is_none());
}

#[tokio::test]
async fn test_local_cache_update() {
    let cache: LocalCache<String> = LocalCache::new();

    cache.set("key1".to_string(), "value1".to_string(), None).await;
    assert_eq!(cache.size().await, 1);

    // 更新已存在的键
    cache.set("key1".to_string(), "updated_value".to_string(), None).await;

    let value = cache.get("key1").await;
    assert_eq!(value, Some("updated_value".to_string()));
    assert_eq!(cache.size().await, 1);
}

#[tokio::test]
async fn test_local_cache_bulk_operations() {
    let cache: LocalCache<String> = LocalCache::new();

    // 批量添加
    for i in 0..10 {
        cache.set(format!("key{}", i), format!("value{}", i), None).await;
    }

    assert_eq!(cache.size().await, 10);

    // 批量获取
    for i in 0..10 {
        let value = cache.get(&format!("key{}", i)).await;
        assert_eq!(value, Some(format!("value{}", i)));
    }

    // 批量删除
    for i in 0..5 {
        cache.del(&format!("key{}", i)).await;
    }

    assert_eq!(cache.size().await, 5);
}

#[tokio::test]
async fn test_local_cache_cleanup_expired() {
    let cache: LocalCache<String> = LocalCache::new();

    // 添加一些带TTL的键
    cache.set("expiring1".to_string(), "value1".to_string(), Some(1)).await;
    cache.set("expiring2".to_string(), "value2".to_string(), Some(1)).await;
    cache.set("permanent".to_string(), "value3".to_string(), None).await;

    assert_eq!(cache.size().await, 3);

    // 等待过期
    sleep(Duration::from_secs(2)).await;

    // 清理过期条目
    let count = cache.cleanup_expired().await;
    assert_eq!(count, 2);
    assert_eq!(cache.size().await, 1);

    // 永久键应该还在
    assert_eq!(cache.get("permanent").await, Some("value3".to_string()));
}

#[tokio::test]
async fn test_local_cache_stats() {
    let cache: LocalCache<String> = LocalCache::new();

    cache.set("key1".to_string(), "value1".to_string(), None).await;
    cache.set("key2".to_string(), "value2".to_string(), None).await;

    // 命中
    cache.get("key1").await;
    cache.get("key2").await;
    cache.get("key1").await; // 重复命中

    // 未命中
    cache.get("key3").await;
    cache.get("key4").await;

    let stats = cache.stats().await;
    assert_eq!(stats.hits, 3);
    assert_eq!(stats.misses, 2);
    assert_eq!(stats.size, 2);
    assert_eq!(stats.hit_rate(), 0.6);
}

#[tokio::test]
async fn test_local_cache_access_count() {
    let cache: LocalCache<String> = LocalCache::new();

    cache.set("key1".to_string(), "value1".to_string(), None).await;

    for _ in 0..5 {
        cache.get("key1").await;
    }

    let entry = cache.get_entry("key1").await;
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().access_count, 5);
}

#[tokio::test]
async fn test_distributed_cache_basic_operations() {
    let cache = LocalDistributedCache::new();

    // 设置和获取
    cache.set("key1", b"value1".to_vec(), None).await.unwrap();
    let value = cache.get("key1").await.unwrap();
    assert_eq!(value, Some(b"value1".to_vec()));

    // 删除
    let removed = cache.del("key1").await.unwrap();
    assert!(removed);

    let value = cache.get("key1").await.unwrap();
    assert!(value.is_none());
}

#[tokio::test]
async fn test_distributed_cache_exists() {
    let cache = LocalDistributedCache::new();

    assert!(!cache.exists("key1").await.unwrap());

    cache.set("key1", b"value1".to_vec(), None).await.unwrap();

    assert!(cache.exists("key1").await.unwrap());
}

#[tokio::test]
async fn test_distributed_cache_ttl() {
    let cache = LocalDistributedCache::new();

    // 无TTL
    cache.set("key1", b"value1".to_vec(), None).await.unwrap();
    let ttl = cache.ttl("key1").await.unwrap();
    assert!(ttl.is_none());

    // 有TTL
    cache.set("key2", b"value2".to_vec(), Some(60)).await.unwrap();
    let ttl = cache.ttl("key2").await.unwrap();
    assert!(ttl.is_some());
    assert!(ttl.unwrap() >= 59);
}

#[tokio::test]
async fn test_distributed_cache_expire() {
    let cache = LocalDistributedCache::new();

    cache.set("key1", b"value1".to_vec(), None).await.unwrap();

    // 设置过期时间
    cache.expire("key1", 60).await.unwrap();

    let ttl = cache.ttl("key1").await.unwrap();
    assert!(ttl.is_some());

    // 对不存在的键expire应该失败
    let result = cache.expire("nonexistent", 60).await;
    assert!(result.is_err());
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
async fn test_distributed_cache_from_config() {
    let config = CacheConfig::default();
    let _cache = LocalDistributedCache::from_config(config);

    // 验证创建成功，测试通过即表示无panic
    assert!(true);
}

#[tokio::test]
async fn test_json_codec_basic() {
    let data = TestData {
        id: "test-id".to_string(),
        name: "test-name".to_string(),
        value: 42,
        tags: vec!["tag1".to_string(), "tag2".to_string()],
    };

    // 序列化
    let bytes = JsonCodec::serialize(&data).unwrap();
    assert!(!bytes.is_empty());

    // 反序列化
    let decoded: TestData = JsonCodec::deserialize(&bytes).unwrap();
    assert_eq!(decoded.id, data.id);
    assert_eq!(decoded.name, data.name);
    assert_eq!(decoded.value, data.value);
    assert_eq!(decoded.tags, data.tags);
}

#[tokio::test]
async fn test_json_codec_complex_types() {
    // 测试复杂类型的序列化
    let mut map = HashMap::new();
    map.insert("key1".to_string(), 100);
    map.insert("key2".to_string(), 200);

    let bytes = JsonCodec::serialize(&map).unwrap();
    let decoded: HashMap<String, i32> = JsonCodec::deserialize(&bytes).unwrap();

    assert_eq!(decoded.len(), 2);
    assert_eq!(decoded.get("key1"), Some(&100));
    assert_eq!(decoded.get("key2"), Some(&200));
}

#[tokio::test]
async fn test_concurrent_cache_access() {
    let cache = Arc::new(LocalCache::<String>::new());
    let counter = Arc::new(AtomicU32::new(0));

    // 创建多个任务并发访问缓存
    let mut handles = vec![];

    for i in 0..10 {
        let cache_clone = cache.clone();
        let counter_clone = counter.clone();

        let handle = tokio::spawn(async move {
            let key = format!("key{}", i);
            let value = format!("value{}", i);

            // 写入
            cache_clone.set(key.clone(), value.clone(), None).await;

            // 读取
            if let Some(retrieved) = cache_clone.get(&key).await {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                assert_eq!(retrieved, value);
            }
        });

        handles.push(handle);
    }

    // 等待所有任务完成
    for handle in handles {
        handle.await.unwrap();
    }

    // 验证所有写入和读取都成功
    assert_eq!(counter.load(Ordering::SeqCst), 10);
    assert_eq!(cache.size().await, 10);
}

#[tokio::test]
async fn test_cache_with_expiration_cleanup() {
    let cache: LocalCache<String> = LocalCache::new();

    // 添加多个带不同TTL的键
    cache.set("short1".to_string(), "val1".to_string(), Some(1)).await;
    cache.set("short2".to_string(), "val2".to_string(), Some(1)).await;
    cache.set("medium1".to_string(), "val3".to_string(), Some(3)).await;
    cache.set("permanent1".to_string(), "val4".to_string(), None).await;
    cache.set("permanent2".to_string(), "val5".to_string(), None).await;

    assert_eq!(cache.size().await, 5);

    // 等待短期键过期
    sleep(Duration::from_secs(2)).await;

    // 清理过期
    let count = cache.cleanup_expired().await;
    assert_eq!(count, 2);
    assert_eq!(cache.size().await, 3);

    // 验证永久键还在
    assert!(cache.exists("permanent1").await);
    assert!(cache.exists("permanent2").await);
    assert!(cache.exists("medium1").await);
}

#[tokio::test]
async fn test_cache_stats_reset() {
    let cache: LocalCache<String> = LocalCache::new();

    cache.set("key1".to_string(), "value1".to_string(), None).await;
    cache.get("key1").await;
    cache.get("nonexistent").await;

    let stats = cache.stats().await;
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);
    assert_eq!(stats.size, 1);

    // 重置统计
    cache.reset_stats().await;

    let stats = cache.stats().await;
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);
    assert_eq!(stats.size, 0);
}

#[tokio::test]
async fn test_cache_remaining_ttl() {
    let cache: LocalCache<String> = LocalCache::new();

    cache.set("key1".to_string(), "value1".to_string(), Some(60)).await;

    let entry = cache.get_entry("key1").await;
    assert!(entry.is_some());
    let entry = entry.unwrap();

    let remaining = entry.remaining_ttl();
    assert!(remaining.is_some());
    let remaining_seconds = remaining.unwrap();

    // 剩余时间应该在59-60秒之间
    assert!(remaining_seconds >= 59);
    assert!(remaining_seconds <= 60);
}

#[tokio::test]
async fn test_distributed_cache_integration() {
    // 测试分布式缓存与序列化的集成
    let cache = LocalDistributedCache::new();

    let data = TestData {
        id: "test-id".to_string(),
        name: "test-name".to_string(),
        value: 100,
        tags: vec!["tag1".to_string()],
    };

    // 序列化并存储
    let bytes = JsonCodec::serialize(&data).unwrap();
    cache.set("test-data", bytes, Some(60)).await.unwrap();

    // 获取并反序列化
    let retrieved_bytes = cache.get("test-data").await.unwrap();
    assert!(retrieved_bytes.is_some());

    let retrieved: TestData = JsonCodec::deserialize(&retrieved_bytes.unwrap()).unwrap();
    assert_eq!(retrieved.id, data.id);
    assert_eq!(retrieved.value, data.value);
}

#[tokio::test]
async fn test_cache_error_handling() {
    let cache: LocalCache<String> = LocalCache::new();

    // 对不存在的键执行各种操作
    assert!(cache.get("nonexistent").await.is_none());
    assert!(!cache.del("nonexistent").await);
    assert!(!cache.exists("nonexistent").await);
}

#[tokio::test]
async fn test_cache_stress_test() {
    let cache = Arc::new(LocalCache::<String>::new());

    // 创建100个并发任务
    let mut handles = vec![];

    for i in 0..100 {
        let cache_clone = cache.clone();

        let handle = tokio::spawn(async move {
            let key = format!("key{}", i % 10); // 只有10个不同的键
            let value = format!("value{}", i);

            // 频繁写入和读取
            cache_clone.set(key.clone(), value.clone(), None).await;
            let _ = cache_clone.get(&key).await;

            // 每隔10次删除一次
            if i % 10 == 9 {
                cache_clone.del(&key).await;
            }
        });

        handles.push(handle);
    }

    // 等待所有任务完成
    for handle in handles {
        handle.await.unwrap();
    }

    // 验证缓存状态一致
    let stats = cache.stats().await;
    // 应该有大量的命中和未命中
    assert!(stats.total_requests() > 0);
    assert!(cache.size().await <= 10);
}

#[tokio::test]
async fn test_distributed_cache_mixed_operations() {
    let cache = LocalDistributedCache::new();

    // 混合基本操作和批量操作
    cache.set("single1", b"val1".to_vec(), None).await.unwrap();
    cache.set("single2", b"val2".to_vec(), None).await.unwrap();

    let batch_items =
        vec![("batch1".to_string(), b"val3".to_vec()), ("batch2".to_string(), b"val4".to_vec())];

    cache.mset(batch_items, None).await.unwrap();

    // 单个获取
    assert_eq!(cache.get("single1").await.unwrap(), Some(b"val1".to_vec()));

    // 批量获取
    let keys = vec!["batch1".to_string(), "batch2".to_string()];
    let values = cache.mget(keys).await.unwrap();
    assert_eq!(values.len(), 2);

    // 验证总共的条目数
    let size = cache.size().await.unwrap();
    assert_eq!(size, Some(4));
}
