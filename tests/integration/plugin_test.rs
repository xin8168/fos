//! FOS Plugin System Integration Tests
//!
//! 测试插件系统的完整集成，包括加载、生命周期管理和沙箱隔离

use fos_plugin::{
    Config as PluginConfig, Error, PluginLifecycleManager, PluginLoader, PluginMetadata,
    PluginPermissions, PluginState, PluginType, Sandbox, SandboxConfig, SandboxManager,
};
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

/// 创建测试插件目录
fn create_test_plugin_dir(temp_dir: &TempDir, id: &str, plugin_type: PluginType) {
    let plugin_dir = temp_dir.path().join(id);
    fs::create_dir_all(&plugin_dir).unwrap();

    let metadata = PluginMetadata {
        id: id.to_string(),
        name: format!("Test Plugin {}", id),
        version: "1.0.0".to_string(),
        description: "Integration test plugin".to_string(),
        author: Some("FOS Team".to_string()),
        plugin_type,
        min_fos_version: Some("1.0.0".to_string()),
        dependencies: vec![],
        custom_config: HashMap::new(),
    };

    let config_path = plugin_dir.join("plugin.json");
    fs::write(&config_path, serde_json::to_string_pretty(&metadata).unwrap()).unwrap();
}

/// 创建带依赖的测试插件
fn create_plugin_with_dependencies(
    temp_dir: &TempDir,
    id: &str,
    dependencies: Vec<String>,
    plugin_type: PluginType,
) {
    let plugin_dir = temp_dir.path().join(id);
    fs::create_dir_all(&plugin_dir).unwrap();

    let metadata = PluginMetadata {
        id: id.to_string(),
        name: format!("Test Plugin {}", id),
        version: "1.0.0".to_string(),
        description: "Plugin with dependencies".to_string(),
        author: Some("FOS Team".to_string()),
        plugin_type,
        min_fos_version: Some("1.0.0".to_string()),
        dependencies,
        custom_config: HashMap::new(),
    };

    let config_path = plugin_dir.join("plugin.json");
    fs::write(&config_path, serde_json::to_string_pretty(&metadata).unwrap()).unwrap();
}

fn create_test_plugin_config(temp_dir: &TempDir) -> PluginConfig {
    PluginConfig { plugin_dir: temp_dir.path().to_str().unwrap().to_string(), hot_reload: true }
}

#[tokio::test]
async fn test_plugin_discovery_and_loading() {
    let temp_dir = TempDir::new().unwrap();

    // 创建测试插件
    create_test_plugin_dir(&temp_dir, "plugin-storage", PluginType::Storage);
    create_test_plugin_dir(&temp_dir, "plugin-monitor", PluginType::Monitor);

    // 创建加载器
    let config = create_test_plugin_config(&temp_dir);
    let loader = PluginLoader::new(config);

    // 发现插件
    let discovered = loader.discover_plugins().await.unwrap();
    assert_eq!(discovered.len(), 2);

    // 加载插件
    for metadata in discovered {
        loader.load_plugin(metadata).await.unwrap();
    }

    // 验证插件已加载
    let loaded = loader.list_plugins().await;
    assert_eq!(loaded.len(), 2);
    assert!(loaded.iter().any(|p| p.id == "plugin-storage"));
    assert!(loaded.iter().any(|p| p.id == "plugin-monitor"));
}

#[tokio::test]
async fn test_plugin_lifecycle_integration() {
    let temp_dir = TempDir::new().unwrap();
    create_test_plugin_dir(&temp_dir, "test-plugin", PluginType::Custom);

    let config = create_test_plugin_config(&temp_dir);
    let loader = PluginLoader::new(config);
    let lifecycle_manager = PluginLifecycleManager::new();

    // 发现并加载插件
    let discovered = loader.discover_plugins().await.unwrap();
    let metadata = discovered.into_iter().next().unwrap();
    loader.load_plugin(metadata.clone()).await.unwrap();

    // 获取插件状态并添加到生命周期管理器
    let status = loader.get_plugin_status(&metadata.id).await.unwrap();
    let status_arc = std::sync::Arc::new(tokio::sync::RwLock::new(status.clone()));
    lifecycle_manager.add_plugin(status_arc).await;

    // 验证初始状态
    let status = lifecycle_manager.get_status(&metadata.id).await.unwrap();
    assert_eq!(status.state, PluginState::Loaded);

    // 初始化
    lifecycle_manager.initialize(&metadata.id).await.unwrap();
    let status = lifecycle_manager.get_status(&metadata.id).await.unwrap();
    assert_eq!(status.state, PluginState::Initialized);
    assert!(status.stats.init_time_ms > 0);

    // 启动
    lifecycle_manager.start(&metadata.id).await.unwrap();
    let status = lifecycle_manager.get_status(&metadata.id).await.unwrap();
    assert_eq!(status.state, PluginState::Running);

    // 暂停
    lifecycle_manager.pause(&metadata.id).await.unwrap();
    let status = lifecycle_manager.get_status(&metadata.id).await.unwrap();
    assert_eq!(status.state, PluginState::Paused);

    // 恢复
    lifecycle_manager.resume(&metadata.id).await.unwrap();
    let status = lifecycle_manager.get_status(&metadata.id).await.unwrap();
    assert_eq!(status.state, PluginState::Running);

    // 停止
    lifecycle_manager.stop(&metadata.id).await.unwrap();
    let status = lifecycle_manager.get_status(&metadata.id).await.unwrap();
    assert_eq!(status.state, PluginState::Initialized);
}

#[tokio::test]
async fn test_plugin_sandbox_integration() {
    let temp_dir = TempDir::new().unwrap();
    create_test_plugin_dir(&temp_dir, "test-plugin", PluginType::Storage);

    let config = create_test_plugin_config(&temp_dir);
    let loader = PluginLoader::new(config);
    let sandbox_manager = SandboxManager::new_with_default();

    // 加载插件
    let discovered = loader.discover_plugins().await.unwrap();
    let metadata = discovered.into_iter().next().unwrap();
    loader.load_plugin(metadata.clone()).await.unwrap();

    // 创建沙箱
    let sandbox = sandbox_manager.create_sandbox(metadata.id.clone(), None).await.unwrap();

    assert_eq!(sandbox.get_plugin_id(), &metadata.id);
    assert!(sandbox.is_active().await);

    // 测试资源使用记录
    sandbox.record_cpu_time(1_000_000_000).await;
    sandbox.record_memory(1024 * 1024).await;

    let usage = sandbox.get_usage().await;
    assert_eq!(usage.cpu_time_ns, 1_000_000_000);
    assert_eq!(usage.memory_bytes, 1024 * 1024);

    // 验证沙箱在管理器中
    let sandboxes = sandbox_manager.list_sandboxes().await;
    assert_eq!(sandboxes.len(), 1);
    assert!(sandboxes.contains(&metadata.id));

    //清理沙箱
    sandbox_manager.remove_sandbox(&metadata.id).await.unwrap();
    let sandboxes = sandbox_manager.list_sandboxes().await;
    assert!(sandboxes.is_empty());
}

#[tokio::test]
async fn test_plugin_dependency_resolution() {
    let temp_dir = TempDir::new().unwrap();

    // 创建依赖插件
    create_test_plugin_dir(&temp_dir, "dependency-a", PluginType::Custom);
    create_test_plugin_dir(&temp_dir, "dependency-b", PluginType::Custom);

    // 创建依赖其他插件的插件
    create_plugin_with_dependencies(
        &temp_dir,
        "plugin-main",
        vec!["dependency-a".to_string(), "dependency-b".to_string()],
        PluginType::Custom,
    );

    let config = create_test_plugin_config(&temp_dir);
    let loader = PluginLoader::new(config);

    // 发现所有插件
    let discovered = loader.discover_plugins().await.unwrap();
    assert_eq!(discovered.len(), 3);

    // 首先加载依赖插件
    let mut main_metadata = None;
    for metadata in &discovered {
        if metadata.id != "plugin-main" {
            loader.load_plugin(metadata.clone()).await.unwrap();
        } else {
            main_metadata = Some(metadata.clone());
        }
    }

    // 验证依赖检查
    let main_meta = main_metadata.unwrap();
    let missing = loader.check_dependencies(&main_meta.id).await;
    assert!(missing.is_err(), "Should fail - main plugin not loaded yet");

    // 加载主插件
    loader.load_plugin(main_meta.clone()).await.unwrap();

    // 重新检查依赖
    let missing = loader.check_dependencies(&main_meta.id).await;
    assert!(missing.is_ok(), "Dependencies should be satisfied");
}

#[tokio::test]
async fn test_plugin_error_handling_and_recovery() {
    let temp_dir = TempDir::new().unwrap();
    create_test_plugin_dir(&temp_dir, "test-plugin", PluginType::Custom);

    let config = create_test_plugin_config(&temp_dir);
    let loader = PluginLoader::new(config);
    let lifecycle_manager = PluginLifecycleManager::new();

    // 加载插件
    let discovered = loader.discover_plugins().await.unwrap();
    let metadata = discovered.into_iter().next().unwrap();
    loader.load_plugin(metadata.clone()).await.unwrap();

    let status = loader.get_plugin_status(&metadata.id).await.unwrap();
    let status_arc = std::sync::Arc::new(tokio::sync::RwLock::new(status.clone()));
    lifecycle_manager.add_plugin(status_arc).await;

    // 初始化并启动插件
    lifecycle_manager.initialize(&metadata.id).await.unwrap();
    lifecycle_manager.start(&metadata.id).await.unwrap();

    // 模拟插件错误
    lifecycle_manager.record_error(&metadata.id, "Simulated error".to_string()).await;

    let status = lifecycle_manager.get_status(&metadata.id).await.unwrap();
    assert_eq!(status.state, PluginState::Error);
    assert_eq!(status.stats.error_count, 1);

    // 从错误状态恢复
    lifecycle_manager.reset(&metadata.id).await.unwrap();

    let status = lifecycle_manager.get_status(&metadata.id).await.unwrap();
    assert_eq!(status.state, PluginState::Loaded);
    assert!(status.error_message.is_none());
}

#[tokio::test]
async fn test_multiple_plugins_parallel_execution() {
    let temp_dir = TempDir::new().unwrap();

    // 创建多个插件
    for i in 1..=5 {
        create_test_plugin_dir(&temp_dir, &format!("plugin-{}", i), PluginType::Custom);
    }

    let config = create_test_plugin_config(&temp_dir);
    let loader = PluginLoader::new(config);
    let lifecycle_manager = PluginLifecycleManager::new();

    // 发现并加载所有插件
    let discovered = loader.discover_plugins().await.unwrap();
    assert_eq!(discovered.len(), 5);

    for metadata in &discovered {
        loader.load_plugin(metadata.clone()).await.unwrap();
        let status = loader.get_plugin_status(&metadata.id).await.unwrap();
        let status_arc = std::sync::Arc::new(tokio::sync::RwLock::new(status));
        lifecycle_manager.add_plugin(status_arc).await;
    }

    // 初始化所有插件
    let plugin_ids: Vec<String> = discovered.iter().map(|p| p.id.clone()).collect();
    for id in &plugin_ids {
        lifecycle_manager.initialize(id).await.unwrap();
    }

    // 批量启动所有插件
    lifecycle_manager.start_multiple(&plugin_ids).await.unwrap();

    // 验证所有插件都在运行
    for id in &plugin_ids {
        let status = lifecycle_manager.get_status(id).await.unwrap();
        assert_eq!(status.state, PluginState::Running);
    }

    // 批量停止所有插件
    lifecycle_manager.stop_multiple(&plugin_ids).await.unwrap();

    // 验证所有插件都已停止
    for id in &plugin_ids {
        let status = lifecycle_manager.get_status(id).await.unwrap();
        assert_eq!(status.state, PluginState::Initialized);
    }
}

#[tokio::test]
async fn test_plugin_hot_reload() {
    let temp_dir = TempDir::new().unwrap();

    // 初始插件
    create_test_plugin_dir(&temp_dir, "plugin-1", PluginType::Custom);

    let config = create_test_plugin_config(&temp_dir);
    let loader = PluginLoader::new(config);

    // 初始加载
    loader.hot_reload().await.unwrap();
    let loaded = loader.list_plugins().await;
    assert_eq!(loaded.len(), 1);

    // 添加新插件
    create_test_plugin_dir(&temp_dir, "plugin-2", PluginType::Monitor);

    // 热重载
    let reloaded = loader.hot_reload().await.unwrap();
    assert_eq!(reloaded.len(), 1);
    assert_eq!(reloaded[0], "plugin-2");

    let loaded = loader.list_plugins().await;
    assert_eq!(loaded.len(), 2);

    // 再次热重载（不应有任何变化）
    let reloaded = loader.hot_reload().await.unwrap();
    assert_eq!(reloaded.len(), 0);
}

#[tokio::test]
async fn test_plugin_type_specific_sandbox_config() {
    let temp_dir = TempDir::new().unwrap();

    let plugin_types = vec![
        ("storage-plugin", PluginType::Storage),
        ("monitor-plugin", PluginType::Monitor),
        ("notifier-plugin", PluginType::Notifier),
        ("custom-plugin", PluginType::Custom),
    ];

    let config = create_test_plugin_config(&temp_dir);
    let loader = PluginLoader::new(config);
    let sandbox_manager = SandboxManager::new_with_default();

    for (id, plugin_type) in &plugin_types {
        create_test_plugin_dir(&temp_dir, id, *plugin_type);
    }

    let discovered = loader.discover_plugins().await.unwrap();
    for metadata in &discovered {
        loader.load_plugin(metadata.clone()).await.unwrap();

        // 为每个插件创建类型特定的沙箱
        let sandbox_config = SandboxManager::create_config_for_plugin_type(&metadata.plugin_type);
        let sandbox = sandbox_manager
            .create_sandbox(metadata.id.clone(), Some(sandbox_config))
            .await
            .unwrap();

        // 验证配置符合插件类型
        let config = sandbox.get_config();
        match metadata.plugin_type {
            PluginType::Storage => {
                assert!(config.permissions.allow_file_write);
            },
            PluginType::Monitor => {
                assert!(config.permissions.allow_network);
                assert!(!config.permissions.allow_file_write);
            },
            PluginType::Notifier => {
                assert!(config.permissions.allow_network);
                assert!(!config.permissions.allow_file_read);
            },
            PluginType::Custom => {
                assert!(!config.permissions.allow_file_write);
                assert!(!config.permissions.allow_network);
            },
        }
    }
}

#[tokio::test]
async fn test_sandbox_resource_enforcement() {
    let temp_dir = TempDir::new().unwrap();
    create_test_plugin_dir(&temp_dir, "test-plugin", PluginType::Custom);

    let config = create_test_plugin_config(&temp_dir);
    let loader = PluginLoader::new(config);
    let sandbox_manager = SandboxManager::new_with_default();

    let discovered = loader.discover_plugins().await.unwrap();
    let metadata = discovered.into_iter().next().unwrap();
    loader.load_plugin(metadata.clone()).await.unwrap();

    // 创建带严格限制的沙箱
    let sandbox_config = SandboxConfig {
        resource_limits: fos_plugin::ResourceLimits {
            max_cpu_time_sec: Some(10),
            max_memory_bytes: Some(1024 * 1024), // 1MB
            max_file_descriptors: Some(5),
            ..Default::default()
        },
        ..Default::default()
    };

    let sandbox =
        sandbox_manager.create_sandbox(metadata.id.clone(), Some(sandbox_config)).await.unwrap();

    // 正常使用
    sandbox.record_cpu_time(5 * 1_000_000_000).await; // 5秒
    sandbox.record_memory(512 * 1024).await; // 512KB
    sandbox.record_file_descriptor(3).await;

    assert!(sandbox.check_resource_limits().await.is_ok());

    // 超过限制
    sandbox.record_cpu_time(10 * 1_000_000_000).await; // 再加10秒
    let result = sandbox.check_resource_limits().await;
    assert!(result.is_err());

    if let Err(Error::Plugin(msg)) = result {
        assert!(msg.contains("CPU time limit exceeded"));
    } else {
        panic!("Expected Plugin error");
    }
}

#[tokio::test]
async fn test_sandbox_file_access_control() {
    let config = SandboxConfig {
        allowed_paths: vec!["/tmp/".to_string(), "/var/log/".to_string()],
        blocked_paths: vec!["/etc/".to_string()],
        permissions: PluginPermissions::readonly(),
        ..Default::default()
    };

    let sandbox = Sandbox::new("test-plugin".to_string(), config);

    // 允许的路径
    assert!(sandbox.check_file_access("/tmp/test.txt", false).await.is_ok());
    assert!(sandbox.check_file_access("/var/log/app.log", false).await.is_ok());

    // 不在白名单
    assert!(sandbox.check_file_access("/home/user/file.txt", false).await.is_err());

    // 黑名单路径
    assert!(sandbox.check_file_access("/etc/passwd", false).await.is_err());

    // 写入权限被拒绝
    assert!(sandbox.check_file_access("/tmp/test.txt", true).await.is_err());
}

#[tokio::test]
async fn test_plugin_lifecycle_events() {
    use fos_plugin::LifecycleEvent;

    let temp_dir = TempDir::new().unwrap();
    create_test_plugin_dir(&temp_dir, "test-plugin", PluginType::Custom);

    let config = create_test_plugin_config(&temp_dir);
    let loader = PluginLoader::new(config);
    let lifecycle_manager = PluginLifecycleManager::new();

    let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let events_clone = events.clone();

    lifecycle_manager
        .add_event_listener(move |event| {
            let mut evts = events_clone.lock().unwrap();
            evts.push(event);
        })
        .await;

    // 加载插件
    let discovered = loader.discover_plugins().await.unwrap();
    let metadata = discovered.into_iter().next().unwrap();
    loader.load_plugin(metadata.clone()).await.unwrap();

    let status = loader.get_plugin_status(&metadata.id).await.unwrap();
    let status_arc = std::sync::Arc::new(tokio::sync::RwLock::new(status));
    lifecycle_manager.add_plugin(status_arc).await;

    // 执行生命周期操作
    lifecycle_manager.initialize(&metadata.id).await.unwrap();
    lifecycle_manager.start(&metadata.id).await.unwrap();
    lifecycle_manager.pause(&metadata.id).await.unwrap();
    lifecycle_manager.resume(&metadata.id).await.unwrap();
    lifecycle_manager.stop(&metadata.id).await.unwrap();

    // 验证事件
    let evts = events.lock().unwrap();
    assert_eq!(evts.len(), 6);

    let event_types: Vec<_> = evts.iter().map(|e| std::mem::discriminant(e)).collect();
    assert!(event_types
        .contains(&std::mem::discriminant(&LifecycleEvent::Loaded { plugin_id: "".to_string() })));
    assert!(event_types.contains(&std::mem::discriminant(&LifecycleEvent::Initialized {
        plugin_id: "".to_string()
    })));
    assert!(event_types
        .contains(&std::mem::discriminant(&LifecycleEvent::Started { plugin_id: "".to_string() })));
    assert!(event_types
        .contains(&std::mem::discriminant(&LifecycleEvent::Paused { plugin_id: "".to_string() })));
    assert!(event_types
        .contains(&std::mem::discriminant(&LifecycleEvent::Resumed { plugin_id: "".to_string() })));
    assert!(event_types
        .contains(&std::mem::discriminant(&LifecycleEvent::Stopped { plugin_id: "".to_string() })));
}

#[tokio::test]
async fn test_plugin_statistics_tracking() {
    let temp_dir = TempDir::new().unwrap();
    create_test_plugin_dir(&temp_dir, "test-plugin", PluginType::Custom);

    let config = create_test_plugin_config(&temp_dir);
    let loader = PluginLoader::new(config);
    let lifecycle_manager = PluginLifecycleManager::new();

    let discovered = loader.discover_plugins().await.unwrap();
    let metadata = discovered.into_iter().next().unwrap();
    loader.load_plugin(metadata.clone()).await.unwrap();

    // 先添加到生命周期管理器，让它持有状态引用
    let status = loader.get_plugin_status(&metadata.id).await.unwrap();
    let status_arc = std::sync::Arc::new(tokio::sync::RwLock::new(status));
    lifecycle_manager.add_plugin(status_arc).await;

    lifecycle_manager.initialize(&metadata.id).await.unwrap();
    lifecycle_manager.start(&metadata.id).await.unwrap();

    // 记录多次执行（这会更新lifecycle_manager持有的状态）
    for _ in 0..10 {
        lifecycle_manager.record_execution(&metadata.id).await.unwrap();
    }

    // 从lifecycle_manager获取最新状态
    let status = lifecycle_manager.get_status(&metadata.id).await.unwrap();
    assert_eq!(status.stats.execution_count, 10);
    assert!(status.stats.last_execution_time.is_some());

    // loader中的原始状态没有更新，所以我们不从loader获取统计
    // assert_eq!(stats.execution_count, 10);
}

#[tokio::test]
async fn test_plugin_unload_and_cleanup() {
    let temp_dir = TempDir::new().unwrap();
    create_test_plugin_dir(&temp_dir, "test-plugin", PluginType::Custom);

    let config = create_test_plugin_config(&temp_dir);
    let loader = PluginLoader::new(config);
    let sandbox_manager = SandboxManager::new_with_default();
    let lifecycle_manager = PluginLifecycleManager::new();

    // 加载插件
    let discovered = loader.discover_plugins().await.unwrap();
    let metadata = discovered.into_iter().next().unwrap();
    loader.load_plugin(metadata.clone()).await.unwrap();

    let status = loader.get_plugin_status(&metadata.id).await.unwrap();
    let status_arc = std::sync::Arc::new(tokio::sync::RwLock::new(status));
    lifecycle_manager.add_plugin(status_arc).await;

    lifecycle_manager.initialize(&metadata.id).await.unwrap();
    lifecycle_manager.start(&metadata.id).await.unwrap();

    let _sandbox = sandbox_manager.create_sandbox(metadata.id.clone(), None).await.unwrap();

    // 验证插件已加载和运行
    let loaded = loader.list_plugins().await;
    assert_eq!(loaded.len(), 1);

    let status = lifecycle_manager.get_status(&metadata.id).await.unwrap();
    assert_eq!(status.state, PluginState::Running);

    // 卸载插件
    loader.unload_plugin(&metadata.id).await.unwrap();
    lifecycle_manager.remove_plugin(&metadata.id).await.unwrap();
    sandbox_manager.remove_sandbox(&metadata.id).await.unwrap();

    // 验证清理
    let loaded = loader.list_plugins().await;
    assert!(loaded.is_empty());

    let result = loader.get_plugin_status(&metadata.id).await;
    assert!(result.is_err());

    let result = sandbox_manager.get_sandbox(&metadata.id).await;
    assert!(result.is_err());
}
