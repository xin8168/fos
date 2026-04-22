//! 启动阶段定义

use serde::{Deserialize, Serialize};

/// 启动阶段枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BootstrapPhase {
    /// 配置加载阶段
    Config,
    /// 存储初始化阶段
    Storage,
    /// 核心模块启动阶段
    Core,
    /// 扩展模块启动阶段
    Extension,
    /// 服务暴露阶段
    Service,
}

impl BootstrapPhase {
    /// 获取阶段名称
    pub fn name(&self) -> &'static str {
        match self {
            BootstrapPhase::Config => "配置加载",
            BootstrapPhase::Storage => "存储初始化",
            BootstrapPhase::Core => "核心模块启动",
            BootstrapPhase::Extension => "扩展模块启动",
            BootstrapPhase::Service => "服务暴露",
        }
    }

    /// 获取阶段顺序号
    pub fn order(&self) -> u8 {
        match self {
            BootstrapPhase::Config => 1,
            BootstrapPhase::Storage => 2,
            BootstrapPhase::Core => 3,
            BootstrapPhase::Extension => 4,
            BootstrapPhase::Service => 5,
        }
    }

    /// 获取所有阶段
    pub fn all() -> Vec<BootstrapPhase> {
        vec![
            BootstrapPhase::Config,
            BootstrapPhase::Storage,
            BootstrapPhase::Core,
            BootstrapPhase::Extension,
            BootstrapPhase::Service,
        ]
    }
}

impl std::fmt::Display for BootstrapPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_name() {
        assert_eq!(BootstrapPhase::Config.name(), "配置加载");
        assert_eq!(BootstrapPhase::Storage.name(), "存储初始化");
    }

    #[test]
    fn test_phase_order() {
        assert_eq!(BootstrapPhase::Config.order(), 1);
        assert_eq!(BootstrapPhase::Service.order(), 5);
    }

    #[test]
    fn test_phase_all() {
        let phases = BootstrapPhase::all();
        assert_eq!(phases.len(), 5);
    }
}
