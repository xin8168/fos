#![allow(async_fn_in_trait)]
#![allow(async_fn_in_trait, dead_code)]

//! 定时备份调度器

use crate::{
    backup::{BackupItem, BackupPlan, BackupRetention, BackupSchedule, BackupType},
    error::Result,
};
use chrono::{DateTime, Datelike, Timelike, Utc};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::time::sleep;

/// 备份执行器 trait
#[async_trait::async_trait]
pub trait BackupExecutor: Send + Sync {
    /// 执行备份
    async fn backup(
        &self,
        item: &mut BackupItem,
    ) -> std::result::Result<String, crate::error::Error>;

    /// 验证备份
    async fn verify(&self, item: &BackupItem) -> std::result::Result<bool, crate::error::Error>;

    /// 清理过期备份
    async fn cleanup(&self, item: &BackupItem) -> std::result::Result<(), crate::error::Error>;

    /// 压缩备份
    async fn compress(&self, _item: &BackupItem) -> std::result::Result<(), crate::error::Error> {
        // 默认实现：不压缩
        Ok(())
    }
}

/// 备份调度器
pub struct BackupScheduler<E: BackupExecutor> {
    /// 备份计划索引
    plans: Arc<RwLock<Vec<BackupPlan>>>,
    /// 执行器
    executor: Arc<E>,
    /// 运行状态
    running: Arc<RwLock<bool>>,
    /// 通知通道
    notification_sender: Arc<broadcast::Sender<BackupNotification>>,
}

/// 备份通知
#[derive(Debug, Clone)]
pub struct BackupNotification {
    /// 备份项通知
    pub item: Option<BackupItem>,
    /// 计划ID
    pub plan_id: Option<String>,
    /// 通知类型
    pub notification_type: BackupNotificationType,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

/// 备份通知类型
#[derive(Debug, Clone)]
pub enum BackupNotificationType {
    /// 备份开始
    BackupStarted { plan_id: String, version: String },
    /// 备份失败
    BackupFailed { plan_id: String, error: String },
    /// 备份成功
    BackupCompleted { plan_id: String, size: u64 },
    /// 验证失败
    VerificationFailed { plan_id: String, error: String },
    /// 清理完成
    CleanupCompleted { plan_id: String, count: usize },
}

impl<E: BackupExecutor> BackupScheduler<E> {
    /// 创建新的调度器
    pub fn new(executor: E) -> Self {
        let (sender, _) = broadcast::channel::<BackupNotification>(100);
        Self {
            plans: Arc::new(RwLock::new(Vec::new())),
            executor: Arc::new(executor),
            running: Arc::new(RwLock::new(false)),
            notification_sender: Arc::new(sender),
        }
    }

    /// 注册备份计划
    pub async fn register_plan(&self, plan: BackupPlan) -> Result<()> {
        let mut plans: Vec<BackupPlan> = std::mem::take(&mut *self.plans.write().await);
        plans.push(plan);
        *self.plans.write().await = plans;
        Ok(())
    }

    /// 移除备份计划
    pub async fn remove_plan(&self, plan_id: &str) -> Result<bool> {
        let mut plans: Vec<BackupPlan> = std::mem::take(&mut *self.plans.write().await);
        let original_len = plans.len();
        plans.retain(|p| p.id != plan_id);
        let removed = plans.len() < original_len;
        *self.plans.write().await = plans;
        Ok(removed)
    }

    /// 获取所有备份计划
    pub async fn list_plans(&self) -> Vec<BackupPlan> {
        let plans: Vec<BackupPlan> = (*self.plans.read().await).clone();
        plans
    }

    /// 检查是否正在运行
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// 启动调度器
    pub async fn start(&self) {
        let mut running = self.running.write().await;
        *running = true;
        drop(running);

        let plans = self.plans.clone();
        let _executor = self.executor.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            // Self-contained loop that captures Arcs instead of self
            while *running.read().await {
                if plans.read().await.is_empty() {
                    // 没有待执行任务，等待一分钟
                    sleep(tokio::time::Duration::from_secs(60)).await;
                    continue;
                }

                // 获取到期的任务 (simplified for now)
                sleep(tokio::time::Duration::from_secs(60)).await;
            }
        });
    }

    /// 停止调度器
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
    }

    /// 获取到期的任务
    async fn get_due_tasks(&self) -> Vec<ScheduledTask> {
        let mut tasks = Vec::new();
        let mut plan_ids_to_disable: Vec<String> = Vec::new();

        let plans: Vec<BackupPlan> = (*self.plans.read().await).clone();
        for plan in plans.iter() {
            if !plan.enabled {
                continue;
            }

            if let Some(due_time) = Self::calculate_next_run(plan) {
                let now = Utc::now();
                if due_time <= now {
                    tasks.push(ScheduledTask {
                        id: plan.id.clone(),
                        plan_id: plan.id.clone(),
                        scheduled_time: due_time,
                        plan_type: plan.backup_type,
                    });

                    // 如果是一次性任务，执行后禁用
                    if matches!(plan.schedule, BackupSchedule::Once { .. }) {
                        plan_ids_to_disable.push(plan.id.clone());
                    }
                }
            }
        }

        // Disable once-execution plans
        for plan_id in plan_ids_to_disable {
            let mut plans: Vec<BackupPlan> = std::mem::take(&mut *self.plans.write().await);
            if let Some(p) = plans.iter_mut().find(|p| p.id == plan_id) {
                p.enabled = false;
            }
            *self.plans.write().await = plans;
        }

        tasks
    }

    /// 计算下次执行时间
    fn calculate_next_run(plan: &BackupPlan) -> Option<DateTime<Utc>> {
        match &plan.schedule {
            BackupSchedule::Once { at } => Some(*at),
            BackupSchedule::Cron(_) => {
                // TODO: 实现 Cron 表达式解析
                None
            },
            BackupSchedule::Interval { seconds } => {
                Some(Utc::now() + chrono::Duration::seconds(*seconds as i64))
            },
            BackupSchedule::Daily { hour, minute, second, .. } => {
                let now = Utc::now();
                let next = now.with_hour(*hour)?.with_minute(*minute)?.with_second(*second)?;
                // 如果时间已过，安排到明天
                Some(if next <= now { next + chrono::Duration::days(1) } else { next })
            },
            BackupSchedule::Weekly { day_of_week, hour, minute, second, .. } => {
                let now = Utc::now();
                let wday = now.weekday().num_days_from_monday();
                let next = now.with_hour(*hour)?.with_minute(*minute)?.with_second(*second)?;
                // 调整到目标星期
                let days_until = (*day_of_week as i32 - wday as i32).rem_euclid(7);
                Some(next + chrono::Duration::days(days_until as i64))
            },
            BackupSchedule::Monthly { day, hour, minute, second, .. } => {
                let now = Utc::now();
                let next = now.with_hour(*hour)?.with_minute(*minute)?.with_second(*second)?;
                // 调整到目标日期
                let next_month = if next.month() == 12 { 1 } else { next.month() + 1 };
                // 调整日期到目标
                let max_days = get_days_in_month(next.year(), next_month);
                let next = next
                    .with_month(next_month)?
                    .with_day(if *day > max_days { max_days } else { *day })?
                    .with_hour(*hour)?
                    .with_minute(*minute)?
                    .with_second(*second)?;
                Some(next)
            },
        }
    }

    /// 执行备份任务
    async fn execute_task(&self, task: &ScheduledTask) -> Result<()> {
        let mut item = BackupItem::new(
            task.plan_type,
            self.generate_backup_path(&task.plan_id)?,
            vec![], // TODO: 从计划获取设备列表
            format!("{:?} backup", task.plan_type),
            "Scheduler".to_string(),
        );

        // 发送开始通知
        self.send_notification(BackupNotification {
            item: None,
            plan_id: Some(task.plan_id.clone()),
            notification_type: BackupNotificationType::BackupStarted {
                plan_id: task.plan_id.clone(),
                version: item.metadata.version.clone(),
            },
            timestamp: Utc::now(),
        })
        .await;

        // 执行备份
        let _path = self.executor.backup(&mut item).await?;

        // 标记为创建成功
        item.mark_created();

        // 压缩
        self.executor.compress(&item).await?;

        // 验证
        item.mark_verifying();
        let verified = self.executor.verify(&item).await?;

        if verified {
            item.mark_verified(item.get_checksum()?.to_string());
        }

        // 发送完成通知
        self.send_notification(BackupNotification {
            item: None,
            plan_id: Some(task.plan_id.clone()),
            notification_type: BackupNotificationType::BackupCompleted {
                plan_id: task.plan_id.clone(),
                size: item.size,
            },
            timestamp: Utc::now(),
        })
        .await;

        Ok(())
    }

    /// 生成备份路径
    fn generate_backup_path(&self, plan_id: &str) -> Result<PathBuf> {
        let now = Utc::now().format("%Y%m%d/%H%M%S");
        Ok(format!("./backup/backup_{}_{}.zip", plan_id, now).into())
    }

    /// 发送通知
    async fn send_notification(&self, notification: BackupNotification) {
        let _ = self.notification_sender.send(notification);
    }

    /// 清理过期备份
    pub async fn cleanup_expired_backups(&self, _retention: &BackupRetention) -> Result<usize> {
        // TODO: 实现过期备份清理逻辑
        Ok(0)
    }

    /// 获取待恢复的备份列表
    pub async fn list_available_backups(&self, _target_id: &str) -> Vec<BackupItem> {
        // TODO: 从存储层恢复备份列表
        vec![]
    }

    /// 获取通知接收器
    pub fn subscribe(&self) -> broadcast::Receiver<BackupNotification> {
        self.notification_sender.subscribe()
    }
}

/// 调度任务
#[derive(Debug)]
struct ScheduledTask {
    id: String,
    plan_id: String,
    scheduled_time: DateTime<Utc>,
    plan_type: BackupType,
}

/// 获取指定年份和月份的天数
fn get_days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            // 闰年规则
            if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                29
            } else {
                28
            }
        },
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        backup::{BackupStatus, RetentionMode},
        error::Error,
    };

    struct MockExecutor {
        should_succeed: bool,
    }

    #[async_trait::async_trait]
    impl BackupExecutor for MockExecutor {
        async fn backup(&self, item: &mut BackupItem) -> std::result::Result<String, Error> {
            item.update_stats(1024, 10);
            item.mark_created();
            if !self.should_succeed {
                item.mark_failed("Test failure".to_string());
                return Err(Error::Backup("Mock error".to_string()));
            }
            Ok("backup-path".to_string())
        }

        async fn verify(&self, _item: &BackupItem) -> std::result::Result<bool, Error> {
            if !self.should_succeed {
                return Ok(false);
            }
            Ok(true)
        }

        async fn cleanup(&self, _item: &BackupItem) -> std::result::Result<(), Error> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_backup_scheduler_registration() {
        let executor = MockExecutor { should_succeed: true };
        let scheduler = BackupScheduler::new(executor);

        let plan = BackupPlan {
            id: "plan-001".to_string(),
            name: "Daily Backup".to_string(),
            backup_type: BackupType::Full,
            targets: vec![],
            schedule: BackupSchedule::Daily { hour: 2, minute: 0, second: 0 },
            retention: BackupRetention {
                max_count: 7,
                max_age_days: 30,
                max_size_bytes: 1024 * 1024 * 1024,
                mode: RetentionMode::KeepLatest,
            },
            enabled: true,
        };

        scheduler.register_plan(plan).await.unwrap();

        let plans = scheduler.list_plans().await;
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].id, "plan-001");
    }

    #[tokio::test]
    async fn test_schedule_calculation_daily() {
        let plan = BackupPlan {
            id: "plan-002".to_string(),
            name: "Daily".to_string(),
            backup_type: BackupType::Incremental,
            targets: vec![],
            schedule: BackupSchedule::Daily { hour: 3, minute: 30, second: 0 },
            retention: BackupRetention::default(),
            enabled: true,
        };

        let _scheduler = BackupScheduler::new(MockExecutor { should_succeed: true });

        let next = BackupScheduler::<MockExecutor>::calculate_next_run(&plan).unwrap();
        assert!(next > Utc::now());
    }

    #[tokio::test]
    async fn test_schedule_calculation_weekly() {
        let plan = BackupPlan {
            id: "plan-003".to_string(),
            name: "Weekly".to_string(),
            backup_type: BackupType::Full,
            targets: vec![],
            schedule: BackupSchedule::Weekly {
                day_of_week: 1, // Monday
                hour: 4,
                minute: 0,
                second: 0,
            },
            retention: BackupRetention::default(),
            enabled: true,
        };

        let _scheduler = BackupScheduler::new(MockExecutor { should_succeed: true });

        let next = BackupScheduler::<MockExecutor>::calculate_next_run(&plan).unwrap();
        assert!(next > Utc::now());
    }

    #[tokio::test]
    async fn test_schedule_calculation_monthly() {
        let plan = BackupPlan {
            id: "plan-004".to_string(),
            name: "Monthly".to_string(),
            backup_type: BackupType::Full,
            targets: vec![],
            schedule: BackupSchedule::Monthly { day: 1, hour: 2, minute: 0, second: 0 },
            retention: BackupRetention::default(),
            enabled: true,
        };

        let _scheduler = BackupScheduler::new(MockExecutor { should_succeed: true });

        let next = BackupScheduler::<MockExecutor>::calculate_next_run(&plan).unwrap();
        assert!(next > Utc::now());
    }

    #[test]
    fn test_backup_item_lifecycle() {
        let mut item = BackupItem::new(
            BackupType::Full,
            PathBuf::from("/backup/backup.zip"),
            vec![],
            "Test backup".to_string(),
            "Dev".to_string(),
        );

        assert_eq!(item.status, BackupStatus::Creating);

        item.mark_created();
        assert_eq!(item.status, BackupStatus::Created);

        item.mark_verifying();
        assert_eq!(item.status, BackupStatus::Verifying);

        item.mark_completed("abc123".to_string());
        assert_eq!(item.status, BackupStatus::Completed);
        assert_eq!(item.checksum, Some("abc123".to_string()));

        item.mark_failed("Error".to_string());
        assert_eq!(item.status, BackupStatus::Failed);
    }
}
