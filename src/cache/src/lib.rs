//! # FOS Cache - 多级缓存模块
//!
//! ## 核心职责
//! - 多级缓存管理
//! - 缓存淘汰策略
//! - 缓存一致性保证
//!
//! ## 安全铁律
//! - 不做规则判断

pub mod cache;
pub mod config;
pub mod distributed;
pub mod entry;
pub mod error;
pub mod local_distributed;
pub mod stats;

pub use cache::{CacheKey, LocalCache};
pub use config::Config;
pub use distributed::{CacheCodec, CacheConfig, CacheType, DistributedCache, JsonCodec};
pub use entry::CacheEntry;
pub use error::{Error, Result};
pub use local_distributed::LocalDistributedCache;
pub use stats::CacheStats;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
