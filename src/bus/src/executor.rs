//! # 任务执行器模块

use crate::error::{BusError, Result};
use crate::{Task, TaskResult};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

/// 执行器配置
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// 工作线程数
    pub workers: usize,

    /// 任务超时（秒）
    pub task_timeout_secs: u64,

    /// 心跳间隔（秒）
    pub heartbeat_interval_secs: u64,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self { workers: 4, task_timeout_secs: 300, heartbeat_interval_secs: 10 }
    }
}

/// 任务执行器
pub struct TaskExecutor {
    /// 配置
    config: ExecutorConfig,

    /// 任务接收器
    task_rx: mpsc::Receiver<Task>,

    /// 结果发送器
    result_tx: mpsc::Sender<(String, TaskResult)>,
}

impl TaskExecutor {
    /// 创建新执行器
    pub fn new(
        config: ExecutorConfig,
        task_rx: mpsc::Receiver<Task>,
        result_tx: mpsc::Sender<(String, TaskResult)>,
    ) -> Self {
        Self { config, task_rx, result_tx }
    }

    /// 运行执行器
    pub async fn run(&mut self) {
        tracing::info!("🚀 任务执行器启动，工作线程数: {}", self.config.workers);

        while let Some(task) = self.task_rx.recv().await {
            tracing::info!("执行任务: {} ({})", task.name, task.id);

            let result_tx = self.result_tx.clone();
            let task_timeout = Duration::from_secs(self.config.task_timeout_secs);

            // 执行任务（带超时）
            let task_id = task.id.clone();

            tokio::spawn(async move {
                let start = std::time::Instant::now();

                let result = match timeout(task_timeout, Self::execute_task(task)).await {
                    Ok(res) => res,
                    Err(_) => TaskResult {
                        success: false,
                        output: String::new(),
                        error: Some("任务执行超时".to_string()),
                        duration_ms: start.elapsed().as_millis() as u64,
                    },
                };

                if let Err(e) = result_tx.send((task_id, result)).await {
                    tracing::error!("发送任务结果失败: {}", e);
                }
            });
        }
    }

    /// 执行单个任务
    async fn execute_task(mut task: Task) -> TaskResult {
        task.start();
        let start = std::time::Instant::now();

        // 执行每个步骤
        while task.has_more_steps() {
            // 模拟执行
            tokio::time::sleep(Duration::from_millis(100)).await;

            // 执行下一步
            task.advance_step("步骤完成".to_string());
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        TaskResult {
            success: true, output: "任务执行成功".to_string(), error: None, duration_ms
        }
    }
}

/// 执行器管理器
pub struct ExecutorManager {
    /// 配置
    #[allow(dead_code)]
    config: ExecutorConfig,

    /// 任务发送器
    task_tx: mpsc::Sender<Task>,

    /// 结果接收器
    result_rx: mpsc::Receiver<(String, TaskResult)>,
}

impl ExecutorManager {
    /// 创建新管理器
    pub fn new(config: ExecutorConfig) -> Self {
        let (task_tx, task_rx) = mpsc::channel(100);
        let (result_tx, result_rx) = mpsc::channel(100);

        // 启动执行器
        let mut executor = TaskExecutor::new(config.clone(), task_rx, result_tx);

        tokio::spawn(async move {
            executor.run().await;
        });

        Self { config, task_tx, result_rx }
    }

    /// 提交任务
    pub async fn submit(&self, task: Task) -> Result<String> {
        let id = task.id.clone();
        self.task_tx.send(task).await.map_err(|e| BusError::Internal(e.to_string()))?;
        Ok(id)
    }

    /// 等待结果
    pub async fn wait_result(&mut self) -> Option<(String, TaskResult)> {
        self.result_rx.recv().await
    }
}

impl Default for ExecutorManager {
    fn default() -> Self {
        Self::new(ExecutorConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_config_default() {
        let config = ExecutorConfig::default();
        assert_eq!(config.workers, 4);
        assert_eq!(config.task_timeout_secs, 300);
    }

    #[tokio::test]
    async fn test_executor_manager_creation() {
        let _manager = ExecutorManager::new(ExecutorConfig::default());
        // 测试基本创建
        assert!(true);
    }

    #[tokio::test]
    async fn test_submit_task() {
        let manager = ExecutorManager::new(ExecutorConfig::default());
        let task = Task::new("测试任务".to_string(), "test".to_string(), vec!["步骤1".to_string()]);

        let result = manager.submit(task).await;
        assert!(result.is_ok());
    }
}
