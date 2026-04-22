//! 沙箱执行器

use crate::error::Result;
use crate::Sandbox;

/// 沙箱执行器
pub struct SandboxExecutor {
    sandbox: Sandbox,
}

impl SandboxExecutor {
    pub fn new(sandbox: Sandbox) -> Self {
        Self { sandbox }
    }

    pub async fn execute(&self, steps: Vec<String>) -> Result<Vec<String>> {
        let mut results = Vec::new();
        for step in steps {
            let result = self.sandbox.execute(&step).await?;
            results.push(result);
        }
        Ok(results)
    }
}
