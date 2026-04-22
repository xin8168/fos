//! 锁管理器

use crate::config::LockConfig;
use crate::error::{Error, Result};
use crate::lock::{Lock, LockId, LockState, LockType};
use crate::queue::LockWaitQueue;
use crate::LockKey;
use std::collections::HashMap;
use std::time::Duration;

/// 锁管理器
pub struct LockManager {
    /// 配置
    config: LockConfig,
    /// 锁存储
    locks: std::sync::RwLock<HashMap<LockKey, Lock>>,
    /// 等待队列
    wait_queues: std::sync::RwLock<HashMap<LockKey, LockWaitQueue>>,
    /// 统计信息
    stats: std::sync::Mutex<LockStats>,
}

/// 锁统计信息
#[derive(Debug, Default)]
struct LockStats {
    /// 总获取次数
    acquire_count: u64,
    /// 成功获取次数
    acquire_success: u64,
    /// 获取失败次数
    acquire_failed: u64,
    /// 释放次数
    release_count: u64,
    /// 超时次数
    timeout_count: u64,
}

impl LockManager {
    /// 创建新的锁管理器
    pub fn new(config: LockConfig) -> Self {
        Self {
            config,
            locks: std::sync::RwLock::new(HashMap::new()),
            wait_queues: std::sync::RwLock::new(HashMap::new()),
            stats: std::sync::Mutex::new(LockStats::default()),
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(LockConfig::default())
    }

    /// 尝试获取锁（非阻塞）
    pub fn try_lock(&self, key: &str, owner: &str) -> Result<Option<LockId>> {
        self.try_lock_with_type(key, owner, LockType::Exclusive)
    }

    /// 尝试获取可重入锁
    pub fn try_lock_reentrant(&self, key: &str, owner: &str) -> Result<Option<LockId>> {
        self.try_lock_with_type(key, owner, LockType::Reentrant)
    }

    /// 尝试获取共享锁
    pub fn try_lock_shared(&self, key: &str, owner: &str) -> Result<Option<LockId>> {
        self.try_lock_with_type(key, owner, LockType::Shared)
    }

    /// 尝试获取指定类型的锁
    fn try_lock_with_type(
        &self,
        key: &str,
        owner: &str,
        lock_type: LockType,
    ) -> Result<Option<LockId>> {
        let mut stats = self.stats.lock().unwrap();
        stats.acquire_count += 1;

        let mut locks = self.locks.write().unwrap();

        if let Some(existing_lock) = locks.get_mut(key) {
            // 检查是否过期
            if existing_lock.is_expired() {
                existing_lock.state = LockState::Expired;
            }

            // 尝试获取现有锁
            if existing_lock.try_acquire()? {
                stats.acquire_success += 1;
                return Ok(Some(existing_lock.id));
            } else {
                stats.acquire_failed += 1;
                return Ok(None);
            }
        }

        // 创建新锁
        let mut lock = match lock_type {
            LockType::Exclusive => Lock::new(key, owner),
            LockType::Reentrant => Lock::reentrant(key, owner),
            LockType::Shared => Lock::shared(key, owner),
        };
        lock.timeout_secs = self.config.lock_timeout_secs;

        lock.try_acquire()?;
        let lock_id = lock.id;
        locks.insert(key.to_string(), lock);

        stats.acquire_success += 1;
        Ok(Some(lock_id))
    }

    /// 获取锁（阻塞）
    pub fn lock(&self, key: &str, owner: &str) -> Result<LockId> {
        self.lock_with_timeout(key, owner, Duration::from_secs(self.config.wait_timeout_secs))
    }

    /// 获取锁（带超时）
    pub fn lock_with_timeout(&self, key: &str, owner: &str, timeout: Duration) -> Result<LockId> {
        let start = std::time::Instant::now();

        loop {
            if let Some(lock_id) = self.try_lock(key, owner)? {
                return Ok(lock_id);
            }

            if start.elapsed() >= timeout {
                let mut stats = self.stats.lock().unwrap();
                stats.timeout_count += 1;
                return Err(Error::Lock(format!("获取锁超时: {}", key)));
            }

            // 短暂等待后重试
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// 释放锁
    pub fn unlock(&self, key: &str, owner: &str) -> Result<bool> {
        let mut locks = self.locks.write().unwrap();
        let mut stats = self.stats.lock().unwrap();
        stats.release_count += 1;

        if let Some(lock) = locks.get_mut(key) {
            if lock.owner != owner {
                return Err(Error::Lock(format!("锁持有者不匹配: {}", key)));
            }

            if lock.release()? {
                // 完全释放，通知等待队列
                drop(locks);
                self.notify_waiters(key);
                return Ok(true);
            }
            return Ok(false);
        }

        Err(Error::Lock(format!("锁不存在: {}", key)))
    }

    /// 强制释放锁（管理员操作）
    pub fn force_unlock(&self, key: &str) -> Result<()> {
        let mut locks = self.locks.write().unwrap();

        if let Some(lock) = locks.remove(key) {
            tracing::warn!("强制释放锁: {} (owner: {})", key, lock.owner);
            drop(locks);
            self.notify_waiters(key);
            Ok(())
        } else {
            Err(Error::Lock(format!("锁不存在: {}", key)))
        }
    }

    /// 刷新锁过期时间
    pub fn refresh(&self, key: &str, owner: &str) -> Result<()> {
        let mut locks = self.locks.write().unwrap();

        if let Some(lock) = locks.get_mut(key) {
            if lock.owner != owner {
                return Err(Error::Lock("无权刷新他人的锁".to_string()));
            }
            lock.refresh()
        } else {
            Err(Error::Lock(format!("锁不存在: {}", key)))
        }
    }

    /// 检查锁是否被持有
    pub fn is_locked(&self, key: &str) -> bool {
        let locks = self.locks.read().unwrap();
        locks.get(key).map_or(false, |l| l.is_locked())
    }

    /// 获取锁信息
    pub fn get_lock(&self, key: &str) -> Option<Lock> {
        let locks = self.locks.read().unwrap();
        locks.get(key).cloned()
    }

    /// 获取锁持有者
    pub fn get_owner(&self, key: &str) -> Option<String> {
        let locks = self.locks.read().unwrap();
        locks.get(key).filter(|l| l.is_locked()).map(|l| l.owner.clone())
    }

    /// 清理过期锁
    pub fn cleanup_expired(&self) -> Result<usize> {
        let mut locks = self.locks.write().unwrap();
        let initial_count = locks.len();

        locks.retain(|_, lock| !lock.is_expired());

        let removed = initial_count - locks.len();
        if removed > 0 {
            tracing::debug!("清理了 {} 个过期锁", removed);
        }

        Ok(removed)
    }

    /// 获取活跃锁数量
    pub fn active_count(&self) -> usize {
        let locks = self.locks.read().unwrap();
        locks.values().filter(|l| l.is_locked()).count()
    }

    /// 获取所有锁
    pub fn get_all_locks(&self) -> Vec<Lock> {
        let locks = self.locks.read().unwrap();
        locks.values().cloned().collect()
    }

    /// 通知等待者
    fn notify_waiters(&self, key: &str) {
        let mut queues = self.wait_queues.write().unwrap();
        if let Some(queue) = queues.get_mut(key) {
            queue.notify_one();
        }
    }

    /// 加入等待队列
    pub fn wait_for_lock(&self, key: &str, owner: &str) -> Result<()> {
        let mut queues = self.wait_queues.write().unwrap();
        let queue = queues.entry(key.to_string()).or_insert_with(LockWaitQueue::new);
        queue.add_waiter(owner);
        Ok(())
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> LockManagerStats {
        let stats = self.stats.lock().unwrap();
        LockManagerStats {
            acquire_count: stats.acquire_count,
            acquire_success: stats.acquire_success,
            acquire_failed: stats.acquire_failed,
            release_count: stats.release_count,
            timeout_count: stats.timeout_count,
            active_locks: self.active_count(),
        }
    }
}

impl Default for LockManager {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// 锁管理器统计信息
#[derive(Debug, Clone)]
pub struct LockManagerStats {
    pub acquire_count: u64,
    pub acquire_success: u64,
    pub acquire_failed: u64,
    pub release_count: u64,
    pub timeout_count: u64,
    pub active_locks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = LockManager::with_defaults();
        assert_eq!(manager.active_count(), 0);
    }

    #[test]
    fn test_try_lock() {
        let manager = LockManager::with_defaults();

        let lock_id = manager.try_lock("test_key", "owner1").unwrap();
        assert!(lock_id.is_some());
        assert!(manager.is_locked("test_key"));
    }

    #[test]
    fn test_lock_release() {
        let manager = LockManager::with_defaults();

        manager.try_lock("test_key", "owner1").unwrap();
        assert!(manager.is_locked("test_key"));

        manager.unlock("test_key", "owner1").unwrap();
        assert!(!manager.is_locked("test_key"));
    }

    #[test]
    fn test_lock_conflict() {
        let manager = LockManager::with_defaults();

        manager.try_lock("test_key", "owner1").unwrap();

        // 第二个获取者应该失败
        let result = manager.try_lock("test_key", "owner2").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_reentrant_lock() {
        let manager = LockManager::with_defaults();

        // 第一次获取
        manager.try_lock_reentrant("test_key", "owner1").unwrap();

        // 同一持有者可以再次获取
        let result = manager.try_lock_reentrant("test_key", "owner1").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_force_unlock() {
        let manager = LockManager::with_defaults();

        manager.try_lock("test_key", "owner1").unwrap();
        manager.force_unlock("test_key").unwrap();

        assert!(!manager.is_locked("test_key"));
    }

    #[test]
    fn test_get_stats() {
        let manager = LockManager::with_defaults();

        manager.try_lock("key1", "owner1").unwrap();
        manager.try_lock("key2", "owner1").unwrap();
        manager.try_lock("key1", "owner2").unwrap(); // 失败

        let stats = manager.get_stats();
        assert_eq!(stats.acquire_count, 3);
        assert_eq!(stats.acquire_success, 2);
        assert_eq!(stats.acquire_failed, 1);
    }

    #[test]
    fn test_cleanup_expired() {
        let manager = LockManager::with_defaults();

        // 手动插入过期锁
        let mut lock = Lock::new("expired_key", "owner1");
        lock.state = LockState::Locked;
        lock.expires_at = Some(chrono::Utc::now() - chrono::Duration::seconds(1));
        manager.locks.write().unwrap().insert("expired_key".to_string(), lock);

        let removed = manager.cleanup_expired().unwrap();
        assert_eq!(removed, 1);
    }
}
