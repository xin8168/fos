//! FOS Bus 集成测试
//!
//! 测试 Bus 模块与其他模块的集成

use fos_bus::{Task, TaskPriority, TaskQueue, TaskScheduler, TaskStatus};

/// 测试任务调度器与校验器的集成
#[tokio::test]
async fn test_scheduler_validator_integration() {
    // 创建调度器
    let scheduler = TaskScheduler::default();

    // 创建有效的任务
    let task = Task::new(
        "清理桌面文件".to_string(),
        "device_control".to_string(),
        vec!["列出文件".to_string(), "删除临时文件".to_string()],
    );

    // 提交任务
    let task_id = scheduler.submit(task).await.unwrap();
    assert!(!task_id.is_empty());

    // 验证任务状态
    let status = scheduler.get_status(&task_id).await.unwrap();
    assert!(matches!(status, TaskStatus::Pending | TaskStatus::Queued));
}

/// 测试任务优先级与执行器集成
#[tokio::test]
async fn test_priority_executor_integration() {
    let scheduler = TaskScheduler::default();

    // 创建不同优先级的任务
    let low_task =
        Task::new("低优先级任务".to_string(), "test".to_string(), vec!["step1".to_string()])
            .with_priority(TaskPriority::Low);

    let high_task =
        Task::new("高优先级任务".to_string(), "test".to_string(), vec!["step1".to_string()])
            .with_priority(TaskPriority::High);

    let critical_task =
        Task::new("紧急任务".to_string(), "test".to_string(), vec!["step1".to_string()])
            .with_priority(TaskPriority::Critical);

    // 提交任务
    let low_id = scheduler.submit(low_task).await.unwrap();
    let high_id = scheduler.submit(high_task).await.unwrap();
    let critical_id = scheduler.submit(critical_task).await.unwrap();

    // 验证所有任务都已提交
    assert!(!low_id.is_empty());
    assert!(!high_id.is_empty());
    assert!(!critical_id.is_empty());
}

/// 测试任务队列容量管理
#[tokio::test]
async fn test_queue_capacity_integration() {
    let queue = TaskQueue::new(3);

    // 创建任务
    let task1 = Task::new("任务1".to_string(), "test".to_string(), vec!["step".to_string()]);
    let task2 = Task::new("任务2".to_string(), "test".to_string(), vec!["step".to_string()]);
    let task3 = Task::new("任务3".to_string(), "test".to_string(), vec!["step".to_string()]);

    // 入队
    assert!(queue.enqueue(task1).await.is_ok());
    assert!(queue.enqueue(task2).await.is_ok());
    assert!(queue.enqueue(task3).await.is_ok());

    // 队列已满
    let task4 = Task::new("任务4".to_string(), "test".to_string(), vec!["step".to_string()]);
    assert!(queue.enqueue(task4).await.is_err());

    // 出队后可以再次入队
    let _ = queue.dequeue().await;
    let task5 = Task::new("任务5".to_string(), "test".to_string(), vec!["step".to_string()]);
    assert!(queue.enqueue(task5).await.is_ok());
}

/// 测试任务取消功能
#[tokio::test]
async fn test_task_cancellation() {
    let scheduler = TaskScheduler::default();

    let task = Task::new("可取消任务".to_string(), "test".to_string(), vec!["step".to_string()]);
    let task_id = scheduler.submit(task).await.unwrap();

    // 取消任务
    let result = scheduler.cancel(&task_id).await;
    assert!(result.is_ok());

    // 验证任务状态
    let status = scheduler.get_status(&task_id).await.unwrap();
    assert!(matches!(status, TaskStatus::Cancelled));
}

/// 测试任务步骤执行
#[test]
fn test_task_step_execution() {
    let mut task = Task::new(
        "多步骤任务".to_string(),
        "test".to_string(),
        vec!["步骤1".to_string(), "步骤2".to_string(), "步骤3".to_string()],
    );

    // 开始执行
    task.start();
    assert_eq!(task.status, TaskStatus::Running);
    assert!(task.started_at.is_some());

    // 完成步骤1
    task.complete_step("结果1".to_string());
    assert_eq!(task.current_step, 1);
    assert!(task.steps[0].completed);
    assert!(task.has_more_steps());

    // 完成步骤2
    task.complete_step("结果2".to_string());
    assert_eq!(task.current_step, 2);
    assert!(task.has_more_steps());

    // 完成步骤3
    task.complete_step("结果3".to_string());
    assert_eq!(task.current_step, 3);
    assert!(!task.has_more_steps());
}

/// 测试任务超时检测
#[test]
fn test_task_timeout_detection() {
    let task = Task::new("超时任务".to_string(), "test".to_string(), vec!["step".to_string()])
        .with_timeout(1); // 1秒超时

    // 未开始时不超时
    assert!(!task.is_timeout());
}

/// 测试任务元数据管理
#[test]
fn test_task_metadata() {
    let task = Task::new("带元数据任务".to_string(), "test".to_string(), vec!["step".to_string()])
        .with_metadata("device_id".to_string(), "device-001".to_string())
        .with_metadata("owner".to_string(), "user-123".to_string());

    assert_eq!(task.metadata.get("device_id"), Some(&"device-001".to_string()));
    assert_eq!(task.metadata.get("owner"), Some(&"user-123".to_string()));
}

/// 测试优先级队列排序
#[tokio::test]
async fn test_priority_queue_ordering() {
    let queue = TaskQueue::new(10);

    // 创建不同优先级的任务
    let low = Task::new("低".to_string(), "test".to_string(), vec!["s".to_string()])
        .with_priority(TaskPriority::Low);
    let normal = Task::new("普通".to_string(), "test".to_string(), vec!["s".to_string()])
        .with_priority(TaskPriority::Normal);
    let high = Task::new("高".to_string(), "test".to_string(), vec!["s".to_string()])
        .with_priority(TaskPriority::High);
    let critical = Task::new("紧急".to_string(), "test".to_string(), vec!["s".to_string()])
        .with_priority(TaskPriority::Critical);

    // 按随机顺序入队
    queue.enqueue(normal).await.unwrap();
    queue.enqueue(critical).await.unwrap();
    queue.enqueue(low).await.unwrap();
    queue.enqueue(high).await.unwrap();

    // 出队顺序应该是 Critical -> High -> Normal -> Low
    let first = queue.dequeue().await.unwrap();
    assert_eq!(first.priority, TaskPriority::Critical);

    let second = queue.dequeue().await.unwrap();
    assert_eq!(second.priority, TaskPriority::High);

    let third = queue.dequeue().await.unwrap();
    assert_eq!(third.priority, TaskPriority::Normal);

    let fourth = queue.dequeue().await.unwrap();
    assert_eq!(fourth.priority, TaskPriority::Low);
}

/// 测试调度器统计信息
#[tokio::test]
async fn test_scheduler_stats() {
    let scheduler = TaskScheduler::default();

    // 提交多个任务
    for i in 0..5 {
        let task = Task::new(format!("任务{}", i), "test".to_string(), vec!["step".to_string()]);
        scheduler.submit(task).await.unwrap();
    }

    // 获取统计
    let stats = scheduler.stats().await;
    assert!(stats.total_tasks >= 5);
}

/// 测试任务队列基本操作
#[tokio::test]
async fn test_queue_basic_operations() {
    let queue = TaskQueue::new(100);

    // 初始状态
    assert!(queue.is_empty().await);
    assert_eq!(queue.len().await, 0);

    // 入队
    let task = Task::new("测试任务".to_string(), "test".to_string(), vec!["step".to_string()]);
    let id = queue.enqueue(task).await.unwrap();

    assert!(!queue.is_empty().await);
    assert_eq!(queue.len().await, 1);

    // 获取任务
    let retrieved = queue.get(&id).await.unwrap();
    assert_eq!(retrieved.name, "测试任务");

    // 出队
    let dequeued = queue.dequeue().await.unwrap();
    assert_eq!(dequeued.id, id);
    assert!(queue.is_empty().await);
}

/// 测试任务状态更新
#[tokio::test]
async fn test_task_status_update() {
    let queue = TaskQueue::new(10);

    let task = Task::new("状态测试".to_string(), "test".to_string(), vec!["step".to_string()]);
    let id = queue.enqueue(task).await.unwrap();

    // 更新状态
    queue.update_status(&id, TaskStatus::Running).await.unwrap();

    let updated = queue.get(&id).await.unwrap();
    assert_eq!(updated.status, TaskStatus::Running);

    // 再次更新
    queue.update_status(&id, TaskStatus::Completed).await.unwrap();

    let completed = queue.get(&id).await.unwrap();
    assert_eq!(completed.status, TaskStatus::Completed);
}
