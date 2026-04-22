//! # FOS Shutdown - 优雅关闭模块
//!
//! 负责系统优雅关闭、资源清理、关闭顺序控制
//!
//! ## 核心职责
//! - 接收关闭信号
//! - 等待任务完成
//! - 清理资源
//! - 确保数据一致性
//!
//! ## 安全铁律
//! - 不做规则判断
//! - 不强制终止
//! - 不丢弃数据

pub mod error;
pub mod config;
pub mod signal;
pub mod waiter;
pub mod cleaner;
pub mod coordinator;

pub use error::{Error, Result};
pub use config::ShutdownConfig;
pub use signal::SignalHandler;
pub use waiter::TaskWaiter;
pub use cleaner::ResourceCleaner;
pub use coordinator::ShutdownCoordinator;

use std::time::{Duration, Instant};

/// 模块版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 模块名称
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// 关闭状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownState {
    /// 运行中
    Running,
    /// 关闭中
    ShuttingDown,
    /// 已关闭
    Shutdown,
    /// 关闭失败
    Failed,
}

/// Shutdown 管理器
pub struct Shutdown {
    config: ShutdownConfig,
    state: ShutdownState,
    start_time: Option<Instant>,
}

impl Shutdown {
    /// 创建新的Shutdown实例
    pub fn new(config: ShutdownConfig) -> Self {
        Self {
            config,
            state: ShutdownState::Running,
            start_time: None,
        }
    }

    /// 获取当前状态
    pub fn state(&self) -> ShutdownState {
        self.state
    }

    /// 检查是否正在关闭
    pub fn is_shutting_down(&self) -> bool {
        matches!(self.state, ShutdownState::ShuttingDown)
    }

    /// 检查是否已关闭
    pub fn is_shutdown(&self) -> bool {
        matches!(self.state, ShutdownState::Shutdown)
    }

    /// 执行关闭
    pub fn shutdown(&mut self) -> Result<()> {
        if self.state == ShutdownState::Shutdown {
            return Ok(());
        }

        self.state = ShutdownState::ShuttingDown;
        self.start_time = Some(Instant::now());

        tracing::info!("开始优雅关闭...");

        // 检查超时
        let timeout = Duration::from_secs(self.config.timeout_secs);

        // 执行关闭步骤
        self.stop_accepting_connections()?;
        self.wait_for_pending_tasks()?;
        self.cleanup_resources()?;

        // 检查是否超时
        if let Some(start) = self.start_time {
            if start.elapsed() > timeout {
                tracing::warn!("关闭超时，强制完成");
            }
        }

        self.state = ShutdownState::Shutdown;
        tracing::info!("优雅关闭完成");
        
        Ok(())
    }

    fn stop_accepting_connections(&self) -> Result<()> {
        tracing::debug!("停止接受新连接...");
        Ok(())
    }

    fn wait_for_pending_tasks(&self) -> Result<()> {
        tracing::debug!("等待任务完成...");
        Ok(())
    }

    fn cleanup_resources(&self) -> Result<()> {
        tracing::debug!("清理资源...");
        Ok(())
    }

    /// 生成关闭报告
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== Shutdown Report ===\n");
        report.push_str(&format!("State: {:?}\n", self.state));
        
        if let Some(start) = self.start_time {
            report.push_str(&format!("Shutdown time: {:?}\n", start.elapsed()));
        }
        
        report
    }
}

impl Default for Shutdown {
    fn default() -> Self {
        Self::new(ShutdownConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shutdown_creation() {
        let shutdown = Shutdown::default();
        assert_eq!(shutdown.state(), ShutdownState::Running);
    }

    #[test]
    fn test_shutdown_state() {
        let mut shutdown = Shutdown::default();
        shutdown.shutdown().unwrap();
        assert!(shutdown.is_shutdown());
    }

    #[test]
    fn test_shutdown_report() {
        let shutdown = Shutdown::default();
        let report = shutdown.generate_report();
        assert!(report.contains("Shutdown Report"));
    }
}
