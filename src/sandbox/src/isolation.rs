//! 隔离层实现
//!
//! 提供文件系统、网络、进程的隔离能力

use crate::error::{Result, SandboxError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 隔离配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationConfig {
    /// 文件系统根目录
    pub filesystem_root: Option<PathBuf>,
    /// 只读路径
    pub readonly_paths: Vec<PathBuf>,
    /// 读写路径
    pub readwrite_paths: Vec<PathBuf>,
    /// 禁止访问路径
    pub denied_paths: Vec<PathBuf>,
    /// 网络命名空间
    pub network_namespace: Option<String>,
    /// 允许的网络地址
    pub allowed_networks: Vec<String>,
    /// 禁止的网络地址
    pub denied_networks: Vec<String>,
    /// 环境变量
    pub environment: HashMap<String, String>,
}

impl Default for IsolationConfig {
    fn default() -> Self {
        Self {
            filesystem_root: None,
            readonly_paths: Vec::new(),
            readwrite_paths: Vec::new(),
            denied_paths: Vec::new(),
            network_namespace: None,
            allowed_networks: Vec::new(),
            denied_networks: Vec::new(),
            environment: HashMap::new(),
        }
    }
}

/// 文件系统隔离状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FilesystemIsolationStatus {
    /// 未激活
    Inactive,
    /// 已激活
    Active,
    /// 已销毁
    Destroyed,
}

/// 网络隔离状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NetworkIsolationStatus {
    /// 未激活
    Inactive,
    /// 已激活
    Active,
    /// 已销毁
    Destroyed,
}

/// 进程隔离状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProcessIsolationStatus {
    /// 未激活
    Inactive,
    /// 已激活
    Active,
    /// 已销毁
    Destroyed,
}

/// 文件系统隔离器
pub struct FilesystemIsolation {
    /// 配置
    config: IsolationConfig,
    /// 状态
    status: Arc<RwLock<FilesystemIsolationStatus>>,
    /// 挂载点
    mount_points: Arc<RwLock<Vec<PathBuf>>>,
}

impl FilesystemIsolation {
    /// 创建新的文件系统隔离
    pub fn new(config: IsolationConfig) -> Self {
        Self {
            config,
            status: Arc::new(RwLock::new(FilesystemIsolationStatus::Inactive)),
            mount_points: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 激活文件系统隔离
    pub async fn activate(&self) -> Result<()> {
        let mut status = self.status.write().await;
        *status = FilesystemIsolationStatus::Active;
        Ok(())
    }

    /// 检查路径访问权限
    pub async fn check_path_access(&self, path: &PathBuf) -> Result<PathAccess> {
        let status = self.status.read().await;
        if *status != FilesystemIsolationStatus::Active {
            return Ok(PathAccess::Denied);
        }

        // 检查禁止路径
        for denied in &self.config.denied_paths {
            if path.starts_with(denied) {
                return Ok(PathAccess::Denied);
            }
        }

        // 检查读写路径
        for rw_path in &self.config.readwrite_paths {
            if path.starts_with(rw_path) {
                return Ok(PathAccess::ReadWrite);
            }
        }

        // 检查只读路径
        for ro_path in &self.config.readonly_paths {
            if path.starts_with(ro_path) {
                return Ok(PathAccess::ReadOnly);
            }
        }

        // 默认拒绝
        Ok(PathAccess::Denied)
    }

    /// 添加挂载点
    pub async fn add_mount_point(&self, path: PathBuf) -> Result<()> {
        let mut mount_points = self.mount_points.write().await;
        mount_points.push(path);
        Ok(())
    }

    /// 销毁隔离
    pub async fn destroy(&self) -> Result<()> {
        let mut status = self.status.write().await;
        *status = FilesystemIsolationStatus::Destroyed;
        Ok(())
    }

    /// 获取状态
    pub async fn status(&self) -> FilesystemIsolationStatus {
        self.status.read().await.clone()
    }
}

/// 路径访问权限
#[derive(Debug, Clone, PartialEq)]
pub enum PathAccess {
    /// 只读
    ReadOnly,
    /// 读写
    ReadWrite,
    /// 拒绝
    Denied,
}

/// 网络隔离器
pub struct NetworkIsolation {
    /// 配置
    config: IsolationConfig,
    /// 状态
    status: Arc<RwLock<NetworkIsolationStatus>>,
}

impl NetworkIsolation {
    /// 创建新的网络隔离
    pub fn new(config: IsolationConfig) -> Self {
        Self { config, status: Arc::new(RwLock::new(NetworkIsolationStatus::Inactive)) }
    }

    /// 激活网络隔离
    pub async fn activate(&self) -> Result<()> {
        let mut status = self.status.write().await;
        *status = NetworkIsolationStatus::Active;
        Ok(())
    }

    /// 检查网络访问权限
    pub async fn check_network_access(&self, address: &str) -> Result<NetworkAccess> {
        let status = self.status.read().await;
        if *status != NetworkIsolationStatus::Active {
            return Ok(NetworkAccess::Denied);
        }

        // 检查禁止地址
        for denied in &self.config.denied_networks {
            if address.starts_with(denied) || address == *denied {
                return Ok(NetworkAccess::Denied);
            }
        }

        // 如果有允许列表，检查是否在允许列表中
        if !self.config.allowed_networks.is_empty() {
            for allowed in &self.config.allowed_networks {
                if address.starts_with(allowed) || address == *allowed {
                    return Ok(NetworkAccess::Allowed);
                }
            }
            return Ok(NetworkAccess::Denied);
        }

        // 没有允许列表，默认允许
        Ok(NetworkAccess::Allowed)
    }

    /// 销毁隔离
    pub async fn destroy(&self) -> Result<()> {
        let mut status = self.status.write().await;
        *status = NetworkIsolationStatus::Destroyed;
        Ok(())
    }

    /// 获取状态
    pub async fn status(&self) -> NetworkIsolationStatus {
        self.status.read().await.clone()
    }
}

/// 网络访问权限
#[derive(Debug, Clone, PartialEq)]
pub enum NetworkAccess {
    /// 允许
    Allowed,
    /// 拒绝
    Denied,
}

/// 进程隔离器
pub struct ProcessIsolation {
    /// 配置
    config: IsolationConfig,
    /// 状态
    status: Arc<RwLock<ProcessIsolationStatus>>,
    /// 子进程ID列表
    child_pids: Arc<RwLock<Vec<u32>>>,
}

impl ProcessIsolation {
    /// 创建新的进程隔离
    pub fn new(config: IsolationConfig) -> Self {
        Self {
            config,
            status: Arc::new(RwLock::new(ProcessIsolationStatus::Inactive)),
            child_pids: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 激活进程隔离
    pub async fn activate(&self) -> Result<()> {
        let mut status = self.status.write().await;
        *status = ProcessIsolationStatus::Active;
        Ok(())
    }

    /// 注册子进程
    pub async fn register_child(&self, pid: u32) -> Result<()> {
        let mut child_pids = self.child_pids.write().await;
        child_pids.push(pid);
        Ok(())
    }

    /// 获取所有子进程
    pub async fn get_child_pids(&self) -> Vec<u32> {
        self.child_pids.read().await.clone()
    }

    /// 终止所有子进程
    pub async fn terminate_all_children(&self) -> Result<usize> {
        let mut child_pids = self.child_pids.write().await;
        let count = child_pids.len();
        child_pids.clear();
        Ok(count)
    }

    /// 销毁隔离
    pub async fn destroy(&self) -> Result<()> {
        // 先终止所有子进程
        self.terminate_all_children().await?;
        let mut status = self.status.write().await;
        *status = ProcessIsolationStatus::Destroyed;
        Ok(())
    }

    /// 获取状态
    pub async fn status(&self) -> ProcessIsolationStatus {
        self.status.read().await.clone()
    }
}

/// 隔离管理器
pub struct IsolationManager {
    /// 文件系统隔离
    pub filesystem: FilesystemIsolation,
    /// 网络隔离
    pub network: NetworkIsolation,
    /// 进程隔离
    pub process: ProcessIsolation,
}

impl IsolationManager {
    /// 创建新的隔离管理器
    pub fn new(config: IsolationConfig) -> Self {
        Self {
            filesystem: FilesystemIsolation::new(config.clone()),
            network: NetworkIsolation::new(config.clone()),
            process: ProcessIsolation::new(config),
        }
    }

    /// 激活所有隔离
    pub async fn activate_all(&self) -> Result<()> {
        self.filesystem.activate().await?;
        self.network.activate().await?;
        self.process.activate().await?;
        Ok(())
    }

    /// 销毁所有隔离
    pub async fn destroy_all(&self) -> Result<()> {
        self.filesystem.destroy().await?;
        self.network.destroy().await?;
        self.process.destroy().await?;
        Ok(())
    }

    /// 检查是否全部激活
    pub async fn is_all_active(&self) -> bool {
        let fs_active = self.filesystem.status().await == FilesystemIsolationStatus::Active;
        let net_active = self.network.status().await == NetworkIsolationStatus::Active;
        let proc_active = self.process.status().await == ProcessIsolationStatus::Active;
        fs_active && net_active && proc_active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_filesystem_isolation_lifecycle() {
        let config = IsolationConfig::default();
        let isolation = FilesystemIsolation::new(config);

        assert_eq!(isolation.status().await, FilesystemIsolationStatus::Inactive);

        isolation.activate().await.unwrap();
        assert_eq!(isolation.status().await, FilesystemIsolationStatus::Active);

        isolation.destroy().await.unwrap();
        assert_eq!(isolation.status().await, FilesystemIsolationStatus::Destroyed);
    }

    #[tokio::test]
    async fn test_path_access_check() {
        let mut config = IsolationConfig::default();
        config.readonly_paths.push(PathBuf::from("/readonly"));
        config.readwrite_paths.push(PathBuf::from("/readwrite"));
        config.denied_paths.push(PathBuf::from("/denied"));

        let isolation = FilesystemIsolation::new(config);
        isolation.activate().await.unwrap();

        assert_eq!(
            isolation.check_path_access(&PathBuf::from("/readonly/file")).await.unwrap(),
            PathAccess::ReadOnly
        );
        assert_eq!(
            isolation.check_path_access(&PathBuf::from("/readwrite/file")).await.unwrap(),
            PathAccess::ReadWrite
        );
        assert_eq!(
            isolation.check_path_access(&PathBuf::from("/denied/file")).await.unwrap(),
            PathAccess::Denied
        );
        assert_eq!(
            isolation.check_path_access(&PathBuf::from("/other/file")).await.unwrap(),
            PathAccess::Denied
        );
    }

    #[tokio::test]
    async fn test_network_isolation() {
        let mut config = IsolationConfig::default();
        config.allowed_networks.push("192.168.1".to_string()); // 使用前缀匹配
        config.denied_networks.push("10.0.0".to_string());

        let isolation = NetworkIsolation::new(config);
        isolation.activate().await.unwrap();

        assert_eq!(
            isolation.check_network_access("192.168.1.100").await.unwrap(),
            NetworkAccess::Allowed
        );
        assert_eq!(
            isolation.check_network_access("10.0.0.1").await.unwrap(),
            NetworkAccess::Denied
        );
    }

    #[tokio::test]
    async fn test_process_isolation() {
        let config = IsolationConfig::default();
        let isolation = ProcessIsolation::new(config);

        isolation.activate().await.unwrap();
        assert_eq!(isolation.status().await, ProcessIsolationStatus::Active);

        isolation.register_child(1234).await.unwrap();
        isolation.register_child(5678).await.unwrap();

        let pids = isolation.get_child_pids().await;
        assert_eq!(pids.len(), 2);

        let count = isolation.terminate_all_children().await.unwrap();
        assert_eq!(count, 2);

        isolation.destroy().await.unwrap();
        assert_eq!(isolation.status().await, ProcessIsolationStatus::Destroyed);
    }

    #[tokio::test]
    async fn test_isolation_manager() {
        let config = IsolationConfig::default();
        let manager = IsolationManager::new(config);

        manager.activate_all().await.unwrap();
        assert!(manager.is_all_active().await);

        manager.destroy_all().await.unwrap();
        assert!(!manager.is_all_active().await);
    }
}
