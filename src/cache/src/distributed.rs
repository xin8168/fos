//! # 分布式缓存接口
//!
//! 分布式缓存的抽象trait定义

use crate::error::Result;
use async_trait::async_trait;
use std::time::Duration;

/// 分布式缓存接口
#[async_trait]
pub trait DistributedCache: Send + Sync {
    /// 设置缓存值
    ///
    /// # 参数
    /// - `key`: 缓存键
    /// - `value`: 缓存值（字节数组）
    /// - `ttl_seconds`: 生存时间（秒），None表示永不过期
    async fn set(&self, key: &str, value: Vec<u8>, ttl_seconds: Option<u64>) -> Result<()>;

    /// 获取缓存值
    ///
    /// # 参数
    /// - `key`: 缓存键
    ///
    /// # 返回
    /// - `Some(Vec<u8>)`: 如果缓存命中
    /// - `None`: 如果缓存未命中
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// 删除缓存条目
    ///
    /// # 参数
    /// - `key`: 缓存键
    ///
    /// # 返回
    /// - `true`: 如果条目存在并被删除
    /// - `false`: 如果条目不存在
    async fn del(&self, key: &str) -> Result<bool>;

    /// 检查缓存条目是否存在
    ///
    /// # 参数
    /// - `key`: 缓存键
    ///
    /// # 返回
    /// - `true`: 如果条目存在
    /// - `false`: 如果条目不存在
    async fn exists(&self, key: &str) -> Result<bool>;

    /// 设置过期时间
    ///
    /// # 参数
    /// - `key`: 缓存键
    /// - `ttl_seconds`: 生存时间（秒）
    async fn expire(&self, key: &str, ttl_seconds: u64) -> Result<()>;

    /// 获取剩余生存时间
    ///
    /// # 参数
    /// - `key`: 缓存键
    ///
    /// # 返回
    /// - `Some(u64)`: 剩余秒数
    /// - `None`: 键不存在或未设置TTL
    async fn ttl(&self, key: &str) -> Result<Option<u64>>;

    /// 清空所有缓存
    async fn clear(&self) -> Result<()>;

    /// 获取缓存大小（条目数）
    ///
    /// 注意：某些后端可能不支持此操作
    async fn size(&self) -> Result<Option<usize>>;

    /// 批量设置
    ///
    /// # 参数
    /// - `items`: 键值对列表
    /// - `ttl_seconds`: 生存时间（可选）
    async fn mset(&self, items: Vec<(String, Vec<u8>)>, ttl_seconds: Option<u64>) -> Result<()>;

    /// 批量获取
    ///
    /// # 参数
    /// - `keys`: 键列表
    ///
    /// # 返回
    /// - 键到值的映射
    async fn mget(&self, keys: Vec<String>) -> Result<Vec<Option<Vec<u8>>>>;
}

/// 分布式缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// 缓存类型
    pub cache_type: CacheType,
    /// 连接字符串（如Redis URL）
    pub connection_string: Option<String>,
    /// 最大连接数
    pub max_connections: u32,
    /// 连接超时时间
    pub connect_timeout: Duration,
    /// 命令超时时间
    pub command_timeout: Duration,
    /// 重试次数
    pub max_retries: u32,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            cache_type: CacheType::Memory,
            connection_string: None,
            max_connections: 10,
            connect_timeout: Duration::from_secs(5),
            command_timeout: Duration::from_secs(3),
            max_retries: 3,
        }
    }
}

/// 缓存类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheType {
    /// 内存缓存
    Memory,
    /// Redis
    #[cfg(feature = "redis")]
    Redis,
}

impl Default for CacheType {
    fn default() -> Self {
        Self::Memory
    }
}

/// 序列化/反序列化trait
pub trait CacheCodec: Send + Sync {
    /// 序列化
    fn serialize<T: serde::Serialize>(value: &T) -> Result<Vec<u8>>;

    /// 反序列化
    fn deserialize<T: for<'de> serde::Deserialize<'de>>(bytes: &[u8]) -> Result<T>;
}

/// JSON编解码器（默认）
pub struct JsonCodec;

impl CacheCodec for JsonCodec {
    fn serialize<T: serde::Serialize>(value: &T) -> Result<Vec<u8>> {
        serde_json::to_vec(value).map_err(|e| crate::error::Error::Cache(e.to_string()))
    }

    fn deserialize<T: for<'de> serde::Deserialize<'de>>(bytes: &[u8]) -> Result<T> {
        serde_json::from_slice(bytes).map_err(|e| crate::error::Error::Cache(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.cache_type, CacheType::Memory);
        assert_eq!(config.max_connections, 10);
        assert!(config.connection_string.is_none());
    }

    #[test]
    fn test_json_codec_serialize() {
        let value = "test_value";
        let bytes = JsonCodec::serialize(&value).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_json_codec_deserialize() {
        let value = "test_value";
        let bytes = JsonCodec::serialize(&value).unwrap();
        let deserialized: String = JsonCodec::deserialize(&bytes).unwrap();
        assert_eq!(deserialized, value);
    }

    #[test]
    fn test_json_codec_serialize_struct() {
        #[derive(Serialize, Deserialize)]
        struct TestStruct {
            name: String,
            count: i32,
        }

        let value = TestStruct { name: "test".to_string(), count: 42 };

        let bytes = JsonCodec::serialize(&value).unwrap();
        let deserialized: TestStruct = JsonCodec::deserialize(&bytes).unwrap();

        assert_eq!(deserialized.name, "test");
        assert_eq!(deserialized.count, 42);
    }
}
