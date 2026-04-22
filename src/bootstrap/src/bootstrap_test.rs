//! Bootstrap模块单元测试

use fos_bootstrap::{
    Bootstrap, BootstrapConfig, BootstrapPhase, DependencyChecker, ModuleInitializer,
    StartupCoordinator,
};
use std::collections::HashMap;

/// 测试：启动阶段顺序正确
#[test]
fn test_bootstrap_phases_order() {
    let phases = Bootstrap::get_phases();

    assert_eq!(phases.len(), 5);
    assert_eq!(phases[0], BootstrapPhase::Config);
    assert_eq!(phases[1], BootstrapPhase::Storage);
    assert_eq!(phases[2], BootstrapPhase::Core);
    assert_eq!(phases[3], BootstrapPhase::Extension);
    assert_eq!(phases[4], BootstrapPhase::Service);
}

/// 测试：默认配置创建
#[test]
fn test_default_config() {
    let config = BootstrapConfig::default();

    assert!(config.enable_health_check);
    assert!(config.enable_dependency_check);
    assert!(config.enable_module_init);
    assert!(config.timeout_secs > 0);
}

/// 测试：依赖检查器创建
#[test]
fn test_dependency_checker_creation() {
    let checker = DependencyChecker::new();

    assert!(checker.checks().len() > 0);
}

/// 测试：模块初始化器创建
#[test]
fn test_module_initializer_creation() {
    let initializer = ModuleInitializer::new();

    assert!(initializer.pending_modules().is_empty());
}

/// 测试：启动协调器创建
#[test]
fn test_startup_coordinator_creation() {
    let coordinator = StartupCoordinator::new();

    assert_eq!(coordinator.current_phase(), None);
    assert!(!coordinator.is_started());
}

/// 测试：启动流程状态转换
#[test]
fn test_bootstrap_state_transitions() {
    let mut bootstrap = Bootstrap::new(BootstrapConfig::default());

    // 初始状态
    assert!(!bootstrap.is_initialized());

    // 执行初始化
    let result = bootstrap.initialize();
    assert!(result.is_ok());
    assert!(bootstrap.is_initialized());
}

/// 测试：启动超时处理
#[test]
fn test_bootstrap_timeout() {
    let mut config = BootstrapConfig::default();
    config.timeout_secs = 1;

    let bootstrap = Bootstrap::new(config);

    // 超时应该返回错误
    // 实际测试中会模拟慢速初始化
}

/// 测试：依赖检查成功
#[test]
fn test_dependency_check_success() {
    let checker = DependencyChecker::new();
    let result = checker.check_all();

    // 在测试环境中，基础依赖应该存在
    assert!(result.is_ok());
}

/// 测试：模块注册
#[test]
fn test_module_registration() {
    let mut initializer = ModuleInitializer::new();

    initializer.register("gateway", 1);
    initializer.register("validator", 2);
    initializer.register("bus", 3);

    assert_eq!(initializer.pending_modules().len(), 3);
}

/// 测试：模块初始化顺序
#[test]
fn test_module_init_order() {
    let mut initializer = ModuleInitializer::new();

    initializer.register("bus", 3);
    initializer.register("gateway", 1);
    initializer.register("validator", 2);

    let order = initializer.get_init_order();

    assert_eq!(order, vec!["gateway", "validator", "bus"]);
}

/// 测试：启动报告生成
#[test]
fn test_bootstrap_report() {
    let bootstrap = Bootstrap::new(BootstrapConfig::default());
    let report = bootstrap.generate_report();

    assert!(report.contains("Bootstrap Report"));
    assert!(report.contains("Phase"));
}
