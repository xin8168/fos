//! 设备心跳模块
//!
//! 提供设备心跳检测和状态监控功能

use crate::enhanced_manager::EnhancedDeviceManager;
use crate::error::{McpError, Result};
use crate::{Device, DeviceStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 心跳状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HeartbeatStatus {
    /// 正常
    Healthy,
    /// 警告
    Warning,
    /// 超时
    Timeout,
    /// 失联
    Lost,
}

/// 心跳记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatRecord {
    /// 设备ID
    pub device_id: String,
    /// 心跳时间
    pub timestamp: DateTime<Utc>,
    /// 心跳状态
    pub status: HeartbeatStatus,
    /// 延迟（毫秒）
    pub latency_ms: u64,
    /// 设备状态
    pub device_status: DeviceStatus,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl HeartbeatRecord {
    /// 创建新心跳记录
    pub fn new(device_id: String, latency_ms: u64) -> Self {
        Self {
            device_id,
            timestamp: Utc::now(),
            status: HeartbeatStatus::Healthy,
            latency_ms,
            device_status: DeviceStatus::Online,
            metadata: HashMap::new(),
        }
    }

    /// 添加元数据
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// 设置状态
    pub fn with_status(mut self, status: HeartbeatStatus) -> Self {
        self.status = status;
        self
    }

    /// 设置设备状态
    pub fn with_device_status(mut self, status: DeviceStatus) -> Self {
        self.device_status = status;
        self
    }
}

use std::collections::HashMap;

/// 心跳配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatConfig {
    /// 心跳间隔（秒）
    pub interval_secs: u64,
    /// 超时阈值（秒）
    pub timeout_secs: u64,
    /// 警告阈值（秒）
    pub warning_secs: u64,
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试间隔（秒）
    pub retry_interval_secs: u64,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            interval_secs: 10,
            timeout_secs: 30,
            warning_secs: 20,
            max_retries: 3,
            retry_interval_secs: 5,
        }
    }
}

/// 设备心跳状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceHeartbeatState {
    /// 设备ID
    pub device_id: String,
    /// 最后心跳时间
    pub last_heartbeat: DateTime<Utc>,
    /// 连续失败次数
    pub consecutive_failures: u32,
    /// 当前状态
    pub status: HeartbeatStatus,
    /// 平均延迟
    pub avg_latency_ms: u64,
    /// 心跳计数
    pub heartbeat_count: u64,
    /// 失败计数
    pub failure_count: u64,
}

impl DeviceHeartbeatState {
    /// 创建新状态
    pub fn new(device_id: String) -> Self {
        Self {
            device_id,
            last_heartbeat: Utc::now(),
            consecutive_failures: 0,
            status: HeartbeatStatus::Healthy,
            avg_latency_ms: 0,
            heartbeat_count: 0,
            failure_count: 0,
        }
    }

    /// 更新心跳
    pub fn update_heartbeat(&mut self, latency_ms: u64) {
        self.last_heartbeat = Utc::now();
        self.consecutive_failures = 0;
        self.heartbeat_count += 1;
        self.status = HeartbeatStatus::Healthy;

        // 计算平均延迟
        let total = self.avg_latency_ms * (self.heartbeat_count - 1) + latency_ms;
        self.avg_latency_ms = total / self.heartbeat_count;
    }

    /// 记录失败
    pub fn record_failure(&mut self) {
        self.consecutive_failures += 1;
        self.failure_count += 1;
    }

    /// 检查状态
    pub fn check_status(&self, config: &HeartbeatConfig) -> HeartbeatStatus {
        let elapsed = (Utc::now() - self.last_heartbeat).num_seconds() as u64;

        if elapsed > config.timeout_secs {
            HeartbeatStatus::Timeout
        } else if elapsed > config.warning_secs {
            HeartbeatStatus::Warning
        } else if self.consecutive_failures >= config.max_retries {
            HeartbeatStatus::Lost
        } else {
            self.status.clone()
        }
    }
}

/// 心跳管理器
pub struct HeartbeatManager {
    /// 设备心跳状态
    states: Arc<RwLock<HashMap<String, DeviceHeartbeatState>>>,
    /// 配置
    config: HeartbeatConfig,
    /// 心跳历史记录
    history: Arc<RwLock<Vec<HeartbeatRecord>>>,
}

impl HeartbeatManager {
    /// 创建新心跳管理器
    pub fn new() -> Self {
        Self::with_config(HeartbeatConfig::default())
    }

    /// 使用配置创建
    pub fn with_config(config: HeartbeatConfig) -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
            config,
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 注册设备
    pub async fn register_device(&self, device_id: String) {
        let state = DeviceHeartbeatState::new(device_id.clone());
        let mut states = self.states.write().await;
        states.insert(device_id, state);
    }

    /// 注销设备
    pub async fn unregister_device(&self, device_id: &str) {
        let mut states = self.states.write().await;
        states.remove(device_id);
    }

    /// 接收心跳
    pub async fn receive_heartbeat(
        &self,
        device_id: &str,
        latency_ms: u64,
    ) -> Result<HeartbeatStatus> {
        let mut states = self.states.write().await;

        if let Some(state) = states.get_mut(device_id) {
            state.update_heartbeat(latency_ms);

            // 记录心跳历史
            let record = HeartbeatRecord::new(device_id.to_string(), latency_ms);
            let mut history = self.history.write().await;
            history.push(record);

            Ok(HeartbeatStatus::Healthy)
        } else {
            Err(McpError::DeviceNotFound(device_id.to_string()))
        }
    }

    /// 检查设备状态
    pub async fn check_device_status(&self, device_id: &str) -> Result<HeartbeatStatus> {
        let states = self.states.read().await;

        if let Some(state) = states.get(device_id) {
            Ok(state.check_status(&self.config))
        } else {
            Err(McpError::DeviceNotFound(device_id.to_string()))
        }
    }

    /// 检查所有设备状态
    pub async fn check_all_devices(&self) -> HashMap<String, HeartbeatStatus> {
        let states = self.states.read().await;
        let mut result = HashMap::new();

        for (device_id, state) in states.iter() {
            result.insert(device_id.clone(), state.check_status(&self.config));
        }

        result
    }

    /// 获取设备心跳状态
    pub async fn get_device_state(&self, device_id: &str) -> Result<DeviceHeartbeatState> {
        let states = self.states.read().await;
        states
            .get(device_id)
            .cloned()
            .ok_or_else(|| McpError::DeviceNotFound(device_id.to_string()))
    }

    /// 获取所有设备心跳状态
    pub async fn get_all_states(&self) -> Vec<DeviceHeartbeatState> {
        let states = self.states.read().await;
        states.values().cloned().collect()
    }

    /// 获取心跳历史
    pub async fn get_history(&self, device_id: Option<&str>) -> Vec<HeartbeatRecord> {
        let history = self.history.read().await;
        match device_id {
            Some(id) => history.iter().filter(|r| r.device_id == id).cloned().collect(),
            None => history.clone(),
        }
    }

    /// 清理过期历史记录
    pub async fn cleanup_history(&self, max_age_secs: u64) -> usize {
        let cutoff = Utc::now() - chrono::Duration::seconds(max_age_secs as i64);
        let mut history = self.history.write().await;
        let before = history.len();
        history.retain(|r| r.timestamp > cutoff);
        before - history.len()
    }

    /// 标记设备失败
    pub async fn mark_device_failure(&self, device_id: &str) -> Result<()> {
        let mut states = self.states.write().await;

        if let Some(state) = states.get_mut(device_id) {
            state.record_failure();
            Ok(())
        } else {
            Err(McpError::DeviceNotFound(device_id.to_string()))
        }
    }

    /// 获取心跳统计
    pub async fn get_stats(&self) -> HeartbeatStats {
        let states = self.states.read().await;

        let mut stats = HeartbeatStats::default();
        stats.total_devices = states.len();

        for state in states.values() {
            match state.check_status(&self.config) {
                HeartbeatStatus::Healthy => stats.healthy += 1,
                HeartbeatStatus::Warning => stats.warning += 1,
                HeartbeatStatus::Timeout => stats.timeout += 1,
                HeartbeatStatus::Lost => stats.lost += 1,
            }
        }

        stats
    }
}

impl Default for HeartbeatManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 心跳统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HeartbeatStats {
    /// 总设备数
    pub total_devices: usize,
    /// 健康设备数
    pub healthy: usize,
    /// 警告设备数
    pub warning: usize,
    /// 超时设备数
    pub timeout: usize,
    /// 失联设备数
    pub lost: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heartbeat_record() {
        let record = HeartbeatRecord::new("device-1".to_string(), 100);
        assert_eq!(record.device_id, "device-1");
        assert_eq!(record.latency_ms, 100);
        assert_eq!(record.status, HeartbeatStatus::Healthy);
    }

    #[test]
    fn test_device_heartbeat_state() {
        let mut state = DeviceHeartbeatState::new("device-1".to_string());
        state.update_heartbeat(50);

        assert_eq!(state.heartbeat_count, 1);
        assert_eq!(state.consecutive_failures, 0);
    }

    #[test]
    fn test_heartbeat_config_default() {
        let config = HeartbeatConfig::default();
        assert_eq!(config.interval_secs, 10);
        assert_eq!(config.timeout_secs, 30);
    }

    #[tokio::test]
    async fn test_heartbeat_manager_register() {
        let manager = HeartbeatManager::new();
        manager.register_device("device-1".to_string()).await;

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_devices, 1);
    }

    #[tokio::test]
    async fn test_heartbeat_manager_receive() {
        let manager = HeartbeatManager::new();
        manager.register_device("device-1".to_string()).await;

        let status = manager.receive_heartbeat("device-1", 100).await.unwrap();
        assert_eq!(status, HeartbeatStatus::Healthy);

        let state = manager.get_device_state("device-1").await.unwrap();
        assert_eq!(state.heartbeat_count, 1);
    }

    #[tokio::test]
    async fn test_heartbeat_manager_failure() {
        let manager = HeartbeatManager::new();
        manager.register_device("device-1".to_string()).await;

        manager.mark_device_failure("device-1").await.unwrap();
        manager.mark_device_failure("device-1").await.unwrap();

        let state = manager.get_device_state("device-1").await.unwrap();
        assert_eq!(state.consecutive_failures, 2);
    }

    #[tokio::test]
    async fn test_heartbeat_manager_unregister() {
        let manager = HeartbeatManager::new();
        manager.register_device("device-1".to_string()).await;
        manager.unregister_device("device-1").await;

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_devices, 0);
    }

    #[tokio::test]
    async fn test_heartbeat_history() {
        let manager = HeartbeatManager::new();
        manager.register_device("device-1".to_string()).await;

        manager.receive_heartbeat("device-1", 100).await.unwrap();
        manager.receive_heartbeat("device-1", 200).await.unwrap();

        let history = manager.get_history(Some("device-1")).await;
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_heartbeat_status_check() {
        let config = HeartbeatConfig::default();
        let mut state = DeviceHeartbeatState::new("device-1".to_string());

        // 正常状态
        assert_eq!(state.check_status(&config), HeartbeatStatus::Healthy);

        // 模拟超时
        state.last_heartbeat = Utc::now() - chrono::Duration::seconds(40);
        assert_eq!(state.check_status(&config), HeartbeatStatus::Timeout);
    }
}
