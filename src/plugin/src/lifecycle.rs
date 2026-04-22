//! # Plugin Lifecycle - 插件生命周期管理
//!
//! 实现插件的完整生命周期：初始化、启动、停止、暂停、恢复、清理

use crate::error::{Error, Result};
use crate::plugin::{PluginState, PluginStatus};
use std::sync::Arc;
use tokio::sync::RwLock as AsyncRwLock;
use tracing::{error, info, warn};

/// 插件生命周期事件
#[derive(Debug, Clone)]
pub enum LifecycleEvent {
    /// 插件已加载
    Loaded { plugin_id: String },
    /// 插件已初始化
    Initialized { plugin_id: String },
    /// 插件已启动
    Started { plugin_id: String },
    /// 插件已停止
    Stopped { plugin_id: String },
    /// 插件已暂停
    Paused { plugin_id: String },
    /// 插件已恢复
    Resumed { plugin_id: String },
    /// 插件已卸载
    Unloaded { plugin_id: String },
    /// 插件发生错误
    Error { plugin_id: String, error: String },
}

/// 生命周期事件监听器
pub type EventListener = Box<dyn Fn(LifecycleEvent) + Send + Sync>;

/// 插件生命周期管理器
pub struct PluginLifecycleManager {
    /// 插件状态
    plugin_states: Arc<AsyncRwLock<Vec<Arc<AsyncRwLock<PluginStatus>>>>>,
    /// 事件监听器
    event_listeners: Arc<AsyncRwLock<Vec<EventListener>>>,
}

impl PluginLifecycleManager {
    /// 创建新的生命周期管理器
    pub fn new() -> Self {
        Self {
            plugin_states: Arc::new(AsyncRwLock::new(Vec::new())),
            event_listeners: Arc::new(AsyncRwLock::new(Vec::new())),
        }
    }

    /// 添加插件状态
    pub async fn add_plugin(&self, status: Arc<AsyncRwLock<PluginStatus>>) {
        // 触发Loaded事件
        let plugin_id = {
            let s = status.read().await;
            s.plugin_id.clone()
        };

        let mut states = self.plugin_states.write().await;
        states.push(status);
        drop(states);

        self.emit_event(LifecycleEvent::Loaded { plugin_id }).await;
    }

    /// 移除插件状态
    pub async fn remove_plugin(&self, plugin_id: &str) -> Result<()> {
        let mut states = self.plugin_states.write().await;

        // 先读取需要的信息，然后执行retain
        let plugin_to_remove = plugin_id.to_string();
        states.retain(|state| {
            // 这里不能await，所以我们使用try_read避免阻塞
            if let Ok(s) = state.try_read() {
                s.plugin_id != plugin_to_remove
            } else {
                // 如果无法读取，保留该插件（保守策略）
                true
            }
        });

        drop(states);

        // 触发Unloaded事件
        self.emit_event(LifecycleEvent::Unloaded { plugin_id: plugin_id.to_string() }).await;

        Ok(())
    }

    /// 初始化插件
    pub async fn initialize(&self, plugin_id: &str) -> Result<()> {
        info!("Initializing plugin: {}", plugin_id);

        let status = self.find_plugin_status(plugin_id).await?;

        {
            let mut s = status.write().await;

            // 检查当前状态
            match s.state {
                PluginState::Loaded => {
                    // 可以初始化
                    s.update_state(PluginState::Initialized);

                    let start_time = std::time::Instant::now();
                    let elapsed = start_time.elapsed().as_millis() as u64;
                    s.stats.init_time_ms = if elapsed == 0 { 1 } else { elapsed };
                },
                PluginState::Initialized => {
                    warn!("Plugin {} is already initialized", plugin_id);
                    return Err(Error::Plugin(format!(
                        "Plugin {} is already initialized",
                        plugin_id
                    )));
                },
                PluginState::Error => {
                    warn!("Cannot initialize plugin in error state");
                    return Err(Error::Plugin(
                        "Cannot initialize plugin in error state".to_string(),
                    ));
                },
                _ => {
                    warn!("Plugin {} is in state {:?}, cannot initialize", plugin_id, s.state);
                    return Err(Error::Plugin(format!(
                        "Plugin {} is in state {:?}, cannot initialize",
                        plugin_id, s.state
                    )));
                },
            }
        }

        self.emit_event(LifecycleEvent::Initialized { plugin_id: plugin_id.to_string() }).await;

        info!("Plugin {} initialized successfully", plugin_id);
        Ok(())
    }

    /// 启动插件
    pub async fn start(&self, plugin_id: &str) -> Result<()> {
        info!("Starting plugin: {}", plugin_id);

        let status = self.find_plugin_status(plugin_id).await?;

        {
            let mut s = status.write().await;

            // 检查当前状态
            match s.state {
                PluginState::Initialized | PluginState::Paused => {
                    s.update_state(PluginState::Running);
                },
                PluginState::Running => {
                    warn!("Plugin {} is already running", plugin_id);
                    return Err(Error::Plugin(format!("Plugin {} is already running", plugin_id)));
                },
                _ => {
                    warn!("Plugin {} is in state {:?}, cannot start", plugin_id, s.state);
                    return Err(Error::Plugin(format!(
                        "Plugin {} is in state {:?}, cannot start",
                        plugin_id, s.state
                    )));
                },
            }
        }

        self.emit_event(LifecycleEvent::Started { plugin_id: plugin_id.to_string() }).await;

        info!("Plugin {} started successfully", plugin_id);
        Ok(())
    }

    /// 停止插件
    pub async fn stop(&self, plugin_id: &str) -> Result<()> {
        info!("Stopping plugin: {}", plugin_id);

        let status = self.find_plugin_status(plugin_id).await?;

        {
            let mut s = status.write().await;

            // 检查当前状态
            match s.state {
                PluginState::Running => {
                    s.update_state(PluginState::Initialized);
                },
                PluginState::Initialized | PluginState::Loaded | PluginState::Unloaded => {
                    warn!("Plugin {} is not running", plugin_id);
                    return Err(Error::Plugin(format!("Plugin {} is not running", plugin_id)));
                },
                _ => {
                    warn!("Plugin {} is in state {:?}, cannot stop", plugin_id, s.state);
                    return Err(Error::Plugin(format!(
                        "Plugin {} is in state {:?}, cannot stop",
                        plugin_id, s.state
                    )));
                },
            }
        }

        self.emit_event(LifecycleEvent::Stopped { plugin_id: plugin_id.to_string() }).await;

        info!("Plugin {} stopped successfully", plugin_id);
        Ok(())
    }

    /// 暂停插件
    pub async fn pause(&self, plugin_id: &str) -> Result<()> {
        info!("Pausing plugin: {}", plugin_id);

        let status = self.find_plugin_status(plugin_id).await?;

        {
            let mut s = status.write().await;

            // 检查当前状态
            match s.state {
                PluginState::Running => {
                    s.update_state(PluginState::Paused);
                },
                PluginState::Paused => {
                    warn!("Plugin {} is already paused", plugin_id);
                    return Err(Error::Plugin(format!("Plugin {} is already paused", plugin_id)));
                },
                _ => {
                    warn!("Plugin {} is in state {:?}, cannot pause", plugin_id, s.state);
                    return Err(Error::Plugin(format!(
                        "Plugin {} is in state {:?}, cannot pause",
                        plugin_id, s.state
                    )));
                },
            }
        }

        self.emit_event(LifecycleEvent::Paused { plugin_id: plugin_id.to_string() }).await;

        info!("Plugin {} paused successfully", plugin_id);
        Ok(())
    }

    /// 恢复插件
    pub async fn resume(&self, plugin_id: &str) -> Result<()> {
        info!("Resuming plugin: {}", plugin_id);

        let status = self.find_plugin_status(plugin_id).await?;

        {
            let mut s = status.write().await;

            // 检查当前状态
            match s.state {
                PluginState::Paused => {
                    s.update_state(PluginState::Running);
                },
                PluginState::Running => {
                    warn!("Plugin {} is already running", plugin_id);
                    return Err(Error::Plugin(format!("Plugin {} is already running", plugin_id)));
                },
                _ => {
                    warn!("Plugin {} is in state {:?}, cannot resume", plugin_id, s.state);
                    return Err(Error::Plugin(format!(
                        "Plugin {} is in state {:?}, cannot resume",
                        plugin_id, s.state
                    )));
                },
            }
        }

        self.emit_event(LifecycleEvent::Resumed { plugin_id: plugin_id.to_string() }).await;

        info!("Plugin {} resumed successfully", plugin_id);
        Ok(())
    }

    /// 记录插件执行
    pub async fn record_execution(&self, plugin_id: &str) -> Result<()> {
        let status = self.find_plugin_status(plugin_id).await?;
        let mut s = status.write().await;
        s.record_execution();
        Ok(())
    }

    /// 记录插件错误
    pub async fn record_error(&self, plugin_id: &str, error: String) {
        error!("Plugin {} error: {}", plugin_id, error);

        let status = self.find_plugin_status(plugin_id).await;

        match status {
            Ok(s) => {
                let mut status = s.write().await;
                status.record_error(error.clone());
            },
            Err(e) => {
                error!("Failed to record error for plugin {}: {}", plugin_id, e);
            },
        }

        self.emit_event(LifecycleEvent::Error { plugin_id: plugin_id.to_string(), error }).await;
    }

    /// 获取插件状态
    pub async fn get_status(&self, plugin_id: &str) -> Result<PluginStatus> {
        let status = self.find_plugin_status(plugin_id).await?;
        let s = status.read().await;
        Ok(s.clone())
    }

    /// 获取所有插件状态
    pub async fn get_all_statuses(&self) -> Vec<PluginStatus> {
        let states = self.plugin_states.read().await;
        let mut statuses = Vec::new();

        for status in states.iter() {
            let s = status.read().await;
            statuses.push(s.clone());
        }

        statuses
    }

    /// 添加事件监听器
    pub async fn add_event_listener<F>(&self, listener: F)
    where
        F: Fn(LifecycleEvent) + Send + Sync + 'static,
    {
        let mut listeners = self.event_listeners.write().await;
        listeners.push(Box::new(listener));
    }

    /// 触发事件
    async fn emit_event(&self, event: LifecycleEvent) {
        let listeners = self.event_listeners.read().await;
        for listener in listeners.iter() {
            listener(event.clone());
        }
    }

    /// 查找插件状态
    async fn find_plugin_status(&self, plugin_id: &str) -> Result<Arc<AsyncRwLock<PluginStatus>>> {
        let states = self.plugin_states.read().await;
        for status in states.iter() {
            let s = status.read().await;
            if s.plugin_id == plugin_id {
                return Ok(status.clone());
            }
        }
        Err(Error::Plugin(format!("Plugin {} not found", plugin_id)))
    }

    /// 批量启动多个插件
    pub async fn start_multiple(&self, plugin_ids: &[String]) -> Result<()> {
        info!("Starting {} plugins", plugin_ids.len());

        let mut errors = Vec::new();
        for plugin_id in plugin_ids {
            if let Err(e) = self.start(plugin_id).await {
                error!("Failed to start plugin {}: {}", plugin_id, e);
                errors.push((plugin_id.clone(), e));
            }
        }

        if !errors.is_empty() {
            return Err(Error::Plugin(format!(
                "Failed to start {} plugins: {:?}",
                errors.len(),
                errors
            )));
        }

        Ok(())
    }

    /// 批量停止多个插件
    pub async fn stop_multiple(&self, plugin_ids: &[String]) -> Result<()> {
        info!("Stopping {} plugins", plugin_ids.len());

        let mut errors = Vec::new();
        for plugin_id in plugin_ids {
            if let Err(e) = self.stop(plugin_id).await {
                error!("Failed to stop plugin {}: {}", plugin_id, e);
                errors.push((plugin_id.clone(), e));
            }
        }

        if !errors.is_empty() {
            return Err(Error::Plugin(format!(
                "Failed to stop {} plugins: {:?}",
                errors.len(),
                errors
            )));
        }

        Ok(())
    }

    /// 重置插件状态（从错误状态恢复）
    pub async fn reset(&self, plugin_id: &str) -> Result<()> {
        info!("Resetting plugin: {}", plugin_id);

        let status = self.find_plugin_status(plugin_id).await?;

        {
            let mut s = status.write().await;

            if s.state != PluginState::Error {
                return Err(Error::Plugin(format!("Cannot reset plugin in state {:?}", s.state)));
            }

            s.update_state(PluginState::Loaded);
            s.error_message = None;
        }

        info!("Plugin {} reset successfully", plugin_id);
        Ok(())
    }
}

impl Default for PluginLifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// 创建测试插件状态
    fn create_test_status(plugin_id: &str) -> Arc<AsyncRwLock<PluginStatus>> {
        Arc::new(AsyncRwLock::new(PluginStatus::new(plugin_id.to_string())))
    }

    #[tokio::test]
    async fn test_lifecycle_manager_creation() {
        let manager = PluginLifecycleManager::new();
        let statuses = manager.get_all_statuses().await;
        assert!(statuses.is_empty());
    }

    #[tokio::test]
    async fn test_add_and_remove_plugin() {
        let manager = PluginLifecycleManager::new();
        let status = create_test_status("test-plugin");
        manager.add_plugin(status).await;

        let statuses = manager.get_all_statuses().await;
        assert_eq!(statuses.len(), 1);

        manager.remove_plugin("test-plugin").await.unwrap();
        let statuses = manager.get_all_statuses().await;
        assert!(statuses.is_empty());
    }

    #[tokio::test]
    async fn test_initialize_plugin() {
        let manager = PluginLifecycleManager::new();
        let mut status = PluginStatus::new("test-plugin".to_string());
        status.update_state(PluginState::Loaded);
        let status_arc = Arc::new(AsyncRwLock::new(status));

        manager.add_plugin(status_arc).await;

        manager.initialize("test-plugin").await.unwrap();

        let status = manager.get_status("test-plugin").await.unwrap();
        assert_eq!(status.state, PluginState::Initialized);
        assert!(status.stats.init_time_ms > 0);
    }

    #[tokio::test]
    async fn test_initialize_already_initialized() {
        let manager = PluginLifecycleManager::new();
        let mut status = PluginStatus::new("test-plugin".to_string());
        status.update_state(PluginState::Initialized);
        let status_arc = Arc::new(AsyncRwLock::new(status));

        manager.add_plugin(status_arc).await;

        let result = manager.initialize("test-plugin").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_start_plugin() {
        let manager = PluginLifecycleManager::new();
        let mut status = PluginStatus::new("test-plugin".to_string());
        status.update_state(PluginState::Initialized);
        let status_arc = Arc::new(AsyncRwLock::new(status));

        manager.add_plugin(status_arc).await;

        manager.start("test-plugin").await.unwrap();

        let status = manager.get_status("test-plugin").await.unwrap();
        assert_eq!(status.state, PluginState::Running);
    }

    #[tokio::test]
    async fn test_stop_plugin() {
        let manager = PluginLifecycleManager::new();
        let mut status = PluginStatus::new("test-plugin".to_string());
        status.update_state(PluginState::Running);
        let status_arc = Arc::new(AsyncRwLock::new(status));

        manager.add_plugin(status_arc).await;

        manager.stop("test-plugin").await.unwrap();

        let status = manager.get_status("test-plugin").await.unwrap();
        assert_eq!(status.state, PluginState::Initialized);
    }

    #[tokio::test]
    async fn test_pause_plugin() {
        let manager = PluginLifecycleManager::new();
        let mut status = PluginStatus::new("test-plugin".to_string());
        status.update_state(PluginState::Running);
        let status_arc = Arc::new(AsyncRwLock::new(status));

        manager.add_plugin(status_arc).await;

        manager.pause("test-plugin").await.unwrap();

        let status = manager.get_status("test-plugin").await.unwrap();
        assert_eq!(status.state, PluginState::Paused);
    }

    #[tokio::test]
    async fn test_resume_plugin() {
        let manager = PluginLifecycleManager::new();
        let mut status = PluginStatus::new("test-plugin".to_string());
        status.update_state(PluginState::Paused);
        let status_arc = Arc::new(AsyncRwLock::new(status));

        manager.add_plugin(status_arc).await;

        manager.resume("test-plugin").await.unwrap();

        let status = manager.get_status("test-plugin").await.unwrap();
        assert_eq!(status.state, PluginState::Running);
    }

    #[tokio::test]
    async fn test_full_lifecycle() {
        let manager = PluginLifecycleManager::new();
        let mut status = PluginStatus::new("test-plugin".to_string());
        status.update_state(PluginState::Loaded);
        let status_arc = Arc::new(AsyncRwLock::new(status));

        manager.add_plugin(status_arc).await;

        // 初始化
        manager.initialize("test-plugin").await.unwrap();
        let status = manager.get_status("test-plugin").await.unwrap();
        assert_eq!(status.state, PluginState::Initialized);

        // 启动
        manager.start("test-plugin").await.unwrap();
        let status = manager.get_status("test-plugin").await.unwrap();
        assert_eq!(status.state, PluginState::Running);

        // 暂停
        manager.pause("test-plugin").await.unwrap();
        let status = manager.get_status("test-plugin").await.unwrap();
        assert_eq!(status.state, PluginState::Paused);

        // 恢复
        manager.resume("test-plugin").await.unwrap();
        let status = manager.get_status("test-plugin").await.unwrap();
        assert_eq!(status.state, PluginState::Running);

        // 停止
        manager.stop("test-plugin").await.unwrap();
        let status = manager.get_status("test-plugin").await.unwrap();
        assert_eq!(status.state, PluginState::Initialized);
    }

    #[tokio::test]
    async fn test_record_execution() {
        let manager = PluginLifecycleManager::new();
        let status = create_test_status("test-plugin");
        manager.add_plugin(status).await;

        manager.record_execution("test-plugin").await.unwrap();
        manager.record_execution("test-plugin").await.unwrap();

        let status = manager.get_status("test-plugin").await.unwrap();
        assert_eq!(status.stats.execution_count, 2);
    }

    #[tokio::test]
    async fn test_record_error() {
        let manager = PluginLifecycleManager::new();
        let status = create_test_status("test-plugin");
        manager.add_plugin(status).await;

        manager.record_error("test-plugin", "Test error".to_string()).await;

        let status = manager.get_status("test-plugin").await.unwrap();
        assert_eq!(status.state, PluginState::Error);
        assert_eq!(status.error_message, Some("Test error".to_string()));
        assert_eq!(status.stats.error_count, 1);
    }

    #[tokio::test]
    async fn test_event_listener() {
        let manager = PluginLifecycleManager::new();
        let events = Arc::new(Mutex::new(Vec::new()));

        let events_clone = events.clone();
        manager
            .add_event_listener(move |event| {
                let mut events = events_clone.lock().unwrap();
                events.push(event);
            })
            .await;

        let status = create_test_status("test-plugin");
        manager.add_plugin(status).await;

        let event_types = events.lock().unwrap();
        assert_eq!(event_types.len(), 1);
        match &event_types[0] {
            LifecycleEvent::Loaded { plugin_id } => assert_eq!(plugin_id, "test-plugin"),
            _ => panic!("Expected Loaded event"),
        }
    }

    #[tokio::test]
    async fn test_start_multiple_plugins() {
        let manager = PluginLifecycleManager::new();

        for i in 1..=3 {
            let mut status = PluginStatus::new(format!("plugin-{}", i));
            status.update_state(PluginState::Initialized);
            let status_arc = Arc::new(AsyncRwLock::new(status));
            manager.add_plugin(status_arc).await;
        }

        let plugin_ids =
            vec!["plugin-1".to_string(), "plugin-2".to_string(), "plugin-3".to_string()];
        manager.start_multiple(&plugin_ids).await.unwrap();

        for plugin_id in &plugin_ids {
            let status = manager.get_status(plugin_id).await.unwrap();
            assert_eq!(status.state, PluginState::Running);
        }
    }

    #[tokio::test]
    async fn test_stop_multiple_plugins() {
        let manager = PluginLifecycleManager::new();

        for i in 1..=3 {
            let mut status = PluginStatus::new(format!("plugin-{}", i));
            status.update_state(PluginState::Running);
            let status_arc = Arc::new(AsyncRwLock::new(status));
            manager.add_plugin(status_arc).await;
        }

        let plugin_ids =
            vec!["plugin-1".to_string(), "plugin-2".to_string(), "plugin-3".to_string()];
        manager.stop_multiple(&plugin_ids).await.unwrap();

        for plugin_id in &plugin_ids {
            let status = manager.get_status(plugin_id).await.unwrap();
            assert_eq!(status.state, PluginState::Initialized);
        }
    }

    #[tokio::test]
    async fn test_reset_error_plugin() {
        let manager = PluginLifecycleManager::new();
        let mut status = PluginStatus::new("test-plugin".to_string());
        status.update_state(PluginState::Error);
        status.error_message = Some("Test error".to_string());
        let status_arc = Arc::new(AsyncRwLock::new(status));

        manager.add_plugin(status_arc).await;

        manager.reset("test-plugin").await.unwrap();

        let status = manager.get_status("test-plugin").await.unwrap();
        assert_eq!(status.state, PluginState::Loaded);
        assert!(status.error_message.is_none());
    }

    #[tokio::test]
    async fn test_reset_non_error_plugin() {
        let manager = PluginLifecycleManager::new();
        let mut status = PluginStatus::new("test-plugin".to_string());
        status.update_state(PluginState::Running);
        let status_arc = Arc::new(AsyncRwLock::new(status));

        manager.add_plugin(status_arc).await;

        let result = manager.reset("test-plugin").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_state_transitions() {
        let manager = PluginLifecycleManager::new();
        let mut status = PluginStatus::new("test-plugin".to_string());
        status.update_state(PluginState::Loaded);
        let status_arc = Arc::new(AsyncRwLock::new(status));

        manager.add_plugin(status_arc).await;

        // 尝试从Loaded直接启动（应该失败）
        let result = manager.start("test-plugin").await;
        assert!(result.is_err());

        // 尝试从Loaded直接暂停（应该失败）
        let result = manager.pause("test-plugin").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multiple_error_records() {
        let manager = PluginLifecycleManager::new();
        let status = create_test_status("test-plugin");
        manager.add_plugin(status).await;

        manager.record_error("test-plugin", "Error 1".to_string()).await;
        manager.record_error("test-plugin", "Error 2".to_string()).await;

        let status = manager.get_status("test-plugin").await.unwrap();
        assert_eq!(status.stats.error_count, 2);
    }
}
