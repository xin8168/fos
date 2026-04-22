//! # FOS Bus - 运动神经层
//!
//! FOS神经元控制器的第三层神经网络：运动神经
//!
//! 类比于人体神经系统的运动神经，负责动作的执行和协调
//!
//! ## 核心职责
//! - 接收脊髓许可的信号（已校验通过）
//! - 任务调度与执行
//! - 执行队列和优先级管理
//! - 超时控制与重试
//! - 反馈闭环
//!
//! ## 信号处理流程
//! 信号许可 → 任务分解 → 优先级队列 → 执行调度 → 具身执行 → 结果反馈
//!
//! ## 安全铁律
//! - 只执行经过脊髓校验的信号
//! - 拒绝执行未授权的任务
//! - 所有执行必须有超时控制
//! - 执行结果必须反馈到控制层

pub mod config;
pub mod error;
pub mod executor;
pub mod queue;
pub mod scheduler;

pub use error::{BusError, Result};
pub use executor::TaskExecutor;
pub use queue::TaskQueue;
pub use scheduler::TaskScheduler;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 任务优先级
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    /// 低优先级
    Low = 1,

    /// 普通优先级
    Normal = 2,

    /// 高优先级
    High = 3,

    /// 紧急
    Critical = 4,
}

/// 任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// 待执行
    Pending,

    /// 排队中
    Queued,

    /// 执行中
    Running,

    /// 已完成
    Completed,

    /// 失败
    Failed,

    /// 已取消
    Cancelled,

    /// 超时
    Timeout,
}

/// 执行任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 任务ID
    pub id: String,

    /// 任务名称
    pub name: String,

    /// 任务类型
    pub task_type: String,

    /// 优先级
    pub priority: TaskPriority,

    /// 状态
    pub status: TaskStatus,

    /// 执行步骤
    pub steps: Vec<TaskStep>,

    /// 当前步骤索引
    pub current_step: usize,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 开始时间
    pub started_at: Option<DateTime<Utc>>,

    /// 完成时间
    pub completed_at: Option<DateTime<Utc>>,

    /// 超时时间（秒）
    pub timeout_secs: u64,

    /// 重试次数
    pub retry_count: u32,

    /// 最大重试次数
    pub max_retries: u32,

    /// 元数据
    pub metadata: HashMap<String, String>,

    /// 执行结果
    pub result: Option<TaskResult>,
}

/// 任务步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStep {
    /// 步骤索引
    pub index: usize,

    /// 步骤名称
    pub name: String,

    /// 步骤内容
    pub content: String,

    /// 是否完成
    pub completed: bool,

    /// 执行结果
    pub result: Option<String>,
}

/// 任务执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// 是否成功
    pub success: bool,

    /// 输出内容
    pub output: String,

    /// 错误信息
    pub error: Option<String>,

    /// 执行时间（毫秒）
    pub duration_ms: u64,
}

impl Task {
    /// 创建新任务
    pub fn new(name: String, task_type: String, steps: Vec<String>) -> Self {
        let task_steps: Vec<TaskStep> = steps
            .into_iter()
            .enumerate()
            .map(|(index, content)| TaskStep {
                index,
                name: format!("步骤{}", index + 1),
                content,
                completed: false,
                result: None,
            })
            .collect();

        Self {
            id: Uuid::new_v4().to_string(),
            name,
            task_type,
            priority: TaskPriority::Normal,
            status: TaskStatus::Pending,
            steps: task_steps,
            current_step: 0,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            timeout_secs: 300, // 5分钟
            retry_count: 0,
            max_retries: 3,
            metadata: HashMap::new(),
            result: None,
        }
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    /// 设置超时时间
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// 设置元数据
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// 开始执行
    pub fn start(&mut self) {
        self.status = TaskStatus::Running;
        self.started_at = Some(Utc::now());
    }

    /// 完成当前步骤
    pub fn complete_step(&mut self, result: String) {
        // 直接使用 current_step 而不是 steps.len()
        if self.current_step < self.steps.len() {
            self.steps[self.current_step].completed = true;
            self.steps[self.current_step].result = Some(result);
            self.current_step += 1;
        }
    }

    /// 检查是否还有更多步骤
    pub fn has_more_steps(&self) -> bool {
        // 直接比较，不调用 len()
        self.current_step < self.steps.len()
    }

    /// 执行下一步
    pub fn advance_step(&mut self, result: String) -> bool {
        let has_more = self.has_more_steps();
        if has_more {
            let idx = self.current_step;
            self.steps[idx].completed = true;
            self.steps[idx].result = Some(result);
            self.current_step += 1;
        }
        has_more
    }

    /// 完成任务
    pub fn complete(&mut self, result: TaskResult) {
        self.status = if result.success { TaskStatus::Completed } else { TaskStatus::Failed };
        self.completed_at = Some(Utc::now());
        self.result = Some(result);
    }

    /// 检查是否超时
    pub fn is_timeout(&self) -> bool {
        if let Some(started_at) = self.started_at {
            let elapsed = (Utc::now() - started_at).num_seconds() as u64;
            return elapsed > self.timeout_secs;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "测试任务".to_string(),
            "device_control".to_string(),
            vec!["步骤1".to_string(), "步骤2".to_string()],
        );

        assert!(!task.id.is_empty());
        assert_eq!(task.name, "测试任务");
        assert_eq!(task.steps.len(), 2);
        assert_eq!(task.priority, TaskPriority::Normal);
    }

    #[test]
    fn test_task_with_priority() {
        let task = Task::new("测试任务".to_string(), "test".to_string(), vec!["步骤".to_string()])
            .with_priority(TaskPriority::High);

        assert_eq!(task.priority, TaskPriority::High);
    }

    #[test]
    fn test_task_steps() {
        let mut task = Task::new(
            "测试任务".to_string(),
            "test".to_string(),
            vec!["步骤1".to_string(), "步骤2".to_string()],
        );

        task.start();
        assert_eq!(task.status, TaskStatus::Running);

        task.complete_step("结果1".to_string());
        assert_eq!(task.current_step, 1);
        assert!(task.steps[0].completed);

        task.complete_step("结果2".to_string());
        assert_eq!(task.current_step, 2);
        assert!(task.steps[1].completed);
    }

    #[test]
    fn test_task_timeout() {
        let task = Task::new("测试任务".to_string(), "test".to_string(), vec!["步骤".to_string()])
            .with_timeout(0);

        assert!(!task.is_timeout()); // 未开始，不超时
    }
}
