//! # 任务定义和执行
//!
//! 定义定时任务的基本结构

use chrono::{DateTime, Timelike, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cron::CronExpression;

/// 任务ID
pub type JobId = String;

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    /// 等待执行
    Pending,
    /// 正在运行
    Running,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
    /// 执行失败
    Failed,
}

/// 任务执行结果
#[derive(Debug, Clone)]
pub struct JobResult {
    /// 是否成功
    pub success: bool,
    /// 错误信息（如果有）
    pub error: Option<String>,
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
}

/// 任务定义
#[derive(Clone)]
pub struct Job {
    /// 任务ID
    pub id: JobId,
    /// 任务名称
    pub name: String,
    /// 任务描述
    pub description: Option<String>,
    /// Cron表达式
    pub cron: CronExpression,
    /// 任务处理器
    pub handler: Arc<dyn JobHandler>,
    /// 是否启用
    pub enabled: bool,
    /// 最大重试次数
    pub max_retries: u32,
    /// 当前重试次数
    pub current_retry: Arc<RwLock<u32>>,
    /// 任务状态
    pub status: Arc<RwLock<JobStatus>>,
    /// 最后执行时间
    pub last_run: Arc<RwLock<Option<DateTime<Utc>>>>,
    /// 下次执行时间
    pub next_run: Arc<RwLock<Option<DateTime<Utc>>>>,
    /// 执行统计
    pub stats: Arc<RwLock<JobStats>>,
}

/// 任务统计
#[derive(Debug, Clone, Default)]
pub struct JobStats {
    /// 总执行次数
    pub total_runs: u64,
    /// 成功次数
    pub success_count: u64,
    /// 失败次数
    pub failure_count: u64,
    /// 总执行时间（毫秒）
    pub total_execution_time_ms: u64,
    /// 平均执行时间（毫秒）
    pub avg_execution_time_ms: u64,
}

/// 任务处理器trait
#[async_trait::async_trait]
pub trait JobHandler: Send + Sync {
    /// 执行任务
    async fn execute(&self) -> JobResult;
}

/// 使用async闭包的任务处理器
pub struct ClosureJobHandler<F>
where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = JobResult> + Send>> + Send + Sync,
{
    f: Arc<F>,
}

impl<F> ClosureJobHandler<F>
where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = JobResult> + Send>> + Send + Sync,
{
    pub fn new(f: F) -> Self {
        Self { f: Arc::new(f) }
    }
}

#[async_trait::async_trait]
impl<F> JobHandler for ClosureJobHandler<F>
where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = JobResult> + Send>> + Send + Sync,
{
    async fn execute(&self) -> JobResult {
        (self.f)().await
    }
}

impl Job {
    /// 创建新任务
    pub fn new(
        id: JobId,
        name: String,
        cron: CronExpression,
        handler: Arc<dyn JobHandler>,
    ) -> Self {
        Self {
            enabled: true,
            max_retries: 3,
            current_retry: Arc::new(RwLock::new(0)),
            status: Arc::new(RwLock::new(JobStatus::Pending)),
            last_run: Arc::new(RwLock::new(None)),
            next_run: Arc::new(RwLock::new(None)),
            stats: Arc::new(RwLock::new(JobStats::default())),
            id,
            name,
            description: None,
            cron,
            handler,
        }
    }

    /// 设置任务描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// 设置最大重试次数
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// 计算下次执行时间
    pub async fn calculate_next_run(&self, from: DateTime<Utc>) {
        let (second, minute, hour) = (from.second() as u8, from.minute() as u8, from.hour() as u8);

        if let Some((next_sec, next_min, next_hour)) = self.cron.next_run(second, minute, hour) {
            let mut next_run = self.next_run.write().await;
            *next_run = Some(
                from.date_naive()
                    .and_hms_opt(next_hour as u32, next_min as u32, next_sec as u32)
                    .map(|dt| dt.and_utc())
                    .unwrap_or_else(|| {
                        // 如果跨越到第二天
                        from.signed_duration_since(Utc::now()).num_days();
                        from
                    }),
            );
        } else {
            // 没有找到今天的时间，设置为明天
            if let Some(next_sec) = self.cron.seconds.iter().next() {
                if let Some(next_min) = self.cron.minutes.iter().next() {
                    if let Some(next_hour) = self.cron.hours.iter().next() {
                        let mut next_run = self.next_run.write().await;
                        if let Some(tomorrow) = from.date_naive().succ_opt() {
                            *next_run = tomorrow
                                .and_hms_opt(
                                    (*next_hour) as u32,
                                    (*next_min) as u32,
                                    (*next_sec) as u32,
                                )
                                .map(|dt| dt.and_utc());
                        }
                    }
                }
            }
        }
    }

    /// 检查是否到执行时间
    pub async fn is_due(&self) -> bool {
        let next_run = self.next_run.read().await;
        if let Some(next) = *next_run {
            return Utc::now() >= next;
        }
        false
    }

    /// 执行任务
    pub async fn execute(&self) -> JobResult {
        // 更新状态为运行中
        {
            let mut status = self.status.write().await;
            *status = JobStatus::Running;
        }

        // 更新最后执行时间
        {
            let mut last_run = self.last_run.write().await;
            *last_run = Some(Utc::now());
        }

        let start = std::time::Instant::now();

        // 执行任务
        let result = self.handler.execute().await;

        // 计算执行时间
        let execution_time_ms = start.elapsed().as_millis() as u64;

        // 更新统计
        {
            let mut stats = self.stats.write().await;
            stats.total_runs += 1;

            if result.success {
                stats.success_count += 1;
                {
                    let mut current_retry = self.current_retry.write().await;
                    *current_retry = 0; // 重置重试次数
                }

                // 更新状态为完成
                let mut status = self.status.write().await;
                *status = JobStatus::Completed;
            } else {
                stats.failure_count += 1;

                // 检查是否需要重试
                {
                    let mut current_retry = self.current_retry.write().await;
                    if *current_retry < self.max_retries {
                        *current_retry += 1;
                        let mut status = self.status.write().await;
                        *status = JobStatus::Pending;
                    } else {
                        let mut status = self.status.write().await;
                        *status = JobStatus::Failed;
                    }
                }
            }

            stats.total_execution_time_ms += execution_time_ms;
            if stats.total_runs > 0 {
                stats.avg_execution_time_ms = stats.total_execution_time_ms / stats.total_runs;
            }
        }

        // 计算下次执行时间
        self.calculate_next_run(Utc::now()).await;

        result
    }

    /// 取消任务
    pub async fn cancel(&self) {
        let mut status = self.status.write().await;
        *status = JobStatus::Cancelled;
    }

    /// 获取任务状态
    pub async fn get_status(&self) -> JobStatus {
        *self.status.read().await
    }

    /// 获取任务统计
    pub async fn get_stats(&self) -> JobStats {
        self.stats.read().await.clone()
    }

    /// 获取下次执行时间
    pub async fn get_next_run(&self) -> Option<DateTime<Utc>> {
        *self.next_run.read().await
    }

    /// 获取上次执行时间
    pub async fn get_last_run(&self) -> Option<DateTime<Utc>> {
        *self.last_run.read().await
    }
}
