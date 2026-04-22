#![allow(dead_code)]

//! Token Bucket限流算法实现

use crate::{
    error::Result,
    limiter::{RateLimitResult, RateLimiter, RateLimiterAlgorithm, RateLimiterStatus},
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Token Bucket限流器
///
/// Token Bucket算法：
/// 1. 桶中最多存储capacity个token
/// 2. 按照恒定速率rate（tokens/秒）向桶中添加token
/// 3. 当桶满时，多余的token会被丢弃
/// 4. 请求需要从桶中获取token才能通过
pub struct TokenBucketLimiter {
    /// 桶的容量
    capacity: u64,
    /// token填充速率（tokens/秒）
    refill_rate: u64,
    /// 当前可用的token数量
    available_tokens: Arc<AtomicU64>,
    /// 上次填充时间（Instant::now()的duration）
    last_refill_time: Arc<AtomicU64>, // 存储纳秒时间戳
}

impl TokenBucketLimiter {
    /// 创建新的Token Bucket限流器
    pub fn new(capacity: u64, refill_rate: u64) -> Self {
        Self {
            capacity,
            refill_rate,
            available_tokens: Arc::new(AtomicU64::new(capacity)),
            last_refill_time: Arc::new(AtomicU64::new(0)),
        }
    }

    /// 填充token
    /// 根据时间差计算应该添加的token数量
    fn refill(&self) {
        let now = self.current_time_nanos();
        let last_refill = self.last_refill_time.load(Ordering::Relaxed);

        let nanos_elapsed = now.saturating_sub(last_refill);
        let seconds_elapsed = nanos_elapsed as f64 / 1_000_000_000.0;

        // 计算应该添加的token数量
        let tokens_to_add = (seconds_elapsed * self.refill_rate as f64) as u64;

        if tokens_to_add > 0 {
            // 更新可用token数量，不超过容量
            let current = self.available_tokens.load(Ordering::Relaxed);
            let new_tokens = current.saturating_add(tokens_to_add).min(self.capacity);
            self.available_tokens.store(new_tokens, Ordering::Relaxed);

            // 更新最后刷新时间
            self.last_refill_time.store(now, Ordering::Relaxed);
        }
    }

    /// 获取当前时间戳（纳秒）
    fn current_time_nanos(&self) -> u64 {
        Instant::now().elapsed().as_nanos() as u64
    }

    /// 尝试获取指定数量的token，不执行refill
    fn try_acquire_without_refill(&self, tokens: u64) -> Result<RateLimitResult> {
        loop {
            let current = self.available_tokens.load(Ordering::Relaxed);

            if current < tokens {
                // 计算需要等待的时间
                let tokens_needed = tokens - current;
                let wait_seconds = tokens_needed as f64 / self.refill_rate as f64;
                let wait_ms = (wait_seconds * 1000.0).ceil() as u64;

                return Ok(RateLimitResult::Limited { wait_ms });
            }

            // 原子性地减少token数量
            match self.available_tokens.compare_exchange_weak(
                current,
                current - tokens,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(RateLimitResult::Allowed),
                Err(_) => continue, // 重试
            }
        }
    }

    /// 计算等待时间
    fn calculate_wait_time(&self, tokens: u64) -> u64 {
        let current = self.available_tokens.load(Ordering::Relaxed);
        if current >= tokens {
            return 0;
        }

        let tokens_needed = tokens - current;
        let wait_seconds = tokens_needed as f64 / self.refill_rate as f64;
        (wait_seconds * 1000.0).ceil() as u64
    }
}

#[async_trait::async_trait]
impl RateLimiter for TokenBucketLimiter {
    async fn try_acquire(&self) -> bool {
        self.refill();
        self.try_acquire_without_refill(1).map(|r| r.is_allowed()).unwrap_or(false)
    }

    async fn try_acquire_many(&self, tokens: u64) -> u64 {
        self.refill();

        // 逐个尝试获取token
        let mut acquired = 0u64;
        for _ in 0..tokens {
            if self.try_acquire_without_refill(1).map(|r| r.is_allowed()).unwrap_or(false) {
                acquired += 1;
            } else {
                break;
            }
        }

        acquired
    }

    async fn get_status(&self) -> RateLimiterStatus {
        self.refill();
        RateLimiterStatus {
            algorithm: RateLimiterAlgorithm::TokenBucket,
            available_tokens: self.available_tokens.load(Ordering::Relaxed),
            capacity: self.capacity,
            refill_rate: self.refill_rate,
            last_refill: self.last_refill_time.load(Ordering::Relaxed),
        }
    }

    async fn reset(&self) {
        self.available_tokens.store(self.capacity, Ordering::Relaxed);
        self.last_refill_time.store(self.current_time_nanos(), Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_bucket_creation() {
        let limiter = TokenBucketLimiter::new(10, 1);

        let status = limiter.get_status().await;
        assert_eq!(status.capacity, 10);
        assert_eq!(status.refill_rate, 1);
        assert_eq!(status.algorithm, RateLimiterAlgorithm::TokenBucket);
    }

    #[tokio::test]
    async fn test_token_bucket_acquire() {
        let limiter = TokenBucketLimiter::new(5, 1);

        // 初始应该有5个token
        assert_eq!(limiter.available_tokens.load(Ordering::Relaxed), 5);

        // 获取token
        assert!(limiter.try_acquire().await);
        assert_eq!(limiter.available_tokens.load(Ordering::Relaxed), 4);

        assert!(limiter.try_acquire().await);
        assert_eq!(limiter.available_tokens.load(Ordering::Relaxed), 3);

        // 获取剩余所有token
        for _ in 0..3 {
            assert!(limiter.try_acquire().await);
        }

        // 没有token了
        assert!(!limiter.try_acquire().await);
        assert_eq!(limiter.available_tokens.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn test_token_bucket_acquire_many() {
        let limiter = TokenBucketLimiter::new(10, 5);

        // 尝试获取太多token
        let acquired = limiter.try_acquire_many(15).await;
        assert_eq!(acquired, 10);
        assert_eq!(limiter.available_tokens.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn test_token_bucket_reset() {
        let limiter = TokenBucketLimiter::new(5, 1);

        // 消耗所有token
        for _ in 0..5 {
            assert!(limiter.try_acquire().await);
        }

        assert!(!limiter.try_acquire().await);

        // 重置
        limiter.reset().await;

        // 重置后应该恢复容量
        assert_eq!(limiter.available_tokens.load(Ordering::Relaxed), 5);
        assert!(limiter.try_acquire().await);
    }

    #[tokio::test]
    async fn test_token_bucket_status() {
        let limiter = TokenBucketLimiter::new(10, 2);

        let status = limiter.get_status().await;
        assert_eq!(status.available_tokens, 10);
        assert_eq!(status.capacity, 10);
        assert_eq!(status.refill_rate, 2);

        // 获取一些token
        for _ in 0..3 {
            limiter.try_acquire().await;
        }

        let status = limiter.get_status().await;
        assert_eq!(status.available_tokens, 7);
    }

    #[test]
    fn test_calculate_wait_time() {
        let limiter = TokenBucketLimiter::new(10, 1); // 1 token/秒

        // 有足够token
        assert_eq!(limiter.calculate_wait_time(5), 0);

        // 需要等待
        let limiter2 = TokenBucketLimiter::new(10, 1);
        limiter2.available_tokens.store(0, Ordering::Relaxed);
        assert_eq!(limiter2.calculate_wait_time(5), 5000); // 5秒等待
    }

    #[tokio::test]
    async fn test_token_bucket_high_rate() {
        let limiter = TokenBucketLimiter::new(1000, 100); // 100 tokens/秒

        // 快速消耗
        let mut acquired = 0u64;
        for _ in 0..100 {
            if limiter.try_acquire().await {
                acquired += 1;
            }
        }

        assert_eq!(acquired, 100);
        assert_eq!(limiter.available_tokens.load(Ordering::Relaxed), 900);
    }

    #[tokio::test]
    async fn test_token_bucket_partial_acquire() {
        let limiter = TokenBucketLimiter::new(5, 1);

        // 获取超过容量的token
        let acquired = limiter.try_acquire_many(10).await;
        assert_eq!(acquired, 5);
        assert_eq!(limiter.available_tokens.load(Ordering::Relaxed), 0);
    }
}
