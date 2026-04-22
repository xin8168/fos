//! # 存储模块

use crate::error::{MemoryError, Result};
use crate::{EventQuery, SuccessEvent};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 内存存储（用于开发和测试）
pub struct InMemoryStorage {
    /// 事件存储
    events: Arc<RwLock<HashMap<String, SuccessEvent>>>,
}

impl InMemoryStorage {
    /// 创建新的内存存储
    pub fn new() -> Self {
        Self { events: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// 存储事件
    pub async fn store(&self, event: SuccessEvent) -> Result<String> {
        let id = event.id.clone();
        let mut events = self.events.write().await;
        events.insert(id.clone(), event);
        Ok(id)
    }

    /// 获取事件
    pub async fn get(&self, id: &str) -> Result<SuccessEvent> {
        let events = self.events.read().await;
        events.get(id).cloned().ok_or_else(|| MemoryError::EventNotFound(id.to_string()))
    }

    /// 查询事件
    pub async fn query(&self, query: EventQuery) -> Result<Vec<SuccessEvent>> {
        let events = self.events.read().await;
        let mut results: Vec<SuccessEvent> = events
            .values()
            .filter(|event| {
                let mut matches = true;

                if let Some(ref name) = query.name {
                    matches &= event.name.contains(name);
                }

                if let Some(ref event_type) = query.event_type {
                    matches &= &event.event_type == event_type;
                }

                if let Some(ref location) = query.location {
                    matches &= &event.location == location;
                }

                if let Some(ref subject) = query.subject {
                    matches &= &event.subject == subject;
                }

                if let Some(start_time) = query.start_time {
                    matches &= event.created_at >= start_time;
                }

                if let Some(end_time) = query.end_time {
                    matches &= event.created_at <= end_time;
                }

                matches
            })
            .cloned()
            .collect();

        // 排序（按创建时间降序）
        results.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // 分页
        if let Some(offset) = query.offset {
            results = results.into_iter().skip(offset).collect();
        }

        if let Some(limit) = query.limit {
            results = results.into_iter().take(limit).collect();
        }

        Ok(results)
    }

    /// 删除事件
    pub async fn delete(&self, id: &str) -> Result<()> {
        let mut events = self.events.write().await;
        events.remove(id).ok_or_else(|| MemoryError::EventNotFound(id.to_string()))?;
        Ok(())
    }

    /// 统计事件数量
    pub async fn count(&self) -> Result<usize> {
        let events = self.events.read().await;
        Ok(events.len())
    }

    /// 清空所有事件
    pub async fn clear(&self) -> Result<()> {
        let mut events = self.events.write().await;
        events.clear();
        Ok(())
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// 存储类型
#[derive(Debug, Clone)]
pub enum StorageType {
    /// 内存存储
    InMemory,

    /// PostgreSQL 存储
    Postgres,

    /// SQLite 存储
    SQLite,
}

/// 存储配置
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// 存储类型
    pub storage_type: StorageType,

    /// 数据库 URL
    pub database_url: Option<String>,

    /// 连接池大小
    pub pool_size: usize,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self { storage_type: StorageType::InMemory, database_url: None, pool_size: 10 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_store_and_get() {
        let storage = InMemoryStorage::new();
        let event = SuccessEvent::new(
            "测试事件".to_string(),
            "device_control".to_string(),
            vec!["步骤1".to_string()],
            "条件1".to_string(),
            "标准1".to_string(),
            "位置1".to_string(),
            "主体1".to_string(),
        );

        let id = storage.store(event.clone()).await.unwrap();
        let retrieved = storage.get(&id).await.unwrap();

        assert_eq!(retrieved.name, event.name);
    }

    #[tokio::test]
    async fn test_query() {
        let storage = InMemoryStorage::new();

        // 存储多个事件
        for i in 0..5 {
            let event = SuccessEvent::new(
                format!("测试事件{}", i),
                "device_control".to_string(),
                vec!["步骤1".to_string()],
                "条件1".to_string(),
                "标准1".to_string(),
                "位置1".to_string(),
                "主体1".to_string(),
            );
            storage.store(event).await.unwrap();
        }

        let query =
            EventQuery { name: Some("测试".to_string()), limit: Some(3), ..Default::default() };

        let results = storage.query(query).await.unwrap();
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_delete() {
        let storage = InMemoryStorage::new();
        let event = SuccessEvent::new(
            "测试事件".to_string(),
            "device_control".to_string(),
            vec!["步骤1".to_string()],
            "条件1".to_string(),
            "标准1".to_string(),
            "位置1".to_string(),
            "主体1".to_string(),
        );

        let id = storage.store(event).await.unwrap();
        storage.delete(&id).await.unwrap();

        let result = storage.get(&id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_count() {
        let storage = InMemoryStorage::new();

        for i in 0..10 {
            let event = SuccessEvent::new(
                format!("事件{}", i),
                "test".to_string(),
                vec!["步骤".to_string()],
                "条件".to_string(),
                "标准".to_string(),
                "位置".to_string(),
                "主体".to_string(),
            );
            storage.store(event).await.unwrap();
        }

        let count = storage.count().await.unwrap();
        assert_eq!(count, 10);
    }
}
