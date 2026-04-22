//! # FOS Validator - 脊髓神经层
//!
//! FOS神经元控制器的第二层神经网络：脊髓神经节
//!
//! 类比于人体神经系统的脊髓，负责信号校验和安全边界控制
//!
//! ## 核心职责
//! - 信号规则校验（是否符合安全规则）
//! - 安全边界检查（是否在允许范围内）
//! - 权限校验（是否有执行权限）
//! - 环境验证（设备状态是否允许）
//! - 危险动作拦截（反射弧机制）
//!
//! ## 信号处理流程
//! 信号标准化 → 规则校验 → 安全边界 → 权限检查 → 许可/拦截 → 传递到运动神经层
//!
//! ## 安全铁律
//! - 所有规则硬编码、不可动态修改
//! - 拒绝任何绕过脊髓神经层的信号
//! - 危险动作直接拦截（反射）
//! - 所有拦截必须记录到审计层

pub mod config;
pub mod engine;
pub mod error;
pub mod redline;
pub mod rules;
pub mod validator;

pub use error::{Result, ValidatorError};
pub use redline::{RedLine, RedLineChecker, RedLineSeverity, RedLineType};
pub use rules::{Rule, RuleResult, RuleType};
pub use validator::Validator;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 校验请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRequest {
    /// 请求ID
    pub id: String,

    /// 事件名称
    pub event: String,

    /// 执行步骤
    pub steps: Vec<String>,

    /// 判断逻辑
    pub judgment_logic: String,

    /// 校验标准
    pub verification_standard: String,

    /// 执行地点
    pub location: String,

    /// 执行主体
    pub subject: String,

    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// 校验结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// 请求ID
    pub request_id: String,

    /// 是否通过
    pub passed: bool,

    /// 错误信息
    pub errors: Vec<String>,

    /// 警告信息
    pub warnings: Vec<String>,

    /// 规则执行结果
    pub rule_results: Vec<RuleResult>,

    /// 校验时间（毫秒）
    pub validation_time_ms: u64,
}

/// 执行上下文
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// 用户ID
    pub user_id: String,

    /// 角色列表
    pub roles: Vec<String>,

    /// 权限列表
    pub permissions: Vec<String>,

    /// 设备信息
    pub device_info: DeviceInfo,

    /// 环境信息
    pub environment: HashMap<String, String>,
}

/// 设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// 设备ID
    pub device_id: String,

    /// 设备类型
    pub device_type: String,

    /// 设备状态
    pub status: DeviceStatus,

    /// 设备能力
    pub capabilities: Vec<String>,
}

/// 设备状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeviceStatus {
    /// 在线
    Online,

    /// 离线
    Offline,

    /// 忙碌
    Busy,

    /// 错误
    Error,

    /// 维护中
    Maintenance,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_request_creation() {
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

        assert_eq!(request.id, "req-001");
        assert_eq!(request.steps.len(), 1);
    }

    #[test]
    fn test_device_status() {
        let status = DeviceStatus::Online;
        assert_eq!(status, DeviceStatus::Online);
    }
}
