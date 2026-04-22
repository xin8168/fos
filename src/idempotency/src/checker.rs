//! 幂等检查器

use crate::config::IdempotencyConfig;
use crate::error::{Error, Result};
use crate::key::{IdempotencyKey, KeyStatus};
use std::collections::HashMap;

/// 幂等检查器
pub struct IdempotencyChecker {
    /// 配置
    config: IdempotencyConfig,
    /// 键存储
    keys: std::sync::RwLock<HashMap<String, IdempotencyKey>>,
    /// 统计
    stats: std::sync::Mutex<CheckerStats>,
}

/// 检查器统计
#[derive(Debug, Default)]
struct CheckerStats {
    total_checks: u64,
    first_time_requests: u64,
    duplicate_requests: u64,
    processing_requests: u64,
}

impl IdempotencyChecker {
    /// 创建新的检查器
    pub fn new(config: IdempotencyConfig) -> Self {
        Self {
            config,
            keys: std::sync::RwLock::new(HashMap::new()),
            stats: std::sync::Mutex::new(CheckerStats::default()),
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(IdempotencyConfig::default())
    }

    /// 检查幂等键
    pub fn check(&self, key: &str, resource: &str, operation: &str) -> Result<CheckResult> {
        let mut stats = self.stats.lock().unwrap();
        stats.total_checks += 1;

        let mut keys = self.keys.write().unwrap();

        // 清理过期键
        if self.config.auto_cleanup {
            keys.retain(|_, v| !v.is_expired());
        }

        if let Some(existing_key) = keys.get(key) {
            match existing_key.status {
                KeyStatus::Completed => {
                    stats.duplicate_requests += 1;
                    Ok(CheckResult::Duplicate(existing_key.clone()))
                },
                KeyStatus::Processing => {
                    stats.processing_requests += 1;
                    Ok(CheckResult::Processing)
                },
                KeyStatus::Failed => {
                    // 可以重试
                    Ok(CheckResult::Retry(existing_key.clone()))
                },
                KeyStatus::Pending => {
                    // 理论上不应该出现这种情况
                    Ok(CheckResult::Processing)
                },
            }
        } else {
            // 新请求
            let new_key =
                IdempotencyKey::new(key, resource, operation, self.config.key_expiry_secs);
            keys.insert(key.to_string(), new_key.clone());
            stats.first_time_requests += 1;
            Ok(CheckResult::FirstTime(new_key))
        }
    }

    /// 标记处理中
    pub fn mark_processing(&self, key: &str) -> Result<()> {
        let mut keys = self.keys.write().unwrap();
        if let Some(k) = keys.get_mut(key) {
            k.mark_processing();
            Ok(())
        } else {
            Err(Error::Idempotency(format!("键不存在: {}", key)))
        }
    }

    /// 标记完成
    pub fn mark_completed(&self, key: &str) -> Result<()> {
        let mut keys = self.keys.write().unwrap();
        if let Some(k) = keys.get_mut(key) {
            k.mark_completed();
            Ok(())
        } else {
            Err(Error::Idempotency(format!("键不存在: {}", key)))
        }
    }

    /// 标记失败
    pub fn mark_failed(&self, key: &str) -> Result<()> {
        let mut keys = self.keys.write().unwrap();
        if let Some(k) = keys.get_mut(key) {
            k.mark_failed();
            Ok(())
        } else {
            Err(Error::Idempotency(format!("键不存在: {}", key)))
        }
    }

    /// 获取键状态
    pub fn get_status(&self, key: &str) -> Option<KeyStatus> {
        let keys = self.keys.read().unwrap();
        keys.get(key).map(|k| k.status)
    }

    /// 检查键是否存在
    pub fn exists(&self, key: &str) -> bool {
        let keys = self.keys.read().unwrap();
        keys.contains_key(key)
    }

    /// 清理过期键
    pub fn cleanup_expired(&self) -> usize {
        let mut keys = self.keys.write().unwrap();
        let initial_len = keys.len();
        keys.retain(|_, v| !v.is_expired());
        initial_len - keys.len()
    }

    /// 获取活跃键数量
    pub fn active_count(&self) -> usize {
        let keys = self.keys.read().unwrap();
        keys.len()
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> CheckerStatsInfo {
        let stats = self.stats.lock().unwrap();
        CheckerStatsInfo {
            total_checks: stats.total_checks,
            first_time_requests: stats.first_time_requests,
            duplicate_requests: stats.duplicate_requests,
            processing_requests: stats.processing_requests,
            active_keys: self.active_count(),
        }
    }
}

impl Default for IdempotencyChecker {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// 检查结果
#[derive(Debug, Clone)]
pub enum CheckResult {
    /// 首次请求
    FirstTime(IdempotencyKey),
    /// 重复请求（已完成）
    Duplicate(IdempotencyKey),
    /// 正在处理中
    Processing,
    /// 可以重试（之前失败）
    Retry(IdempotencyKey),
}

impl CheckResult {
    /// 是否为首次请求
    pub fn is_first_time(&self) -> bool {
        matches!(self, CheckResult::FirstTime(_))
    }

    /// 是否为重复请求
    pub fn is_duplicate(&self) -> bool {
        matches!(self, CheckResult::Duplicate(_))
    }

    /// 是否正在处理
    pub fn is_processing(&self) -> bool {
        matches!(self, CheckResult::Processing)
    }

    /// 是否可以重试
    pub fn is_retry(&self) -> bool {
        matches!(self, CheckResult::Retry(_))
    }

    /// 获取键引用
    pub fn get_key(&self) -> Option<&IdempotencyKey> {
        match self {
            CheckResult::FirstTime(k) => Some(k),
            CheckResult::Duplicate(k) => Some(k),
            CheckResult::Processing => None,
            CheckResult::Retry(k) => Some(k),
        }
    }
}

/// 统计信息
#[derive(Debug, Clone)]
pub struct CheckerStatsInfo {
    pub total_checks: u64,
    pub first_time_requests: u64,
    pub duplicate_requests: u64,
    pub processing_requests: u64,
    pub active_keys: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checker_creation() {
        let checker = IdempotencyChecker::with_defaults();
        assert_eq!(checker.active_count(), 0);
    }

    #[test]
    fn test_first_time_request() {
        let checker = IdempotencyChecker::with_defaults();

        let result = checker.check("key1", "order", "create").unwrap();
        assert!(result.is_first_time());
        assert_eq!(checker.active_count(), 1);
    }

    #[test]
    fn test_duplicate_request() {
        let checker = IdempotencyChecker::with_defaults();

        // 第一次请求
        checker.check("key1", "order", "create").unwrap();
        checker.mark_processing("key1").unwrap();
        checker.mark_completed("key1").unwrap();

        // 重复请求
        let result = checker.check("key1", "order", "create").unwrap();
        assert!(result.is_duplicate());
    }

    #[test]
    fn test_processing_status() {
        let checker = IdempotencyChecker::with_defaults();

        checker.check("key1", "order", "create").unwrap();
        checker.mark_processing("key1").unwrap();

        let result = checker.check("key1", "order", "create").unwrap();
        assert!(result.is_processing());
    }

    #[test]
    fn test_failed_retry() {
        let checker = IdempotencyChecker::with_defaults();

        checker.check("key1", "order", "create").unwrap();
        checker.mark_processing("key1").unwrap();
        checker.mark_failed("key1").unwrap();

        let result = checker.check("key1", "order", "create").unwrap();
        assert!(result.is_retry());
    }

    #[test]
    fn test_cleanup_expired() {
        let config =
            IdempotencyConfig { key_expiry_secs: 0, auto_cleanup: true, max_cache_entries: 10000 };
        let checker = IdempotencyChecker::new(config);

        checker.check("key1", "order", "create").unwrap();

        let removed = checker.cleanup_expired();
        assert_eq!(removed, 1);
    }

    #[test]
    fn test_get_stats() {
        let checker = IdempotencyChecker::with_defaults();

        checker.check("key1", "order", "create").unwrap();
        checker.check("key2", "order", "create").unwrap();

        let stats = checker.get_stats();
        assert_eq!(stats.total_checks, 2);
        assert_eq!(stats.first_time_requests, 2);
    }
}
