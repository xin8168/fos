//! 自愈机制

use crate::error::{Error, Result};
use std::collections::HashMap;

/// 自愈策略
pub type HealingStrategy = Box<dyn Fn(&str) -> Result<bool> + Send + Sync>;

/// 自愈记录
#[derive(Debug, Clone)]
pub struct HealingRecord {
    /// 问题名称
    pub issue: String,
    /// 尝试次数
    pub attempts: u32,
    /// 是否成功
    pub success: bool,
    /// 最后尝试时间
    pub last_attempt: chrono::DateTime<chrono::Utc>,
}

/// 自愈机制
pub struct SelfHealing {
    /// 已注册的策略
    strategies: std::sync::Mutex<HashMap<String, HealingStrategy>>,
    /// 治愈记录
    records: std::sync::Mutex<HashMap<String, HealingRecord>>,
    /// 最大重试次数
    max_retries: u32,
}

impl SelfHealing {
    /// 创建新的自愈机制
    pub fn new() -> Self {
        Self {
            strategies: std::sync::Mutex::new(HashMap::new()),
            records: std::sync::Mutex::new(HashMap::new()),
            max_retries: 3,
        }
    }

    /// 注册自愈策略
    pub fn register<F>(&self, issue: &str, strategy: F)
    where
        F: Fn(&str) -> Result<bool> + Send + Sync + 'static,
    {
        self.strategies.lock().unwrap().insert(issue.to_string(), Box::new(strategy));
    }

    /// 尝试自愈
    pub fn attempt(&self, issue: &str) -> Result<bool> {
        let strategies = self.strategies.lock().unwrap();

        if let Some(strategy) = strategies.get(issue) {
            let result = strategy(issue);

            // 记录尝试
            self.record_attempt(issue, result.is_ok() && *result.as_ref().unwrap_or(&false));

            result
        } else {
            Err(Error::HealingFailed(format!("未找到问题 '{}' 的自愈策略", issue)))
        }
    }

    /// 记录尝试
    fn record_attempt(&self, issue: &str, success: bool) {
        let mut records = self.records.lock().unwrap();

        let record = records.entry(issue.to_string()).or_insert(HealingRecord {
            issue: issue.to_string(),
            attempts: 0,
            success: false,
            last_attempt: chrono::Utc::now(),
        });

        record.attempts += 1;
        record.success = success;
        record.last_attempt = chrono::Utc::now();
    }

    /// 获取治愈记录
    pub fn get_record(&self, issue: &str) -> Option<HealingRecord> {
        self.records.lock().unwrap().get(issue).cloned()
    }

    /// 获取所有记录
    pub fn get_all_records(&self) -> Vec<HealingRecord> {
        self.records.lock().unwrap().values().cloned().collect()
    }

    /// 清除记录
    pub fn clear_records(&self) {
        self.records.lock().unwrap().clear();
    }

    /// 检查是否超过最大重试次数
    pub fn should_retry(&self, issue: &str) -> bool {
        if let Some(record) = self.get_record(issue) {
            record.attempts < self.max_retries
        } else {
            true
        }
    }
}

impl Default for SelfHealing {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_self_healing_creation() {
        let healing = SelfHealing::new();
        assert!(healing.get_all_records().is_empty());
    }

    #[test]
    fn test_register_strategy() {
        let healing = SelfHealing::new();
        healing.register("test_issue", |_| Ok(true));

        let result = healing.attempt("test_issue");
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_attempt_unknown_issue() {
        let healing = SelfHealing::new();
        let result = healing.attempt("unknown");
        assert!(result.is_err());
    }

    #[test]
    fn test_record_attempt() {
        let healing = SelfHealing::new();
        healing.register("test", |_| Ok(true));
        healing.attempt("test").unwrap();

        let record = healing.get_record("test");
        assert!(record.is_some());
        assert_eq!(record.unwrap().attempts, 1);
    }

    #[test]
    fn test_should_retry() {
        let healing = SelfHealing::new();
        assert!(healing.should_retry("new_issue"));
    }
}
