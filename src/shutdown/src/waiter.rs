//! 任务等待器

use crate::error::{Error, Result};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// 运行中
    Running,
    /// 完成
    Completed,
    /// 失败
    Failed,
}

/// 任务信息
#[derive(Debug, Clone)]
pub struct TaskInfo {
    /// 任务ID
    pub id: String,
    /// 任务名称
    pub name: String,
    /// 任务状态
    pub status: TaskStatus,
}

/// 任务等待器
pub struct TaskWaiter {
    /// 运行中的任务数
    running_count: AtomicUsize,
    /// 任务列表
    tasks: std::sync::Mutex<HashMap<String, TaskInfo>>,
}

impl TaskWaiter {
    /// 创建新的任务等待器
    pub fn new() -> Self {
        Self { running_count: AtomicUsize::new(0), tasks: std::sync::Mutex::new(HashMap::new()) }
    }

    /// 注册任务
    pub fn register_task(&self, id: &str, name: &str) {
        let task =
            TaskInfo { id: id.to_string(), name: name.to_string(), status: TaskStatus::Running };
        self.tasks.lock().unwrap().insert(id.to_string(), task);
        self.running_count.fetch_add(1, Ordering::SeqCst);
    }

    /// 完成任务
    pub fn complete_task(&self, id: &str) -> Result<()> {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(id) {
            task.status = TaskStatus::Completed;
            self.running_count.fetch_sub(1, Ordering::SeqCst);
        }
        Ok(())
    }

    /// 任务失败
    pub fn fail_task(&self, id: &str) -> Result<()> {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(id) {
            task.status = TaskStatus::Failed;
            self.running_count.fetch_sub(1, Ordering::SeqCst);
        }
        Ok(())
    }

    /// 获取运行中任务数
    pub fn running_count(&self) -> usize {
        self.running_count.load(Ordering::SeqCst)
    }

    /// 等待所有任务完成
    pub async fn wait_all(&self, timeout: Duration) -> Result<()> {
        let start = std::time::Instant::now();

        loop {
            if self.running_count() == 0 {
                return Ok(());
            }

            if start.elapsed() > timeout {
                return Err(Error::Timeout(format!(
                    "等待任务超时，仍有 {} 个任务运行中",
                    self.running_count()
                )));
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// 获取所有任务列表
    pub fn get_tasks(&self) -> Vec<TaskInfo> {
        self.tasks.lock().unwrap().values().cloned().collect()
    }

    /// 清空任务列表
    pub fn clear(&self) {
        self.tasks.lock().unwrap().clear();
        self.running_count.store(0, Ordering::SeqCst);
    }
}

impl Default for TaskWaiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_waiter_creation() {
        let waiter = TaskWaiter::new();
        assert_eq!(waiter.running_count(), 0);
    }

    #[test]
    fn test_register_task() {
        let waiter = TaskWaiter::new();
        waiter.register_task("task-1", "测试任务");

        assert_eq!(waiter.running_count(), 1);
    }

    #[test]
    fn test_complete_task() {
        let waiter = TaskWaiter::new();
        waiter.register_task("task-1", "测试任务");
        waiter.complete_task("task-1").unwrap();

        assert_eq!(waiter.running_count(), 0);
    }

    #[test]
    fn test_get_tasks() {
        let waiter = TaskWaiter::new();
        waiter.register_task("task-1", "任务1");

        let tasks = waiter.get_tasks();
        assert_eq!(tasks.len(), 1);
    }
}
