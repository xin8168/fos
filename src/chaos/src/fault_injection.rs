//! 故障注入模块

use serde::{Deserialize, Serialize};

/// 故障类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum FaultType {
    /// 服务不可用
    ServiceUnavailable,
    /// 超时
    Timeout,
    /// 数据损坏
    DataCorruption,
    /// 资源耗尽
    ResourceExhaustion,
    /// 网络分区
    NetworkPartition,
    /// 时钟偏移
    ClockSkew,
}

impl FaultType {
    pub fn name(&self) -> &str {
        match self {
            FaultType::ServiceUnavailable => "ServiceUnavailable",
            FaultType::Timeout => "Timeout",
            FaultType::DataCorruption => "DataCorruption",
            FaultType::ResourceExhaustion => "ResourceExhaustion",
            FaultType::NetworkPartition => "NetworkPartition",
            FaultType::ClockSkew => "ClockSkew",
        }
    }
}

/// 故障注入器
pub struct FaultInjector {
    name: String,
    injected_faults: Vec<FaultType>,
}

impl FaultInjector {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), injected_faults: Vec::new() }
    }

    pub fn inject_fault(&mut self, fault_type: FaultType) {
        self.injected_faults.push(fault_type);
    }

    pub fn clear_faults(&mut self) {
        self.injected_faults.clear();
    }

    pub fn get_injected_faults(&self) -> &[FaultType] {
        &self.injected_faults
    }

    pub fn simulate_service_unavailable(&self) -> bool {
        self.injected_faults.contains(&FaultType::ServiceUnavailable)
    }

    pub fn simulate_timeout(&self) -> bool {
        self.injected_faults.contains(&FaultType::Timeout)
    }

    pub fn simulate_data_corruption(&self) -> bool {
        self.injected_faults.contains(&FaultType::DataCorruption)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fault_type_name() {
        assert_eq!(FaultType::ServiceUnavailable.name(), "ServiceUnavailable");
        assert_eq!(FaultType::Timeout.name(), "Timeout");
    }

    #[test]
    fn test_fault_injector() {
        let mut injector = FaultInjector::new("TestInjector");
        assert!(injector.get_injected_faults().is_empty());

        injector.inject_fault(FaultType::ServiceUnavailable);
        assert_eq!(injector.get_injected_faults().len(), 1);
        assert!(injector.simulate_service_unavailable());
        assert!(!injector.simulate_timeout());

        injector.clear_faults();
        assert!(injector.get_injected_faults().is_empty());
    }
}
