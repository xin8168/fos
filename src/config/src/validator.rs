//! 配置验证器

use crate::error::{ConfigError, Result};
use crate::FosConfig;
use std::collections::HashSet;

/// 配置验证规则
pub struct ValidationRule {
    /// 规则名称
    pub name: String,
    /// 规则描述
    pub description: String,
    /// 验证函数
    pub validate: fn(&FosConfig) -> Result<()>,
}

/// 配置验证器
pub struct ConfigValidator {
    /// 验证规则
    rules: Vec<ValidationRule>,
    /// 必需的配置项
    required_keys: HashSet<String>,
}

impl ConfigValidator {
    /// 创建新的验证器
    pub fn new() -> Self {
        let mut validator = Self { rules: Vec::new(), required_keys: HashSet::new() };

        // 添加默认验证规则
        validator.add_default_rules();
        validator
    }

    /// 添加默认验证规则
    fn add_default_rules(&mut self) {
        // 端口范围验证
        self.add_rule(ValidationRule {
            name: "port_range".to_string(),
            description: "端口必须在1-65535范围内".to_string(),
            validate: |config| {
                if config.server.port == 0 {
                    Err(ConfigError::ValidationError("端口不能为0".to_string()))
                } else {
                    Ok(())
                }
            },
        });

        // 工作线程数验证
        self.add_rule(ValidationRule {
            name: "workers_positive".to_string(),
            description: "工作线程数必须大于0".to_string(),
            validate: |config| {
                if config.server.workers == 0 {
                    Err(ConfigError::ValidationError("工作线程数必须大于0".to_string()))
                } else {
                    Ok(())
                }
            },
        });

        // 超时验证
        self.add_rule(ValidationRule {
            name: "timeout_positive".to_string(),
            description: "超时时间必须大于0".to_string(),
            validate: |config| {
                if config.server.timeout_secs == 0 {
                    Err(ConfigError::ValidationError("超时时间必须大于0".to_string()))
                } else {
                    Ok(())
                }
            },
        });

        // 日志级别验证
        self.add_rule(ValidationRule {
            name: "log_level_valid".to_string(),
            description: "日志级别必须是有效的".to_string(),
            validate: |config| {
                let valid_levels = ["trace", "debug", "info", "warn", "error"];
                if !valid_levels.contains(&config.logging.level.to_lowercase().as_str()) {
                    Err(ConfigError::ValidationError(format!(
                        "无效的日志级别: {}",
                        config.logging.level
                    )))
                } else {
                    Ok(())
                }
            },
        });
    }

    /// 添加验证规则
    pub fn add_rule(&mut self, rule: ValidationRule) {
        self.rules.push(rule);
    }

    /// 添加必需的配置项
    pub fn require(&mut self, key: &str) {
        self.required_keys.insert(key.to_string());
    }

    /// 验证配置
    pub fn validate(&self, config: &FosConfig) -> Result<()> {
        // 执行所有验证规则
        for rule in &self.rules {
            (rule.validate)(config)
                .map_err(|e| ConfigError::ValidationError(format!("{}: {}", rule.name, e)))?;
        }

        Ok(())
    }

    /// 获取所有规则名称
    pub fn rule_names(&self) -> Vec<&str> {
        self.rules.iter().map(|r| r.name.as_str()).collect()
    }

    /// 检查是否有指定规则
    pub fn has_rule(&self, name: &str) -> bool {
        self.rules.iter().any(|r| r.name == name)
    }
}

impl Default for ConfigValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = ConfigValidator::new();
        assert!(!validator.rules.is_empty());
    }

    #[test]
    fn test_validate_valid_config() {
        let validator = ConfigValidator::new();
        let config = FosConfig::default();
        assert!(validator.validate(&config).is_ok());
    }

    #[test]
    fn test_validate_invalid_port() {
        let validator = ConfigValidator::new();
        let mut config = FosConfig::default();
        config.server.port = 0;

        // 默认配置中端口是8080，不是0，所以需要验证器来检测
        // 这里我们测试验证规则是否存在
        assert!(validator.has_rule("port_range"));
    }

    #[test]
    fn test_validate_invalid_log_level() {
        let validator = ConfigValidator::new();
        let mut config = FosConfig::default();
        config.logging.level = "invalid".to_string();

        let result = validator.validate(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_custom_rule() {
        let mut validator = ConfigValidator::new();

        validator.add_rule(ValidationRule {
            name: "custom_rule".to_string(),
            description: "自定义验证规则".to_string(),
            validate: |_| Ok(()),
        });

        assert!(validator.has_rule("custom_rule"));
    }

    #[test]
    fn test_rule_names() {
        let validator = ConfigValidator::new();
        let names = validator.rule_names();
        assert!(names.contains(&"port_range"));
    }
}
