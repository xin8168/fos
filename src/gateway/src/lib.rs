//! # FOS Gateway - 感觉神经层
//!
//! FOS神经元控制器的第一层神经网络：感觉神经
//!
//! ## 类比：人体神经系统
//! 类比于人体神经系统的感觉神经，负责接收外界信号并解析
//!
//! ## 核心职责
//! - 神经信号输入接收（来自大脑/AI的意图信号）
//! - 信号格式校验（六维锚定 + 四要素执行规范）
//! - 信号解析与标准化
//! - 协议转换
//!
//! ## 信号处理流程
//! 输入信号 → 格式校验 → 语义解析 → 信号标准化 → 传递到脊髓神经层
//!
//! ## 安全铁律
//! - 所有信号必须经过格式校验
//! - 拒绝任何不符合协议的信号
//! - 拒绝任何绕过感觉神经层的信号

pub mod config;
pub mod error;
pub mod handler;
pub mod middleware;
pub mod protocol;
pub mod server;
pub mod token;
pub mod validator;

pub use error::{GatewayError, Result};
pub use server::{GatewayConfig, GatewayServer, GatewayStats};
pub use token::{TokenConfig, TokenInfo, TokenManager, TokenType};
pub use validator::{FosValidator, ValidatorConfig};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// FOS 6维锚定结构
///
/// FOS 的核心设计：每一个执行动作必须有 6 个锚定点
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SixAnchor {
    /// 事件: 用户想要达成的目标
    pub event: String,

    /// 步骤: 明确的执行步骤序列
    pub steps: Vec<String>,

    /// 判断逻辑: 每一步的判断条件
    pub judgment_logic: String,

    /// 校验标准: 最终结果的成功标准
    pub verification_standard: String,

    /// 地点: 执行环境/设备
    pub location: String,

    /// 主体: 执行者身份
    pub subject: String,
}

/// FOS 4要素执行结构
///
/// 每个执行步骤必须包含的 4 个要素
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FourElement {
    /// 动作: 具体的操作行为
    pub action: String,

    /// 对象: 操作的目标对象
    pub target: String,

    /// 条件: 执行前提条件
    pub condition: String,

    /// 结果: 预期的执行结果
    pub expected_result: String,
}

/// FOS 完整命令结构
///
/// 包含完整的 FOS 执行指令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FosCommand {
    /// 命令唯一ID
    pub id: String,

    /// 6维锚定
    pub anchor: SixAnchor,

    /// 4要素执行（每个步骤的详细定义）
    pub elements: Vec<FourElement>,

    /// 时间戳
    pub timestamp: i64,

    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// FOS 事件结构
///
/// 代表一个完整的 FOS 执行事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FosEvent {
    /// 事件ID
    pub id: String,

    /// 事件名称
    pub name: String,

    /// 事件类型
    pub event_type: EventType,

    /// 事件状态
    pub status: EventStatus,

    /// 6维锚定
    pub anchor: SixAnchor,

    /// 执行结果
    pub result: Option<ExecutionResult>,

    /// 时间戳
    pub created_at: i64,

    pub updated_at: i64,
}

/// 事件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// 设备控制
    DeviceControl,

    /// 文件操作
    FileOperation,

    /// 系统命令
    SystemCommand,

    /// 网络操作
    NetworkOperation,

    /// 自动化任务
    AutomationTask,

    /// SKILL 执行
    SkillExecution,
}

/// 事件状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventStatus {
    /// 待处理
    Pending,

    /// 校验中
    Validating,

    /// 执行中
    Executing,

    /// 成功
    Success,

    /// 失败
    Failed,

    /// 已拦截
    Blocked,

    /// 已回滚
    RolledBack,
}

/// 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// 是否成功
    pub success: bool,

    /// 输出内容
    pub output: String,

    /// 错误信息
    pub error: Option<String>,

    /// 执行时间（毫秒）
    pub duration_ms: u64,

    /// 步骤结果
    pub step_results: Vec<StepResult>,
}

/// 步骤执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// 步骤索引
    pub step_index: usize,

    /// 是否成功
    pub success: bool,

    /// 输出
    pub output: String,

    /// 错误
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_six_anchor_creation() {
        let anchor = SixAnchor {
            event: "清理电脑桌面无用文件".to_string(),
            steps: vec![
                "列出桌面文件".to_string(),
                "筛选7天未修改文件".to_string(),
                "移动到归档文件夹".to_string(),
            ],
            judgment_logic: "文件大小<100MB且7天未修改".to_string(),
            verification_standard: "归档文件夹出现对应文件，桌面无残留".to_string(),
            location: "我的Windows电脑".to_string(),
            subject: "我".to_string(),
        };

        assert_eq!(anchor.event, "清理电脑桌面无用文件");
        assert_eq!(anchor.steps.len(), 3);
    }

    #[test]
    fn test_four_element_creation() {
        let element = FourElement {
            action: "列出文件".to_string(),
            target: "桌面文件列表".to_string(),
            condition: "桌面存在文件".to_string(),
            expected_result: "获得完整文件列表".to_string(),
        };

        assert_eq!(element.action, "列出文件");
    }

    #[test]
    fn test_fos_command_serialization() {
        let command = FosCommand {
            id: "cmd-001".to_string(),
            anchor: SixAnchor {
                event: "测试事件".to_string(),
                steps: vec!["步骤1".to_string()],
                judgment_logic: "条件1".to_string(),
                verification_standard: "标准1".to_string(),
                location: "位置1".to_string(),
                subject: "主体1".to_string(),
            },
            elements: vec![],
            timestamp: 1700000000,
            metadata: HashMap::new(),
        };

        let json = serde_json::to_string(&command).unwrap();
        assert!(json.contains("测试事件"));
    }
}
