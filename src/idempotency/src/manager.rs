//! 幂等管理器

use crate::cache::ResultCache;
use crate::checker::{CheckResult, CheckerStatsInfo, IdempotencyChecker};
use crate::config::IdempotencyConfig;
use crate::error::Result;
use crate::key::KeyStatus;

/// 幂等管理器
pub struct IdempotencyManager {
    checker: IdempotencyChecker,
    cache: ResultCache,
    config: IdempotencyConfig,
}

impl IdempotencyManager {
    pub fn new(config: IdempotencyConfig) -> Self {
        Self { checker: IdempotencyChecker::new(config.clone()), cache: ResultCache::new(), config }
    }

    pub fn with_defaults() -> Self {
        Self::new(IdempotencyConfig::default())
    }

    /// 执行幂等操作
    pub fn execute<F, T>(&self, key: &str, resource: &str, operation: &str, f: F) -> Result<T>
    where
        F: FnOnce() -> T,
        T: serde::Serialize + serde::de::DeserializeOwned + Clone,
    {
        match self.checker.check(key, resource, operation)? {
            CheckResult::FirstTime(_) => {
                self.checker.mark_processing(key)?;
                let result = f();
                let json = serde_json::to_value(&result).unwrap_or_default();
                self.cache.store(key, json, self.config.key_expiry_secs);
                self.checker.mark_completed(key)?;
                Ok(result)
            },
            CheckResult::Duplicate(_) => {
                if let Some(cached) = self.cache.get(key) {
                    Ok(serde_json::from_value(cached.data).unwrap())
                } else {
                    Err(crate::error::Error::Idempotency("缓存结果不存在".into()))
                }
            },
            CheckResult::Processing => {
                Err(crate::error::Error::Idempotency("请求正在处理中".into()))
            },
            CheckResult::Retry(_) => {
                self.checker.mark_processing(key)?;
                let result = f();
                let json = serde_json::to_value(&result).unwrap_or_default();
                self.cache.store(key, json, self.config.key_expiry_secs);
                self.checker.mark_completed(key)?;
                Ok(result)
            },
        }
    }

    pub fn check(&self, key: &str, resource: &str, operation: &str) -> Result<CheckResult> {
        self.checker.check(key, resource, operation)
    }

    pub fn mark_processing(&self, key: &str) -> Result<()> {
        self.checker.mark_processing(key)
    }

    pub fn mark_completed(&self, key: &str) -> Result<()> {
        self.checker.mark_completed(key)
    }

    pub fn mark_failed(&self, key: &str) -> Result<()> {
        self.checker.mark_failed(key)
    }

    pub fn get_cached_result(&self, key: &str) -> Option<serde_json::Value> {
        self.cache.get(key).map(|r| r.data)
    }

    pub fn store_result(&self, key: &str, data: serde_json::Value) {
        self.cache.store(key, data, self.config.key_expiry_secs);
    }

    pub fn get_status(&self, key: &str) -> Option<KeyStatus> {
        self.checker.get_status(key)
    }

    pub fn cleanup(&self) -> usize {
        self.checker.cleanup_expired() + self.cache.cleanup_expired()
    }

    pub fn get_stats(&self) -> ManagerStats {
        ManagerStats { checker: self.checker.get_stats(), cache: self.cache.get_stats() }
    }
}

impl Default for IdempotencyManager {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[derive(Debug, Clone)]
pub struct ManagerStats {
    pub checker: CheckerStatsInfo,
    pub cache: crate::cache::CacheStats,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = IdempotencyManager::with_defaults();
        assert!(manager.checker.active_count() == 0);
    }

    #[test]
    fn test_execute_first_time() {
        let manager = IdempotencyManager::with_defaults();
        let result: i32 = manager.execute("key1", "order", "create", || 42).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_execute_duplicate() {
        let manager = IdempotencyManager::with_defaults();

        let r1: i32 = manager.execute("key1", "order", "create", || 42).unwrap();
        let r2: i32 = manager.execute("key1", "order", "create", || 100).unwrap();

        assert_eq!(r1, 42);
        assert_eq!(r2, 42); // 返回缓存结果
    }

    #[test]
    fn test_manual_flow() {
        let manager = IdempotencyManager::with_defaults();

        let result = manager.check("key1", "order", "create").unwrap();
        assert!(result.is_first_time());

        manager.mark_processing("key1").unwrap();
        manager.store_result("key1", serde_json::json!({"id": 1}));
        manager.mark_completed("key1").unwrap();

        let result = manager.check("key1", "order", "create").unwrap();
        assert!(result.is_duplicate());
    }
}
