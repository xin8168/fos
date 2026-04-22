//! 设备管理

use crate::error::{McpError, Result};
use crate::{Device, DeviceStatus};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 设备管理器
pub struct DeviceManager {
    devices: Arc<RwLock<HashMap<String, Device>>>,
}

impl DeviceManager {
    pub fn new() -> Self {
        Self { devices: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn register(&self, device: Device) -> Result<String> {
        let id = device.id.clone();
        let mut devices = self.devices.write().await;
        devices.insert(id.clone(), device);
        Ok(id)
    }

    pub async fn unregister(&self, id: &str) -> Result<()> {
        let mut devices = self.devices.write().await;
        devices.remove(id).ok_or_else(|| McpError::DeviceNotFound(id.to_string()))?;
        Ok(())
    }

    pub async fn get(&self, id: &str) -> Result<Device> {
        let devices = self.devices.read().await;
        devices.get(id).cloned().ok_or_else(|| McpError::DeviceNotFound(id.to_string()))
    }

    pub async fn update_status(&self, id: &str, status: DeviceStatus) -> Result<()> {
        let mut devices = self.devices.write().await;
        if let Some(device) = devices.get_mut(id) {
            device.status = status;
            Ok(())
        } else {
            Err(McpError::DeviceNotFound(id.to_string()))
        }
    }

    pub async fn list(&self) -> Vec<Device> {
        let devices = self.devices.read().await;
        devices.values().cloned().collect()
    }
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}
