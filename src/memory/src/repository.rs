//! # 事件仓库模块

use crate::error::Result;
use crate::storage::InMemoryStorage;
use crate::{EventQuery, SuccessEvent};

/// 事件仓库
pub struct EventRepository {
    storage: InMemoryStorage,
}

impl EventRepository {
    /// 创建新的事件仓库
    pub fn new() -> Self {
        Self { storage: InMemoryStorage::new() }
    }

    /// 保存成功事件
    pub async fn save(&self, event: SuccessEvent) -> Result<String> {
        self.storage.store(event).await
    }

    /// 根据 ID 获取事件
    pub async fn find_by_id(&self, id: &str) -> Result<SuccessEvent> {
        self.storage.get(id).await
    }

    /// 查询事件
    pub async fn find(&self, query: EventQuery) -> Result<Vec<SuccessEvent>> {
        self.storage.query(query).await
    }

    /// 删除事件
    pub async fn delete(&self, id: &str) -> Result<()> {
        self.storage.delete(id).await
    }

    /// 统计事件数量
    pub async fn count(&self) -> Result<usize> {
        self.storage.count().await
    }

    /// 按名称搜索事件
    pub async fn search_by_name(&self, name: &str) -> Result<Vec<SuccessEvent>> {
        let query = EventQuery { name: Some(name.to_string()), ..Default::default() };
        self.find(query).await
    }

    /// 按类型查询事件
    pub async fn find_by_type(&self, event_type: &str) -> Result<Vec<SuccessEvent>> {
        let query = EventQuery { event_type: Some(event_type.to_string()), ..Default::default() };
        self.find(query).await
    }

    /// 按地点查询事件
    pub async fn find_by_location(&self, location: &str) -> Result<Vec<SuccessEvent>> {
        let query = EventQuery { location: Some(location.to_string()), ..Default::default() };
        self.find(query).await
    }

    /// 按主体查询事件
    pub async fn find_by_subject(&self, subject: &str) -> Result<Vec<SuccessEvent>> {
        let query = EventQuery { subject: Some(subject.to_string()), ..Default::default() };
        self.find(query).await
    }

    /// 复用成功事件（创建副本）
    pub async fn reuse(&self, id: &str) -> Result<SuccessEvent> {
        let original = self.find_by_id(id).await?;

        // 创建新事件（保持相同内容）
        let mut reused = SuccessEvent::new(
            original.name.clone(),
            original.event_type.clone(),
            original.steps.clone(),
            original.judgment_logic.clone(),
            original.verification_standard.clone(),
            original.location.clone(),
            original.subject.clone(),
        );

        // 复制元数据
        reused = reused.with_metadata(original.metadata.clone());

        Ok(reused)
    }

    /// 获取最近的事件
    pub async fn find_recent(&self, limit: usize) -> Result<Vec<SuccessEvent>> {
        let query = EventQuery { limit: Some(limit), ..Default::default() };
        self.find(query).await
    }
}

impl Default for EventRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_save_and_find() {
        let repo = EventRepository::new();
        let event = SuccessEvent::new(
            "测试事件".to_string(),
            "device_control".to_string(),
            vec!["步骤1".to_string()],
            "条件1".to_string(),
            "标准1".to_string(),
            "位置1".to_string(),
            "主体1".to_string(),
        );

        let id = repo.save(event).await.unwrap();
        let found = repo.find_by_id(&id).await.unwrap();

        assert_eq!(found.name, "测试事件");
    }

    #[tokio::test]
    async fn test_search_by_name() {
        let repo = EventRepository::new();

        // 存储多个事件
        for i in 0..3 {
            let event = SuccessEvent::new(
                format!("测试事件{}", i),
                "test".to_string(),
                vec!["步骤".to_string()],
                "条件".to_string(),
                "标准".to_string(),
                "位置".to_string(),
                "主体".to_string(),
            );
            repo.save(event).await.unwrap();
        }

        let results = repo.search_by_name("测试").await.unwrap();
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_reuse_event() {
        let repo = EventRepository::new();
        let event = SuccessEvent::new(
            "原事件".to_string(),
            "device_control".to_string(),
            vec!["步骤1".to_string()],
            "条件1".to_string(),
            "标准1".to_string(),
            "位置1".to_string(),
            "主体1".to_string(),
        );

        let id = repo.save(event).await.unwrap();
        let reused = repo.reuse(&id).await.unwrap();

        assert_eq!(reused.name, "原事件");
        assert_ne!(reused.id, id); // ID 应该不同
    }

    #[tokio::test]
    async fn test_find_recent() {
        let repo = EventRepository::new();

        // 存储多个事件
        for i in 0..5 {
            let event = SuccessEvent::new(
                format!("事件{}", i),
                "test".to_string(),
                vec!["步骤".to_string()],
                "条件".to_string(),
                "标准".to_string(),
                "位置".to_string(),
                "主体".to_string(),
            );
            repo.save(event).await.unwrap();
        }

        let recent = repo.find_recent(3).await.unwrap();
        assert_eq!(recent.len(), 3);
    }
}
