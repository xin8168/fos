//! 限流器trait定义和核心类型

use crate::error::{Error, Result};

/// 限流器trait
#[async_trait::async_trait]
pub trait RateLimiter: Send + Sync {
    /// 尝试获取token
    /// 返回true表示获取成功，false表示被限流
    async fn try_acquire(&self) -> bool;

    /// 尝试获取指定数量的token
    /// 返回实际获取的token数量
    async fn try_acquire_many(&self, tokens: u64) -> u64;

    /// 获取限流器状态
    async fn get_status(&self) -> RateLimiterStatus;

    /// 重置限流器状态
    async fn reset(&self);
}

/// 限流算法类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimiterAlgorithm {
    /// 令牌桶算法
    TokenBucket,
    /// 漏桶算法
    LeakyBucket,
}

/// 限流器状态
#[derive(Debug, Clone)]
pub struct RateLimiterStatus {
    /// 算法类型
    pub algorithm: RateLimiterAlgorithm,
    /// 当前可用的token数量
    pub available_tokens: u64,
    /// 桶的总容量
    pub capacity: u64,
    /// token填充速率（tokens/秒）
    pub refill_rate: u64,
    /// 上次填充时间戳（纳秒）
    pub last_refill: u64,
}

/// 限流结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitResult {
    /// 允许通过
    Allowed,
    /// 被限流
    Limited {
        /// 等待时间（毫秒）
        wait_ms: u64,
    },
}

impl RateLimitResult {
    /// 检查是否允许通过
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed)
    }

    /// 获取等待时间（毫秒）
    pub fn wait_ms(&self) -> Option<u64> {
        match self {
            Self::Allowed => None,
            Self::Limited { wait_ms } => Some(*wait_ms),
        }
    }
}

/// 限流配置
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// 限流算法
    pub algorithm: RateLimiterAlgorithm,
    /// 桶容量
    pub capacity: u64,
    /// 填充速率（tokens/秒）
    pub rate: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self { algorithm: RateLimiterAlgorithm::TokenBucket, capacity: 10, rate: 1 }
    }
}

impl RateLimitConfig {
    /// 创建Token Bucket配置
    pub fn token_bucket(capacity: u64, rate: u64) -> Self {
        Self { algorithm: RateLimiterAlgorithm::TokenBucket, capacity, rate }
    }

    /// 创建Leaky Bucket配置
    pub fn leaky_bucket(capacity: u64, rate: u64) -> Self {
        Self { algorithm: RateLimiterAlgorithm::LeakyBucket, capacity, rate }
    }

    /// 验证配置
    pub fn validate(&self) -> Result<()> {
        if self.capacity == 0 {
            return Err(Error::Config("capacity must be greater than 0".to_string()));
        }
        if self.rate == 0 {
            return Err(Error::Config("rate must be greater than 0".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_result() {
        let allowed = RateLimitResult::Allowed;
        assert!(allowed.is_allowed());
        assert!(allowed.wait_ms().is_none());

        let limited = RateLimitResult::Limited { wait_ms: 100 };
        assert!(!limited.is_allowed());
        assert_eq!(limited.wait_ms(), Some(100));
    }

    #[test]
    fn test_rate_limit_config() {
        let config = RateLimitConfig::default();
        assert_eq!(config.capacity, 10);
        assert_eq!(config.rate, 1);
        assert_eq!(config.algorithm, RateLimiterAlgorithm::TokenBucket);

        let token_config = RateLimitConfig::token_bucket(100, 10);
        assert_eq!(token_config.capacity, 100);
        assert_eq!(token_config.rate, 10);
        assert_eq!(token_config.algorithm, RateLimiterAlgorithm::TokenBucket);

        let leaky_config = RateLimitConfig::leaky_bucket(50, 5);
        assert_eq!(leaky_config.capacity, 50);
        assert_eq!(leaky_config.rate, 5);
        assert_eq!(leaky_config.algorithm, RateLimiterAlgorithm::LeakyBucket);
    }

    #[test]
    fn test_config_validation() {
        // 有效配置
        let config = RateLimitConfig::token_bucket(100, 10);
        assert!(config.validate().is_ok());

        // 无效配置：capacity为0
        let invalid_config1 =
            RateLimitConfig { capacity: 0, rate: 10, algorithm: RateLimiterAlgorithm::TokenBucket };
        assert!(invalid_config1.validate().is_err());

        // 无效配置：rate为0
        let invalid_config2 = RateLimitConfig {
            capacity: 100,
            rate: 0,
            algorithm: RateLimiterAlgorithm::TokenBucket,
        };
        assert!(invalid_config2.validate().is_err());
    }
}
