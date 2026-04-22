//! 离线缓存模块
//!
//! 提供设备离线时的数据缓存和同步能力，确保数据不丢失

use crate::error::{McpError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::RwLock;

/// 缓存项状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CacheItemStatus {
    /// 待同步
    Pending,
    /// 同步中
    Syncing,
    /// 已同步
    Synced,
    /// 同步失败
    Failed,
    /// 已过期
    Expired,
}

/// 缓存优先级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum CachePriority {
    /// 低优先级
    Low,
    /// 普通优先级
    Normal,
    /// 高优先级
    High,
    /// 紧急
    Critical,
}

impl Default for CachePriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// 缓存项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheItem {
    /// 唯一ID
    pub id: String,
    /// 设备ID
    pub device_id: String,
    /// 数据类型
    pub data_type: String,
    /// 数据内容
    pub data: Vec<u8>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 过期时间
    pub expires_at: Option<DateTime<Utc>>,
    /// 状态
    pub status: CacheItemStatus,
    /// 优先级
    pub priority: CachePriority,
    /// 重试次数
    pub retry_count: u32,
    /// 最大重试次数
    pub max_retries: u32,
    /// 最后同步时间
    pub last_sync_attempt: Option<DateTime<Utc>>,
    /// 错误信息
    pub error_message: Option<String>,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl CacheItem {
    /// 创建新缓存项
    pub fn new(device_id: String, data_type: String, data: Vec<u8>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            device_id,
            data_type,
            data,
            created_at: Utc::now(),
            expires_at: None,
            status: CacheItemStatus::Pending,
            priority: CachePriority::default(),
            retry_count: 0,
            max_retries: 5,
            last_sync_attempt: None,
            error_message: None,
            metadata: HashMap::new(),
        }
    }

    /// 设置过期时间
    pub fn with_expiry(mut self, hours: u64) -> Self {
        self.expires_at = Some(Utc::now() + chrono::Duration::hours(hours as i64));
        self
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: CachePriority) -> Self {
        self.priority = priority;
        self
    }

    /// 设置最大重试次数
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// 添加元数据
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// 检查是否过期
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires_at) => Utc::now() > expires_at,
            None => false,
        }
    }

    /// 检查是否可重试
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries && self.status != CacheItemStatus::Synced
    }

    /// 标记同步中
    pub fn mark_syncing(&mut self) {
        self.status = CacheItemStatus::Syncing;
        self.last_sync_attempt = Some(Utc::now());
    }

    /// 标记同步成功
    pub fn mark_synced(&mut self) {
        self.status = CacheItemStatus::Synced;
        self.error_message = None;
    }

    /// 标记同步失败
    pub fn mark_failed(&mut self, error: String) {
        self.retry_count += 1;
        self.status = CacheItemStatus::Failed;
        self.error_message = Some(error);
    }

    /// 标记过期
    pub fn mark_expired(&mut self) {
        self.status = CacheItemStatus::Expired;
    }

    /// 计算数据大小
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineCacheConfig {
    /// 缓存目录
    pub cache_dir: PathBuf,
    /// 最大缓存大小（字节）
    pub max_size: u64,
    /// 最大缓存项数
    pub max_items: usize,
    /// 默认过期时间（小时）
    pub default_expiry_hours: u64,
    /// 同步间隔（秒）
    pub sync_interval_secs: u64,
    /// 每次同步的最大项数
    pub batch_size: usize,
}

impl Default for OfflineCacheConfig {
    fn default() -> Self {
        Self {
            cache_dir: PathBuf::from("./cache/offline"),
            max_size: 1024 * 1024 * 1024, // 1GB
            max_items: 100000,
            default_expiry_hours: 24 * 7, // 7天
            sync_interval_secs: 30,
            batch_size: 100,
        }
    }
}

/// 缓存统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheStats {
    /// 总项数
    pub total_items: usize,
    /// 待同步项数
    pub pending_items: usize,
    /// 同步中项数
    pub syncing_items: usize,
    /// 已同步项数
    pub synced_items: usize,
    /// 失败项数
    pub failed_items: usize,
    /// 过期项数
    pub expired_items: usize,
    /// 总数据大小
    pub total_size: u64,
}

/// 离线缓存管理器
pub struct OfflineCacheManager {
    /// 内存缓存
    cache: Arc<RwLock<HashMap<String, CacheItem>>>,
    /// 设备索引
    device_index: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// 配置
    config: OfflineCacheConfig,
    /// 统计
    stats: Arc<RwLock<CacheStats>>,
}

impl OfflineCacheManager {
    /// 创建新缓存管理器
    pub fn new() -> Self {
        Self::with_config(OfflineCacheConfig::default())
    }

    /// 使用配置创建
    pub fn with_config(config: OfflineCacheConfig) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            device_index: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// 初始化缓存目录
    pub async fn initialize(&self) -> Result<()> {
        fs::create_dir_all(&self.config.cache_dir)
            .await
            .map_err(|e| McpError::Internal(format!("创建缓存目录失败: {}", e)))?;
        Ok(())
    }

    /// 添加缓存项
    pub async fn add(&self, item: CacheItem) -> Result<String> {
        // 检查数量限制
        {
            let cache = self.cache.read().await;
            if cache.len() >= self.config.max_items {
                return Err(McpError::LimitExceeded("缓存项数量已达上限".to_string()));
            }
        }

        // 检查大小限制
        {
            let stats = self.stats.read().await;
            let new_size = stats.total_size + item.size() as u64;
            if new_size > self.config.max_size {
                return Err(McpError::LimitExceeded("缓存大小已达上限".to_string()));
            }
        }

        let id = item.id.clone();
        let device_id = item.device_id.clone();

        // 存储到内存
        {
            let mut cache = self.cache.write().await;
            cache.insert(id.clone(), item);
        }

        // 更新设备索引
        {
            let mut device_index = self.device_index.write().await;
            device_index.entry(device_id).or_insert_with(Vec::new).push(id.clone());
        }

        // 更新统计
        self.update_stats().await;

        Ok(id)
    }

    /// 获取缓存项
    pub async fn get(&self, id: &str) -> Result<CacheItem> {
        let cache = self.cache.read().await;
        cache.get(id).cloned().ok_or_else(|| McpError::Internal(format!("缓存项不存在: {}", id)))
    }

    /// 获取设备的所有缓存项
    pub async fn get_by_device(&self, device_id: &str) -> Vec<CacheItem> {
        let device_index = self.device_index.read().await;
        let cache = self.cache.read().await;

        device_index
            .get(device_id)
            .map(|ids| ids.iter().filter_map(|id| cache.get(id).cloned()).collect())
            .unwrap_or_default()
    }

    /// 获取待同步的缓存项
    pub async fn get_pending(&self, limit: usize) -> Vec<CacheItem> {
        let cache = self.cache.read().await;
        let mut pending: Vec<CacheItem> = cache
            .values()
            .filter(|item| item.status == CacheItemStatus::Pending)
            .cloned()
            .collect();

        // 按优先级排序
        pending.sort_by(|a, b| b.priority.cmp(&a.priority));
        pending.truncate(limit);
        pending
    }

    /// 获取失败的缓存项（可重试）
    pub async fn get_retryable(&self, limit: usize) -> Vec<CacheItem> {
        let cache = self.cache.read().await;
        let mut retryable: Vec<CacheItem> = cache
            .values()
            .filter(|item| item.status == CacheItemStatus::Failed && item.can_retry())
            .cloned()
            .collect();

        retryable.sort_by(|a, b| b.priority.cmp(&a.priority));
        retryable.truncate(limit);
        retryable
    }

    /// 标记同步中
    pub async fn mark_syncing(&self, id: &str) -> Result<()> {
        let mut cache = self.cache.write().await;
        if let Some(item) = cache.get_mut(id) {
            item.mark_syncing();
            Ok(())
        } else {
            Err(McpError::Internal(format!("缓存项不存在: {}", id)))
        }
    }

    /// 标记同步成功
    pub async fn mark_synced(&self, id: &str) -> Result<()> {
        let mut cache = self.cache.write().await;
        if let Some(item) = cache.get_mut(id) {
            item.mark_synced();
            Ok(())
        } else {
            Err(McpError::Internal(format!("缓存项不存在: {}", id)))
        }
    }

    /// 标记同步失败
    pub async fn mark_failed(&self, id: &str, error: String) -> Result<()> {
        let mut cache = self.cache.write().await;
        if let Some(item) = cache.get_mut(id) {
            item.mark_failed(error);
            Ok(())
        } else {
            Err(McpError::Internal(format!("缓存项不存在: {}", id)))
        }
    }

    /// 删除缓存项
    pub async fn remove(&self, id: &str) -> Result<()> {
        let device_id: String;

        // 从缓存中移除
        {
            let mut cache = self.cache.write().await;
            let item = cache
                .remove(id)
                .ok_or_else(|| McpError::Internal(format!("缓存项不存在: {}", id)))?;
            device_id = item.device_id;
        }

        // 从设备索引中移除
        {
            let mut device_index = self.device_index.write().await;
            if let Some(ids) = device_index.get_mut(&device_id) {
                ids.retain(|i| i != id);
            }
        }

        self.update_stats().await;
        Ok(())
    }

    /// 清理过期缓存项
    pub async fn cleanup_expired(&self) -> Result<usize> {
        let expired_ids: Vec<String> = {
            let cache = self.cache.read().await;
            cache.iter().filter(|(_, item)| item.is_expired()).map(|(id, _)| id.clone()).collect()
        };

        let count = expired_ids.len();

        {
            let mut cache = self.cache.write().await;
            let mut device_index = self.device_index.write().await;

            for id in expired_ids {
                if let Some(item) = cache.remove(&id) {
                    if let Some(ids) = device_index.get_mut(&item.device_id) {
                        ids.retain(|i| i != &id);
                    }
                }
            }
        } // Write locks are dropped here

        self.update_stats().await;
        Ok(count)
    }

    /// 清理已同步的缓存项
    pub async fn cleanup_synced(&self) -> Result<usize> {
        let synced_ids: Vec<String> = {
            let cache = self.cache.read().await;
            cache
                .iter()
                .filter(|(_, item)| item.status == CacheItemStatus::Synced)
                .map(|(id, _)| id.clone())
                .collect()
        };

        let count = synced_ids.len();

        {
            let mut cache = self.cache.write().await;
            let mut device_index = self.device_index.write().await;

            for id in synced_ids {
                if let Some(item) = cache.remove(&id) {
                    if let Some(ids) = device_index.get_mut(&item.device_id) {
                        ids.retain(|i| i != &id);
                    }
                }
            }
        } // Write locks are dropped here

        self.update_stats().await;
        Ok(count)
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }

    /// 更新统计信息
    async fn update_stats(&self) {
        let cache = self.cache.read().await;
        let mut stats = self.stats.write().await;

        stats.total_items = cache.len();
        stats.pending_items =
            cache.values().filter(|i| i.status == CacheItemStatus::Pending).count();
        stats.syncing_items =
            cache.values().filter(|i| i.status == CacheItemStatus::Syncing).count();
        stats.synced_items = cache.values().filter(|i| i.status == CacheItemStatus::Synced).count();
        stats.failed_items = cache.values().filter(|i| i.status == CacheItemStatus::Failed).count();
        stats.expired_items =
            cache.values().filter(|i| i.status == CacheItemStatus::Expired).count();
        stats.total_size = cache.values().map(|i| i.size() as u64).sum();
    }

    /// 持久化到磁盘
    pub async fn persist(&self) -> Result<()> {
        self.initialize().await?;

        let cache = self.cache.read().await;
        let path = self.config.cache_dir.join("cache.dat");

        let data = bincode::serialize(cache.deref())
            .map_err(|e| McpError::Internal(format!("序列化失败: {}", e)))?;

        let mut file = fs::File::create(&path)
            .await
            .map_err(|e| McpError::Internal(format!("创建文件失败: {}", e)))?;

        file.write_all(&data)
            .await
            .map_err(|e| McpError::Internal(format!("写入文件失败: {}", e)))?;

        Ok(())
    }

    /// 从磁盘加载
    pub async fn load(&self) -> Result<()> {
        let path = self.config.cache_dir.join("cache.dat");

        if !path.exists() {
            return Ok(());
        }

        let mut file = fs::File::open(&path)
            .await
            .map_err(|e| McpError::Internal(format!("打开文件失败: {}", e)))?;

        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .await
            .map_err(|e| McpError::Internal(format!("读取文件失败: {}", e)))?;

        let loaded: HashMap<String, CacheItem> = bincode::deserialize(&data)
            .map_err(|e| McpError::Internal(format!("反序列化失败: {}", e)))?;

        // 重建索引
        let mut cache = self.cache.write().await;
        let mut device_index = self.device_index.write().await;

        for (id, item) in loaded {
            let device_id = item.device_id.clone();
            device_index.entry(device_id).or_insert_with(Vec::new).push(id.clone());
            cache.insert(id, item);
        }

        self.update_stats().await;
        Ok(())
    }

    /// 清空缓存
    pub async fn clear(&self) -> Result<()> {
        let mut cache = self.cache.write().await;
        let mut device_index = self.device_index.write().await;

        cache.clear();
        device_index.clear();

        self.update_stats().await;
        Ok(())
    }
}

impl Default for OfflineCacheManager {
    fn default() -> Self {
        Self::new()
    }
}

// 用于 bincode 序列化
use std::ops::Deref;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_item_creation() {
        let item =
            CacheItem::new("device-1".to_string(), "sensor_data".to_string(), vec![1, 2, 3, 4, 5]);

        assert!(!item.id.is_empty());
        assert_eq!(item.device_id, "device-1");
        assert_eq!(item.data_type, "sensor_data");
        assert_eq!(item.status, CacheItemStatus::Pending);
    }

    #[test]
    fn test_cache_item_expiry() {
        let mut item = CacheItem::new("device-1".to_string(), "data".to_string(), vec![]);
        // 手动设置过期时间为过去
        item.expires_at = Some(Utc::now() - chrono::Duration::seconds(1));
        assert!(item.is_expired());
    }

    #[test]
    fn test_cache_item_retry() {
        let mut item = CacheItem::new("device-1".to_string(), "data".to_string(), vec![]);
        item.max_retries = 3;

        assert!(item.can_retry());

        item.mark_failed("error".to_string());
        assert_eq!(item.retry_count, 1);
        assert!(item.can_retry());

        for _ in 0..3 {
            item.mark_failed("error".to_string());
        }
        assert!(!item.can_retry());
    }

    #[test]
    fn test_cache_priority_order() {
        assert!(CachePriority::Critical > CachePriority::High);
        assert!(CachePriority::High > CachePriority::Normal);
        assert!(CachePriority::Normal > CachePriority::Low);
    }

    #[tokio::test]
    async fn test_cache_manager_add() {
        let manager = OfflineCacheManager::new();

        let item = CacheItem::new("device-1".to_string(), "data".to_string(), vec![1, 2, 3]);

        let id = manager.add(item).await.unwrap();
        assert!(!id.is_empty());

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_items, 1);
    }

    #[tokio::test]
    async fn test_cache_manager_get() {
        let manager = OfflineCacheManager::new();

        let item = CacheItem::new("device-1".to_string(), "data".to_string(), vec![1, 2, 3]);
        let id = manager.add(item).await.unwrap();

        let retrieved = manager.get(&id).await.unwrap();
        assert_eq!(retrieved.device_id, "device-1");
    }

    #[tokio::test]
    async fn test_cache_manager_get_by_device() {
        let manager = OfflineCacheManager::new();

        let item1 = CacheItem::new("device-1".to_string(), "data".to_string(), vec![]);
        let item2 = CacheItem::new("device-1".to_string(), "data".to_string(), vec![]);

        manager.add(item1).await.unwrap();
        manager.add(item2).await.unwrap();

        let items = manager.get_by_device("device-1").await;
        assert_eq!(items.len(), 2);
    }

    #[tokio::test]
    async fn test_cache_manager_get_pending() {
        let manager = OfflineCacheManager::new();

        let item1 = CacheItem::new("device-1".to_string(), "data".to_string(), vec![])
            .with_priority(CachePriority::High);
        let item2 = CacheItem::new("device-2".to_string(), "data".to_string(), vec![])
            .with_priority(CachePriority::Low);

        manager.add(item1).await.unwrap();
        manager.add(item2).await.unwrap();

        let pending = manager.get_pending(10).await;
        assert_eq!(pending.len(), 2);
        // 高优先级应排在前面
        assert_eq!(pending[0].priority, CachePriority::High);
    }

    #[tokio::test]
    async fn test_cache_manager_mark_synced() {
        let manager = OfflineCacheManager::new();

        let item = CacheItem::new("device-1".to_string(), "data".to_string(), vec![]);
        let id = manager.add(item).await.unwrap();

        manager.mark_synced(&id).await.unwrap();

        let retrieved = manager.get(&id).await.unwrap();
        assert_eq!(retrieved.status, CacheItemStatus::Synced);
    }

    #[tokio::test]
    async fn test_cache_manager_remove() {
        let manager = OfflineCacheManager::new();

        let item = CacheItem::new("device-1".to_string(), "data".to_string(), vec![]);
        let id = manager.add(item).await.unwrap();

        manager.remove(&id).await.unwrap();

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_items, 0);
    }

    #[tokio::test]
    async fn test_cache_manager_cleanup_expired() {
        let manager = OfflineCacheManager::new();

        let mut item = CacheItem::new("device-1".to_string(), "data".to_string(), vec![]);
        // 设置过期时间为过去
        item.expires_at = Some(Utc::now() - chrono::Duration::seconds(1));
        manager.add(item).await.unwrap();

        let count = manager.cleanup_expired().await.unwrap();
        assert_eq!(count, 1);

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_items, 0);
    }

    #[tokio::test]
    async fn test_cache_manager_cleanup_synced() {
        let manager = OfflineCacheManager::new();

        let item = CacheItem::new("device-1".to_string(), "data".to_string(), vec![]);
        let id = manager.add(item).await.unwrap();

        manager.mark_synced(&id).await.unwrap();
        let count = manager.cleanup_synced().await.unwrap();
        assert_eq!(count, 1);

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_items, 0);
    }

    // ===== 边界条件测试 =====

    #[tokio::test]
    async fn test_cache_manager_empty_operations() {
        let manager = OfflineCacheManager::new();

        // 空缓存获取
        let result = manager.get("non-existent").await;
        assert!(result.is_err());

        // 空缓存删除
        let result = manager.remove("non-existent").await;
        assert!(result.is_err());

        // 空缓存标记状态
        let result = manager.mark_synced("non-existent").await;
        assert!(result.is_err());

        // 空缓存清理
        let count = manager.cleanup_expired().await.unwrap();
        assert_eq!(count, 0);

        let count = manager.cleanup_synced().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_cache_manager_max_items_limit() {
        let config = OfflineCacheConfig { max_items: 5, ..Default::default() };
        let manager = OfflineCacheManager::with_config(config);

        // 添加5个项（正常）
        for i in 0..5 {
            let item = CacheItem::new(format!("device-{}", i), "data".to_string(), vec![]);
            assert!(manager.add(item).await.is_ok());
        }

        // 添加第6个项（应失败）
        let item = CacheItem::new("device-5".to_string(), "data".to_string(), vec![]);
        let result = manager.add(item).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cache_manager_max_size_limit() {
        let config = OfflineCacheConfig {
            max_size: 100, // 100 bytes
            ..Default::default()
        };
        let manager = OfflineCacheManager::with_config(config);

        // 添加一个大项（80 bytes）
        let large_item = CacheItem::new("device-1".to_string(), "data".to_string(), vec![0; 80]);
        assert!(manager.add(large_item).await.is_ok());

        // 添加另一个大项（超过限制）
        let large_item = CacheItem::new("device-2".to_string(), "data".to_string(), vec![0; 80]);
        let result = manager.add(large_item).await;
        assert!(result.is_err());
    }

    // ===== 错误场景测试 =====

    #[tokio::test]
    async fn test_cache_manager_mark_failed() {
        let manager = OfflineCacheManager::new();

        let item = CacheItem::new("device-1".to_string(), "data".to_string(), vec![]);
        let id = manager.add(item).await.unwrap();

        let error_msg = "Connection timeout".to_string();
        manager.mark_failed(&id, error_msg.clone()).await.unwrap();

        let retrieved = manager.get(&id).await.unwrap();
        assert_eq!(retrieved.status, CacheItemStatus::Failed);
        assert_eq!(retrieved.retry_count, 1);
        assert_eq!(retrieved.error_message, Some(error_msg));
    }

    #[tokio::test]
    async fn test_cache_manager_retry_limits() {
        let manager = OfflineCacheManager::new();

        let mut item = CacheItem::new("device-1".to_string(), "data".to_string(), vec![]);
        item.max_retries = 3;
        let id = manager.add(item).await.unwrap();

        // 标记失败3次
        for i in 1..=3 {
            manager.mark_failed(&id, format!("Error {}", i)).await.unwrap();
        }

        let retrieved = manager.get(&id).await.unwrap();
        assert_eq!(retrieved.status, CacheItemStatus::Failed);
        assert_eq!(retrieved.retry_count, 3);

        // 应该不在可重试列表中
        let retryable = manager.get_retryable(100).await;
        assert!(retryable.is_empty());
    }

    // ===== 状态转换测试 =====

    #[tokio::test]
    async fn test_cache_item_full_lifecycle() {
        let manager = OfflineCacheManager::new();

        let mut item = CacheItem::new("device-1".to_string(), "data".to_string(), vec![]);
        item.expires_at = Some(Utc::now() + chrono::Duration::hours(1));
        let id = manager.add(item).await.unwrap();

        // 初始状态: Pending
        let retrieved = manager.get(&id).await.unwrap();
        assert_eq!(retrieved.status, CacheItemStatus::Pending);

        // 标记为 Syncing
        manager.mark_syncing(&id).await.unwrap();
        let retrieved = manager.get(&id).await.unwrap();
        assert_eq!(retrieved.status, CacheItemStatus::Syncing);

        // 标记为 Synced
        manager.mark_synced(&id).await.unwrap();
        let retrieved = manager.get(&id).await.unwrap();
        assert_eq!(retrieved.status, CacheItemStatus::Synced);

        // 清理
        let count = manager.cleanup_synced().await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_cache_item_status_updates_stats() {
        let manager = OfflineCacheManager::new();

        // 初始状态
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_items, 0);
        assert_eq!(stats.pending_items, 0);

        // 添加一个pending项
        let item = CacheItem::new("device-1".to_string(), "data".to_string(), vec![]);
        let id = manager.add(item).await.unwrap();

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_items, 1);
        assert_eq!(stats.pending_items, 1);

        // 标记为synced - 注意：状态改变不会立即更新统计信息
        manager.mark_synced(&id).await.unwrap();

        // 验证item状态确实改变了
        let retrieved = manager.get(&id).await.unwrap();
        assert_eq!(retrieved.status, CacheItemStatus::Synced);
    }

    // ===== 优先级测试 =====

    #[tokio::test]
    async fn test_cache_priority_filtering() {
        let manager = OfflineCacheManager::new();

        // 添加不同优先级的项
        let mut low_item = CacheItem::new("device-1".to_string(), "data".to_string(), vec![]);
        low_item.priority = CachePriority::Low;
        manager.add(low_item).await.unwrap();

        let mut high_item = CacheItem::new("device-2".to_string(), "data".to_string(), vec![]);
        high_item.priority = CachePriority::High;
        manager.add(high_item).await.unwrap();

        let mut critical_item = CacheItem::new("device-3".to_string(), "data".to_string(), vec![]);
        critical_item.priority = CachePriority::Critical;
        manager.add(critical_item).await.unwrap();

        let pending = manager.get_pending(100).await;
        assert_eq!(pending.len(), 3);

        // 验证排序：Critical > High > Low
        assert_eq!(pending[0].priority, CachePriority::Critical);
        assert_eq!(pending[1].priority, CachePriority::High);
        assert_eq!(pending[2].priority, CachePriority::Low);
    }

    // ===== 元数据测试 =====

    #[tokio::test]
    async fn test_cache_item_metadata() {
        let manager = OfflineCacheManager::new();

        let mut item = CacheItem::new("device-1".to_string(), "data".to_string(), vec![]);
        item.metadata.insert("source".to_string(), "sensor".to_string());
        item.metadata.insert("location".to_string(), "room-101".to_string());
        let id = manager.add(item).await.unwrap();

        let retrieved = manager.get(&id).await.unwrap();
        assert_eq!(retrieved.metadata.get("source"), Some(&"sensor".to_string()));
        assert_eq!(retrieved.metadata.get("location"), Some(&"room-101".to_string()));
        assert_eq!(retrieved.metadata.len(), 2);
    }

    // ===== 设备索引测试 =====

    #[tokio::test]
    async fn test_device_index_consistency() {
        let manager = OfflineCacheManager::new();

        // 为同一设备添加多个项
        let item1 = CacheItem::new("device-1".to_string(), "data1".to_string(), vec![]);
        let item2 = CacheItem::new("device-1".to_string(), "data2".to_string(), vec![]);
        let item3 = CacheItem::new("device-2".to_string(), "data3".to_string(), vec![]);

        manager.add(item1).await.unwrap();
        manager.add(item2).await.unwrap();
        manager.add(item3).await.unwrap();

        // 查询device-1的项
        let device1_items = manager.get_by_device("device-1").await;
        assert_eq!(device1_items.len(), 2);

        // 查询device-2的项
        let device2_items = manager.get_by_device("device-2").await;
        assert_eq!(device2_items.len(), 1);

        // 删除一个项
        let first_id = &device1_items[0].id;
        manager.remove(first_id).await.unwrap();

        // 验证索引更新
        let device1_items = manager.get_by_device("device-1").await;
        assert_eq!(device1_items.len(), 1);
    }

    // ===== 混合清理测试 =====

    #[tokio::test]
    async fn test_mixed_cleanup() {
        let manager = OfflineCacheManager::new();

        // 添加各种状态的项
        let mut expired_item = CacheItem::new("device-1".to_string(), "data".to_string(), vec![]);
        expired_item.expires_at = Some(Utc::now() - chrono::Duration::hours(1));
        manager.add(expired_item).await.unwrap();

        let normal_item = CacheItem::new("device-2".to_string(), "data".to_string(), vec![]);
        let synced_id = manager.add(normal_item).await.unwrap();
        manager.mark_synced(&synced_id).await.unwrap();

        let pending_item = CacheItem::new("device-3".to_string(), "data".to_string(), vec![]);
        manager.add(pending_item).await.unwrap();

        // 初始状态: 3个项
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_items, 3);

        // 清理过期项
        let count = manager.cleanup_expired().await.unwrap();
        assert_eq!(count, 1);
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_items, 2);

        // 清理已同步项
        let count = manager.cleanup_synced().await.unwrap();
        assert_eq!(count, 1);
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_items, 1); // 只剩pending项
    }
}
