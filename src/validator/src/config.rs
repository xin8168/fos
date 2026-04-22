//! Validator 配置模块

use serde::{Deserialize, Serialize};

/// Validator 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    /// 是否启用严格模式
    pub strict_mode: bool,

    /// 最大递归深度
    pub max_recursion_depth: usize,

    /// 校验超时（毫秒）
    pub validation_timeout_ms: u64,

    /// 是否启用缓存
    pub enable_cache: bool,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            strict_mode: true,
            max_recursion_depth: 10,
            validation_timeout_ms: 5000,
            enable_cache: true,
        }
    }
}

impl ValidatorConfig {
    /// 创建新配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 验证配置
    pub fn validate(&self) -> Result<(), String> {
        if self.max_recursion_depth == 0 {
            return Err("max_recursion_depth must be greater than 0".to_string());
        }
        if self.validation_timeout_ms == 0 {
            return Err("validation_timeout_ms must be greater than 0".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ValidatorConfig::default();
        assert!(config.strict_mode);
        assert_eq!(config.max_recursion_depth, 10);
    }

    #[test]
    fn test_config_validation() {
        let config = ValidatorConfig::default();
        assert!(config.validate().is_ok());
    }
}
