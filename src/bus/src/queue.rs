//! # 任务队列模块

use crate::error::{BusError, Result};
use crate::{Task, TaskPriority, TaskStatus};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// 优先级任务包装（用于二叉堆排序）
#[derive(Debug, Clone)]
struct PriorityTask {
    priority: TaskPriority,
    created_at: chrono::DateTime<chrono::Utc>,
    task: Task,
}

impl Eq for PriorityTask {}

impl PartialEq for PriorityTask {
    fn eq(&self, other: &Self) -> bool {
        self.task.id == other.task.id
    }
}

impl PartialOrd for PriorityTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // 优先级高的排前面
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => {
                // 创建时间早的排前面
                other.created_at.cmp(&self.created_at)
            },
            ordering => ordering,
        }
    }
}

/// 任务队列
pub struct TaskQueue {
    /// 优先级队列
    queue: Arc<Mutex<BinaryHeap<PriorityTask>>>,

    /// 任务映射（用于快速查找）
    tasks: Arc<RwLock<std::collections::HashMap<String, Task>>>,

    /// 最大容量
    max_size: usize,
}

impl TaskQueue {
    /// 创建新队列
    pub fn new(max_size: usize) -> Self {
        Self {
            queue: Arc::new(Mutex::new(BinaryHeap::new())),
            tasks: Arc::new(RwLock::new(std::collections::HashMap::new())),
            max_size,
        }
    }

    /// 入队任务
    pub async fn enqueue(&self, task: Task) -> Result<String> {
        // 检查队列容量
        {
            let tasks = self.tasks.read().await;
            if tasks.len() >= self.max_size {
                return Err(BusError::QueueFull);
            }
        }

        let id = task.id.clone();

        // 添加到映射
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(id.clone(), task.clone());
        }

        // 添加到优先级队列
        {
            let mut queue = self.queue.lock().await;
            queue.push(PriorityTask { priority: task.priority, created_at: task.created_at, task });
        }

        Ok(id)
    }

    /// 出队任务
    pub async fn dequeue(&self) -> Result<Task> {
        let mut queue = self.queue.lock().await;

        loop {
            if let Some(priority_task) = queue.pop() {
                let task = priority_task.task;

                // 检查任务状态
                if task.status == TaskStatus::Pending {
                    // 从映射中移除
                    let mut tasks = self.tasks.write().await;
                    tasks.remove(&task.id);

                    return Ok(task);
                }
            } else {
                return Err(BusError::TaskNotFound("队列为空".to_string()));
            }
        }
    }

    /// 获取队列长度
    pub async fn len(&self) -> usize {
        let queue = self.queue.lock().await;
        queue.len()
    }

    /// 检查队列是否为空
    pub async fn is_empty(&self) -> bool {
        let queue = self.queue.lock().await;
        queue.is_empty()
    }

    /// 获取任务
    pub async fn get(&self, id: &str) -> Result<Task> {
        let tasks = self.tasks.read().await;
        tasks.get(id).cloned().ok_or_else(|| BusError::TaskNotFound(id.to_string()))
    }

    /// 更新任务状态
    pub async fn update_status(&self, id: &str, status: TaskStatus) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(id) {
            task.status = status;
            Ok(())
        } else {
            Err(BusError::TaskNotFound(id.to_string()))
        }
    }

    /// 移除任务
    pub async fn remove(&self, id: &str) -> Result<Task> {
        let mut tasks = self.tasks.write().await;
        tasks.remove(id).ok_or_else(|| BusError::TaskNotFound(id.to_string()))
    }

    /// 清空队列
    pub async fn clear(&self) {
        let mut queue = self.queue.lock().await;
        queue.clear();

        let mut tasks = self.tasks.write().await;
        tasks.clear();
    }

    /// 获取所有待执行任务数量
    pub async fn pending_count(&self) -> usize {
        let tasks = self.tasks.read().await;
        tasks.values().filter(|t| t.status == TaskStatus::Pending).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enqueue_dequeue() {
        let queue = TaskQueue::new(100);
        let task = Task::new("测试任务".to_string(), "test".to_string(), vec!["步骤".to_string()]);

        let id = queue.enqueue(task.clone()).await.unwrap();
        assert_eq!(queue.len().await, 1);

        let dequeued = queue.dequeue().await.unwrap();
        assert_eq!(dequeued.id, id);
    }

    #[tokio::test]
    async fn test_priority_order() {
        let queue = TaskQueue::new(100);

        // 入队不同优先级的任务
        let low = Task::new("低".to_string(), "test".to_string(), vec!["".to_string()])
            .with_priority(TaskPriority::Low);
        let high = Task::new("高".to_string(), "test".to_string(), vec!["".to_string()])
            .with_priority(TaskPriority::High);
        let normal = Task::new("普通".to_string(), "test".to_string(), vec!["".to_string()])
            .with_priority(TaskPriority::Normal);

        queue.enqueue(low).await.unwrap();
        queue.enqueue(high).await.unwrap();
        queue.enqueue(normal).await.unwrap();

        // 出队顺序应该是: 高 -> 普通 -> 低
        let first = queue.dequeue().await.unwrap();
        assert_eq!(first.priority, TaskPriority::High);

        let second = queue.dequeue().await.unwrap();
        assert_eq!(second.priority, TaskPriority::Normal);

        let third = queue.dequeue().await.unwrap();
        assert_eq!(third.priority, TaskPriority::Low);
    }

    #[tokio::test]
    async fn test_queue_full() {
        let queue = TaskQueue::new(2);

        let task1 = Task::new("任务1".to_string(), "test".to_string(), vec!["".to_string()]);
        let task2 = Task::new("任务2".to_string(), "test".to_string(), vec!["".to_string()]);
        let task3 = Task::new("任务3".to_string(), "test".to_string(), vec!["".to_string()]);

        queue.enqueue(task1).await.unwrap();
        queue.enqueue(task2).await.unwrap();

        let result = queue.enqueue(task3).await;
        assert!(matches!(result, Err(BusError::QueueFull)));
    }
}
