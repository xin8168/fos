//! 增强设备管理模块
//!
//! 提供完整的设备生命周期管理

use crate::error::{McpError, Result};
use crate::{Device, DeviceStatus, DeviceType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 设备连接信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConnection {
    /// 连接ID
    pub connection_id: String,
    /// 设备ID
    pub device_id: String,
    /// 连接地址
    pub address: String,
    /// 连接时间
    pub connected_at: DateTime<Utc>,
    /// 最后活跃时间
    pub last_active: DateTime<Utc>,
    /// 连接协议
    pub protocol: String,
    /// 连接状态
    pub status: ConnectionStatus,
}

impl DeviceConnection {
    /// 创建新连接
    pub fn new(device_id: String, address: String, protocol: String) -> Self {
        let now = Utc::now();
        Self {
            connection_id: uuid::Uuid::new_v4().to_string(),
            device_id,
            address,
            connected_at: now,
            last_active: now,
            protocol,
            status: ConnectionStatus::Connected,
        }
    }

    /// 更新活跃时间
    pub fn update_active(&mut self) {
        self.last_active = Utc::now();
    }

    /// 检查是否超时
    pub fn is_timeout(&self, timeout_secs: u64) -> bool {
        let elapsed = Utc::now() - self.last_active;
        elapsed.num_seconds() > timeout_secs as i64
    }
}

/// 连接状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionStatus {
    /// 已连接
    Connected,
    /// 断开中
    Disconnecting,
    /// 已断开
    Disconnected,
    /// 重连中
    Reconnecting,
}

/// 设备会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSession {
    /// 会话ID
    pub session_id: String,
    /// 设备ID
    pub device_id: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 过期时间
    pub expires_at: DateTime<Utc>,
    /// 会话数据
    pub data: HashMap<String, serde_json::Value>,
}

impl DeviceSession {
    /// 创建新会话
    pub fn new(device_id: String, ttl_secs: u64) -> Self {
        let now = Utc::now();
        Self {
            session_id: uuid::Uuid::new_v4().to_string(),
            device_id,
            created_at: now,
            expires_at: now + chrono::Duration::seconds(ttl_secs as i64),
            data: HashMap::new(),
        }
    }

    /// 检查是否过期
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// 刷新会话
    pub fn refresh(&mut self, ttl_secs: u64) {
        self.expires_at = Utc::now() + chrono::Duration::seconds(ttl_secs as i64);
    }

    /// 设置数据
    pub fn set_data(&mut self, key: String, value: serde_json::Value) {
        self.data.insert(key, value);
    }

    /// 获取数据
    pub fn get_data(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }
}

/// 设备统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceStats {
    /// 总设备数
    pub total: usize,
    /// 在线设备数
    pub online: usize,
    /// 离线设备数
    pub offline: usize,
    /// 错误设备数
    pub error: usize,
    /// 维护中设备数
    pub maintenance: usize,
    /// 总连接数
    pub connections: usize,
    /// 活跃会话数
    pub active_sessions: usize,
}

/// 增强型设备管理器
pub struct EnhancedDeviceManager {
    /// 设备列表
    devices: Arc<RwLock<HashMap<String, Device>>>,
    /// 连接列表
    connections: Arc<RwLock<HashMap<String, DeviceConnection>>>,
    /// 会话列表
    sessions: Arc<RwLock<HashMap<String, DeviceSession>>>,
    /// 设备到连接的映射
    device_connections: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// 配置
    config: DeviceManagerConfig,
}

/// 设备管理器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceManagerConfig {
    /// 心跳超时（秒）
    pub heartbeat_timeout: u64,
    /// 会话TTL（秒）
    pub session_ttl: u64,
    /// 最大设备数
    pub max_devices: usize,
    /// 最大连接数
    pub max_connections: usize,
}

impl Default for DeviceManagerConfig {
    fn default() -> Self {
        Self { heartbeat_timeout: 30, session_ttl: 3600, max_devices: 1000, max_connections: 10000 }
    }
}

impl EnhancedDeviceManager {
    /// 创建新管理器
    pub fn new() -> Self {
        Self::with_config(DeviceManagerConfig::default())
    }

    /// 使用配置创建
    pub fn with_config(config: DeviceManagerConfig) -> Self {
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            device_connections: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// 注册设备
    pub async fn register_device(&self, mut device: Device) -> Result<String> {
        // 检查数量限制
        {
            let devices = self.devices.read().await;
            if devices.len() >= self.config.max_devices {
                return Err(McpError::LimitExceeded("设备数量已达上限".to_string()));
            }
        }

        let id = device.id.clone();
        device.status = DeviceStatus::Offline;

        let mut devices = self.devices.write().await;
        devices.insert(id.clone(), device);

        Ok(id)
    }

    /// 注销设备
    pub async fn unregister_device(&self, device_id: &str) -> Result<()> {
        // 断开所有连接
        let conn_ids: Vec<String> = {
            let device_connections = self.device_connections.read().await;
            device_connections.get(device_id).cloned().unwrap_or_default()
        };

        for conn_id in conn_ids {
            let _ = self.disconnect(&conn_id).await;
        }

        // 移除设备
        let mut devices = self.devices.write().await;
        devices.remove(device_id).ok_or_else(|| McpError::DeviceNotFound(device_id.to_string()))?;

        // 清理映射
        let mut device_connections = self.device_connections.write().await;
        device_connections.remove(device_id);

        Ok(())
    }

    /// 获取设备
    pub async fn get_device(&self, device_id: &str) -> Result<Device> {
        let devices = self.devices.read().await;
        devices
            .get(device_id)
            .cloned()
            .ok_or_else(|| McpError::DeviceNotFound(device_id.to_string()))
    }

    /// 更新设备状态
    pub async fn update_device_status(&self, device_id: &str, status: DeviceStatus) -> Result<()> {
        let mut devices = self.devices.write().await;
        if let Some(device) = devices.get_mut(device_id) {
            device.status = status;
            Ok(())
        } else {
            Err(McpError::DeviceNotFound(device_id.to_string()))
        }
    }

    /// 连接设备
    pub async fn connect(
        &self,
        device_id: String,
        address: String,
        protocol: String,
    ) -> Result<String> {
        // 检查设备存在
        {
            let devices = self.devices.read().await;
            if !devices.contains_key(&device_id) {
                return Err(McpError::DeviceNotFound(device_id.clone()));
            }
        }

        // 检查连接数量
        {
            let connections = self.connections.read().await;
            if connections.len() >= self.config.max_connections {
                return Err(McpError::LimitExceeded("连接数量已达上限".to_string()));
            }
        }

        let connection = DeviceConnection::new(device_id.clone(), address, protocol);
        let conn_id = connection.connection_id.clone();

        // 存储连接
        {
            let mut connections = self.connections.write().await;
            connections.insert(conn_id.clone(), connection);
        }

        // 更新映射
        {
            let mut device_connections = self.device_connections.write().await;
            device_connections
                .entry(device_id.clone())
                .or_insert_with(Vec::new)
                .push(conn_id.clone());
        }

        // 更新设备状态
        self.update_device_status(&device_id, DeviceStatus::Online).await?;

        Ok(conn_id)
    }

    /// 断开连接
    pub async fn disconnect(&self, connection_id: &str) -> Result<()> {
        let device_id: String;

        // 移除连接
        {
            let mut connections = self.connections.write().await;
            let conn = connections
                .remove(connection_id)
                .ok_or_else(|| McpError::ConnectionNotFound(connection_id.to_string()))?;
            device_id = conn.device_id;
        }

        // 更新映射
        {
            let mut device_connections = self.device_connections.write().await;
            if let Some(conn_ids) = device_connections.get_mut(&device_id) {
                conn_ids.retain(|id| id != connection_id);
            }
        }

        Ok(())
    }

    /// 创建会话
    pub async fn create_session(&self, device_id: String) -> Result<String> {
        let session = DeviceSession::new(device_id, self.config.session_ttl);
        let session_id = session.session_id.clone();

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);

        Ok(session_id)
    }

    /// 获取会话
    pub async fn get_session(&self, session_id: &str) -> Result<DeviceSession> {
        let sessions = self.sessions.read().await;
        sessions
            .get(session_id)
            .cloned()
            .ok_or_else(|| McpError::SessionNotFound(session_id.to_string()))
    }

    /// 刷新会话
    pub async fn refresh_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.refresh(self.config.session_ttl);
            Ok(())
        } else {
            Err(McpError::SessionNotFound(session_id.to_string()))
        }
    }

    /// 清理过期会话
    pub async fn cleanup_expired_sessions(&self) -> Result<usize> {
        let mut sessions = self.sessions.write().await;
        let before = sessions.len();
        sessions.retain(|_, s| !s.is_expired());
        Ok(before - sessions.len())
    }

    /// 更新连接活跃时间
    pub async fn update_connection_active(&self, connection_id: &str) -> Result<()> {
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(connection_id) {
            conn.update_active();
            Ok(())
        } else {
            Err(McpError::ConnectionNotFound(connection_id.to_string()))
        }
    }

    /// 获取设备所有连接
    pub async fn get_device_connections(&self, device_id: &str) -> Vec<DeviceConnection> {
        let device_connections = self.device_connections.read().await;
        let connections = self.connections.read().await;

        device_connections
            .get(device_id)
            .map(|ids| ids.iter().filter_map(|id| connections.get(id).cloned()).collect())
            .unwrap_or_default()
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> DeviceStats {
        let devices = self.devices.read().await;
        let connections = self.connections.read().await;
        let sessions = self.sessions.read().await;

        let mut stats = DeviceStats {
            total: devices.len(),
            connections: connections.len(),
            active_sessions: sessions.len(),
            ..Default::default()
        };

        for device in devices.values() {
            match device.status {
                DeviceStatus::Online => stats.online += 1,
                DeviceStatus::Offline => stats.offline += 1,
                DeviceStatus::Error => stats.error += 1,
                DeviceStatus::Maintenance => stats.maintenance += 1,
                _ => {},
            }
        }

        stats
    }

    /// 按类型列出设备
    pub async fn list_devices_by_type(&self, device_type: &DeviceType) -> Vec<Device> {
        let devices = self.devices.read().await;
        devices.values().filter(|d| d.device_type == *device_type).cloned().collect()
    }

    /// 列出所有设备
    pub async fn list_all_devices(&self) -> Vec<Device> {
        let devices = self.devices.read().await;
        devices.values().cloned().collect()
    }
}

impl Default for EnhancedDeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_device() {
        let manager = EnhancedDeviceManager::new();
        let device = Device::default();

        let id = manager.register_device(device).await.unwrap();
        assert!(!id.is_empty());

        let stats = manager.get_stats().await;
        assert_eq!(stats.total, 1);
    }

    #[tokio::test]
    async fn test_unregister_device() {
        let manager = EnhancedDeviceManager::new();
        let device = Device::default();
        let id = device.id.clone();

        manager.register_device(device).await.unwrap();
        manager.unregister_device(&id).await.unwrap();

        let stats = manager.get_stats().await;
        assert_eq!(stats.total, 0);
    }

    #[tokio::test]
    async fn test_device_connection() {
        let manager = EnhancedDeviceManager::new();
        let device = Device::default();
        let device_id = device.id.clone();

        manager.register_device(device).await.unwrap();

        let conn_id = manager
            .connect(device_id.clone(), "127.0.0.1:8080".to_string(), "tcp".to_string())
            .await
            .unwrap();
        assert!(!conn_id.is_empty());

        let device = manager.get_device(&device_id).await.unwrap();
        assert_eq!(device.status, DeviceStatus::Online);
    }

    #[tokio::test]
    async fn test_device_session() {
        let manager = EnhancedDeviceManager::new();
        let device = Device::default();
        let device_id = device.id.clone();

        manager.register_device(device).await.unwrap();

        let session_id = manager.create_session(device_id).await.unwrap();
        let session = manager.get_session(&session_id).await.unwrap();
        assert!(!session.is_expired());
    }

    #[tokio::test]
    async fn test_session_expiry() {
        let mut session = DeviceSession::new("device-1".to_string(), 0);
        session.expires_at = Utc::now() - chrono::Duration::seconds(1);
        assert!(session.is_expired());
    }

    #[tokio::test]
    async fn test_connection_timeout() {
        let mut conn = DeviceConnection::new(
            "device-1".to_string(),
            "127.0.0.1:8080".to_string(),
            "tcp".to_string(),
        );
        conn.last_active = Utc::now() - chrono::Duration::seconds(60);
        assert!(conn.is_timeout(30));
    }

    #[tokio::test]
    async fn test_device_stats() {
        let manager = EnhancedDeviceManager::new();

        let device1 = Device::default();
        let id1 = device1.id.clone();

        let device2 = Device::default();
        let id2 = device2.id.clone();

        manager.register_device(device1).await.unwrap();
        manager.register_device(device2).await.unwrap();

        // 手动更新状态
        manager.update_device_status(&id1, DeviceStatus::Online).await.unwrap();

        let stats = manager.get_stats().await;
        assert_eq!(stats.total, 2);
        assert_eq!(stats.online, 1);
        assert_eq!(stats.offline, 1);
    }

    #[tokio::test]
    async fn test_list_devices_by_type() {
        let manager = EnhancedDeviceManager::new();

        let mut device1 = Device::default();
        device1.device_type = DeviceType::IoT;

        let mut device2 = Device::default();
        device2.device_type = DeviceType::Computer;

        manager.register_device(device1).await.unwrap();
        manager.register_device(device2).await.unwrap();

        let iot_devices = manager.list_devices_by_type(&DeviceType::IoT).await;
        assert_eq!(iot_devices.len(), 1);

        let computer_devices = manager.list_devices_by_type(&DeviceType::Computer).await;
        assert_eq!(computer_devices.len(), 1);
    }

    #[test]
    fn test_device_connection_creation() {
        let conn = DeviceConnection::new(
            "device-1".to_string(),
            "192.168.1.1:8080".to_string(),
            "grpc".to_string(),
        );

        assert!(!conn.connection_id.is_empty());
        assert_eq!(conn.protocol, "grpc");
        assert_eq!(conn.status, ConnectionStatus::Connected);
    }

    #[test]
    fn test_session_data() {
        let mut session = DeviceSession::new("device-1".to_string(), 3600);
        session.set_data("key1".to_string(), serde_json::json!("value1"));

        assert_eq!(session.get_data("key1"), Some(&serde_json::json!("value1")));
        assert_eq!(session.get_data("key2"), None);
    }
}
