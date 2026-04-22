//! # 延迟队列
//!
//! 支持延迟执行任务的队列管理

use crate::error::{Error, Result};
use crate::job::{JobHandler, JobResult, JobStats};
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock as AsyncRwLock;
use tracing::info;

/// 延迟任务ID
pub type DelayedJobId = String;

/// 延迟任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelayedJobStatus {
    /// 等待中
    Waiting,
    /// 准备执行
    Ready,
    /// 正在执行
    Running,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
    /// 已过期
    Expired,
}

/// 延迟任务
#[derive(Clone)]
pub struct DelayedJob {
    /// 任务ID
    pub id: DelayedJobId,
    /// 任务名称
    pub name: String,
    /// 任务描述
    pub description: Option<String>,
    /// 执行时间
    pub execute_at: DateTime<Utc>,
    /// 过期时间（可选）
    pub expires_at: Option<DateTime<Utc>>,
    /// 任务处理器
    pub handler: Arc<dyn JobHandler>,
    /// 最大重试次数
    pub max_retries: u32,
    /// 当前重试次数
    pub current_retry: Arc<AsyncRwLock<u32>>,
    /// 任务状态
    pub status: Arc<AsyncRwLock<DelayedJobStatus>>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 执行统计
    pub stats: Arc<AsyncRwLock<JobStats>>,
    /// 执行结果
    pub result: Arc<AsyncRwLock<Option<JobResult>>>,
}

impl DelayedJob {
    /// 创建新的延迟任务
    pub fn new(
        id: DelayedJobId,
        name: String,
        execute_at: DateTime<Utc>,
        handler: Arc<dyn JobHandler>,
    ) -> Self {
        Self {
            id,
            name,
            description: None,
            execute_at,
            expires_at: None,
            handler,
            max_retries: 3,
            current_retry: Arc::new(AsyncRwLock::new(0)),
            status: Arc::new(AsyncRwLock::new(DelayedJobStatus::Waiting)),
            created_at: Utc::now(),
            stats: Arc::new(AsyncRwLock::new(JobStats::default())),
            result: Arc::new(AsyncRwLock::new(None)),
        }
    }

    /// 设置任务描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// 设置过期时间
    pub fn with_expires_at(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// 设置最大重试次数
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// 检查是否到执行时间（同步版本）
    fn is_ready_sync(&self) -> bool {
        if let Ok(status) = self.status.try_read() {
            if *status != DelayedJobStatus::Waiting {
                return false;
            }
            Utc::now() >= self.execute_at
        } else {
            false
        }
    }

    /// 检查是否到执行时间
    pub async fn is_ready(&self) -> bool {
        self.is_ready_sync()
    }

    /// 检查是否已过期（同步版本）
    fn is_expired_sync(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            return Utc::now() > expires_at;
        }
        false
    }

    /// 检查是否已过期
    pub async fn is_expired(&self) -> bool {
        self.is_expired_sync()
    }

    /// 检查是否可执行
    pub async fn can_execute(&self) -> bool {
        let status = self.status.read().await;

        if *status != DelayedJobStatus::Ready {
            return false;
        }

        if self.is_expired_sync() {
            return false;
        }

        true
    }

    /// 执行任务
    pub async fn execute(&self) -> JobResult {
        // 更新状态为执行中
        {
            let mut status = self.status.write().await;
            *status = DelayedJobStatus::Running;
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

                // 重置重试次数
                {
                    let mut retry = self.current_retry.write().await;
                    *retry = 0;
                }

                // 更新状态为完成
                let mut status = self.status.write().await;
                *status = DelayedJobStatus::Completed;
            } else {
                stats.failure_count += 1;

                // 检查是否需要重试
                {
                    let retry = self.current_retry.read().await;
                    if *retry < self.max_retries {
                        drop(retry);
                        let mut retry = self.current_retry.write().await;
                        *retry += 1;

                        // 更新状态为准备执行
                        let mut status = self.status.write().await;
                        *status = DelayedJobStatus::Ready;
                    } else {
                        // 更新状态为取消
                        let mut status = self.status.write().await;
                        *status = DelayedJobStatus::Cancelled;
                    }
                }
            }

            stats.total_execution_time_ms += execution_time_ms;
            if stats.total_runs > 0 {
                stats.avg_execution_time_ms = stats.total_execution_time_ms / stats.total_runs;
            }
        }

        // 保存执行结果
        {
            let mut result_store = self.result.write().await;
            *result_store = Some(result.clone());
        }

        result
    }

    /// 取消任务
    pub async fn cancel(&self) {
        let mut status = self.status.write().await;
        *status = DelayedJobStatus::Cancelled;
    }

    /// 获取任务状态
    pub async fn get_status(&self) -> DelayedJobStatus {
        *self.status.read().await
    }

    /// 获取任务统计
    pub async fn get_stats(&self) -> JobStats {
        self.stats.read().await.clone()
    }

    /// 获取执行结果
    pub async fn get_result(&self) -> Option<JobResult> {
        self.result.read().await.clone()
    }

    /// 获取剩余等待时间
    pub async fn remaining_time(&self) -> Option<Duration> {
        let now = Utc::now();
        if now < self.execute_at {
            Some(self.execute_at - now)
        } else {
            None
        }
    }
}

/// 延迟队列管理器
pub struct DelayedQueue {
    /// 延迟任务
    jobs: Arc<AsyncRwLock<HashMap<DelayedJobId, Arc<DelayedJob>>>>,
}

impl DelayedQueue {
    /// 创建新的延迟队列
    pub fn new() -> Self {
        Self { jobs: Arc::new(AsyncRwLock::new(HashMap::new())) }
    }

    /// 添加延迟任务
    pub async fn add(&self, job: DelayedJob) -> Result<()> {
        // 验证执行时间
        if job.execute_at <= Utc::now() {
            return Err(Error::Schedule("Execute time must be in the future".to_string()));
        }

        // 验证过期时间（如果设置了，必须是在未来）
        if let Some(expires_at) = job.expires_at {
            if expires_at <= Utc::now() {
                return Err(Error::Schedule("Expires time must be in the future".to_string()));
            }
        }

        let job_id = job.id.clone();
        let job_id_for_log = job_id.clone();
        let mut jobs = self.jobs.write().await;
        jobs.insert(job_id, Arc::new(job));

        info!("Added delayed job: {}", job_id_for_log);
        Ok(())
    }

    /// 移除延迟任务
    pub async fn remove(&self, job_id: &str) -> Result<()> {
        let mut jobs = self.jobs.write().await;

        if jobs.remove(job_id).is_some() {
            info!("Removed delayed job: {}", job_id);
            Ok(())
        } else {
            Err(Error::Schedule(format!("Delayed job not found: {}", job_id)))
        }
    }

    /// 获取延迟任务
    pub async fn get(&self, job_id: &str) -> Result<Arc<DelayedJob>> {
        let jobs = self.jobs.read().await;
        jobs.get(job_id)
            .cloned()
            .ok_or_else(|| Error::Schedule(format!("Delayed job not found: {}", job_id)))
    }

    /// 取消延迟任务
    pub async fn cancel(&self, job_id: &str) -> Result<()> {
        let job = self.get(job_id).await?;
        job.cancel().await;
        info!("Cancelled delayed job: {}", job_id);
        Ok(())
    }

    /// 获取所有准备执行的任务
    pub async fn get_ready_jobs(&self) -> Vec<Arc<DelayedJob>> {
        let jobs = self.jobs.read().await;
        let ready_jobs: Vec<Arc<DelayedJob>> = jobs
            .iter()
            .filter(|(_, job)| job.is_ready_sync() && !job.is_expired_sync())
            .map(|(_, job)| job.clone())
            .collect();

        ready_jobs
    }

    /// 获取所有已过期的任务
    pub async fn get_expired_jobs(&self) -> Vec<Arc<DelayedJob>> {
        let jobs = self.jobs.read().await;
        let expired: Vec<Arc<DelayedJob>> = jobs
            .iter()
            .filter(|(_, job)| job.is_expired_sync())
            .map(|(_, job)| job.clone())
            .collect();

        expired
    }

    /// 移除已过期的任务
    pub async fn remove_expired(&self) -> usize {
        let expired = self.get_expired_jobs().await;
        let mut jobs = self.jobs.write().await;

        let mut count = 0;
        for job in &expired {
            if jobs.remove(&job.id).is_some() {
                count += 1;
            }
        }

        if count > 0 {
            info!("Removed {} expired jobs", count);
        }

        count
    }

    /// 列出所有任务ID
    pub async fn list_jobs(&self) -> Vec<DelayedJobId> {
        let jobs = self.jobs.read().await;
        jobs.keys().cloned().collect()
    }

    /// 获取任务数量
    pub async fn len(&self) -> usize {
        let jobs = self.jobs.read().await;
        jobs.len()
    }

    /// 检查是否为空
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }

    /// 清除所有任务
    pub async fn clear(&self) {
        let mut jobs = self.jobs.write().await;
        jobs.clear();
        info!("Cleared delayed queue");
    }
}

impl Default for DelayedQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::ClosureJobHandler;

    async fn create_test_delayed_job(delay_seconds: i64) -> DelayedJob {
        let execute_at = Utc::now() + Duration::seconds(delay_seconds);

        let executed = std::sync::Arc::new(std::sync::RwLock::new(false));

        let handler: Arc<dyn JobHandler> = Arc::new(ClosureJobHandler::new(move || {
            let executed = executed.clone();
            Box::pin(async move {
                *executed.write().unwrap() = true;
                JobResult { success: true, error: None, execution_time_ms: 100 }
            })
        }));

        DelayedJob::new("test-job".to_string(), "Test Delayed Job".to_string(), execute_at, handler)
    }

    #[tokio::test]
    async fn test_delayed_queue_creation() {
        let queue = DelayedQueue::new();

        assert!(queue.is_empty().await);
        assert_eq!(queue.len().await, 0);
    }

    #[tokio::test]
    async fn test_add_delayed_job() {
        let queue = DelayedQueue::new();

        let job = create_test_delayed_job(10).await;
        queue.add(job).await.unwrap();

        assert_eq!(queue.len().await, 1);
        assert!(queue.list_jobs().await.contains(&"test-job".to_string()));
    }

    #[tokio::test]
    async fn test_add_job_in_past() {
        let queue = DelayedQueue::new();

        let execute_at = Utc::now() - Duration::seconds(10);
        let handler: Arc<dyn JobHandler> = Arc::new(ClosureJobHandler::new(|| {
            Box::pin(async move { JobResult { success: true, error: None, execution_time_ms: 0 } })
        }));

        let job =
            DelayedJob::new("test-job".to_string(), "Test Job".to_string(), execute_at, handler);

        let result = queue.add(job).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_remove_delayed_job() {
        let queue = DelayedQueue::new();

        let job = create_test_delayed_job(10).await;
        queue.add(job).await.unwrap();

        queue.remove("test-job").await.unwrap();

        assert!(queue.is_empty().await);
    }

    #[tokio::test]
    async fn test_remove_expired() {
        let queue = DelayedQueue::new();

        // 创建任务 - execute_at在2秒后，expires_at在0.5秒后
        // 这样任务会在等待期就过期
        let execute_at = Utc::now() + Duration::seconds(2);
        let expires_at = Utc::now() + Duration::milliseconds(500);

        let handler: Arc<dyn JobHandler> = Arc::new(ClosureJobHandler::new(|| {
            Box::pin(async move { JobResult { success: true, error: None, execution_time_ms: 0 } })
        }));

        let job =
            DelayedJob::new("test-job".to_string(), "Test Job".to_string(), execute_at, handler)
                .with_expires_at(expires_at);

        queue.add(job).await.unwrap();

        // 等待任务过期
        tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;

        let count = queue.remove_expired().await;
        assert_eq!(count, 1);
        assert!(queue.is_empty().await);
    }

    #[tokio::test]
    async fn test_cancel_job() {
        let queue = DelayedQueue::new();

        let job = create_test_delayed_job(10).await;
        queue.add(job).await.unwrap();

        queue.cancel("test-job").await.unwrap();

        let job = queue.get("test-job").await.unwrap();
        assert_eq!(job.get_status().await, DelayedJobStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_get_ready_jobs() {
        let queue = DelayedQueue::new();

        let job_id1 = "job-1".to_string();
        let job_id2 = "job-2".to_string();

        // 添加一个即将到期的任务
        let execute_at1 = Utc::now() + Duration::milliseconds(50);
        let handler: Arc<dyn JobHandler> = Arc::new(ClosureJobHandler::new(|| {
            Box::pin(async move { JobResult { success: true, error: None, execution_time_ms: 0 } })
        }));
        let job1 = DelayedJob::new(job_id1, "Test Job 1".to_string(), execute_at1, handler.clone());
        queue.add(job1).await.unwrap();

        // 添加一个还未到期的任务
        let execute_at2 = Utc::now() + Duration::seconds(10);
        let job2 = DelayedJob::new(job_id2, "Test Job 2".to_string(), execute_at2, handler);
        queue.add(job2).await.unwrap();

        // 等待第一个任务到期
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let ready = queue.get_ready_jobs().await;
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "job-1");
    }

    #[tokio::test]
    async fn test_get_expired_jobs() {
        let queue = DelayedQueue::new();

        // 创建带过期时间的任务 - execute_at在1秒后，expires_at在0.5秒后
        // 这样任务会在等待期就过期
        let execute_at = Utc::now() + Duration::seconds(1);
        let expires_at = Utc::now() + Duration::milliseconds(500);

        let handler: Arc<dyn JobHandler> = Arc::new(ClosureJobHandler::new(|| {
            Box::pin(async move { JobResult { success: true, error: None, execution_time_ms: 0 } })
        }));

        let job =
            DelayedJob::new("test-job".to_string(), "Test Job".to_string(), execute_at, handler)
                .with_expires_at(expires_at);

        queue.add(job).await.unwrap();

        // 等待任务过期
        tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;

        let expired = queue.get_expired_jobs().await;
        assert_eq!(expired.len(), 1);
        // 任务应该还处于Waiting状态，因为is_expired只检查时间
        assert!(expired[0].is_expired().await);
    }

    #[tokio::test]
    async fn test_job_execution() {
        let queue = DelayedQueue::new();

        let executed = std::sync::Arc::new(std::sync::RwLock::new(false));
        let executed_for_handler = executed.clone();

        let handler: Arc<dyn JobHandler> = Arc::new(ClosureJobHandler::new(move || {
            let executed = executed_for_handler.clone();
            Box::pin(async move {
                *executed.write().unwrap() = true;
                JobResult { success: true, error: None, execution_time_ms: 100 }
            })
        }));

        let execute_at = Utc::now() + Duration::milliseconds(100);
        let job =
            DelayedJob::new("test-job".to_string(), "Test Job".to_string(), execute_at, handler);

        queue.add(job).await.unwrap();

        // 等待任务到期
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

        let ready = queue.get_ready_jobs().await;
        assert_eq!(ready.len(), 1);

        // 执行任务
        let result = ready[0].execute().await;
        assert!(result.success);
        assert!(*executed.read().unwrap());

        let stats = ready[0].get_stats().await;
        assert_eq!(stats.total_runs, 1);
    }

    #[tokio::test]
    async fn test_remaining_time() {
        let queue = DelayedQueue::new();

        let job = create_test_delayed_job(10).await;
        queue.add(job).await.unwrap();

        let job = queue.get("test-job").await.unwrap();
        let remaining = job.remaining_time().await;

        assert!(remaining.is_some());
        assert!(remaining.unwrap() < Duration::seconds(11));
        assert!(remaining.unwrap() > Duration::seconds(9));
    }

    #[tokio::test]
    async fn test_clear_queue() {
        let queue = DelayedQueue::new();

        // 添加多个任务
        for i in 1..=3 {
            let job_id = format!("test-job-{}", i);
            let execute_at = Utc::now() + Duration::seconds(10 * i as i64);
            let handler: Arc<dyn JobHandler> = Arc::new(ClosureJobHandler::new(|| {
                Box::pin(
                    async move { JobResult { success: true, error: None, execution_time_ms: 0 } },
                )
            }));
            let job = DelayedJob::new(job_id, format!("Test Job {}", i), execute_at, handler);
            queue.add(job).await.unwrap();
        }

        assert_eq!(queue.len().await, 3);

        queue.clear().await;

        assert!(queue.is_empty().await);
    }
}
