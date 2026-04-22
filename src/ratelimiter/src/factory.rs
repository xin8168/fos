//! 限流器工厂 - 简化版本

use crate::{
    error::Result,
    leaky_bucket::LeakyBucketLimiter,
    limiter::{RateLimitConfig, RateLimiter, RateLimiterAlgorithm},
    token_bucket::TokenBucketLimiter,
};
use std::sync::Arc;

/// 限流器工厂
pub struct RateLimiterFactory;

impl RateLimiterFactory {
    /// 创建Token Bucket限流器
    pub fn create_token_bucket(capacity: u64, rate: u64) -> Arc<dyn RateLimiter> {
        Arc::new(TokenBucketLimiter::new(capacity, rate))
    }

    /// 创建Leaky Bucket限流器
    pub fn create_leaky_bucket(capacity: u64, rate: u64) -> Arc<dyn RateLimiter> {
        Arc::new(LeakyBucketLimiter::new(capacity, rate))
    }

    /// 创建Token Bucket限流器（通过配置）
    pub fn create_from_config(config: &RateLimitConfig) -> Result<Arc<dyn RateLimiter>> {
        config.validate()?;

        match config.algorithm {
            RateLimiterAlgorithm::TokenBucket => {
                Ok(Self::create_token_bucket(config.capacity, config.rate))
            },
            RateLimiterAlgorithm::LeakyBucket => {
                Ok(Self::create_leaky_bucket(config.capacity, config.rate))
            },
        }
    }

    /// 创建默认限流器
    pub fn create_default() -> Arc<dyn RateLimiter> {
        Arc::new(TokenBucketLimiter::new(10, 1))
    }

    /// 创建API限流器
    pub fn create_api_limiter() -> Arc<dyn RateLimiter> {
        Arc::new(TokenBucketLimiter::new(5, 1))
    }

    /// 创建数据库限流器
    pub fn create_db_limiter() -> Arc<dyn RateLimiter> {
        Arc::new(LeakyBucketLimiter::new(10, 10))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_factory_default() {
        let limiter = RateLimiterFactory::create_default();
        assert!(limiter.try_acquire().await);
    }

    #[tokio::test]
    async fn test_factory_from_config() {
        let config = RateLimitConfig::token_bucket(100, 10);
        let limiter = RateLimiterFactory::create_from_config(&config).unwrap();

        assert!(limiter.try_acquire().await);
    }

    #[test]
    fn test_factory_invalid_config() {
        let invalid_config =
            RateLimitConfig { algorithm: RateLimiterAlgorithm::TokenBucket, capacity: 0, rate: 10 };
        assert!(RateLimiterFactory::create_from_config(&invalid_config).is_err());
    }
}
