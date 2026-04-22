//! 关闭协调器

use crate::cleaner::ResourceCleaner;
use crate::config::ShutdownConfig;
use crate::error::Result;
use crate::signal::SignalHandler;
use crate::waiter::TaskWaiter;
use std::time::Instant;

/// 关闭阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownPhase {
    /// 停止接受连接
    StopConnections,
    /// 等待任务完成
    WaitForTasks,
    /// 清理资源
    Cleanup,
    /// 完成
    Done,
}

/// 关闭统计信息
#[derive(Debug, Clone)]
pub struct ShutdownStats {
    /// 开始时间
    pub start_time: Instant,
    /// 当前阶段
    pub current_phase: ShutdownPhase,
    /// 已完成阶段数
    pub completed_phases: u8,
}

impl ShutdownStats {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            current_phase: ShutdownPhase::StopConnections,
            completed_phases: 0,
        }
    }

    /// 获取运行时间（毫秒）
    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }
}

impl Default for ShutdownStats {
    fn default() -> Self {
        Self::new()
    }
}

/// 关闭协调器
pub struct ShutdownCoordinator {
    /// 配置
    config: ShutdownConfig,
    /// 信号处理器
    signal_handler: SignalHandler,
    /// 任务等待器
    task_waiter: TaskWaiter,
    /// 资源清理器
    resource_cleaner: ResourceCleaner,
    /// 统计信息
    stats: std::sync::Mutex<Option<ShutdownStats>>,
}

impl ShutdownCoordinator {
    /// 创建新的关闭协调器
    pub fn new() -> Self {
        Self {
            config: ShutdownConfig::default(),
            signal_handler: SignalHandler::new(),
            task_waiter: TaskWaiter::new(),
            resource_cleaner: ResourceCleaner::new(),
            stats: std::sync::Mutex::new(None),
        }
    }

    /// 使用配置创建
    pub fn with_config(config: ShutdownConfig) -> Self {
        Self {
            config,
            signal_handler: SignalHandler::new(),
            task_waiter: TaskWaiter::new(),
            resource_cleaner: ResourceCleaner::new(),
            stats: std::sync::Mutex::new(None),
        }
    }

    /// 获取信号处理器引用
    pub fn signal_handler(&self) -> &SignalHandler {
        &self.signal_handler
    }

    /// 获取任务等待器引用
    pub fn task_waiter(&self) -> &TaskWaiter {
        &self.task_waiter
    }

    /// 获取资源清理器引用
    pub fn resource_cleaner(&self) -> &ResourceCleaner {
        &self.resource_cleaner
    }

    /// 执行关闭流程
    pub fn shutdown(&self) -> Result<()> {
        let mut stats = self.stats.lock().unwrap();
        *stats = Some(ShutdownStats::new());
        drop(stats);

        tracing::info!("开始优雅关闭流程");

        // 阶段1: 停止接受连接
        self.stop_connections()?;

        // 阶段2: 等待任务完成
        self.wait_tasks()?;

        // 阶段3: 清理资源
        self.cleanup_resources()?;

        tracing::info!("优雅关闭完成");
        Ok(())
    }

    fn stop_connections(&self) -> Result<()> {
        tracing::debug!("阶段: 停止接受新连接");
        self.update_phase(ShutdownPhase::StopConnections);
        Ok(())
    }

    fn wait_tasks(&self) -> Result<()> {
        tracing::debug!("阶段: 等待任务完成");
        self.update_phase(ShutdownPhase::WaitForTasks);

        if self.config.wait_for_tasks {
            let timeout = std::time::Duration::from_secs(self.config.task_wait_timeout_secs);
            if let Ok(rt) = tokio::runtime::Handle::try_current() {
                rt.block_on(self.task_waiter.wait_all(timeout))?;
            }
        }

        Ok(())
    }

    fn cleanup_resources(&self) -> Result<()> {
        tracing::debug!("阶段: 清理资源");
        self.update_phase(ShutdownPhase::Cleanup);
        self.resource_cleaner.cleanup_all()
    }

    fn update_phase(&self, phase: ShutdownPhase) {
        if let Some(stats) = self.stats.lock().unwrap().as_mut() {
            stats.current_phase = phase;
            stats.completed_phases += 1;
        }
    }

    /// 生成关闭报告
    pub fn report(&self) -> String {
        let stats = self.stats.lock().unwrap();

        format!(
            "=== 关闭报告 ===\n\
             信号: {:?}\n\
             运行任务数: {}\n\
             已清理: {}\n\
             运行时间: {}ms\n",
            self.signal_handler.get_signal(),
            self.task_waiter.running_count(),
            self.resource_cleaner.is_cleaned(),
            stats.as_ref().map(|s| s.elapsed_ms()).unwrap_or(0)
        )
    }
}

impl Default for ShutdownCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinator_creation() {
        let coordinator = ShutdownCoordinator::new();
        assert!(!coordinator.signal_handler().is_shutdown_requested());
    }

    #[test]
    fn test_shutdown() {
        let coordinator = ShutdownCoordinator::new();
        let result = coordinator.shutdown();
        assert!(result.is_ok());
    }

    #[test]
    fn test_report() {
        let coordinator = ShutdownCoordinator::new();
        let report = coordinator.report();
        assert!(report.contains("关闭报告"));
    }
}
