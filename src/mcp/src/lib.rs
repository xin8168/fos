//! # FOS MCP - 神经末梢层
//!
//! FOS神经元控制器的第四层神经网络：神经末梢/效应器
//!
//! 类比于人体神经系统的神经末梢，负责将神经信号转换为设备动作
//!
//! ## 核心职责
//! - 设备注册与管理（统一设备接入）
//! - 心跳监控（设备健康状态）
//! - 动作执行（具体设备控制）
//! - 离线缓存（网络中断时本地执行）
//!
//! ## MCP协议
//! MCP (Model Context Protocol) - 统一的设备控制协议
//!
//! ## 信号处理流程
//! 设备注册 → 心跳监控 → 动作接收 → 设备执行 → 状态反馈

pub mod device;
pub mod enhanced_manager;
pub mod error;
pub mod heartbeat;
pub mod manager;
pub mod offline_cache;
pub mod protocol;

pub use device::{Device, DeviceStatus, DeviceType};
pub use enhanced_manager::{
    ConnectionStatus, DeviceConnection, DeviceManagerConfig, DeviceSession, DeviceStats,
    EnhancedDeviceManager,
};
pub use error::{McpError, Result};
pub use heartbeat::{
    DeviceHeartbeatState, HeartbeatConfig, HeartbeatManager, HeartbeatRecord, HeartbeatStats,
    HeartbeatStatus,
};
pub use manager::DeviceManager;
pub use offline_cache::{
    CacheItem, CacheItemStatus, CachePriority, CacheStats, OfflineCacheConfig, OfflineCacheManager,
};

/// 生成设备ID
pub fn device_id() -> String {
    format!("dev-{}", chrono::Utc::now().timestamp_millis())
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_id() {
        let id = device_id();
        assert!(id.starts_with("dev-"));
    }
}
