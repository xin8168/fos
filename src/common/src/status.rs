//! FOS 事件类型和状态枚举

use serde::{Deserialize, Serialize};

/// 事件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

impl EventType {
    /// 获取事件类型的字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::DeviceControl => "device_control",
            EventType::FileOperation => "file_operation",
            EventType::SystemCommand => "system_command",
            EventType::NetworkOperation => "network_operation",
            EventType::AutomationTask => "automation_task",
            EventType::SkillExecution => "skill_execution",
        }
    }
}

/// 事件状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

impl EventStatus {
    /// 获取事件状态的字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            EventStatus::Pending => "pending",
            EventStatus::Validating => "validating",
            EventStatus::Executing => "executing",
            EventStatus::Success => "success",
            EventStatus::Failed => "failed",
            EventStatus::Blocked => "blocked",
            EventStatus::RolledBack => "rolled_back",
        }
    }

    /// 判断事件是否处于终态
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            EventStatus::Success
                | EventStatus::Failed
                | EventStatus::Blocked
                | EventStatus::RolledBack
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_as_str() {
        assert_eq!(EventType::DeviceControl.as_str(), "device_control");
        assert_eq!(EventType::FileOperation.as_str(), "file_operation");
    }

    #[test]
    fn test_event_status_as_str() {
        assert_eq!(EventStatus::Pending.as_str(), "pending");
        assert_eq!(EventStatus::Success.as_str(), "success");
    }

    #[test]
    fn test_event_status_is_terminal() {
        assert!(EventStatus::Success.is_terminal());
        assert!(EventStatus::Failed.is_terminal());
        assert!(EventStatus::Blocked.is_terminal());
        assert!(EventStatus::RolledBack.is_terminal());
        assert!(!EventStatus::Pending.is_terminal());
        assert!(!EventStatus::Executing.is_terminal());
    }
}
