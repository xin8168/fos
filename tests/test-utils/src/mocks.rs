//! Mock数据生成器
//!
//! 提供各种Mock数据生成函数

use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use uuid::Uuid;

/// 生成Mock事件ID
pub fn mock_event_id() -> String {
    format!("evt-{}", Uuid::new_v4())
}

/// 生成Mock命令ID
pub fn mock_command_id() -> String {
    format!("cmd-{}", Uuid::new_v4())
}

/// 生成Mock任务ID
pub fn mock_task_id() -> String {
    format!("task-{}", Uuid::new_v4())
}

/// 生成Mock设备ID
pub fn mock_device_id() -> String {
    format!("dev-{}", Uuid::new_v4())
}

/// 生成Mock时间戳
pub fn mock_timestamp() -> i64 {
    Utc::now().timestamp()
}

/// 生成Mock事件
pub fn mock_event(name: &str) -> Value {
    json!({
        "id": mock_event_id(),
        "name": name,
        "event_type": "mock",
        "status": "pending",
        "created_at": Utc::now().to_rfc3339()
    })
}

/// 生成Mock FOS命令
pub fn mock_fos_command(event: &str) -> Value {
    json!({
        "id": mock_command_id(),
        "anchor": {
            "event": event,
            "steps": vec!["mock_step_1", "mock_step_2"],
            "judgment_logic": "mock_logic",
            "verification_standard": "mock_standard",
            "location": "mock_location",
            "subject": "mock_subject"
        },
        "timestamp": mock_timestamp(),
        "metadata": json!({})
    })
}

/// 生成Mock执行结果（成功）
pub fn mock_success_result() -> Value {
    json!({
        "success": true,
        "output": "Mock execution successful",
        "duration_ms": 100,
        "step_results": vec![
            {"step_index": 0, "success": true, "output": "Step 1 done"},
            {"step_index": 1, "success": true, "output": "Step 2 done"}
        ]
    })
}

/// 生成Mock执行结果（失败）
pub fn mock_failure_result(reason: &str) -> Value {
    json!({
        "success": false,
        "error": reason,
        "duration_ms": 50,
        "step_results": vec![
            {"step_index": 0, "success": true, "output": "Step 1 done"},
            {"step_index": 1, "success": false, "output": "", "error": reason}
        ]
    })
}

/// 生成Mock设备状态
pub fn mock_device_status(online: bool) -> Value {
    json!({
        "device_id": mock_device_id(),
        "device_type": "computer",
        "status": if online { "online" } else { "offline" },
        "last_heartbeat": Utc::now().to_rfc3339(),
        "capabilities": vec!["file_operation", "command_execution"]
    })
}

/// 生成Mock权限上下文
pub fn mock_permission_context(is_admin: bool) -> Value {
    if is_admin {
        json!({
            "user_id": "admin-user",
            "roles": vec!["admin"],
            "permissions": vec!["read", "write", "execute", "admin"]
        })
    } else {
        json!({
            "user_id": "normal-user",
            "roles": vec!["user"],
            "permissions": vec!["read", "execute"]
        })
    }
}

/// 生成Mock规则
pub fn mock_rule(rule_type: &str) -> Value {
    json!({
        "id": format!("rule-{}", Uuid::new_v4()),
        "rule_type": rule_type,
        "name": format!("Mock {} Rule", rule_type),
        "condition": "mock_condition",
        "action": "mock_action",
        "enabled": true
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_event_id() {
        let id = mock_event_id();
        assert!(id.starts_with("evt-"));
    }

    #[test]
    fn test_mock_event() {
        let event = mock_event("测试事件");
        assert_eq!(event["name"], "测试事件");
    }

    #[test]
    fn test_mock_success_result() {
        let result = mock_success_result();
        assert_eq!(result["success"], true);
    }

    #[test]
    fn test_mock_failure_result() {
        let result = mock_failure_result("测试错误");
        assert_eq!(result["success"], false);
    }
}
