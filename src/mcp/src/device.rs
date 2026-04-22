//! 设备结构定义

use serde::{Deserialize, Serialize};

/// 设备类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    Computer,
    IoT,
    Industrial,
    Mobile,
    Network,
    Other,
}

/// 设备状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeviceStatus {
    Online,
    Offline,
    Busy,
    Error,
    Maintenance,
}

/// 设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
    pub status: DeviceStatus,
    pub capabilities: Vec<String>,
    pub config: serde_json::Value,
}

impl Default for Device {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            device_type: DeviceType::Other,
            status: DeviceStatus::Offline,
            capabilities: vec![],
            config: serde_json::json!({}),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_creation() {
        let device = Device::default();
        assert!(!device.id.is_empty()); // UUID格式
        assert_eq!(device.device_type, DeviceType::Other);
        assert_eq!(device.status, DeviceStatus::Offline);
    }
}
