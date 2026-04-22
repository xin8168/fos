//! # 任务调度器模块

use crate::error::Result;
use crate::executor::ExecutorManager;
use crate::queue::TaskQueue;
use crate::{Task, TaskPriority, TaskResult, TaskStatus};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 调度器配置
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// 队列最大容量
    pub queue_size: usize,

    /// 调度间隔（毫秒）
    pub schedule_interval_ms: u64,

    /// 是否启用优先级调度
    pub enable_priority: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self { queue_size: 1000, schedule_interval_ms: 100, enable_priority: true }
    }
}

/// 任务调度器
pub struct TaskScheduler {
    /// 配置
    config: SchedulerConfig,

    /// 任务队列
    queue: Arc<TaskQueue>,

    /// 执行器管理器
    executor: Arc<RwLock<ExecutorManager>>,

    /// 运行状态
    running: Arc<RwLock<bool>>,
}

impl TaskScheduler {
    /// 创建新调度器
    pub fn new(config: SchedulerConfig) -> Self {
        let queue = Arc::new(TaskQueue::new(config.queue_size));
        let executor = Arc::new(RwLock::new(ExecutorManager::default()));
        let running = Arc::new(RwLock::new(false));

        Self { config, queue, executor, running }
    }

    /// 启动调度器
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = true;

        tracing::info!("🚀 任务调度器启动");

        Ok(())
    }

    /// 停止调度器
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;

        tracing::info!("🛑 任务调度器停止");

        Ok(())
    }

    /// 提交任务
    pub async fn submit(&self, task: Task) -> Result<String> {
        // 入队
        let id = self.queue.enqueue(task).await?;

        tracing::debug!("任务已入队: {}", id);

        Ok(id)
    }

    /// 提交高优先级任务
    pub async fn submit_high_priority(
        &self,
        name: String,
        task_type: String,
        steps: Vec<String>,
    ) -> Result<String> {
        let task = Task::new(name, task_type, steps).with_priority(TaskPriority::High);

        self.submit(task).await
    }

    /// 提交紧急任务
    pub async fn submit_critical(
        &self,
        name: String,
        task_type: String,
        steps: Vec<String>,
    ) -> Result<String> {
        let task = Task::new(name, task_type, steps).with_priority(TaskPriority::Critical);

        self.submit(task).await
    }

    /// 获取任务状态
    pub async fn get_status(&self, id: &str) -> Result<TaskStatus> {
        let task = self.queue.get(id).await?;
        Ok(task.status)
    }

    /// 取消任务
    pub async fn cancel(&self, id: &str) -> Result<()> {
        self.queue.update_status(id, TaskStatus::Cancelled).await?;
        tracing::info!("任务已取消: {}", id);
        Ok(())
    }

    /// 获取队列统计信息
    pub async fn stats(&self) -> SchedulerStats {
        let queue_len = self.queue.len().await;
        let pending_count = self.queue.pending_count().await;

        SchedulerStats {
            total_tasks: queue_len,
            pending_tasks: pending_count,
            running_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
        }
    }

    /// 调度循环（后台任务）
    pub async fn schedule_loop(&self) {
        let interval = tokio::time::Duration::from_millis(self.config.schedule_interval_ms);

        loop {
            let running = *self.running.read().await;
            if !running {
                break;
            }

            // 尝试从队列获取任务
            if let Ok(task) = self.queue.dequeue().await {
                let executor = self.executor.read().await;
                if let Err(e) = executor.submit(task).await {
                    tracing::error!("提交任务到执行器失败: {}", e);
                }
            }

            tokio::time::sleep(interval).await;
        }
    }
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new(SchedulerConfig::default())
    }
}

/// 调度器统计信息
#[derive(Debug, Clone)]
pub struct SchedulerStats {
    /// 总任务数
    pub total_tasks: usize,

    /// 待执行任务数
    pub pending_tasks: usize,

    /// 执行中任务数
    pub running_tasks: usize,

    /// 已完成任务数
    pub completed_tasks: usize,

    /// 失败任务数
    pub failed_tasks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let scheduler = TaskScheduler::new(SchedulerConfig::default());
        assert!(true);
    }

    #[tokio::test]
    async fn test_submit_task() {
        let scheduler = TaskScheduler::new(SchedulerConfig::default());
        let task = Task::new("测试任务".to_string(), "test".to_string(), vec!["步骤1".to_string()]);

        let result = scheduler.submit(task).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_status() {
        let scheduler = TaskScheduler::new(SchedulerConfig::default());
        let task = Task::new("测试任务".to_string(), "test".to_string(), vec!["步骤1".to_string()]);

        let id = scheduler.submit(task).await.unwrap();
        let status = scheduler.get_status(&id).await.unwrap();
        assert_eq!(status, TaskStatus::Pending);
    }

    #[tokio::test]
    async fn test_cancel_task() {
        let scheduler = TaskScheduler::new(SchedulerConfig::default());
        let task = Task::new("测试任务".to_string(), "test".to_string(), vec!["步骤1".to_string()]);

        let id = scheduler.submit(task).await.unwrap();
        scheduler.cancel(&id).await.unwrap();

        let status = scheduler.get_status(&id).await.unwrap();
        assert_eq!(status, TaskStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_stats() {
        let scheduler = TaskScheduler::new(SchedulerConfig::default());

        for i in 0..5 {
            let task =
                Task::new(format!("任务{}", i), "test".to_string(), vec!["步骤".to_string()]);
            scheduler.submit(task).await.unwrap();
        }

        let stats = scheduler.stats().await;
        assert_eq!(stats.total_tasks, 5);
    }
}
