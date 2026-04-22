//! 熔断器核心类型定义

/// 熔断器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// 关闭状态（正常工作）
    Closed,
    /// 开启状态（熔断中）
    Open,
    /// 半开状态（尝试恢复）
    HalfOpen,
}

/// 熔断器配置
#[derive(Debug, Clone)]
pub struct CircuitConfig {
    /// 失败阈值（连续失败次数达到此值时熔断）
    pub failure_threshold: u32,
    /// 成功阈值（半开状态下连续成功次数达到此值时恢复）
    pub success_threshold: u32,
    /// 超时时间（闭路请求超时算失败，毫秒）
    pub timeout_ms: u64,
    /// 半开状态持续时间（毫秒）
    pub half_open_timeout_ms: u64,
    /// 开路状态持续时间（毫秒）
    pub open_timeout_ms: u64,
}

impl Default for CircuitConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout_ms: 30000,           // 30秒
            half_open_timeout_ms: 60000, // 60秒
            open_timeout_ms: 60000,      // 60秒
        }
    }
}

impl CircuitConfig {
    /// 创建新的熔断器配置
    pub fn new(
        failure_threshold: u32,
        success_threshold: u32,
        timeout_ms: u64,
        half_open_timeout_ms: u64,
        open_timeout_ms: u64,
    ) -> Self {
        Self {
            failure_threshold,
            success_threshold,
            timeout_ms,
            half_open_timeout_ms,
            open_timeout_ms,
        }
    }

    /// 验证配置
    pub fn validate(&self) -> crate::error::Result<()> {
        if self.failure_threshold == 0 {
            return Err(crate::error::Error::Config("failure_threshold must be > 0".to_string()));
        }
        if self.success_threshold == 0 {
            return Err(crate::error::Error::Config("success_threshold must be > 0".to_string()));
        }
        if self.timeout_ms == 0 {
            return Err(crate::error::Error::Config("timeout_ms must be > 0".to_string()));
        }
        Ok(())
    }
}

/// 请求结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestResult {
    /// 成功
    Success,
    /// 失败
    Failure,
    /// 超时
    Timeout,
}

/// 熔断器状态
#[derive(Debug, Clone)]
pub struct CircuitStatus {
    /// 当前状态
    pub state: CircuitState,
    /// 连续失败次数
    pub consecutive_failures: u32,
    /// 半开状态连续成功次数
    pub consecutive_successes: u32,
    /// 最后状态变更时间（毫秒时间戳）
    pub last_state_change: u64,
}

/// 熔断器决策
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitDecision {
    /// 允许通过
    Allow,
    /// 拒绝（已熔断）
    Reject {
        /// 重试时间（毫秒）
        retry_after_ms: u64,
    },
}

impl CircuitDecision {
    /// 检查是否允许通过
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allow)
    }

    /// 获取重试时间
    pub fn retry_after_ms(&self) -> Option<u64> {
        match self {
            Self::Allow => None,
            Self::Reject { retry_after_ms } => Some(*retry_after_ms),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_config_default() {
        let config = CircuitConfig::default();
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.success_threshold, 2);
        assert_eq!(config.timeout_ms, 30000);
        assert_eq!(config.half_open_timeout_ms, 60000);
        assert_eq!(config.open_timeout_ms, 60000);
    }

    #[test]
    fn test_circuit_config_validation() {
        // 有效配置
        let config = CircuitConfig::new(5, 2, 30000, 60000, 60000);
        assert!(config.validate().is_ok());

        // 无效配置：failure_threshold为0
        let invalid1 = CircuitConfig { failure_threshold: 0, ..Default::default() };
        assert!(invalid1.validate().is_err());

        // 无效配置：success_threshold为0
        let invalid2 = CircuitConfig { success_threshold: 0, ..Default::default() };
        assert!(invalid2.validate().is_err());
    }

    #[test]
    fn test_circuit_decision() {
        let allow = CircuitDecision::Allow;
        assert!(allow.is_allowed());
        assert!(allow.retry_after_ms().is_none());

        let reject = CircuitDecision::Reject { retry_after_ms: 1000 };
        assert!(!reject.is_allowed());
        assert_eq!(reject.retry_after_ms(), Some(1000));
    }
}
