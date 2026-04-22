//! # FOS RateLimiter - 限流控制模块
//!
//! ## 核心职责
//! - 请求限流控制
//! - 多种限流算法支持
//! - 限流状态监控
//!
//! ## 安全铁律
//! - 不做规则判断

pub mod circuit;
pub mod circuit_breaker;
pub mod config;
pub mod error;
pub mod factory;
pub mod leaky_bucket;
pub mod limiter;
pub mod token_bucket;

pub use circuit::{CircuitConfig, CircuitState, CircuitStatus, RequestResult};
pub use circuit_breaker::{CircuitBreaker, ExecutionError};
pub use factory::RateLimiterFactory;
pub use leaky_bucket::LeakyBucketLimiter;
pub use limiter::{
    RateLimitConfig, RateLimitResult, RateLimiter, RateLimiterAlgorithm, RateLimiterStatus,
};
pub use token_bucket::TokenBucketLimiter;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
