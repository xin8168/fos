//! 熔断器实现

use super::circuit::{CircuitConfig, CircuitDecision, CircuitState, CircuitStatus};
use std::sync::atomic::{AtomicI64, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// 熔断器
///
/// Circuit Breaker模式：
/// 1. Closed（关闭）: 正常工作，请求通过，记录失败次数
/// 2. Open（开启）: 熔断中，拒绝所有请求，等待open_timeout_ms后进入HalfOpen
/// 3. HalfOpen（半开）: 尝试恢复，允许部分请求通过，成功则恢复Closed，失败则回到Open
pub struct CircuitBreaker {
    /// 配置
    config: CircuitConfig,
    /// 当前状态
    state: AtomicI64, // 存储CircuitState枚举值：0=Closed, 1=Open, 2=HalfOpen
    /// 连续失败次数
    consecutive_failures: Arc<AtomicU32>,
    /// 半开状态连续成功次数
    consecutive_successes: Arc<AtomicU32>,
    /// 最后状态变更时间（纳秒时间戳）
    last_state_change: Arc<AtomicI64>,
}

impl CircuitBreaker {
    /// 创建新的熔断器
    pub fn new(config: CircuitConfig) -> Self {
        Self {
            config,
            state: AtomicI64::new(0), // Closed
            consecutive_failures: Arc::new(AtomicU32::new(0)),
            consecutive_successes: Arc::new(AtomicU32::new(0)),
            last_state_change: Arc::new(AtomicI64::new(Self::now_nanos())),
        }
    }

    /// 使用默认配置创建
    pub fn with_default_config() -> Self {
        Self::new(CircuitConfig::default())
    }

    /// 尝试执行请求（返回是否允许）
    pub fn try_execute<F, R, E>(&self, f: F) -> std::result::Result<R, ExecutionError<E>>
    where
        F: FnOnce() -> std::result::Result<R, E>,
    {
        // 检查是否允许通过
        let decision = self.decide();
        if !decision.is_allowed() {
            return Err(ExecutionError::CircuitOpen {
                retry_after_ms: decision.retry_after_ms().unwrap_or(0),
            });
        }

        // 执行请求
        let result = f();

        // 根据结果更新状态
        match &result {
            Ok(_) => self.on_success(),
            Err(_) => self.on_failure(),
        }

        result.map_err(|e| ExecutionError::ExecutionFailed(e))
    }

    /// 异步版本（通过tokio::spawn模拟）
    pub async fn try_execute_async<F, R, E, Fut>(
        &self,
        f: F,
    ) -> std::result::Result<R, ExecutionError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = std::result::Result<R, E>>,
    {
        // 检查是否允许通过
        let decision = self.decide();
        if !decision.is_allowed() {
            return Err(ExecutionError::CircuitOpen {
                retry_after_ms: decision.retry_after_ms().unwrap_or(0),
            });
        }

        // 执行请求
        let result = f().await;

        // 根据结果更新状态
        match &result {
            Ok(_) => self.on_success(),
            Err(_) => self.on_failure(),
        }

        result.map_err(|e| ExecutionError::ExecutionFailed(e))
    }

    /// 决定是否允许请求通过
    pub fn decide(&self) -> CircuitDecision {
        let current_state_nanos = CircuitBreaker::now_nanos();
        let last_change = self.last_state_change.load(Ordering::Relaxed);
        let elapsed_ms = (current_state_nanos - last_change) / 1_000_000;

        let state_value = self.state.load(Ordering::Relaxed);
        let state = Self::state_from_value(state_value);

        match state {
            CircuitState::Closed => {
                // 关闭状态：总是允许
                CircuitDecision::Allow
            },
            CircuitState::Open => {
                // 开启状态：检查是否超时
                if elapsed_ms >= self.config.open_timeout_ms as i64 {
                    // 超时，转入半开状态
                    self.transition_to(CircuitState::HalfOpen);
                    CircuitDecision::Allow
                } else {
                    // 仍在熔断中，拒绝请求
                    let retry_after = (self.config.open_timeout_ms as i64 - elapsed_ms) as u64;
                    CircuitDecision::Reject { retry_after_ms: retry_after }
                }
            },
            CircuitState::HalfOpen => {
                // 半开状态：允许部分请求通过
                CircuitDecision::Allow
            },
        }
    }

    /// 处理成功
    pub fn on_success(&self) {
        let state_value = self.state.load(Ordering::Relaxed);
        let state = Self::state_from_value(state_value);

        match state {
            CircuitState::Closed => {
                // 关闭状态：重置失败计数
                self.consecutive_failures.store(0, Ordering::Relaxed);
            },
            CircuitState::Open => {
                // 开启状态：不应该调用
                tracing::warn!("on_success called while circuit is open");
            },
            CircuitState::HalfOpen => {
                // 半开状态：记录成功次数
                let successes = self.consecutive_successes.fetch_add(1, Ordering::Relaxed) + 1;

                // 如果达到成功阈值，恢复到关闭状态
                if successes >= self.config.success_threshold {
                    self.transition_to(CircuitState::Closed);
                }
            },
        }
    }

    /// 处理失败
    pub fn on_failure(&self) {
        let state_value = self.state.load(Ordering::Relaxed);
        let state = Self::state_from_value(state_value);

        match state {
            CircuitState::Closed => {
                // 关闭状态：记录失败次数
                let failures = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;

                // 如果达到失败阈值，熔断
                if failures >= self.config.failure_threshold {
                    self.transition_to(CircuitState::Open);
                }
            },
            CircuitState::Open => {
                // 开启状态：不应该调用（请求本来就不应该通过）
                tracing::warn!("on_failure called while circuit is open");
            },
            CircuitState::HalfOpen => {
                // 半开状态：失败则回到开启状态
                self.transition_to(CircuitState::Open);
            },
        }
    }

    /// 处理超时
    pub fn on_timeout(&self) {
        // 超时也视为失败
        self.on_failure();
    }

    /// 获取当前状态
    pub fn get_state(&self) -> CircuitState {
        let state_value = self.state.load(Ordering::Relaxed);
        Self::state_from_value(state_value)
    }

    /// 获取完整状态
    pub fn get_status(&self) -> CircuitStatus {
        let state_value = self.state.load(Ordering::Relaxed);
        CircuitStatus {
            state: Self::state_from_value(state_value),
            consecutive_failures: self.consecutive_failures.load(Ordering::Relaxed),
            consecutive_successes: self.consecutive_successes.load(Ordering::Relaxed),
            last_state_change: self.last_state_change.load(Ordering::Relaxed) as u64,
        }
    }

    /// 重置熔断器到关闭状态
    pub fn reset(&self) {
        self.transition_to(CircuitState::Closed);
    }

    /// 检查是否熔断中
    pub fn is_open(&self) -> bool {
        self.get_state() == CircuitState::Open
    }

    /// 检查是否关闭
    pub fn is_closed(&self) -> bool {
        self.get_state() == CircuitState::Closed
    }

    /// 检查是否半开
    pub fn is_half_open(&self) -> bool {
        self.get_state() == CircuitState::HalfOpen
    }

    /// 状态转换
    fn transition_to(&self, new_state: CircuitState) {
        if Self::state_from_value(self.state.load(Ordering::Relaxed)) != new_state {
            tracing::info!(
                "Circuit breaker state transition: {:?} -> {:?}",
                self.get_state(),
                new_state
            );

            self.state.store(Self::state_to_value(new_state), Ordering::Relaxed);
            self.last_state_change.store(Self::now_nanos(), Ordering::Relaxed);

            // 根据新状态重置计数器
            match new_state {
                CircuitState::Closed => {
                    self.consecutive_failures.store(0, Ordering::Relaxed);
                    self.consecutive_successes.store(0, Ordering::Relaxed);
                },
                CircuitState::Open => {
                    self.consecutive_successes.store(0, Ordering::Relaxed);
                },
                CircuitState::HalfOpen => {
                    self.consecutive_successes.store(0, Ordering::Relaxed);
                },
            }
        }
    }

    /// 将CircuitState枚举转换为数值
    fn state_to_value(state: CircuitState) -> i64 {
        match state {
            CircuitState::Closed => 0,
            CircuitState::Open => 1,
            CircuitState::HalfOpen => 2,
        }
    }

    /// 从数值转换为CircuitState枚举
    fn state_from_value(value: i64) -> CircuitState {
        match value {
            0 => CircuitState::Closed,
            1 => CircuitState::Open,
            2 => CircuitState::HalfOpen,
            _ => CircuitState::Closed,
        }
    }

    /// 获取当前时间戳（纳秒）
    fn now_nanos() -> i64 {
        Instant::now().elapsed().as_nanos() as i64
    }
}

/// 执行错误
#[derive(Debug, Clone)]
pub enum ExecutionError<E> {
    /// 熔断器开启
    CircuitOpen {
        /// 重试时间（毫秒）
        retry_after_ms: u64,
    },
    /// 执行失败
    ExecutionFailed(E),
}

impl<E> ExecutionError<E> {
    /// 检查是否成功（非CircuitOpen）
    pub fn is_ok(&self) -> bool {
        !matches!(self, Self::CircuitOpen { .. })
    }
}

impl<E> std::fmt::Display for ExecutionError<E>
where
    E: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CircuitOpen { retry_after_ms } => {
                write!(f, "Circuit breaker is open, retry after {}ms", retry_after_ms)
            },
            Self::ExecutionFailed(e) => write!(f, "Execution failed: {}", e),
        }
    }
}

impl<E: std::error::Error> std::error::Error for ExecutionError<E> {}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_breaker(threshold: u32) -> CircuitBreaker {
        CircuitBreaker::new(CircuitConfig {
            failure_threshold: threshold,
            success_threshold: 2,
            timeout_ms: 1000,
            half_open_timeout_ms: 1000,
            open_timeout_ms: 100, // 100ms用于快速测试
        })
    }

    #[test]
    fn test_circuit_breaker_creation() {
        let breaker = CircuitBreaker::with_default_config();
        assert!(breaker.is_closed());
        assert!(!breaker.is_open());
        assert!(!breaker.is_half_open());
    }

    #[test]
    fn test_circuit_breaker_fails_until_threshold() {
        let breaker = create_test_breaker(3);

        // 前两次失败应该保持关闭状态
        breaker.on_failure();
        assert!(breaker.is_closed());
        assert_eq!(breaker.get_status().consecutive_failures, 1);

        breaker.on_failure();
        assert!(breaker.is_closed());
        assert_eq!(breaker.get_status().consecutive_failures, 2);

        // 第三次失败应该触发熔断
        breaker.on_failure();
        assert!(breaker.is_open());
        assert_eq!(breaker.get_status().consecutive_failures, 3);
    }

    #[test]
    fn test_circuit_breaker_allows_requests() {
        let breaker = create_test_breaker(3);

        // 关闭状态应该允许
        let decision_1 = breaker.decide();
        assert!(decision_1.is_allowed());

        // 模拟两次失败（未达到阈值）
        breaker.on_failure();
        breaker.on_failure();
        assert!(breaker.is_closed());

        // 应该仍然允许
        let decision_2 = breaker.decide();
        assert!(decision_2.is_allowed());
    }

    #[test]
    fn test_circuit_breaker_opens_on_threshold() {
        let breaker = create_test_breaker(2);

        // 两次失败应该触发熔断
        breaker.on_failure();
        breaker.on_failure();

        assert!(breaker.is_open());

        // 开启状态应该拒绝
        let decision = breaker.decide();
        assert!(!decision.is_allowed());
        assert!(decision.retry_after_ms().is_some());
    }

    #[test]
    fn test_circuit_breaker_half_open_recovery() {
        let breaker = create_test_breaker(2);

        // 触发熔断
        breaker.on_failure();
        breaker.on_failure();
        assert!(breaker.is_open());

        // 手动设置状态为半开（模拟时间超时）
        breaker.transition_to(CircuitState::HalfOpen);
        assert!(breaker.is_half_open());

        // 半开状态应该允许请求
        let decision = breaker.decide();
        assert!(decision.is_allowed());

        // 两次成功应该恢复到关闭
        breaker.on_success();
        assert!(breaker.is_half_open());
        assert_eq!(breaker.get_status().consecutive_successes, 1);

        breaker.on_success();
        assert!(breaker.is_closed());
        assert_eq!(breaker.get_status().consecutive_successes, 0);
        assert_eq!(breaker.get_status().consecutive_failures, 0);
    }

    #[test]
    fn test_circuit_breaker_half_open_failure() {
        let breaker = create_test_breaker(2);

        // 触发熔断
        breaker.on_failure();
        breaker.on_failure();
        assert!(breaker.is_open());

        // 手动设置状态为半开
        breaker.transition_to(CircuitState::HalfOpen);

        // 半开状态下失败应该回到开启
        breaker.on_failure();
        assert!(breaker.is_open());
    }

    #[test]
    fn test_circuit_breaker_reset() {
        let breaker = create_test_breaker(2);

        // 触发熔断
        breaker.on_failure();
        breaker.on_failure();
        assert!(breaker.is_open());

        // 重置应该恢复到关闭
        breaker.reset();
        assert!(breaker.is_closed());
        assert_eq!(breaker.get_status().consecutive_failures, 0);
    }

    #[test]
    fn test_circuit_breaker_success_on_closed() {
        let breaker = create_test_breaker(3);

        // 添加一些失败
        breaker.on_failure();
        breaker.on_failure();
        assert_eq!(breaker.get_status().consecutive_failures, 2);

        // 成功应该重置失败计数
        breaker.on_success();
        assert!(breaker.is_closed());
        assert_eq!(breaker.get_status().consecutive_failures, 0);
    }

    #[test]
    fn test_circuit_breaker_timeout() {
        let breaker = create_test_breaker(3); // 改为3，这样才能在第三次触发熔断

        // 超时应该视为失败
        breaker.on_timeout();
        assert!(breaker.is_closed());

        breaker.on_timeout();
        assert!(breaker.is_closed());

        // 第三次超时应该触发熔断
        breaker.on_timeout();
        assert!(breaker.is_open());
    }

    #[test]
    fn test_circuit_breaker_status() {
        let breaker = CircuitBreaker::with_default_config();

        let status = breaker.get_status();
        assert_eq!(status.state, CircuitState::Closed);
        assert_eq!(status.consecutive_failures, 0);
        assert_eq!(status.consecutive_successes, 0);

        // 添加失败
        breaker.on_failure();
        let status = breaker.get_status();
        assert_eq!(status.consecutive_failures, 1);
    }
}
