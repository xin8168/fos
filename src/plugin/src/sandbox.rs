//! # Plugin Sandbox - 插件沙箱隔离
//!
//! 实现插件的安全隔离机制，包括资源限制、权限控制、环境隔离

use crate::error::{Error, Result};
use crate::plugin::PluginType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock as AsyncRwLock;
use tracing::info;

/// 资源限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// 最大CPU时间（秒）
    pub max_cpu_time_sec: Option<u64>,
    /// 最大内存使用（字节）
    pub max_memory_bytes: Option<u64>,
    /// 最大文件描述符数量
    pub max_file_descriptors: Option<u32>,
    /// 最大网络连接数
    pub max_network_connections: Option<u32>,
    /// 最大磁盘写入（字节）
    pub max_disk_write_bytes: Option<u64>,
    /// 最大磁盘读取（字节）
    pub max_disk_read_bytes: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_time_sec: Some(60),                     // 默认60秒CPU时间
            max_memory_bytes: Some(512 * 1024 * 1024),      // 默认512MB内存
            max_file_descriptors: Some(100),                // 默认100个文件描述符
            max_network_connections: Some(10),              // 默认10个网络连接
            max_disk_write_bytes: Some(1024 * 1024 * 1024), // 默认1GB写入
            max_disk_read_bytes: Some(1024 * 1024 * 1024),  // 默认1GB读取
        }
    }
}

/// 插件权限
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct PluginPermissions {
    /// 文件读取权限
    pub allow_file_read: bool,
    /// 文件写入权限
    pub allow_file_write: bool,
    /// 网络访问权限
    pub allow_network: bool,
    /// 子进程执行权限
    pub allow_process_spawn: bool,
    /// 系统调用权限
    pub allow_syscalls: bool,
    /// 环境变量访问权限
    pub allow_env_read: bool,
    /// 环境变量写入权限
    pub allow_env_write: bool,
}

impl Default for PluginPermissions {
    fn default() -> Self {
        // 最小权限原则：默认只允许读取
        Self {
            allow_file_read: true,
            allow_file_write: false,
            allow_network: false,
            allow_process_spawn: false,
            allow_syscalls: false,
            allow_env_read: true,
            allow_env_write: false,
        }
    }
}

impl PluginPermissions {
    /// 创建允许所有权限的策略（仅用于可信插件）
    pub fn allow_all() -> Self {
        Self {
            allow_file_read: true,
            allow_file_write: true,
            allow_network: true,
            allow_process_spawn: true,
            allow_syscalls: true,
            allow_env_read: true,
            allow_env_write: true,
        }
    }

    /// 创建只读策略
    pub fn readonly() -> Self {
        Self {
            allow_file_read: true,
            allow_file_write: false,
            allow_network: false,
            allow_process_spawn: false,
            allow_syscalls: false,
            allow_env_read: true,
            allow_env_write: false,
        }
    }

    /// 检查是否允许文件读取
    pub fn check_file_read(&self) -> Result<()> {
        if !self.allow_file_read {
            return Err(Error::Plugin("File read permission denied".to_string()));
        }
        Ok(())
    }

    /// 检查是否允许文件写入
    pub fn check_file_write(&self) -> Result<()> {
        if !self.allow_file_write {
            return Err(Error::Plugin("File write permission denied".to_string()));
        }
        Ok(())
    }

    /// 检查是否允许网络访问
    pub fn check_network(&self) -> Result<()> {
        if !self.allow_network {
            return Err(Error::Plugin("Network access permission denied".to_string()));
        }
        Ok(())
    }

    /// 检查是否允许子进程执行
    pub fn check_process_spawn(&self) -> Result<()> {
        if !self.allow_process_spawn {
            return Err(Error::Plugin("Process spawn permission denied".to_string()));
        }
        Ok(())
    }

    /// 检查是否允许系统调用
    pub fn check_syscalls(&self) -> Result<()> {
        if !self.allow_syscalls {
            return Err(Error::Plugin("System call permission denied".to_string()));
        }
        Ok(())
    }

    /// 检查是否允许环境变量读取
    pub fn check_env_read(&self) -> Result<()> {
        if !self.allow_env_read {
            return Err(Error::Plugin("Environment variable read permission denied".to_string()));
        }
        Ok(())
    }

    /// 检查是否允许环境变量写入
    pub fn check_env_write(&self) -> Result<()> {
        if !self.allow_env_write {
            return Err(Error::Plugin("Environment variable write permission denied".to_string()));
        }
        Ok(())
    }
}

/// 沙箱配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// 资源限制
    pub resource_limits: ResourceLimits,
    /// 插件权限
    pub permissions: PluginPermissions,
    /// 工作目录
    pub working_directory: Option<String>,
    /// 允许的文件路径（白名单）
    pub allowed_paths: Vec<String>,
    /// 禁止的文件路径（黑名单）
    pub blocked_paths: Vec<String>,
    /// 环境变量
    pub environment: HashMap<String, String>,
    /// 是否启用网络隔离
    pub enable_network_isolation: bool,
    /// 是否启用文件系统隔离
    pub enable_fs_isolation: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            resource_limits: ResourceLimits::default(),
            permissions: PluginPermissions::default(),
            working_directory: None,
            allowed_paths: vec![],
            blocked_paths: vec![],
            environment: HashMap::new(),
            enable_network_isolation: false,
            enable_fs_isolation: false,
        }
    }
}

/// 资源使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU使用时间（纳秒）
    pub cpu_time_ns: u64,
    /// 内存使用（字节）
    pub memory_bytes: u64,
    /// 文件描述符数量
    pub file_descriptors: u32,
    /// 网络连接数
    pub network_connections: u32,
    /// 磁盘写入（字节）
    pub disk_write_bytes: u64,
    /// 磁盘读取（字节）
    pub disk_read_bytes: u64,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            cpu_time_ns: 0,
            memory_bytes: 0,
            file_descriptors: 0,
            network_connections: 0,
            disk_write_bytes: 0,
            disk_read_bytes: 0,
        }
    }
}

/// 沙箱实例
#[derive(Clone)]
pub struct Sandbox {
    /// 插件ID
    plugin_id: String,
    /// 沙箱配置
    config: SandboxConfig,
    /// 资源使用统计
    usage: Arc<AsyncRwLock<ResourceUsage>>,
    /// 是否活跃
    is_active: Arc<AsyncRwLock<bool>>,
}

impl Sandbox {
    /// 创建新的沙箱
    pub fn new(plugin_id: String, config: SandboxConfig) -> Self {
        info!("Creating sandbox for plugin: {}", plugin_id);

        Self {
            plugin_id,
            config,
            usage: Arc::new(AsyncRwLock::new(ResourceUsage::default())),
            is_active: Arc::new(AsyncRwLock::new(true)),
        }
    }

    /// 检查是否活跃
    pub async fn is_active(&self) -> bool {
        *self.is_active.read().await
    }

    /// 停用沙箱
    pub async fn deactivate(&self) {
        let mut active = self.is_active.write().await;
        *active = false;
    }

    /// 获取资源使用统计
    pub async fn get_usage(&self) -> ResourceUsage {
        self.usage.read().await.clone()
    }

    /// 更新资源使用
    async fn update_usage<F>(&self, updater: F)
    where
        F: FnOnce(&mut ResourceUsage),
    {
        let mut usage = self.usage.write().await;
        updater(&mut usage);
    }

    /// 检查资源限制
    pub async fn check_resource_limits(&self) -> Result<()> {
        let usage = self.usage.read().await;

        // 检查CPU时间
        if let Some(max_cpu) = self.config.resource_limits.max_cpu_time_sec {
            let cpu_time_sec = usage.cpu_time_ns / 1_000_000_000;
            if cpu_time_sec > max_cpu {
                return Err(Error::Plugin(format!(
                    "CPU time limit exceeded: {}s > {}s",
                    cpu_time_sec, max_cpu
                )));
            }
        }

        // 检查内存
        if let Some(max_mem) = self.config.resource_limits.max_memory_bytes {
            if usage.memory_bytes > max_mem {
                return Err(Error::Plugin(format!(
                    "Memory limit exceeded: {} bytes > {} bytes",
                    usage.memory_bytes, max_mem
                )));
            }
        }

        // 检查文件描述符
        if let Some(max_fds) = self.config.resource_limits.max_file_descriptors {
            if usage.file_descriptors > max_fds {
                return Err(Error::Plugin(format!(
                    "File descriptor limit exceeded: {} > {}",
                    usage.file_descriptors, max_fds
                )));
            }
        }

        // 检查网络连接
        if let Some(max_net) = self.config.resource_limits.max_network_connections {
            if usage.network_connections > max_net {
                return Err(Error::Plugin(format!(
                    "Network connection limit exceeded: {} > {}",
                    usage.network_connections, max_net
                )));
            }
        }

        // 检查磁盘写入
        if let Some(max_write) = self.config.resource_limits.max_disk_write_bytes {
            if usage.disk_write_bytes > max_write {
                return Err(Error::Plugin(format!(
                    "Disk write limit exceeded: {} bytes > {} bytes",
                    usage.disk_write_bytes, max_write
                )));
            }
        }

        // 检查磁盘读取
        if let Some(max_read) = self.config.resource_limits.max_disk_read_bytes {
            if usage.disk_read_bytes > max_read {
                return Err(Error::Plugin(format!(
                    "Disk read limit exceeded: {} bytes > {} bytes",
                    usage.disk_read_bytes, max_read
                )));
            }
        }

        Ok(())
    }

    /// 检查文件访问权限
    pub async fn check_file_access(&self, path: &str, write: bool) -> Result<()> {
        // 检查基本权限
        if write {
            self.config.permissions.check_file_write()?;
        } else {
            self.config.permissions.check_file_read()?;
        }

        // 检查黑名单
        for blocked in &self.config.blocked_paths {
            if path.starts_with(blocked) {
                return Err(Error::Plugin(format!("Access to path blocked: {}", path)));
            }
        }

        // 如果启用了白名单且非空，检查白名单
        if !self.config.allowed_paths.is_empty() {
            let allowed = self.config.allowed_paths.iter().any(|allowed| path.starts_with(allowed));

            if !allowed {
                return Err(Error::Plugin(format!("Access to path not allowed: {}", path)));
            }
        }

        Ok(())
    }

    /// 记录CPU使用
    pub async fn record_cpu_time(&self, nanoseconds: u64) {
        self.update_usage(|usage| usage.cpu_time_ns += nanoseconds).await;
    }

    /// 记录内存使用
    pub async fn record_memory(&self, bytes: u64) {
        self.update_usage(|usage| usage.memory_bytes = bytes).await;
    }

    /// 记录文件描述符使用
    pub async fn record_file_descriptor(&self, count: u32) {
        self.update_usage(|usage| usage.file_descriptors = count).await;
    }

    /// 记录网络连接
    pub async fn record_network_connection(&self) {
        self.update_usage(|usage| usage.network_connections += 1).await;
    }

    /// 释放网络连接
    pub async fn release_network_connection(&self) {
        self.update_usage(|usage| {
            usage.network_connections = usage.network_connections.saturating_sub(1);
        })
        .await;
    }

    /// 记录磁盘写入
    pub async fn record_disk_write(&self, bytes: u64) {
        self.update_usage(|usage| usage.disk_write_bytes += bytes).await;
    }

    /// 记录磁盘读取
    pub async fn record_disk_read(&self, bytes: u64) {
        self.update_usage(|usage| usage.disk_read_bytes += bytes).await;
    }

    /// 重置资源使用统计
    pub async fn reset_usage(&self) {
        self.update_usage(|usage| {
            *usage = ResourceUsage::default();
        })
        .await;
    }

    /// 获取配置
    pub fn get_config(&self) -> &SandboxConfig {
        &self.config
    }

    /// 获取插件ID
    pub fn get_plugin_id(&self) -> &str {
        &self.plugin_id
    }
}

/// 沙箱管理器
pub struct SandboxManager {
    /// 沙箱实例
    sandboxes: Arc<AsyncRwLock<HashMap<String, Arc<Sandbox>>>>,
    /// 默认沙箱配置
    default_config: SandboxConfig,
}

impl SandboxManager {
    /// 创建新的沙箱管理器
    pub fn new(default_config: SandboxConfig) -> Self {
        Self { sandboxes: Arc::new(AsyncRwLock::new(HashMap::new())), default_config }
    }

    pub fn new_with_default() -> Self {
        Self::new(SandboxConfig::default())
    }

    /// 根据插件类型创建沙箱配置
    pub fn create_config_for_plugin_type(plugin_type: &PluginType) -> SandboxConfig {
        let mut config = SandboxConfig::default();

        match plugin_type {
            PluginType::Storage => {
                // 存储插件需要文件读写权限
                config.permissions = PluginPermissions {
                    allow_file_read: true,
                    allow_file_write: true,
                    allow_network: false,
                    allow_process_spawn: false,
                    allow_syscalls: false,
                    allow_env_read: true,
                    allow_env_write: false,
                };
                config.resource_limits.max_memory_bytes = Some(1024 * 1024 * 1024);
                // 1GB
            },
            PluginType::Monitor => {
                // 监控插件需要网络权限
                config.permissions = PluginPermissions {
                    allow_file_read: true,
                    allow_file_write: false,
                    allow_network: true,
                    allow_process_spawn: false,
                    allow_syscalls: false,
                    allow_env_read: true,
                    allow_env_write: false,
                };
                config.resource_limits.max_memory_bytes = Some(256 * 1024 * 1024);
                // 256MB
            },
            PluginType::Notifier => {
                // 通知插件需要网络权限
                config.permissions = PluginPermissions {
                    allow_file_read: false,
                    allow_file_write: false,
                    allow_network: true,
                    allow_process_spawn: false,
                    allow_syscalls: false,
                    allow_env_read: true,
                    allow_env_write: false,
                };
                config.resource_limits.max_memory_bytes = Some(128 * 1024 * 1024);
                // 128MB
            },
            PluginType::Custom => {
                // 自定义插件使用默认配置（最小权限）
                config.permissions = PluginPermissions::default();
            },
        }

        config
    }

    /// 创建沙箱
    pub async fn create_sandbox(
        &self,
        plugin_id: String,
        config: Option<SandboxConfig>,
    ) -> Result<Arc<Sandbox>> {
        let config = config.unwrap_or_else(|| self.default_config.clone());

        let sandbox = Arc::new(Sandbox::new(plugin_id.clone(), config));

        let mut sandboxes = self.sandboxes.write().await;
        sandboxes.insert(plugin_id.clone(), sandbox.clone());

        info!("Created sandbox for plugin: {}", plugin_id);
        Ok(sandbox)
    }

    /// 移除沙箱
    pub async fn remove_sandbox(&self, plugin_id: &str) -> Result<()> {
        let mut sandboxes = self.sandboxes.write().await;

        if let Some(sandbox) = sandboxes.remove(plugin_id) {
            sandbox.deactivate().await;
            info!("Removed sandbox for plugin: {}", plugin_id);
            Ok(())
        } else {
            Err(Error::Plugin(format!("Sandbox not found for plugin: {}", plugin_id)))
        }
    }

    /// 获取沙箱
    pub async fn get_sandbox(&self, plugin_id: &str) -> Result<Arc<Sandbox>> {
        let sandboxes = self.sandboxes.read().await;
        sandboxes
            .get(plugin_id)
            .cloned()
            .ok_or_else(|| Error::Plugin(format!("Sandbox not found: {}", plugin_id)))
    }

    /// 获取所有沙箱
    pub async fn list_sandboxes(&self) -> Vec<String> {
        let sandboxes = self.sandboxes.read().await;
        sandboxes.keys().cloned().collect()
    }

    /// 清理不活跃的沙箱
    pub async fn cleanup_inactive(&self) -> usize {
        let mut sandboxes = self.sandboxes.write().await;
        let mut removed = 0;

        let inactive_ids: Vec<String> = sandboxes
            .iter()
            .filter(|(_, sandbox)| {
                // 异步检查需要小心，这里使用try_read
                if let Ok(active) = sandbox.is_active.try_read() {
                    !*active
                } else {
                    false // 如果无法读取，保留沙箱
                }
            })
            .map(|(id, _)| id.clone())
            .collect();

        for id in inactive_ids {
            if let Some(sandbox) = sandboxes.remove(&id) {
                sandbox.deactivate().await;
                removed += 1;
                info!("Cleaned up inactive sandbox: {}", id);
            }
        }

        removed
    }
}

impl Default for SandboxManager {
    fn default() -> Self {
        Self::new_with_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert!(limits.max_cpu_time_sec.is_some());
        assert!(limits.max_memory_bytes.is_some());
    }

    #[test]
    fn test_plugin_permissions_default() {
        let perms = PluginPermissions::default();
        assert!(perms.allow_file_read);
        assert!(!perms.allow_file_write);
        assert!(!perms.allow_network);
    }

    #[test]
    fn test_plugin_permissions_allow_all() {
        let perms = PluginPermissions::allow_all();
        assert!(perms.allow_file_read);
        assert!(perms.allow_file_write);
        assert!(perms.allow_network);
    }

    #[test]
    fn test_plugin_permissions_readonly() {
        let perms = PluginPermissions::readonly();
        assert!(perms.allow_file_read);
        assert!(!perms.allow_file_write);
        assert!(!perms.allow_network);
    }

    #[test]
    fn test_permission_checks() {
        let perms = PluginPermissions::readonly();

        assert!(perms.check_file_read().is_ok());
        assert!(perms.check_file_write().is_err());
        assert!(perms.check_network().is_err());
        assert!(perms.check_process_spawn().is_err());
    }

    #[tokio::test]
    async fn test_sandbox_creation() {
        let config = SandboxConfig::default();
        let sandbox = Sandbox::new("test-plugin".to_string(), config);

        assert_eq!(sandbox.get_plugin_id(), "test-plugin");
        assert!(sandbox.is_active().await);
    }

    #[tokio::test]
    async fn test_sandbox_deactivate() {
        let sandbox = Sandbox::new("test-plugin".to_string(), SandboxConfig::default());

        sandbox.deactivate().await;
        assert!(!sandbox.is_active().await);
    }

    #[tokio::test]
    async fn test_resource_usage_tracking() {
        let sandbox = Sandbox::new("test-plugin".to_string(), SandboxConfig::default());

        sandbox.record_cpu_time(1_000_000_000).await; // 1秒
        sandbox.record_memory(1024 * 1024).await; // 1MB
        sandbox.record_disk_write(512 * 1024).await; // 512KB

        let usage = sandbox.get_usage().await;
        assert_eq!(usage.cpu_time_ns, 1_000_000_000);
        assert_eq!(usage.memory_bytes, 1024 * 1024);
        assert_eq!(usage.disk_write_bytes, 512 * 1024);
    }

    #[tokio::test]
    async fn test_resource_limit_check() {
        let config = SandboxConfig {
            resource_limits: ResourceLimits {
                max_cpu_time_sec: Some(30),
                max_memory_bytes: Some(1024 * 1024), // 1MB
                ..Default::default()
            },
            ..Default::default()
        };

        let sandbox = Sandbox::new("test-plugin".to_string(), config);

        // 正常使用
        sandbox.record_cpu_time(10 * 1_000_000_000).await; // 10秒
        sandbox.record_memory(512 * 1024).await; // 512KB
        assert!(sandbox.check_resource_limits().await.is_ok());

        // 超过CPU时间
        sandbox.record_cpu_time(25 * 1_000_000_000).await; // 再加25秒，总计35秒
        assert!(sandbox.check_resource_limits().await.is_err());
    }

    #[tokio::test]
    async fn test_file_access_permission() {
        let config = SandboxConfig {
            allowed_paths: vec!["/tmp/".to_string()],
            blocked_paths: vec!["/etc/".to_string()],
            ..Default::default()
        };

        let sandbox = Sandbox::new("test-plugin".to_string(), config);

        // 允许的路径
        assert!(sandbox.check_file_access("/tmp/test.txt", false).await.is_ok());

        // 不允许的路径（白名单）
        assert!(sandbox.check_file_access("/var/log/test.txt", false).await.is_err());

        // 禁止的路径（黑名单）
        assert!(sandbox.check_file_access("/etc/passwd", false).await.is_err());
    }

    #[tokio::test]
    async fn test_file_write_permission() {
        let config =
            SandboxConfig { permissions: PluginPermissions::readonly(), ..Default::default() };

        let sandbox = Sandbox::new("test-plugin".to_string(), config);

        // 只读配置不允许写入
        assert!(sandbox.check_file_access("/tmp/test.txt", true).await.is_err());
    }

    #[tokio::test]
    async fn test_network_connection_tracking() {
        let sandbox = Sandbox::new("test-plugin".to_string(), SandboxConfig::default());

        sandbox.record_network_connection().await;
        let usage = sandbox.get_usage().await;
        assert_eq!(usage.network_connections, 1);

        sandbox.record_network_connection().await;
        let usage = sandbox.get_usage().await;
        assert_eq!(usage.network_connections, 2);

        sandbox.release_network_connection().await;
        let usage = sandbox.get_usage().await;
        assert_eq!(usage.network_connections, 1);
    }

    #[tokio::test]
    async fn test_usage_reset() {
        let sandbox = Sandbox::new("test-plugin".to_string(), SandboxConfig::default());

        sandbox.record_cpu_time(1_000_000_000).await;
        sandbox.record_memory(1024 * 1024).await;

        sandbox.reset_usage().await;

        let usage = sandbox.get_usage().await;
        assert_eq!(usage.cpu_time_ns, 0);
        assert_eq!(usage.memory_bytes, 0);
    }

    #[tokio::test]
    async fn test_sandbox_manager_creation() {
        let manager = SandboxManager::new_with_default();
        let sandboxes = manager.list_sandboxes().await;
        assert!(sandboxes.is_empty());
    }

    #[tokio::test]
    async fn test_sandbox_manager_create_and_remove() {
        let manager = SandboxManager::new_with_default();

        manager.create_sandbox("test-plugin".to_string(), None).await.unwrap();

        let sandboxes = manager.list_sandboxes().await;
        assert_eq!(sandboxes.len(), 1);
        assert!(sandboxes.contains(&"test-plugin".to_string()));

        manager.remove_sandbox("test-plugin").await.unwrap();

        let sandboxes = manager.list_sandboxes().await;
        assert!(sandboxes.is_empty());
    }

    #[tokio::test]
    async fn test_sandbox_manager_get_sandbox() {
        let manager = SandboxManager::new_with_default();
        let sandbox = manager.create_sandbox("test-plugin".to_string(), None).await.unwrap();

        let retrieved = manager.get_sandbox("test-plugin").await.unwrap();
        assert_eq!(retrieved.get_plugin_id(), sandbox.get_plugin_id());
    }

    #[tokio::test]
    async fn test_sandbox_manager_get_nonexistent() {
        let manager = SandboxManager::new_with_default();
        let result = manager.get_sandbox("nonexistent").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_create_config_for_plugin_type() {
        let storage_config = SandboxManager::create_config_for_plugin_type(&PluginType::Storage);
        assert!(storage_config.permissions.allow_file_write);

        let monitor_config = SandboxManager::create_config_for_plugin_type(&PluginType::Monitor);
        assert!(monitor_config.permissions.allow_network);

        let notifier_config = SandboxManager::create_config_for_plugin_type(&PluginType::Notifier);
        assert!(notifier_config.permissions.allow_network);

        let custom_config = SandboxManager::create_config_for_plugin_type(&PluginType::Custom);
        assert!(!custom_config.permissions.allow_file_write);
    }

    #[tokio::test]
    async fn test_sandbox_serialization() {
        let config = SandboxConfig::default();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: SandboxConfig = serde_json::from_str(&serialized).unwrap();

        assert_eq!(
            config.resource_limits.max_cpu_time_sec,
            deserialized.resource_limits.max_cpu_time_sec
        );
    }

    #[tokio::test]
    async fn test_permissions_serialization() {
        let perms = PluginPermissions::default();
        let serialized = serde_json::to_string(&perms).unwrap();
        let deserialized: PluginPermissions = serde_json::from_str(&serialized).unwrap();

        assert_eq!(perms, deserialized);
    }

    #[tokio::test]
    async fn test_multiple_sandboxes() {
        let manager = SandboxManager::new_with_default();

        for i in 1..=3 {
            manager.create_sandbox(format!("plugin-{}", i), None).await.unwrap();
        }

        let sandboxes = manager.list_sandboxes().await;
        assert_eq!(sandboxes.len(), 3);

        for id in 1..=3 {
            let sandbox = manager.get_sandbox(&format!("plugin-{}", id)).await.unwrap();
            assert!(sandbox.is_active().await);
        }
    }
}
