//! 测试夹具（Fixtures）
//!
//! 提供预定义的测试数据

use serde_json::json;

/// 创建测试用的FOS命令
pub fn fixture_fos_command() -> serde_json::Value {
    json!({
        "id": "test-cmd-001",
        "anchor": {
            "event": "测试事件",
            "steps": ["步骤1", "步骤2", "步骤3"],
            "judgment_logic": "测试判断逻辑",
            "verification_standard": "测试校验标准",
            "location": "测试环境",
            "subject": "测试主体"
        },
        "timestamp": 1700000000,
        "metadata": {}
    })
}

/// 创建测试用的成功事件
pub fn fixture_success_event() -> serde_json::Value {
    json!({
        "id": "test-event-001",
        "name": "测试成功事件",
        "event_type": "test",
        "status": "success",
        "steps": ["步骤1", "步骤2"],
        "result": {
            "success": true,
            "output": "测试输出"
        }
    })
}

/// 创建测试用的拦截日志
pub fn fixture_audit_log() -> serde_json::Value {
    json!({
        "id": "test-audit-001",
        "log_type": "format_blocked",
        "original_command": "测试命令",
        "reason": "格式错误",
        "created_at": "2026-03-09T00:00:00Z"
    })
}

/// 创建测试用的设备信息
pub fn fixture_device_info() -> serde_json::Value {
    json!({
        "device_id": "test-device-001",
        "device_type": "computer",
        "status": "online",
        "capabilities": ["file_operation", "command_execution"]
    })
}

/// 创建测试用的权限上下文
pub fn fixture_permission_context() -> serde_json::Value {
    json!({
        "user_id": "test-user-001",
        "roles": ["admin"],
        "permissions": ["read", "write", "execute"]
    })
}

/// 创建测试用的配置
pub fn fixture_config() -> serde_json::Value {
    json!({
        "gateway": {
            "host": "127.0.0.1",
            "port": 8080
        },
        "validator": {
            "strict_mode": true
        },
        "bus": {
            "max_concurrent_tasks": 100
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_fos_command() {
        let cmd = fixture_fos_command();
        assert_eq!(cmd["id"], "test-cmd-001");
    }

    #[test]
    fn test_fixture_success_event() {
        let event = fixture_success_event();
        assert_eq!(event["status"], "success");
    }
}
