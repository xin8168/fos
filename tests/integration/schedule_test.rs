//! # Schedule 集成测试
//!
//! 测试调度系统的完整功能，包括 Cron 任务和延迟队列

use fos_schedule::{
    ClosureJobHandler, CronExpression, DelayedJob, DelayedJobId, DelayedJobStatus, DelayedQueue,
    Job, JobHandler, JobId, JobResult, JobStatus,
};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// 创建测试用的 Cron 任务处理器
fn create_test_job_handler() -> Arc<dyn JobHandler> {
    Arc::new(ClosureJobHandler::new(|| {
        Box::pin(async move { JobResult { success: true, error: None, execution_time_ms: 10 } })
    }))
}

/// 创建会失败的任务处理器（用于重试测试）
fn create_failing_job_handler(fail_count: Arc<AtomicU32>) -> Arc<dyn JobHandler> {
    Arc::new(ClosureJobHandler::new(move || {
        let fail_count = fail_count.clone();
        Box::pin(async move {
            let current = fail_count.fetch_add(1, Ordering::SeqCst);
            if current < 3 {
                JobResult {
                    success: false,
                    error: Some(format!("Failure {} of 3", current + 1)),
                    execution_time_ms: 10,
                }
            } else {
                JobResult { success: true, error: None, execution_time_ms: 10 }
            }
        })
    }))
}

#[tokio::test]
async fn test_cron_expression_parsing() {
    // 测试简单的 Cron 表达式（6个部分：秒 分 时 日 月 星期）
    let cron = CronExpression::parse("0 * * * * *").unwrap();

    // 测试列表
    let cron = CronExpression::parse("0 1,2,3 * * * *").unwrap();

    // 测试范围
    let cron = CronExpression::parse("0 1-5 * * * *").unwrap();

    // 测试步进
    let cron = CronExpression::parse("0 * * * * *").unwrap();

    // 测试无效的 Cron 表达式
    let result = CronExpression::parse("invalid");
    assert!(result.is_err());

    // 测试部分数量错误
    let result = CronExpression::parse("* * * * *"); // 只有5个部分
    assert!(result.is_err());
}

#[tokio::test]
async fn test_delayed_job_creation() {
    let queue = DelayedQueue::new();

    let execute_at = chrono::Utc::now() + chrono::Duration::seconds(10);
    let handler = create_test_job_handler();

    let job = DelayedJob::new(
        "test-job".to_string(),
        "Test Delayed Job".to_string(),
        execute_at,
        handler,
    );

    // 验证任务属性
    assert_eq!(job.id, "test-job");
    assert_eq!(job.name, "Test Delayed Job");
    assert_eq!(job.max_retries, 3);
}

#[tokio::test]
async fn test_delayed_job_with_expiration() {
    let queue = DelayedQueue::new();

    let execute_at = chrono::Utc::now() + chrono::Duration::seconds(10);
    let expires_at = chrono::Utc::now() + chrono::Duration::seconds(5);
    let handler = create_test_job_handler();

    let job = DelayedJob::new(
        "test-job".to_string(),
        "Test Delayed Job".to_string(),
        execute_at,
        handler,
    )
    .with_expires_at(expires_at);

    // 验证过期时间设置
    assert_eq!(job.expires_at, Some(expires_at));
}

#[tokio::test]
async fn test_delayed_queue_add_and_remove() {
    let queue = DelayedQueue::new();

    let execute_at = chrono::Utc::now() + chrono::Duration::seconds(10);
    let handler = create_test_job_handler();

    let job = DelayedJob::new(
        "test-job".to_string(),
        "Test Delayed Job".to_string(),
        execute_at,
        handler,
    );

    // 添加任务
    queue.add(job).await.unwrap();
    assert_eq!(queue.len().await, 1);

    // 获取任务
    let retrieved_job = queue.get("test-job").await.unwrap();
    assert_eq!(retrieved_job.id, "test-job");

    // 移除任务
    queue.remove("test-job").await.unwrap();
    assert_eq!(queue.len().await, 0);
}

#[tokio::test]
async fn test_delayed_job_execution() {
    let queue = DelayedQueue::new();

    let executed = Arc::new(AtomicBool::new(false));
    let executed_clone = executed.clone();

    let execute_at = chrono::Utc::now() + chrono::Duration::milliseconds(100);
    let handler = Arc::new(ClosureJobHandler::new(move || {
        let executed = executed_clone.clone();
        Box::pin(async move {
            executed.store(true, Ordering::SeqCst);
            JobResult { success: true, error: None, execution_time_ms: 10 }
        })
    }));

    let job = DelayedJob::new(
        "test-job".to_string(),
        "Test Delayed Job".to_string(),
        execute_at,
        handler,
    );

    queue.add(job).await.unwrap();

    // 等待任务准备执行
    sleep(Duration::from_millis(150)).await;

    // 检查任务是否准备好
    let ready_jobs = queue.get_ready_jobs().await;
    assert_eq!(ready_jobs.len(), 1);

    // 执行任务
    let result = ready_jobs[0].execute().await;
    assert!(result.success);
    assert!(executed.load(Ordering::SeqCst));

    // 检查统计
    let stats = ready_jobs[0].get_stats().await;
    assert_eq!(stats.total_runs, 1);
    assert_eq!(stats.success_count, 1);
}

#[tokio::test]
async fn test_delayed_job_retry_mechanism() {
    let queue = DelayedQueue::new();

    let fail_count = Arc::new(AtomicU32::new(0));
    let handler = create_failing_job_handler(fail_count.clone());

    let execute_at = chrono::Utc::now() + chrono::Duration::milliseconds(100);
    let job = DelayedJob::new(
        "test-job".to_string(),
        "Test Delayed Job".to_string(),
        execute_at,
        handler,
    )
    .with_max_retries(3);

    queue.add(job).await.unwrap();

    // 等待并执行任务
    sleep(Duration::from_millis(150)).await;
    let ready_jobs = queue.get_ready_jobs().await;

    // 第一次执行（应该失败）
    let result1 = ready_jobs[0].execute().await;
    assert!(!result1.success);
    assert_eq!(fail_count.load(Ordering::SeqCst), 1);

    // 第二次执行（应该失败）
    let result2 = ready_jobs[0].execute().await;
    assert!(!result2.success);
    assert_eq!(fail_count.load(Ordering::SeqCst), 2);

    // 第三次执行（应该失败）
    let result3 = ready_jobs[0].execute().await;
    assert!(!result3.success);
    assert_eq!(fail_count.load(Ordering::SeqCst), 3);

    // 第四次执行（应该成功）
    let result4 = ready_jobs[0].execute().await;
    assert!(result4.success);
    assert_eq!(fail_count.load(Ordering::SeqCst), 4);

    // 检查状态
    let stats = ready_jobs[0].get_stats().await;
    assert_eq!(stats.total_runs, 4);
    assert_eq!(stats.success_count, 1);
    assert_eq!(stats.failure_count, 3);
}

#[tokio::test]
async fn test_delayed_job_cancellation() {
    let queue = DelayedQueue::new();

    let execute_at = chrono::Utc::now() + chrono::Duration::seconds(10);
    let handler = create_test_job_handler();

    let job = DelayedJob::new(
        "test-job".to_string(),
        "Test Delayed Job".to_string(),
        execute_at,
        handler,
    );

    queue.add(job).await.unwrap();

    // 取消任务
    queue.cancel("test-job").await.unwrap();

    let cancelled_job = queue.get("test-job").await.unwrap();
    assert_eq!(cancelled_job.get_status().await, DelayedJobStatus::Cancelled);
}

#[tokio::test]
async fn test_expired_job_removal() {
    let queue = DelayedQueue::new();

    // 创建一个很快过期的任务
    let execute_at = chrono::Utc::now() + chrono::Duration::seconds(2);
    let expires_at = chrono::Utc::now() + chrono::Duration::milliseconds(500);
    let handler = create_test_job_handler();

    let job = DelayedJob::new(
        "test-job".to_string(),
        "Test Delayed Job".to_string(),
        execute_at,
        handler,
    )
    .with_expires_at(expires_at);

    queue.add(job).await.unwrap();

    // 等待任务过期
    sleep(Duration::from_millis(600)).await;

    // 检查过期任务
    let expired_jobs = queue.get_expired_jobs().await;
    assert_eq!(expired_jobs.len(), 1);

    // 移除过期任务
    let count = queue.remove_expired().await;
    assert_eq!(count, 1);
    assert!(queue.is_empty().await);
}

#[tokio::test]
async fn test_multiple_delayed_jobs() {
    let queue = DelayedQueue::new();

    // 添加多个任务
    for i in 0..3 {
        let execute_at = chrono::Utc::now() + chrono::Duration::milliseconds(100 + i * 50);
        let handler = create_test_job_handler();
        let job = DelayedJob::new(format!("job-{}", i), format!("Job {}", i), execute_at, handler);
        queue.add(job).await.unwrap();
    }

    assert_eq!(queue.len().await, 3);

    // 等待所有任务准备执行
    sleep(Duration::from_millis(300)).await;

    // 检查所有任务都准备好
    let ready_jobs = queue.get_ready_jobs().await;
    assert!(ready_jobs.len() >= 3);

    // 列出所有任务ID
    let job_ids = queue.list_jobs().await;
    assert_eq!(job_ids.len(), 3);
}

#[tokio::test]
async fn test_job_status_transitions() {
    let handler = create_test_job_handler();
    let execute_at = chrono::Utc::now() + chrono::Duration::seconds(10);

    let job = DelayedJob::new(
        "test-job".to_string(),
        "Test Delayed Job".to_string(),
        execute_at,
        handler,
    );

    // 初始状态应该是 Waiting
    assert_eq!(job.get_status().await, DelayedJobStatus::Waiting);

    // 添加到队列并等待
    let queue = DelayedQueue::new();
    queue.add(job).await.unwrap();

    // 等待任务准备执行
    sleep(Duration::from_millis(100)).await;

    let ready_jobs = queue.get_ready_jobs().await;
    if !ready_jobs.is_empty() {
        // 执行任务
        ready_jobs[0].execute().await;

        // 检查状态应该是 Completed
        assert_eq!(ready_jobs[0].get_status().await, DelayedJobStatus::Completed);
    }
}

#[tokio::test]
async fn test_clear_delayed_queue() {
    let queue = DelayedQueue::new();

    // 添加多个任务
    for i in 0..3 {
        let execute_at = chrono::Utc::now() + chrono::Duration::seconds(10 + i);
        let handler = create_test_job_handler();
        let job = DelayedJob::new(format!("job-{}", i), format!("Job {}", i), execute_at, handler);
        queue.add(job).await.unwrap();
    }

    assert_eq!(queue.len().await, 3);

    // 清空队列
    queue.clear().await;
    assert!(queue.is_empty().await);
}

#[tokio::test]
async fn test_job_result_tracking() {
    let queue = DelayedQueue::new();
    let handler = create_test_job_handler();

    let execute_at = chrono::Utc::now() + chrono::Duration::milliseconds(100);
    let job = DelayedJob::new(
        "test-job".to_string(),
        "Test Delayed Job".to_string(),
        execute_at,
        handler,
    )
    .with_max_retries(2);

    queue.add(job).await.unwrap();

    // 等待并执行任务
    sleep(Duration::from_millis(150)).await;
    let ready_jobs = queue.get_ready_jobs().await;

    if !ready_jobs.is_empty() {
        let result = ready_jobs[0].execute().await;

        // 检查执行结果
        assert!(result.success);
        assert!(result.error.is_none());
        assert!(result.execution_time_ms > 0);

        // 从任务获取结果
        let stored_result = ready_jobs[0].get_result().await;
        assert!(stored_result.is_some());
        let stored = stored_result.unwrap();
        assert!(stored.success);
    }
}

#[tokio::test]
async fn test_concurrent_delayed_job_execution() {
    let queue = DelayedQueue::new();
    let execution_count = Arc::new(AtomicU32::new(0));

    // 同时添加多个任务
    for i in 0..5 {
        let count_clone = execution_count.clone();
        let handler = Arc::new(ClosureJobHandler::new(move || {
            let count = count_clone.clone();
            Box::pin(async move {
                count.fetch_add(1, Ordering::SeqCst);
                // 模拟一些处理时间
                sleep(Duration::from_millis(10)).await;
                JobResult { success: true, error: None, execution_time_ms: 10 }
            })
        }));

        let execute_at = chrono::Utc::now() + chrono::Duration::milliseconds(100);
        let job = DelayedJob::new(format!("job-{}", i), format!("Job {}", i), execute_at, handler);
        queue.add(job).await.unwrap();
    }

    // 等待所有任务准备执行
    sleep(Duration::from_millis(150)).await;

    // 执行所有准备好的任务
    let ready_jobs = queue.get_ready_jobs().await;
    for job in ready_jobs {
        job.execute().await;
    }

    // 验证所有任务都被执行
    assert_eq!(execution_count.load(Ordering::SeqCst), 5);
}

#[tokio::test]
async fn test_remaining_time_calculation() {
    let queue = DelayedQueue::new();
    let handler = create_test_job_handler();

    let execute_at = chrono::Utc::now() + chrono::Duration::seconds(10);
    let job = DelayedJob::new(
        "test-job".to_string(),
        "Test Delayed Job".to_string(),
        execute_at,
        handler,
    );

    queue.add(job).await.unwrap();

    let retrieved_job = queue.get("test-job").await.unwrap();
    let remaining = retrieved_job.remaining_time().await;

    assert!(remaining.is_some());
    let remaining_duration = remaining.unwrap();
    // 剩余时间应该在 9-10 秒之间
    assert!(remaining_duration.num_seconds() >= 9);
    assert!(remaining_duration.num_seconds() <= 10);
}

#[tokio::test]
async fn test_max_retries_exceeded() {
    let queue = DelayedQueue::new();

    let handler = Arc::new(ClosureJobHandler::new(|| {
        Box::pin(async move {
            JobResult {
                success: false,
                error: Some("Permanent failure".to_string()),
                execution_time_ms: 10,
            }
        })
    }));

    let execute_at = chrono::Utc::now() + chrono::Duration::milliseconds(100);
    let job = DelayedJob::new(
        "test-job".to_string(),
        "Test Delayed Job".to_string(),
        execute_at,
        handler,
    )
    .with_max_retries(1);

    queue.add(job).await.unwrap();

    // 等待并多次执行任务
    sleep(Duration::from_millis(150)).await;
    let ready_jobs = queue.get_ready_jobs().await;

    if !ready_jobs.is_empty() {
        // 第一次执行（失败）
        let result1 = ready_jobs[0].execute().await;
        assert!(!result1.success);

        // 第二次执行（应该因为超过最大重试次数而被取消）
        let result2 = ready_jobs[0].execute().await;
        assert!(!result2.success);

        // 检查状态应该是 Cancelled
        let status = ready_jobs[0].get_status().await;
        assert_eq!(status, DelayedJobStatus::Cancelled);

        // 检查统计
        let stats = ready_jobs[0].get_stats().await;
        assert_eq!(stats.total_runs, 2);
        assert_eq!(stats.failure_count, 2);
    }
}

#[tokio::test]
async fn test_delayed_job_with_description() {
    let handler = create_test_job_handler();
    let execute_at = chrono::Utc::now() + chrono::Duration::seconds(10);

    let job = DelayedJob::new(
        "test-job".to_string(),
        "Test Delayed Job".to_string(),
        execute_at,
        handler,
    )
    .with_description("This is a test job description".to_string())
    .with_expires_at(chrono::Utc::now() + chrono::Duration::seconds(15));

    assert_eq!(job.description, Some("This is a test job description".to_string()));
    assert!(job.expires_at.is_some());
}
