//! 沙箱核心实现

use crate::error::{Result, SandboxError};
use crate::{SandboxConfig, SandboxStatus};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 沙箱实例
pub struct Sandbox {
    /// 配置
    config: SandboxConfig,

    /// 状态
    status: Arc<RwLock<SandboxStatus>>,

    /// 执行结果
    result: Arc<RwLock<Option<String>>>,
}

impl Sandbox {
    /// 创建新沙箱
    pub fn new(config: SandboxConfig) -> Self {
        Self {
            config,
            status: Arc::new(RwLock::new(SandboxStatus::Idle)),
            result: Arc::new(RwLock::new(None)),
        }
    }

    /// 启动沙箱
    pub async fn start(&self) -> Result<()> {
        let mut status = self.status.write().await;
        *status = SandboxStatus::Running;
        Ok(())
    }

    /// 执行命令
    pub async fn execute(&self, command: &str) -> Result<String> {
        // TODO: 实现真实的沙箱执行
        // 当前返回模拟结果
        let output = format!("沙箱执行成功: {}", command);

        let mut result = self.result.write().await;
        *result = Some(output.clone());

        Ok(output)
    }

    /// 停止沙箱
    pub async fn stop(&self) -> Result<()> {
        let mut status = self.status.write().await;
        *status = SandboxStatus::Success;
        Ok(())
    }

    /// 销毁沙箱
    pub async fn destroy(&self) -> Result<()> {
        let mut status = self.status.write().await;
        *status = SandboxStatus::Destroyed;
        Ok(())
    }

    /// 获取状态
    pub async fn status(&self) -> SandboxStatus {
        self.status.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sandbox_lifecycle() {
        let sandbox = Sandbox::new(SandboxConfig::default());

        sandbox.start().await.unwrap();
        assert_eq!(sandbox.status().await, SandboxStatus::Running);

        let result = sandbox.execute("test command").await.unwrap();
        assert!(result.contains("成功"));

        sandbox.stop().await.unwrap();
        assert_eq!(sandbox.status().await, SandboxStatus::Success);
    }
}
