//! MCP模块集成测试
//!
//! 测试设备管理、心跳监控、离线缓存之间的协作

use fos_mcp::{
    CacheItem, Device, DeviceStatus, DeviceType, EnhancedDeviceManager, HeartbeatConfig,
    HeartbeatManager, HeartbeatStatus, OfflineCacheManager,
};
use std::time::Duration;
use tokio::time::sleep;

/// 测试设备注册 → 心跳监控 集成
/// 验证设备注册后心跳监控正常工作
#[tokio::test]
async fn test_device_registration_heartbeat_integration() {
    // 创建设备管理器
    let device_manager = EnhancedDeviceManager::new();

    // 创建心跳管理器
    let heartbeat_manager = HeartbeatManager::new();

    // 注册设备
    let device_id = uuid::Uuid::new_v4().to_string();
    let device = Device {
        id: device_id.clone(),
        name: "test-device".to_string(),
        device_type: DeviceType::Sensor,
        status: DeviceStatus::Online,
        capabilities: vec!["temperature".to_string()],
        config: serde_json::json!({}),
    };
    device_manager.register_device(device).await.unwrap();

    // 心跳管理器注册
    heartbeat_manager.register_device(device_id.clone()).await;

    // 发送心跳
    heartbeat_manager.receive_heartbeat(&device_id, 100).await.unwrap();

    // 验证设备在心跳管理器中
    let registered = heartbeat_manager.is_registered(&device_id).await;
    assert!(registered);

    // 验证心跳状态
    let status = heartbeat_manager.check_device_status(&device_id).await.unwrap();
    assert_eq!(status, HeartbeatStatus::Healthy);

    // 注销设备
    device_manager.unregister_device(&device_id).await.unwrap();
    heartbeat_manager.unregister_device(&device_id).await;
}

/// 测试心跳超时 → 离线缓存触发
/// 验证设备离线时自动触发缓存
#[tokio::test]
async fn test_heartbeat_timeout_offline_cache_integration() {
    // 创建设备管理器
    let device_manager = EnhancedDeviceManager::new();

    // 创建心跳管理器（短超时）
    let heartbeat_manager = HeartbeatManager::with_config(HeartbeatConfig {
        interval_secs: 1,
        timeout_secs: 2,
        history_limit: 10,
    });
    use fos_mcp::CacheItemStatus;

    // 创建离线缓存管理器
    let cache_manager = OfflineCacheManager::new();

    // 注册设备
    let device_id = uuid::Uuid::new_v4().to_string();
    let device = Device {
        id: device_id.clone(),
        name: "test-device".to_string(),
        device_type: DeviceType::Sensor,
        status: DeviceStatus::Online,
        capabilities: vec!["temperature".to_string()],
        config: serde_json::json!({}),
    };
    device_manager.register_device(device).await.unwrap();

    // 心跳管理器注册
    heartbeat_manager.register_device(device_id.clone()).await;

    // 发送初始心跳
    heartbeat_manager.receive_heartbeat(&device_id, 100).await.unwrap();

    // 等待超时
    sleep(Duration::from_secs(3)).await;

    // 检查心跳状态
    let status = heartbeat_manager.check_device_status(&device_id).await.unwrap();
    assert_eq!(status, HeartbeatStatus::Timeout, "设备应该超时");

    // 模拟离线数据缓存
    let offline_data = vec![1, 2, 3, 4, 5];
    let cache_item = CacheItem::new(device_id.clone(), "sensor-data".to_string(), offline_data);
    cache_manager.add(cache_item).await.unwrap();

    // 验证缓存添加成功
    let cached_items = cache_manager.get_by_device(&device_id).await;
    assert_eq!(cached_items.len(), 1, "应该有一个离线缓存项");
}

/// 测试离线缓存 → 设备重连 → 数据同步
/// 验证完整的离线-重连工作流
#[tokio::test]
async fn test_offline_cache_device_reconnect_workflow() {
    // 创建各管理器
    let device_manager = EnhancedDeviceManager::new();
    let heartbeat_manager = HeartbeatManager::new();
    let cache_manager = OfflineCacheManager::new();
    use fos_mcp::CacheItemStatus;
    use fos_mcp::CachePriority;

    // 注册设备
    let device_id = uuid::Uuid::new_v4().to_string();
    let device = Device {
        id: device_id.clone(),
        name: "test-device".to_string(),
        device_type: DeviceType::Sensor,
        status: DeviceStatus::Online,
        capabilities: vec!["temperature".to_string()],
        config: serde_json::json!({}),
    };
    device_manager.register_device(device).await.unwrap();
    heartbeat_manager.register_device(device_id.clone()).await;

    // 阶段1: 设备上线，发送心跳
    heartbeat_manager.receive_heartbeat(&device_id, 100).await.unwrap();

    let status = heartbeat_manager.check_device_status(&device_id).await.unwrap();
    assert_eq!(status, HeartbeatStatus::Healthy);

    // 阶段2: 设备离线，缓存数据
    let data1 = vec![10, 20, 30];
    let mut item1 = CacheItem::new(device_id.clone(), "sensor-data-1".to_string(), data1);
    item1.priority = CachePriority::High;
    let id1 = cache_manager.add(item1).await.unwrap();

    let data2 = vec![40, 50, 60];
    let item2 = CacheItem::new(device_id.clone(), "sensor-data-2".to_string(), data2);
    let id2 = cache_manager.add(item2).await.unwrap();

    // 验证缓存
    let pending = cache_manager.get_pending(100).await;
    assert_eq!(pending.len(), 2);

    // 阶段3: 设备重新上线
    heartbeat_manager.receive_heartbeat(&device_id, 105).await.unwrap();

    // 阶段4: 同步数据
    cache_manager.mark_syncing(&id1).await.unwrap();
    cache_manager.mark_synced(&id1).await.unwrap();

    cache_manager.mark_syncing(&id2).await.unwrap();
    cache_manager.mark_synced(&id2).await.unwrap();

    // 验证同步完成
    let synced = cache_manager.get_by_device(&device_id).await;
    assert_eq!(synced.len(), 2);
    assert!(synced.iter().all(|item| item.status == CacheItemStatus::Synced));

    // 阶段5: 清理已同步项
    let count = cache_manager.cleanup_synced().await.unwrap();
    assert_eq!(count, 2);

    // 验证缓存已清理
    let cached = cache_manager.get_by_device(&device_id).await;
    assert_eq!(cached.len(), 0);
}

/// 测试设备连接状态管理
#[tokio::test]
async fn test_device_connection_management() {
    let device_manager = EnhancedDeviceManager::new();
    let heartbeat_manager = HeartbeatManager::new();

    // 注册设备
    let device_id = uuid::Uuid::new_v4().to_string();
    let device = Device {
        id: device_id.clone(),
        name: "test-device".to_string(),
        device_type: DeviceType::Actuator,
        status: DeviceStatus::Online,
        capabilities: vec!["switch".to_string()],
        config: serde_json::json!({}),
    };
    device_manager.register_device(device).await.unwrap();
    heartbeat_manager.register_device(device_id.clone()).await;

    // 建立连接
    let connection_id = device_manager
        .connect(device_id.clone(), "192.168.1.100:8080".to_string(), "tcp".to_string())
        .await
        .unwrap();

    // 发送心跳
    heartbeat_manager.receive_heartbeat(&device_id, 100).await.unwrap();

    // 验证连接状态
    let connection = device_manager.get_connection(&connection_id).await.expect("连接应该存在");
    assert!(connection.is_active);

    // 验证心跳状态
    let status = heartbeat_manager.check_device_status(&device_id).await.unwrap();
    assert_eq!(status, HeartbeatStatus::Healthy);

    // 断开连接
    device_manager.disconnect(&connection_id).await.unwrap();
}

/// 测试多设备并发心跳和缓存
#[tokio::test]
async fn test_multiple_devices_concurrent_heartbeat_cache() {
    let device_manager = EnhancedDeviceManager::new();
    let heartbeat_manager = HeartbeatManager::new();
    let cache_manager = OfflineCacheManager::new();

    // 创建多个设备
    let mut device_ids = Vec::new();
    for i in 0..3 {
        let device_id = uuid::Uuid::new_v4().to_string();
        let device = Device {
            id: device_id.clone(),
            name: format!("test-device-{}", i),
            device_type: if i % 2 == 0 { DeviceType::Sensor } else { DeviceType::Actuator },
            status: DeviceStatus::Online,
            capabilities: vec![],
            config: serde_json::json!({}),
        };
        device_manager.register_device(device).await.unwrap();
        heartbeat_manager.register_device(device_id.clone()).await;
        device_ids.push(device_id);
    }

    // 所有设备发送心跳
    for id in &device_ids {
        heartbeat_manager.receive_heartbeat(id, 100).await.unwrap();
    }

    // 验证所有设备状态健康
    for id in &device_ids {
        let status = heartbeat_manager.check_device_status(id).await.unwrap();
        assert_eq!(status, HeartbeatStatus::Healthy);
    }

    // 为每个设备添加缓存
    for (i, id) in device_ids.iter().enumerate() {
        let data = vec![i as u8 * 10; 5];
        let item = CacheItem::new(id.clone(), "sensor-data".to_string(), data);
        cache_manager.add(item).await.unwrap();
    }

    // 验证所有缓存
    let pending = cache_manager.get_pending(100).await;
    assert_eq!(pending.len(), 3);

    // 按设备分类统计
    for id in &device_ids {
        let items = cache_manager.get_by_device(id).await;
        assert_eq!(items.len(), 1);
    }

    // 清理所有设备
    for id in device_ids {
        device_manager.unregister_device(&id).await.unwrap();
        heartbeat_manager.unregister_device(&id).await;
    }
}

/// 测试设备统计信息集成
#[tokio::test]
async fn test_statistics_integration() {
    let device_manager = EnhancedDeviceManager::new();
    let heartbeat_manager = HeartbeatManager::new();
    let cache_manager = OfflineCacheManager::new();

    // 初始统计
    let device_stats = device_manager.get_stats().await;
    assert_eq!(device_stats.total, 0);

    let cache_stats = cache_manager.get_stats().await;
    assert_eq!(cache_stats.total_items, 0);

    // 注册设备
    let device_id = uuid::Uuid::new_v4().to_string();
    let device = Device {
        id: device_id.clone(),
        name: "test-device".to_string(),
        device_type: DeviceType::Sensor,
        status: DeviceStatus::Online,
        capabilities: vec![],
        config: serde_json::json!({}),
    };
    device_manager.register_device(device).await.unwrap();
    heartbeat_manager.register_device(device_id.clone()).await;

    // 统计更新
    let device_stats = device_manager.get_stats().await;
    assert_eq!(device_stats.total, 1);

    // 添加缓存
    let item = CacheItem::new(device_id.clone(), "sensor-data".to_string(), vec![1, 2, 3]);
    cache_manager.add(item).await.unwrap();

    // 缓存统计更新
    let cache_stats = cache_manager.get_stats().await;
    assert_eq!(cache_stats.total_items, 1);
    assert_eq!(cache_stats.pending_items, 1);

    // 标记已同步并清理
    let synced_items = cache_manager.get_by_device(&device_id).await;
    if let Some(item) = synced_items.first() {
        cache_manager.mark_synced(&item.id).await.unwrap();
        cache_manager.cleanup_synced().await.unwrap();
    }

    // 缓存统计清理后
    let cache_stats = cache_manager.get_stats().await;
    assert_eq!(cache_stats.total_items, 0);

    // 清理
    device_manager.unregister_device(&device_id).await.unwrap();
    heartbeat_manager.unregister_device(&device_id).await;
}

/// 测试端到端工作流
/// 完整模拟：设备注册 → 连接 → 心跳 → 离线 → 缓存 → 重连 → 同步 → 清理
#[tokio::test]
async fn test_end_to_end_workflow() {
    // 初始化所有管理器
    let device_manager = EnhancedDeviceManager::new();
    let heartbeat_manager = HeartbeatManager::new();
    let cache_manager = OfflineCacheManager::new();
    use fos_mcp::CacheItemStatus;

    // 1. 设备注册
    let device_id = uuid::Uuid::new_v4().to_string();
    let device = Device {
        id: device_id.clone(),
        name: "sensor-001".to_string(),
        device_type: DeviceType::Sensor,
        status: DeviceStatus::Online,
        capabilities: vec!["temperature".to_string()],
        config: serde_json::json!({}),
    };
    device_manager.register_device(device.clone()).await.unwrap();
    heartbeat_manager.register_device(device_id.clone()).await;

    let device_stats = device_manager.get_stats().await;
    assert_eq!(device_stats.total, 1);

    // 2. 建立连接
    let connection_id = device_manager
        .connect(device_id.clone(), "192.168.1.100:8080".to_string(), "tcp".to_string())
        .await
        .unwrap();

    // 3. 心跳监控
    heartbeat_manager.receive_heartbeat(&device_id, 100).await.unwrap();

    let status = heartbeat_manager.check_device_status(&device_id).await.unwrap();
    assert_eq!(status, HeartbeatStatus::Healthy);

    // 4. 采集数据并缓存（模拟设备离线前）
    let item1 = CacheItem::new(device_id.clone(), "temperature".to_string(), vec![22, 23, 24]);
    let id1 = cache_manager.add(item1).await.unwrap();

    let item2 = CacheItem::new(device_id.clone(), "humidity".to_string(), vec![45, 46, 47]);
    let id2 = cache_manager.add(item2).await.unwrap();

    let cache_stats = cache_manager.get_stats().await;
    assert_eq!(cache_stats.total_items, 2);
    assert_eq!(cache_stats.pending_items, 2);

    // 5. 设备离线（心跳超时，通过模拟长时间间隔）
    sleep(Duration::from_secs(3)).await;

    let status = heartbeat_manager.check_device_status(&device_id).await.unwrap();
    assert_eq!(status, HeartbeatStatus::Timeout);

    // 6. 设备重连
    heartbeat_manager.receive_heartbeat(&device_id, 105).await.unwrap();

    let status = heartbeat_manager.check_device_status(&device_id).await.unwrap();
    assert_eq!(status, HeartbeatStatus::Healthy);

    // 7. 同步离线数据
    cache_manager.mark_syncing(&id1).await.unwrap();
    cache_manager.mark_synced(&id1).await.unwrap();

    cache_manager.mark_syncing(&id2).await.unwrap();
    cache_manager.mark_synced(&id2).await.unwrap();

    // 8. 清理已同步数据
    let count = cache_manager.cleanup_synced().await.unwrap();
    assert_eq!(count, 2);

    let cache_stats = cache_manager.get_stats().await;
    assert_eq!(cache_stats.total_items, 0);

    // 9. 断开连接并注销设备
    device_manager.disconnect(&connection_id).await.unwrap();
    device_manager.unregister_device(&device_id).await.unwrap();
    heartbeat_manager.unregister_device(&device_id).await;

    let device_stats = device_manager.get_stats().await;
    assert_eq!(device_stats.total, 0);

    // 工作流完成
}
