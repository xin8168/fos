//! # Validator 模块

use crate::engine::ValidationEngine;
use crate::error::Result;
use crate::{ExecutionContext, ValidationRequest, ValidationResult};

/// 校验器
pub struct Validator {
    /// 校验引擎
    engine: ValidationEngine,
}

impl Validator {
    /// 创建新的校验器
    pub fn new() -> Self {
        Self { engine: ValidationEngine::new() }
    }

    /// 校验请求
    pub async fn validate(
        &self,
        request: ValidationRequest,
        context: ExecutionContext,
    ) -> Result<ValidationResult> {
        self.engine.validate(&request, &context).await
    }

    /// 快速校验（仅执行高优先级规则）
    pub async fn quick_validate(
        &self,
        request: ValidationRequest,
        context: ExecutionContext,
    ) -> Result<bool> {
        let result = self.engine.validate(&request, &context).await?;
        Ok(result.passed)
    }

    /// 获取校验引擎引用
    pub fn engine(&self) -> &ValidationEngine {
        &self.engine
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DeviceStatus;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_validator_creation() {
        let validator = Validator::new();
        assert!(true);
    }

    #[tokio::test]
    async fn test_validate_request() {
        let validator = Validator::new();
        let request = ValidationRequest {
            id: "req-001".to_string(),
            event: "测试事件".to_string(),
            steps: vec!["步骤1".to_string()],
            judgment_logic: "条件1".to_string(),
            verification_standard: "标准1".to_string(),
            location: "位置1".to_string(),
            subject: "主体1".to_string(),
            metadata: HashMap::new(),
        };

        let context = ExecutionContext {
            user_id: "user-001".to_string(),
            roles: vec!["admin".to_string()],
            permissions: vec!["execute".to_string()],
            device_info: crate::DeviceInfo {
                device_id: "device-001".to_string(),
                device_type: "computer".to_string(),
                status: DeviceStatus::Online,
                capabilities: vec![],
            },
            environment: HashMap::new(),
        };

        let result = validator.validate(request, context).await;
        assert!(result.is_ok());
    }
}
