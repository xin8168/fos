//! FOS Monitoring - 监控模块

pub mod metrics;
pub mod health;
pub mod alert;

pub use metrics::MetricsCollector;
pub use health::HealthChecker;
pub use alert::AlertManager;

use serde::{Deserialize, Serialize};

/// 监控状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorStatus {
    pub healthy: bool,
    pub uptime_secs: u64,
    pub metrics: serde_json::Value,
}

impl Default for MonitorStatus {
    fn default() -> Self {
        Self {
            healthy: true,
            uptime_secs: 0,
            metrics: serde_json::json!({}),
        }
    }
}
