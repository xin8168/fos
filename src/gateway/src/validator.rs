//! FOS 协议校验规则
//!
//! 实现严格的 6维锚定 + 4要素执行校验

use crate::error::{GatewayError, Result};
use crate::{FourElement, SixAnchor};
use regex::Regex;

/// 校验器配置
#[derive(Debug)]
pub struct ValidatorConfig {
    /// 事件名称最大长度
    pub max_event_length: usize,
    /// 步骤最大数量
    pub max_steps_count: usize,
    /// 步骤最大长度
    pub max_step_length: usize,
    /// 判断逻辑最大长度
    pub max_judgment_length: usize,
    /// 校验标准最大长度
    pub max_verification_length: usize,
    /// 地点最大长度
    pub max_location_length: usize,
    /// 主体最大长度
    pub max_subject_length: usize,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            max_event_length: 200,
            max_steps_count: 20,
            max_step_length: 500,
            max_judgment_length: 500,
            max_verification_length: 500,
            max_location_length: 200,
            max_subject_length: 100,
        }
    }
}

/// FOS 协议校验器
pub struct FosValidator {
    config: ValidatorConfig,
    /// 中文和字母数字正则
    name_regex: Regex,
    /// 危险字符正则
    dangerous_regex: Regex,
}

impl std::fmt::Debug for FosValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FosValidator").field("config", &self.config).finish_non_exhaustive()
    }
}

impl FosValidator {
    /// 创建新校验器
    pub fn new() -> Self {
        Self {
            config: ValidatorConfig::default(),
            name_regex: Regex::new(r"^[\u4e00-\u9fa5a-zA-Z0-9_\-\s]+$").unwrap(),
            dangerous_regex: Regex::new(r#"[<>&"'\\;|`$]"#).unwrap(),
        }
    }

    /// 使用自定义配置创建校验器
    pub fn with_config(config: ValidatorConfig) -> Self {
        Self {
            config,
            name_regex: Regex::new(r"^[\u4e00-\u9fa5a-zA-Z0-9_\-\s]+$").unwrap(),
            dangerous_regex: Regex::new(r#"[<>&"'\\;|`$]"#).unwrap(),
        }
    }

    /// 校验 6维锚定
    pub fn validate_six_anchor(&self, anchor: &SixAnchor) -> Result<()> {
        self.validate_event(&anchor.event)?;
        self.validate_steps(&anchor.steps)?;
        self.validate_judgment_logic(&anchor.judgment_logic)?;
        self.validate_verification_standard(&anchor.verification_standard)?;
        self.validate_location(&anchor.location)?;
        self.validate_subject(&anchor.subject)?;
        Ok(())
    }

    /// 校验事件名称
    pub fn validate_event(&self, event: &str) -> Result<()> {
        if event.is_empty() {
            return Err(GatewayError::MissingField("事件(event)".to_string()));
        }
        if event.len() > self.config.max_event_length {
            return Err(GatewayError::InvalidField {
                field: "事件".to_string(),
                value: format!("长度超过{}字符", self.config.max_event_length),
            });
        }
        if !self.name_regex.is_match(event) {
            return Err(GatewayError::InvalidField {
                field: "事件".to_string(),
                value: "包含非法字符".to_string(),
            });
        }
        if self.dangerous_regex.is_match(event) {
            return Err(GatewayError::InvalidField {
                field: "事件".to_string(),
                value: "包含危险字符".to_string(),
            });
        }
        Ok(())
    }

    /// 校验步骤
    pub fn validate_steps(&self, steps: &[String]) -> Result<()> {
        if steps.is_empty() {
            return Err(GatewayError::MissingField("步骤(steps)".to_string()));
        }
        if steps.len() > self.config.max_steps_count {
            return Err(GatewayError::InvalidField {
                field: "步骤".to_string(),
                value: format!("步骤数量超过{}个", self.config.max_steps_count),
            });
        }
        for (i, step) in steps.iter().enumerate() {
            if step.is_empty() {
                return Err(GatewayError::InvalidField {
                    field: format!("步骤[{}]", i),
                    value: "空步骤".to_string(),
                });
            }
            if step.len() > self.config.max_step_length {
                return Err(GatewayError::InvalidField {
                    field: format!("步骤[{}]", i),
                    value: format!("长度超过{}字符", self.config.max_step_length),
                });
            }
            if self.dangerous_regex.is_match(step) {
                return Err(GatewayError::InvalidField {
                    field: format!("步骤[{}]", i),
                    value: "包含危险字符".to_string(),
                });
            }
        }
        Ok(())
    }

    /// 校验判断逻辑
    pub fn validate_judgment_logic(&self, logic: &str) -> Result<()> {
        if logic.is_empty() {
            return Err(GatewayError::MissingField("判断逻辑(judgment_logic)".to_string()));
        }
        if logic.len() > self.config.max_judgment_length {
            return Err(GatewayError::InvalidField {
                field: "判断逻辑".to_string(),
                value: format!("长度超过{}字符", self.config.max_judgment_length),
            });
        }
        Ok(())
    }

    /// 校验校验标准
    pub fn validate_verification_standard(&self, standard: &str) -> Result<()> {
        if standard.is_empty() {
            return Err(GatewayError::MissingField("校验标准(verification_standard)".to_string()));
        }
        if standard.len() > self.config.max_verification_length {
            return Err(GatewayError::InvalidField {
                field: "校验标准".to_string(),
                value: format!("长度超过{}字符", self.config.max_verification_length),
            });
        }
        Ok(())
    }

    /// 校验地点
    pub fn validate_location(&self, location: &str) -> Result<()> {
        if location.is_empty() {
            return Err(GatewayError::MissingField("地点(location)".to_string()));
        }
        if location.len() > self.config.max_location_length {
            return Err(GatewayError::InvalidField {
                field: "地点".to_string(),
                value: format!("长度超过{}字符", self.config.max_location_length),
            });
        }
        Ok(())
    }

    /// 校验主体
    pub fn validate_subject(&self, subject: &str) -> Result<()> {
        if subject.is_empty() {
            return Err(GatewayError::MissingField("主体(subject)".to_string()));
        }
        if subject.len() > self.config.max_subject_length {
            return Err(GatewayError::InvalidField {
                field: "主体".to_string(),
                value: format!("长度超过{}字符", self.config.max_subject_length),
            });
        }
        Ok(())
    }

    /// 校验 4要素
    pub fn validate_four_element(&self, element: &FourElement) -> Result<()> {
        if element.action.is_empty() {
            return Err(GatewayError::MissingField("4要素-动作(action)".to_string()));
        }
        if element.target.is_empty() {
            return Err(GatewayError::MissingField("4要素-对象(target)".to_string()));
        }
        if self.dangerous_regex.is_match(&element.action)
            || self.dangerous_regex.is_match(&element.target)
        {
            return Err(GatewayError::InvalidField {
                field: "4要素".to_string(),
                value: "包含危险字符".to_string(),
            });
        }
        Ok(())
    }

    /// 校验所有 4要素
    pub fn validate_four_elements(&self, elements: &[FourElement]) -> Result<()> {
        for (i, element) in elements.iter().enumerate() {
            self.validate_four_element(element).map_err(|e| {
                if let GatewayError::MissingField(field) = e {
                    GatewayError::InvalidField {
                        field: format!("elements[{}].{}", i, field),
                        value: "缺失".to_string(),
                    }
                } else {
                    e
                }
            })?;
        }
        Ok(())
    }
}

impl Default for FosValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_anchor() {
        let validator = FosValidator::new();
        let anchor = SixAnchor {
            event: "清理电脑桌面无用文件".to_string(),
            steps: vec!["列出桌面文件".to_string(), "筛选7天未修改文件".to_string()],
            judgment_logic: "文件大小<100MB且7天未修改".to_string(),
            verification_standard: "归档文件夹出现对应文件".to_string(),
            location: "我的Windows电脑".to_string(),
            subject: "我".to_string(),
        };
        assert!(validator.validate_six_anchor(&anchor).is_ok());
    }

    #[test]
    fn test_validate_empty_event() {
        let validator = FosValidator::new();
        let anchor = SixAnchor {
            event: String::new(),
            steps: vec!["步骤1".to_string()],
            judgment_logic: "条件1".to_string(),
            verification_standard: "标准1".to_string(),
            location: "位置1".to_string(),
            subject: "主体1".to_string(),
        };
        assert!(validator.validate_six_anchor(&anchor).is_err());
    }

    #[test]
    fn test_validate_empty_steps() {
        let validator = FosValidator::new();
        let anchor = SixAnchor {
            event: "测试事件".to_string(),
            steps: vec![],
            judgment_logic: "条件1".to_string(),
            verification_standard: "标准1".to_string(),
            location: "位置1".to_string(),
            subject: "主体1".to_string(),
        };
        assert!(validator.validate_six_anchor(&anchor).is_err());
    }

    #[test]
    fn test_validate_four_element() {
        let validator = FosValidator::new();
        let element = FourElement {
            action: "列出文件".to_string(),
            target: "桌面文件列表".to_string(),
            condition: "桌面存在文件".to_string(),
            expected_result: "获得完整文件列表".to_string(),
        };
        assert!(validator.validate_four_element(&element).is_ok());
    }
}
