//! 启动协调器

use crate::checker::DependencyChecker;
use crate::error::Result;
use crate::initializer::ModuleInitializer;
use crate::phases::BootstrapPhase;
use std::time::Instant;

/// 启动状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupState {
    /// 未启动
    NotStarted,
    /// 启动中
    Starting,
    /// 运行中
    Running,
    /// 启动失败
    Failed,
    /// 已停止
    Stopped,
}

/// 启动统计信息
#[derive(Debug, Clone)]
pub struct StartupStats {
    /// 启动开始时间
    pub start_time: Instant,
    /// 当前阶段
    pub current_phase: Option<BootstrapPhase>,
    /// 已完成阶段数
    pub completed_phases: u8,
    /// 总阶段数
    pub total_phases: u8,
}

impl StartupStats {
    /// 创建新的统计信息
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            current_phase: None,
            completed_phases: 0,
            total_phases: 5,
        }
    }

    /// 获取启动进度百分比
    pub fn progress_percent(&self) -> f64 {
        if self.total_phases == 0 {
            return 100.0;
        }
        (self.completed_phases as f64 / self.total_phases as f64) * 100.0
    }

    /// 获取已运行时间（毫秒）
    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }
}

impl Default for StartupStats {
    fn default() -> Self {
        Self::new()
    }
}

/// 启动协调器
pub struct StartupCoordinator {
    /// 当前状态
    state: StartupState,
    /// 当前阶段
    current_phase: Option<BootstrapPhase>,
    /// 统计信息
    stats: StartupStats,
    /// 依赖检查器
    dependency_checker: DependencyChecker,
    /// 模块初始化器
    #[allow(dead_code)]
    module_initializer: ModuleInitializer,
}

impl StartupCoordinator {
    /// 创建新的启动协调器
    pub fn new() -> Self {
        Self {
            state: StartupState::NotStarted,
            current_phase: None,
            stats: StartupStats::new(),
            dependency_checker: DependencyChecker::new(),
            module_initializer: ModuleInitializer::new(),
        }
    }

    /// 获取当前状态
    pub fn state(&self) -> StartupState {
        self.state
    }

    /// 获取当前阶段
    pub fn current_phase(&self) -> Option<BootstrapPhase> {
        self.current_phase
    }

    /// 检查是否已启动
    pub fn is_started(&self) -> bool {
        matches!(self.state, StartupState::Running)
    }

    /// 获取统计信息
    pub fn stats(&self) -> &StartupStats {
        &self.stats
    }

    /// 执行启动流程
    pub fn startup(&mut self) -> Result<()> {
        if self.state != StartupState::NotStarted {
            return Ok(());
        }

        self.state = StartupState::Starting;
        self.stats = StartupStats::new();

        // 1. 依赖检查
        self.check_dependencies()?;

        // 2. 执行各阶段
        let phases = BootstrapPhase::all();
        for phase in phases {
            self.execute_phase(phase)?;
        }

        self.state = StartupState::Running;
        Ok(())
    }

    /// 检查依赖
    fn check_dependencies(&mut self) -> Result<()> {
        tracing::info!("执行依赖检查...");
        self.dependency_checker.check_all()?;
        tracing::info!("依赖检查通过");
        Ok(())
    }

    /// 执行单个阶段
    fn execute_phase(&mut self, phase: BootstrapPhase) -> Result<()> {
        tracing::info!("执行启动阶段: {}", phase);

        self.current_phase = Some(phase);

        // 模拟阶段执行
        match phase {
            BootstrapPhase::Config => self.phase_config()?,
            BootstrapPhase::Storage => self.phase_storage()?,
            BootstrapPhase::Core => self.phase_core()?,
            BootstrapPhase::Extension => self.phase_extension()?,
            BootstrapPhase::Service => self.phase_service()?,
        }

        self.stats.completed_phases += 1;
        tracing::info!("阶段完成: {} (进度: {:.0}%)", phase, self.stats.progress_percent());

        Ok(())
    }

    fn phase_config(&self) -> Result<()> {
        tracing::debug!("配置阶段执行中...");
        Ok(())
    }

    fn phase_storage(&self) -> Result<()> {
        tracing::debug!("存储阶段执行中...");
        Ok(())
    }

    fn phase_core(&self) -> Result<()> {
        tracing::debug!("核心模块阶段执行中...");
        Ok(())
    }

    fn phase_extension(&self) -> Result<()> {
        tracing::debug!("扩展模块阶段执行中...");
        Ok(())
    }

    fn phase_service(&self) -> Result<()> {
        tracing::debug!("服务阶段执行中...");
        Ok(())
    }

    /// 停止服务
    pub fn shutdown(&mut self) -> Result<()> {
        if self.state != StartupState::Running {
            return Ok(());
        }

        tracing::info!("开始停止服务...");

        // 执行停止逻辑

        self.state = StartupState::Stopped;
        tracing::info!("服务已停止");

        Ok(())
    }

    /// 生成启动报告
    pub fn report(&self) -> String {
        format!(
            "=== 启动报告 ===\n\
             状态: {:?}\n\
             当前阶段: {:?}\n\
             已完成阶段: {}/{}\n\
             进度: {:.0}%\n\
             运行时间: {}ms\n",
            self.state,
            self.current_phase,
            self.stats.completed_phases,
            self.stats.total_phases,
            self.stats.progress_percent(),
            self.stats.elapsed_ms()
        )
    }
}

impl Default for StartupCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinator_creation() {
        let coordinator = StartupCoordinator::new();
        assert_eq!(coordinator.state(), StartupState::NotStarted);
        assert!(!coordinator.is_started());
    }

    #[test]
    fn test_coordinator_startup() {
        let mut coordinator = StartupCoordinator::new();
        let result = coordinator.startup();

        assert!(result.is_ok());
        assert!(coordinator.is_started());
        assert_eq!(coordinator.state(), StartupState::Running);
    }

    #[test]
    fn test_startup_stats() {
        let stats = StartupStats::new();
        assert_eq!(stats.completed_phases, 0);
        assert_eq!(stats.total_phases, 5);
    }

    #[test]
    fn test_progress_percent() {
        let mut stats = StartupStats::new();
        stats.completed_phases = 2;

        assert_eq!(stats.progress_percent(), 40.0);
    }

    #[test]
    fn test_report() {
        let coordinator = StartupCoordinator::new();
        let report = coordinator.report();

        assert!(report.contains("启动报告"));
        assert!(report.contains("NotStarted"));
    }
}
