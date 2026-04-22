//! # FOS EventLog - 事件日志模块
//!
//! ## 核心职责
//! - 事件记录与追踪
//! - 日志持久化
//! - 事件查询与回放
//!
//! ## 安全铁律
//! - 不做规则判断

pub mod aggregator;
pub mod config;
pub mod error;
pub mod tracer;

pub use aggregator::{AggregationQuery, EventStats, GroupedStats, LogAggregator};
pub use config::Config;
pub use error::{Error, Result};
pub use tracer::{
    EventId, EventLevel, EventLog, EventStatus, EventTracer, EventType, SpanId, TraceContext,
    TraceId,
};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
