//! 测试上下文
//!
//! 提供测试执行环境的管理

use serde_json::Value;
use std::collections::HashMap;

/// 测试上下文
///
/// 管理测试执行过程中的状态和配置
#[derive(Debug, Clone)]
pub struct TestContext {
    /// 上下文ID
    pub id: String,
    /// 测试名称
    pub name: String,
    /// 开始时间
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// 上下文数据
    pub data: HashMap<String, Value>,
}

impl TestContext {
    /// 创建新的测试上下文
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            start_time: chrono::Utc::now(),
            data: HashMap::new(),
        }
    }

    /// 创建带名称的测试上下文
    pub fn with_name(name: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            start_time: chrono::Utc::now(),
            data: HashMap::new(),
        }
    }

    /// 设置数据
    pub fn set(&mut self, key: &str, value: Value) -> &mut Self {
        self.data.insert(key.to_string(), value);
        self
    }

    /// 获取数据
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    /// 获取数据（字符串）
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.data.get(key).and_then(|v| v.as_str())
    }

    /// 获取数据（整数）
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.data.get(key).and_then(|v| v.as_i64())
    }

    /// 获取数据（布尔值）
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.data.get(key).and_then(|v| v.as_bool())
    }

    /// 检查键是否存在
    pub fn has(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// 删除数据
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.data.remove(key)
    }

    /// 清空数据
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// 获取执行时间（毫秒）
    pub fn elapsed_ms(&self) -> i64 {
        (chrono::Utc::now() - self.start_time).num_milliseconds()
    }
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_context_new() {
        let ctx = TestContext::new();
        assert!(!ctx.id.is_empty());
    }

    #[test]
    fn test_context_with_name() {
        let ctx = TestContext::with_name("test_example");
        assert_eq!(ctx.name, "test_example");
    }

    #[test]
    fn test_context_set_get() {
        let mut ctx = TestContext::new();
        ctx.set("key", json!("value"));

        assert_eq!(ctx.get_str("key"), Some("value"));
    }

    #[test]
    fn test_context_has() {
        let mut ctx = TestContext::new();
        assert!(!ctx.has("key"));

        ctx.set("key", json!("value"));
        assert!(ctx.has("key"));
    }

    #[test]
    fn test_context_remove() {
        let mut ctx = TestContext::new();
        ctx.set("key", json!("value"));

        let removed = ctx.remove("key");
        assert_eq!(removed, Some(json!("value")));
        assert!(!ctx.has("key"));
    }

    #[test]
    fn test_context_elapsed() {
        let ctx = TestContext::new();
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(ctx.elapsed_ms() >= 10);
    }
}
