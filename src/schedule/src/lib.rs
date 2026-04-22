//! # FOS Schedule - 定时调度模块
//!
//! ## 核心职责
//! - 定时任务调度
//! - 任务执行管理
//! - 调度策略配置
//!
//! ## 安全铁律
//! - 不做规则判断

pub mod config;
pub mod cron;
pub mod delayed_queue;
pub mod error;
pub mod job;

pub use config::Config;
pub use cron::CronExpression;
pub use delayed_queue::{DelayedJob, DelayedJobId, DelayedJobStatus, DelayedQueue};
pub use error::{Error, Result};
pub use job::{ClosureJobHandler, Job, JobHandler, JobId, JobResult, JobStats, JobStatus};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
