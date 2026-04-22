#![allow(dead_code)]

//! Leaky Bucket限流算法实现

use crate::{
    error::Result,
    limiter::{RateLimitResult, RateLimiter, RateLimiterAlgorithm, RateLimiterStatus},
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Leaky Bucket限流器
///
/// Leaky Bucket算法（流量整形版本）：
/// 1. 请求按照固定速率处理（tokens/秒）
/// 2. 漏桶固定容量，超出容量的请求被丢弃
/// 3. 桶中的请求按照固定速率流出
/// 4. 当桶满时，新的请求会被限流
pub struct LeakyBucketLimiter {
    /// 桶的容量
    capacity: u64,
    /// 漏出速率（tokens/秒）
    drain_rate: u64,
    /// 当前桶中的请求数量
    bucket_level: Arc<AtomicU64>,
    /// 上次漏出时间（纳秒时间戳）
    last_drain_time: Arc<AtomicU64>,
}

impl LeakyBucketLimiter {
    /// 创建新的Leaky Bucket限流器
    pub fn new(capacity: u64, drain_rate: u64) -> Self {
        Self {
            capacity,
            drain_rate,
            bucket_level: Arc::new(AtomicU64::new(0)),
            last_drain_time: Arc::new(AtomicU64::new(0)),
        }
    }

    /// 漏出请求
    /// 根据时间差计算应该流出的请求数量
    fn drain(&self) {
        let now = self.current_time_nanos();
        let last_drain = self.last_drain_time.load(Ordering::Relaxed);

        let nanos_elapsed = now.saturating_sub(last_drain);
        let seconds_elapsed = nanos_elapsed as f64 / 1_000_000_000.0;

        // 计算应该流出的请求数量
        let requests_to_drain = (seconds_elapsed * self.drain_rate as f64) as u64;

        if requests_to_drain > 0 {
            // 减少桶中的请求数量
            let current = self.bucket_level.load(Ordering::Relaxed);
            let new_level = current.saturating_sub(requests_to_drain);
            self.bucket_level.store(new_level, Ordering::Relaxed);

            // 更新最后漏出时间
            self.last_drain_time.store(now, Ordering::Relaxed);
        }
    }

    /// 获取当前时间戳（纳秒）
    fn current_time_nanos(&self) -> u64 {
        Instant::now().elapsed().as_nanos() as u64
    }

    /// 尝试添加请求到桶中
    fn try_add_request(&self, tokens: u64) -> Result<RateLimitResult> {
        loop {
            let current_level = self.bucket_level.load(Ordering::Relaxed);

            // 检查桶是否有足够空间
            if current_level + tokens > self.capacity {
                // 计算需要等待的时间
                let space_needed = (current_level + tokens) - self.capacity;
                let wait_seconds = space_needed as f64 / self.drain_rate as f64;
                let wait_ms = (wait_seconds * 1000.0).ceil() as u64;

                return Ok(RateLimitResult::Limited { wait_ms });
            }

            // 原子性地增加桶中的请求数量
            match self.bucket_level.compare_exchange_weak(
                current_level,
                current_level + tokens,
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
        let current_level = self.bucket_level.load(Ordering::Relaxed);

        if current_level + tokens <= self.capacity {
            return 0;
        }

        let space_needed = (current_level + tokens) - self.capacity;
        let wait_seconds = space_needed as f64 / self.drain_rate as f64;
        (wait_seconds * 1000.0).ceil() as u64
    }
}

#[async_trait::async_trait]
impl RateLimiter for LeakyBucketLimiter {
    async fn try_acquire(&self) -> bool {
        self.drain();
        self.try_add_request(1).map(|r| r.is_allowed()).unwrap_or(false)
    }

    async fn try_acquire_many(&self, tokens: u64) -> u64 {
        self.drain();

        // 逐个尝试添加请求
        let mut acquired = 0u64;
        for _ in 0..tokens {
            if self.try_add_request(1).map(|r| r.is_allowed()).unwrap_or(false) {
                acquired += 1;
            } else {
                break;
            }
        }

        acquired
    }

    async fn get_status(&self) -> RateLimiterStatus {
        self.drain();
        RateLimiterStatus {
            algorithm: RateLimiterAlgorithm::LeakyBucket,
            // 对于Leaky Bucket，桶中剩余空间 = capacity - level
            available_tokens: self
                .capacity
                .saturating_sub(self.bucket_level.load(Ordering::Relaxed)),
            capacity: self.capacity,
            refill_rate: self.drain_rate,
            last_refill: self.last_drain_time.load(Ordering::Relaxed),
        }
    }

    async fn reset(&self) {
        self.bucket_level.store(0, Ordering::Relaxed);
        self.last_drain_time.store(self.current_time_nanos(), Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_leaky_bucket_creation() {
        let limiter = LeakyBucketLimiter::new(10, 1);

        let status = limiter.get_status().await;
        assert_eq!(status.capacity, 10);
        assert_eq!(status.refill_rate, 1);
        assert_eq!(status.algorithm, RateLimiterAlgorithm::LeakyBucket);
        assert_eq!(status.available_tokens, 10); // 初始桶是空的
    }

    #[tokio::test]
    async fn test_leaky_bucket_acquire() {
        let limiter = LeakyBucketLimiter::new(5, 1);

        // 初始桶是空的，可以添加5个请求
        assert_eq!(limiter.bucket_level.load(Ordering::Relaxed), 0);
        assert!(limiter.try_acquire().await);
        assert_eq!(limiter.bucket_level.load(Ordering::Relaxed), 1);

        assert!(limiter.try_acquire().await);
        assert_eq!(limiter.bucket_level.load(Ordering::Relaxed), 2);

        // 添加到容量上限
        for _ in 0..3 {
            assert!(limiter.try_acquire().await);
        }

        // 桶满了
        assert!(!limiter.try_acquire().await);
        assert_eq!(limiter.bucket_level.load(Ordering::Relaxed), 5);
    }

    #[tokio::test]
    async fn test_leaky_bucket_acquire_many() {
        let limiter = LeakyBucketLimiter::new(10, 5);

        // 尝试添加超过容量的请求
        let acquired = limiter.try_acquire_many(15).await;
        assert_eq!(acquired, 10);
        assert_eq!(limiter.bucket_level.load(Ordering::Relaxed), 10);
    }

    #[tokio::test]
    async fn test_leaky_bucket_reset() {
        let limiter = LeakyBucketLimiter::new(5, 1);

        // 添加请求填满桶
        for _ in 0..5 {
            assert!(limiter.try_acquire().await);
        }

        assert!(!limiter.try_acquire().await);

        // 重置
        limiter.reset().await;

        // 重置后桶应该为空
        assert_eq!(limiter.bucket_level.load(Ordering::Relaxed), 0);
        assert!(limiter.try_acquire().await);
    }

    #[tokio::test]
    async fn test_leaky_bucket_status() {
        let limiter = LeakyBucketLimiter::new(10, 2);

        let status = limiter.get_status().await;
        assert_eq!(status.available_tokens, 10); // 初始空桶，10空间
        assert_eq!(status.capacity, 10);
        assert_eq!(status.refill_rate, 2);

        // 添加一些请求
        for _ in 0..3 {
            limiter.try_acquire().await;
        }

        let status = limiter.get_status().await;
        assert_eq!(status.available_tokens, 7); // 10 - 3 = 7可用空间
    }

    #[test]
    fn test_calculate_wait_time() {
        let limiter = LeakyBucketLimiter::new(10, 1); // 1 request/秒

        // 有足够空间
        assert_eq!(limiter.calculate_wait_time(5), 0);

        // 需要等待
        let limiter2 = LeakyBucketLimiter::new(10, 1);
        limiter2.bucket_level.store(8, Ordering::Relaxed); // 桶中已有8个
        assert_eq!(limiter2.calculate_wait_time(5), 3000); // 需要3秒等待(8+5-10=3)
    }

    #[tokio::test]
    async fn test_leaky_bucket_high_rate() {
        let limiter = LeakyBucketLimiter::new(1000, 100); // 100 requests/秒

        // 快速添加请求
        let mut acquired = 0u64;
        for _ in 0..100 {
            if limiter.try_acquire().await {
                acquired += 1;
            }
        }

        assert_eq!(acquired, 100);
        assert_eq!(limiter.bucket_level.load(Ordering::Relaxed), 100);
    }

    #[tokio::test]
    async fn test_leaky_bucket_partial_acquire() {
        let limiter = LeakyBucketLimiter::new(5, 1);

        // 添加超过容量的请求
        let acquired = limiter.try_acquire_many(10).await;
        assert_eq!(acquired, 5);
        assert_eq!(limiter.bucket_level.load(Ordering::Relaxed), 5);
    }

    #[tokio::test]
    async fn test_leaky_bucket_available_tokens() {
        let limiter = LeakyBucketLimiter::new(10, 2);

        // 初始：空桶，10个可用token
        assert_eq!(limiter.bucket_level.load(Ordering::Relaxed), 0);
        let status = limiter.get_status().await;
        assert_eq!(status.available_tokens, 10);

        // 添加3个请求
        for _ in 0..3 {
            limiter.try_acquire().await;
        }

        // 3个请求在桶中，只有7个可用token
        let status = limiter.get_status().await;
        assert_eq!(status.available_tokens, 7);
    }
}
