//! 健康状态定义

use serde::{Deserialize, Serialize};

/// 健康状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    /// 健康
    Healthy,
    /// 降级
    Degraded,
    /// 不健康
    Unhealthy,
    /// 未知
    Unknown,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "健康"),
            HealthStatus::Degraded => write!(f, "降级"),
            HealthStatus::Unhealthy => write!(f, "不健康"),
            HealthStatus::Unknown => write!(f, "未知"),
        }
    }
}

/// 健康级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthLevel {
    /// 健康
    Healthy,
    /// 降级
    Degraded,
    /// 不健康
    Unhealthy,
}

impl HealthLevel {
    /// 转换为状态
    pub fn to_status(&self) -> HealthStatus {
        match self {
            HealthLevel::Healthy => HealthStatus::Healthy,
            HealthLevel::Degraded => HealthStatus::Degraded,
            HealthLevel::Unhealthy => HealthStatus::Unhealthy,
        }
    }
}

impl Default for HealthLevel {
    fn default() -> Self {
        HealthLevel::Healthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_display() {
        assert_eq!(HealthStatus::Healthy.to_string(), "健康");
        assert_eq!(HealthStatus::Unhealthy.to_string(), "不健康");
    }

    #[test]
    fn test_health_level_to_status() {
        assert_eq!(HealthLevel::Healthy.to_status(), HealthStatus::Healthy);
    }
}
