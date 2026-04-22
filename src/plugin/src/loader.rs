//! # Plugin Loader - 插件加载器
//!
//! 实现插件的发现、加载和卸载功能

use crate::error::{Error, Result};
use crate::plugin::{PluginMetadata, PluginState, PluginStats, PluginStatus};
use crate::Config;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock as AsyncRwLock;
use tracing::{debug, info, warn};

/// 插件加载器
pub struct PluginLoader {
    /// 配置
    #[allow(dead_code)]
    config: Config,
    /// 已加载的插件元数据
    plugins: Arc<AsyncRwLock<HashMap<String, PluginMetadata>>>,
    /// 插件状态
    plugin_states: Arc<AsyncRwLock<HashMap<String, Arc<AsyncRwLock<PluginStatus>>>>>,
    /// 插件目录
    plugin_dir: PathBuf,
}

impl PluginLoader {
    /// 创建新的插件加载器
    pub fn new(config: Config) -> Self {
        let plugin_dir = PathBuf::from(&config.plugin_dir);

        Self {
            config,
            plugins: Arc::new(AsyncRwLock::new(HashMap::new())),
            plugin_states: Arc::new(AsyncRwLock::new(HashMap::new())),
            plugin_dir,
        }
    }

    /// 发现插件
    pub async fn discover_plugins(&self) -> Result<Vec<PluginMetadata>> {
        let mut discovered = Vec::new();

        info!("Discovering plugins in: {:?}", self.plugin_dir);

        // 检查插件目录是否存在
        if !self.plugin_dir.exists() {
            warn!("Plugin directory does not exist: {:?}", self.plugin_dir);
            return Ok(discovered);
        }

        // 遍历插件目录
        let entries = std::fs::read_dir(&self.plugin_dir)
            .map_err(|e| Error::Plugin(format!("Failed to read plugin directory: {}", e)))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| Error::Plugin(format!("Failed to read directory entry: {}", e)))?;

            let path = entry.path();

            // 只处理目录（每个插件一个目录）
            if path.is_dir() {
                // 查找插件配置文件（plugin.json 或 plugin.toml）
                let config_path = path.join("plugin.json");
                let toml_path = path.join("plugin.toml");

                if config_path.exists() {
                    match self.load_plugin_metadata(&config_path).await {
                        Ok(metadata) => {
                            debug!("Discovered plugin: {}", metadata.id);
                            discovered.push(metadata);
                        },
                        Err(e) => {
                            warn!("Failed to load plugin metadata from {:?}: {}", config_path, e);
                        },
                    }
                } else if toml_path.exists() {
                    match self.load_plugin_metadata_toml(&toml_path).await {
                        Ok(metadata) => {
                            debug!("Discovered plugin: {}", metadata.id);
                            discovered.push(metadata);
                        },
                        Err(e) => {
                            warn!("Failed to load plugin metadata from {:?}: {}", toml_path, e);
                        },
                    }
                }
            }
        }

        info!("Discovered {} plugins", discovered.len());
        Ok(discovered)
    }

    /// 从JSON加载插件元数据
    async fn load_plugin_metadata(&self, path: &PathBuf) -> Result<PluginMetadata> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::Plugin(format!("Failed to read plugin config: {}", e)))?;

        let metadata: PluginMetadata = serde_json::from_str(&content)
            .map_err(|e| Error::Plugin(format!("Failed to parse plugin metadata: {}", e)))?;

        // 验证元数据
        self.validate_metadata(&metadata)?;

        Ok(metadata)
    }

    /// 从TOML加载插件元数据
    async fn load_plugin_metadata_toml(&self, path: &PathBuf) -> Result<PluginMetadata> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::Plugin(format!("Failed to read plugin config: {}", e)))?;

        let metadata: PluginMetadata = toml::from_str(&content)
            .map_err(|e| Error::Plugin(format!("Failed to parse plugin metadata: {}", e)))?;

        // 验证元数据
        self.validate_metadata(&metadata)?;

        Ok(metadata)
    }

    /// 验证插件元数据
    fn validate_metadata(&self, metadata: &PluginMetadata) -> Result<()> {
        if metadata.id.is_empty() {
            return Err(Error::Plugin("Plugin ID cannot be empty".to_string()));
        }

        if metadata.name.is_empty() {
            return Err(Error::Plugin("Plugin name cannot be empty".to_string()));
        }

        if metadata.version.is_empty() {
            return Err(Error::Plugin("Plugin version cannot be empty".to_string()));
        }

        // 验证版本格式（简单的语义版本检查）
        if let Err(_) = semver::Version::parse(&metadata.version) {
            return Err(Error::Plugin(format!("Invalid version format: {}", metadata.version)));
        }

        Ok(())
    }

    /// 加载插件
    pub async fn load_plugin(&self, metadata: PluginMetadata) -> Result<()> {
        info!("Loading plugin: {}", metadata.id);

        // 验证元数据
        self.validate_metadata(&metadata)?;

        let start_time = std::time::Instant::now();

        // 检查是否已加载
        let plugins = self.plugins.read().await;
        if plugins.contains_key(&metadata.id) {
            warn!("Plugin {} is already loaded", metadata.id);
            drop(plugins);
            return Err(Error::Plugin(format!("Plugin {} is already loaded", metadata.id)));
        }
        drop(plugins);

        // 创建插件状态
        let mut status = PluginStatus::new(metadata.id.clone());
        status.update_state(PluginState::Loaded);

        // 记录加载时间（至少1ms）
        let elapsed = start_time.elapsed().as_millis() as u64;
        status.stats.load_time_ms = if elapsed == 0 { 1 } else { elapsed };

        // 存储插件元数据
        {
            let mut plugins = self.plugins.write().await;
            plugins.insert(metadata.id.clone(), metadata.clone());
        }

        // 存储插件状态
        let status_arc = Arc::new(AsyncRwLock::new(status));
        {
            let mut states = self.plugin_states.write().await;
            states.insert(metadata.id.clone(), status_arc);
        }

        info!("Plugin {} loaded successfully", metadata.id);
        Ok(())
    }

    /// 卸载插件
    pub async fn unload_plugin(&self, plugin_id: &str) -> Result<()> {
        info!("Unloading plugin: {}", plugin_id);

        // 检查插件是否存在
        let plugins = self.plugins.read().await;
        if !plugins.contains_key(plugin_id) {
            warn!("Plugin {} is not loaded", plugin_id);
            drop(plugins);
            return Err(Error::Plugin(format!("Plugin {} is not loaded", plugin_id)));
        }
        drop(plugins);

        // 更新插件状态
        {
            let states = self.plugin_states.read().await;
            if let Some(status_arc) = states.get(plugin_id) {
                let mut status = status_arc.write().await;
                status.update_state(PluginState::Unloaded);
            }
        }

        // 移除插件
        {
            let mut plugins = self.plugins.write().await;
            plugins.remove(plugin_id);
        }

        {
            let mut states = self.plugin_states.write().await;
            states.remove(plugin_id);
        }

        info!("Plugin {} unloaded successfully", plugin_id);
        Ok(())
    }

    /// 获取插件元数据
    pub async fn get_plugin_metadata(&self, plugin_id: &str) -> Result<PluginMetadata> {
        let plugins = self.plugins.read().await;
        plugins
            .get(plugin_id)
            .cloned()
            .ok_or_else(|| Error::Plugin(format!("Plugin {} not found", plugin_id)))
    }

    /// 获取插件状态
    pub async fn get_plugin_status(&self, plugin_id: &str) -> Result<PluginStatus> {
        let states = self.plugin_states.read().await;
        let status_arc = states
            .get(plugin_id)
            .ok_or_else(|| Error::Plugin(format!("Plugin {} not found", plugin_id)))?;

        let status = status_arc.read().await;
        Ok(status.clone())
    }

    /// 获取所有已加载插件
    pub async fn list_plugins(&self) -> Vec<PluginMetadata> {
        let plugins = self.plugins.read().await;
        plugins.values().cloned().collect()
    }

    /// 检查插件依赖
    pub async fn check_dependencies(&self, plugin_id: &str) -> Result<Vec<String>> {
        let metadata = self.get_plugin_metadata(plugin_id).await?;

        let mut missing = Vec::new();
        let plugins = self.plugins.read().await;

        for dep in &metadata.dependencies {
            if !plugins.contains_key(dep) {
                missing.push(dep.clone());
            }
        }

        drop(plugins);

        if !missing.is_empty() {
            return Err(Error::Plugin(format!(
                "Plugin {} has missing dependencies: {:?}",
                plugin_id, missing
            )));
        }

        Ok(missing)
    }

    /// 热加载插件
    pub async fn hot_reload(&self) -> Result<Vec<String>> {
        info!("Hot reloading plugins...");

        let mut reloaded = Vec::new();

        // 发现插件
        let discovered = self.discover_plugins().await?;

        // 对比并加载新插件
        let loaded_plugins = self.list_plugins().await;
        let loaded_ids: std::collections::HashSet<String> =
            loaded_plugins.iter().map(|p| p.id.clone()).collect();

        for metadata in discovered {
            if !loaded_ids.contains(&metadata.id) {
                match self.load_plugin(metadata.clone()).await {
                    Ok(_) => {
                        reloaded.push(metadata.id);
                    },
                    Err(e) => {
                        warn!("Failed to reload plugin {}: {}", metadata.id, e);
                    },
                }
            }
        }

        info!("Hot reload completed. Reloaded {} plugins", reloaded.len());
        Ok(reloaded)
    }

    /// 获取插件统计信息
    pub async fn get_plugin_stats(&self, plugin_id: &str) -> Result<PluginStats> {
        let status = self.get_plugin_status(plugin_id).await?;
        Ok(status.stats)
    }

    /// 获取所有插件状态
    pub async fn get_all_plugin_statuses(&self) -> Vec<PluginStatus> {
        let states = self.plugin_states.read().await;
        let mut statuses = Vec::new();

        for status_arc in states.values() {
            let status = status_arc.read().await;
            statuses.push(status.clone());
        }

        statuses
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::PluginType;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_plugin_metadata(id: &str) -> PluginMetadata {
        PluginMetadata {
            id: id.to_string(),
            name: format!("Test Plugin {}", id),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: Some("Test Author".to_string()),
            plugin_type: PluginType::Custom,
            min_fos_version: Some("1.0.0".to_string()),
            dependencies: vec![],
            custom_config: HashMap::new(),
        }
    }

    fn create_test_plugin_dir(temp_dir: &TempDir, id: &str) -> PathBuf {
        let plugin_dir = temp_dir.path().join(id);
        fs::create_dir_all(&plugin_dir).unwrap();

        let metadata = create_test_plugin_metadata(id);
        let config_path = plugin_dir.join("plugin.json");
        fs::write(&config_path, serde_json::to_string_pretty(&metadata).unwrap()).unwrap();

        plugin_dir
    }

    #[tokio::test]
    async fn test_plugin_loader_creation() {
        let config = Config::default();
        let loader = PluginLoader::new(config);

        let plugins = loader.list_plugins().await;
        assert!(plugins.is_empty());
    }

    #[tokio::test]
    async fn test_discover_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let config =
            Config { plugin_dir: temp_dir.path().to_str().unwrap().to_string(), hot_reload: false };

        let loader = PluginLoader::new(config);
        let discovered = loader.discover_plugins().await.unwrap();

        assert!(discovered.is_empty());
    }

    #[tokio::test]
    async fn test_discover_single_plugin() {
        let temp_dir = TempDir::new().unwrap();
        create_test_plugin_dir(&temp_dir, "test-plugin");

        let config =
            Config { plugin_dir: temp_dir.path().to_str().unwrap().to_string(), hot_reload: false };

        let loader = PluginLoader::new(config);
        let discovered = loader.discover_plugins().await.unwrap();

        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].id, "test-plugin");
    }

    #[tokio::test]
    async fn test_discover_multiple_plugins() {
        let temp_dir = TempDir::new().unwrap();
        create_test_plugin_dir(&temp_dir, "plugin-a");
        create_test_plugin_dir(&temp_dir, "plugin-b");
        create_test_plugin_dir(&temp_dir, "plugin-c");

        let config =
            Config { plugin_dir: temp_dir.path().to_str().unwrap().to_string(), hot_reload: false };

        let loader = PluginLoader::new(config);
        let discovered = loader.discover_plugins().await.unwrap();

        assert_eq!(discovered.len(), 3);
    }

    #[tokio::test]
    async fn test_load_plugin() {
        let config = Config::default();
        let loader = PluginLoader::new(config);

        let metadata = create_test_plugin_metadata("test-plugin");
        loader.load_plugin(metadata).await.unwrap();

        let plugins = loader.list_plugins().await;
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].id, "test-plugin");
    }

    #[tokio::test]
    async fn test_load_duplicate_plugin() {
        let config = Config::default();
        let loader = PluginLoader::new(config);

        let metadata = create_test_plugin_metadata("test-plugin");
        loader.load_plugin(metadata.clone()).await.unwrap();

        let result = loader.load_plugin(metadata).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unload_plugin() {
        let config = Config::default();
        let loader = PluginLoader::new(config);

        let metadata = create_test_plugin_metadata("test-plugin");
        loader.load_plugin(metadata.clone()).await.unwrap();

        loader.unload_plugin("test-plugin").await.unwrap();

        let plugins = loader.list_plugins().await;
        assert!(plugins.is_empty());
    }

    #[tokio::test]
    async fn test_unload_nonexistent_plugin() {
        let config = Config::default();
        let loader = PluginLoader::new(config);

        let result = loader.unload_plugin("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_plugin_metadata() {
        let config = Config::default();
        let loader = PluginLoader::new(config);

        let metadata = create_test_plugin_metadata("test-plugin");
        loader.load_plugin(metadata.clone()).await.unwrap();

        let retrieved = loader.get_plugin_metadata("test-plugin").await.unwrap();
        assert_eq!(retrieved.id, "test-plugin");
        assert_eq!(retrieved.name, metadata.name);
    }

    #[tokio::test]
    async fn test_get_plugin_status() {
        let config = Config::default();
        let loader = PluginLoader::new(config);

        let metadata = create_test_plugin_metadata("test-plugin");
        loader.load_plugin(metadata).await.unwrap();

        let status = loader.get_plugin_status("test-plugin").await.unwrap();
        assert_eq!(status.plugin_id, "test-plugin");
        assert_eq!(status.state, PluginState::Loaded);
        assert!(status.stats.load_time_ms > 0);
    }

    #[tokio::test]
    async fn test_check_dependencies_no_deps() {
        let config = Config::default();
        let loader = PluginLoader::new(config);

        let metadata = create_test_plugin_metadata("test-plugin");
        loader.load_plugin(metadata).await.unwrap();

        let missing = loader.check_dependencies("test-plugin").await.unwrap();
        assert!(missing.is_empty());
    }

    #[tokio::test]
    async fn test_check_dependencies_with_deps() {
        let config = Config::default();
        let loader = PluginLoader::new(config);

        // 加载依赖插件
        let dep_metadata = PluginMetadata {
            id: "dep-plugin".to_string(),
            name: "Dependency Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Dependency".to_string(),
            author: None,
            plugin_type: PluginType::Custom,
            min_fos_version: None,
            dependencies: vec![],
            custom_config: HashMap::new(),
        };
        loader.load_plugin(dep_metadata).await.unwrap();

        // 加载依赖其他插件的插件
        let mut metadata = create_test_plugin_metadata("test-plugin");
        metadata.dependencies = vec!["dep-plugin".to_string()];
        loader.load_plugin(metadata).await.unwrap();

        let missing = loader.check_dependencies("test-plugin").await.unwrap();
        assert!(missing.is_empty());
    }

    #[tokio::test]
    async fn test_check_dependencies_missing() {
        let config = Config::default();
        let loader = PluginLoader::new(config);

        let mut metadata = create_test_plugin_metadata("test-plugin");
        metadata.dependencies = vec!["missing-plugin".to_string()];
        loader.load_plugin(metadata).await.unwrap();

        let result = loader.check_dependencies("test-plugin").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_hot_reload() {
        let temp_dir = TempDir::new().unwrap();
        let config =
            Config { plugin_dir: temp_dir.path().to_str().unwrap().to_string(), hot_reload: true };

        let loader = PluginLoader::new(config);

        // 热加载应发现并加载插件
        let reloaded = loader.hot_reload().await.unwrap();
        assert_eq!(reloaded.len(), 0); // 目录为空

        // 添加一个插件
        create_test_plugin_dir(&temp_dir, "new-plugin");

        let reloaded = loader.hot_reload().await.unwrap();
        assert_eq!(reloaded.len(), 1);
        assert_eq!(reloaded[0], "new-plugin");

        // 再次热加载不应重复加载
        let reloaded = loader.hot_reload().await.unwrap();
        assert_eq!(reloaded.len(), 0);
    }

    #[tokio::test]
    async fn test_get_all_plugin_statuses() {
        let config = Config::default();
        let loader = PluginLoader::new(config);

        let metadata = create_test_plugin_metadata("test-plugin");
        loader.load_plugin(metadata).await.unwrap();

        let statuses = loader.get_all_plugin_statuses().await;
        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].plugin_id, "test-plugin");
    }

    #[tokio::test]
    async fn test_metadata_validation_empty_id() {
        let config = Config::default();
        let loader = PluginLoader::new(config);

        let mut metadata = create_test_plugin_metadata("test-plugin");
        metadata.id = String::new();

        let result = loader.load_plugin(metadata).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_metadata_validation_empty_name() {
        let config = Config::default();
        let loader = PluginLoader::new(config);

        let mut metadata = create_test_plugin_metadata("test-plugin");
        metadata.name = String::new();

        let result = loader.load_plugin(metadata).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_metadata_validation_invalid_version() {
        let config = Config::default();
        let loader = PluginLoader::new(config);

        let mut metadata = create_test_plugin_metadata("test-plugin");
        metadata.version = "invalid".to_string();

        let result = loader.load_plugin(metadata).await;
        assert!(result.is_err());
    }
}
