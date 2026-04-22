//! FOS Memory 集成测试
//!
//! 测试 Memory 模块与其他模块的集成

use fos_memory::{
    ChangeType, EventQuery, EventRepository, EventVersion, InMemoryStorage, SuccessEvent,
    VersionManager,
};

/// 测试事件存储与仓库集成
#[tokio::test]
async fn test_storage_repository_integration() {
    let repo = EventRepository::new();

    // 创建事件
    let event = SuccessEvent::new(
        "清理桌面".to_string(),
        "device_control".to_string(),
        vec!["列出文件".to_string(), "删除临时文件".to_string()],
        "文件为临时文件".to_string(),
        "桌面干净".to_string(),
        "我的电脑".to_string(),
        "用户".to_string(),
    );

    // 存储事件
    let id = repo.save(event.clone()).await.unwrap();

    // 验证存储
    let found = repo.find_by_id(&id).await.unwrap();
    assert_eq!(found.name, "清理桌面");
    assert_eq!(found.steps.len(), 2);
}

/// 测试事件查询集成
#[tokio::test]
async fn test_query_integration() {
    let repo = EventRepository::new();

    // 存储多个事件
    for i in 0..5 {
        let event = SuccessEvent::new(
            format!("事件{}", i),
            if i % 2 == 0 { "type_a".to_string() } else { "type_b".to_string() },
            vec!["步骤".to_string()],
            "条件".to_string(),
            "标准".to_string(),
            format!("位置{}", i % 2),
            "用户".to_string(),
        );
        repo.save(event).await.unwrap();
    }

    // 按类型查询
    let type_a_events = repo.find_by_type("type_a").await.unwrap();
    assert_eq!(type_a_events.len(), 3);

    let type_b_events = repo.find_by_type("type_b").await.unwrap();
    assert_eq!(type_b_events.len(), 2);
}

/// 测试版本管理与存储集成
#[tokio::test]
async fn test_version_storage_integration() {
    let repo = EventRepository::new();
    let version_manager = VersionManager::new();

    // 创建并存储事件
    let event = SuccessEvent::new(
        "版本测试".to_string(),
        "test".to_string(),
        vec!["步骤1".to_string()],
        "条件".to_string(),
        "标准".to_string(),
        "位置".to_string(),
        "用户".to_string(),
    );
    let id = repo.save(event).await.unwrap();

    // 创建初始版本
    let version = version_manager.create_initial_version(&id, "user-1".to_string()).await.unwrap();
    assert_eq!(version, 1);

    // 创建更新版本
    let version2 = version_manager
        .create_content_update(&id, "更新内容".to_string(), "user-2".to_string())
        .await
        .unwrap();
    assert_eq!(version2, 2);

    // 验证版本历史
    let history = version_manager.get_history(&id).await.unwrap();
    assert_eq!(history.version_count(), 2);
}

/// 测试事件复用功能
#[tokio::test]
async fn test_event_reuse() {
    let repo = EventRepository::new();

    // 创建原始事件
    let original = SuccessEvent::new(
        "原始事件".to_string(),
        "test".to_string(),
        vec!["步骤1".to_string(), "步骤2".to_string()],
        "条件".to_string(),
        "标准".to_string(),
        "位置".to_string(),
        "主体".to_string(),
    )
    .with_metadata(serde_json::json!({"key": "value"}));

    let id = repo.save(original).await.unwrap();

    // 复用事件
    let reused = repo.reuse(&id).await.unwrap();

    // 验证复用
    assert_eq!(reused.name, "原始事件");
    assert_ne!(reused.id, id); // ID 应该不同
    assert_eq!(reused.steps.len(), 2);
}

/// 测试事件删除与版本管理
#[tokio::test]
async fn test_delete_with_version() {
    let repo = EventRepository::new();
    let version_manager = VersionManager::new();

    // 创建事件
    let event = SuccessEvent::new(
        "待删除".to_string(),
        "test".to_string(),
        vec!["步骤".to_string()],
        "条件".to_string(),
        "标准".to_string(),
        "位置".to_string(),
        "主体".to_string(),
    );
    let id = repo.save(event).await.unwrap();

    // 创建版本
    version_manager.create_initial_version(&id, "user".to_string()).await.unwrap();

    // 删除事件
    repo.delete(&id).await.unwrap();

    // 验证事件已删除
    let result = repo.find_by_id(&id).await;
    assert!(result.is_err());

    // 版本历史仍可查询（用于审计）
    let history = version_manager.get_history(&id).await;
    assert!(history.is_ok());
}

/// 测试最近事件查询
#[tokio::test]
async fn test_recent_events() {
    let repo = EventRepository::new();

    // 存储多个事件
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
        repo.save(event).await.unwrap();
    }

    // 获取最近5个
    let recent = repo.find_recent(5).await.unwrap();
    assert_eq!(recent.len(), 5);
}

/// 测试复杂查询
#[tokio::test]
async fn test_complex_query() {
    let storage = InMemoryStorage::new();

    // 存储多个事件
    for i in 0..10 {
        let event = SuccessEvent::new(
            format!("测试事件{}", i),
            if i < 5 { "type_a".to_string() } else { "type_b".to_string() },
            vec!["步骤".to_string()],
            "条件".to_string(),
            "标准".to_string(),
            "位置".to_string(),
            if i % 2 == 0 { "user_a".to_string() } else { "user_b".to_string() },
        );
        storage.store(event).await.unwrap();
    }

    // 复杂查询
    let query = EventQuery {
        event_type: Some("type_a".to_string()),
        subject: Some("user_a".to_string()),
        limit: Some(10),
        ..Default::default()
    };

    let results = storage.query(query).await.unwrap();

    // 应该只有 type_a 且 subject 为 user_a 的事件
    for event in &results {
        assert_eq!(event.event_type, "type_a");
        assert_eq!(event.subject, "user_a");
    }
}

/// 测试版本回滚
#[tokio::test]
async fn test_version_rollback() {
    let version_manager = VersionManager::new();

    // 创建初始版本
    version_manager.create_initial_version("event-001", "user-1".to_string()).await.unwrap();

    // 创建多个更新版本
    version_manager
        .create_content_update("event-001", "更新1".to_string(), "user-2".to_string())
        .await
        .unwrap();
    version_manager
        .create_content_update("event-001", "更新2".to_string(), "user-3".to_string())
        .await
        .unwrap();

    // 当前版本应该是 3
    let current = version_manager.get_current_version("event-001").await.unwrap();
    assert_eq!(current, 3);

    // 回滚到版本 1
    let rollback_version =
        version_manager.create_rollback_version("event-001", 1, "admin".to_string()).await.unwrap();

    // 验证回滚版本
    let version = version_manager.get_version("event-001", rollback_version).await.unwrap();
    assert_eq!(version.change_type, ChangeType::RolledBack);
}

/// 测试版本历史完整性
#[tokio::test]
async fn test_version_history_integrity() {
    let version_manager = VersionManager::new();

    // 创建完整版本链
    version_manager.create_initial_version("event-001", "creator".to_string()).await.unwrap();
    version_manager
        .create_content_update("event-001", "内容更新".to_string(), "editor1".to_string())
        .await
        .unwrap();
    version_manager
        .create_metadata_update("event-001", "元数据更新".to_string(), "editor2".to_string())
        .await
        .unwrap();
    version_manager
        .create_steps_update("event-001", "步骤更新".to_string(), "editor3".to_string())
        .await
        .unwrap();

    // 获取完整历史
    let history = version_manager.get_history("event-001").await.unwrap();

    // 验证完整性
    assert_eq!(history.version_count(), 4);

    // 验证版本顺序
    let versions = &history.versions;
    assert_eq!(versions[0].change_type, ChangeType::Created);
    assert_eq!(versions[1].change_type, ChangeType::ContentUpdated);
    assert_eq!(versions[2].change_type, ChangeType::MetadataUpdated);
    assert_eq!(versions[3].change_type, ChangeType::StepsUpdated);
}

/// 测试多事件版本管理
#[tokio::test]
async fn test_multiple_event_versions() {
    let version_manager = VersionManager::new();

    // 为多个事件创建版本
    for i in 0..5 {
        let event_id = format!("event-multi-{}", i);
        version_manager.create_initial_version(&event_id, "user".to_string()).await.unwrap();
    }

    // 统计
    let count = version_manager.count_histories().await.unwrap();
    assert_eq!(count, 5);

    // 每个事件都有独立的历史（版本号会递增但历史是独立的）
    for i in 0..5 {
        let event_id = format!("event-multi-{}", i);
        let version_count = version_manager.get_version_count(&event_id).await.unwrap();
        assert_eq!(version_count, 1); // 每个事件只有1个版本
    }
}
