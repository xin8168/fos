//! 锁核心结构定义

use crate::error::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

/// 锁ID类型
pub type LockId = Uuid;

/// 锁定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lock {
    /// 锁ID
    pub id: LockId,
    /// 锁键名
    pub key: String,
    /// 锁持有者
    pub owner: String,
    /// 锁类型
    pub lock_type: LockType,
    /// 锁状态
    pub state: LockState,
    /// 获取时间
    pub acquired_at: Option<DateTime<Utc>>,
    /// 过期时间
    pub expires_at: Option<DateTime<Utc>>,
    /// 重入计数
    pub reentrant_count: u32,
    /// 锁超时（秒）
    pub timeout_secs: u64,
    /// 元数据
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

impl Lock {
    /// 创建新锁
    pub fn new(key: &str, owner: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            key: key.to_string(),
            owner: owner.to_string(),
            lock_type: LockType::Exclusive,
            state: LockState::Unlocked,
            acquired_at: None,
            expires_at: None,
            reentrant_count: 0,
            timeout_secs: 30,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// 创建可重入锁
    pub fn reentrant(key: &str, owner: &str) -> Self {
        let mut lock = Self::new(key, owner);
        lock.lock_type = LockType::Reentrant;
        lock
    }

    /// 创建共享锁
    pub fn shared(key: &str, owner: &str) -> Self {
        let mut lock = Self::new(key, owner);
        lock.lock_type = LockType::Shared;
        lock
    }

    /// 设置超时时间
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// 尝试获取锁
    pub fn try_acquire(&mut self) -> Result<bool> {
        match self.state {
            LockState::Unlocked => {
                self.acquire()?;
                Ok(true)
            },
            LockState::Locked => {
                // 检查是否是同一持有者（可重入）
                if self.lock_type == LockType::Reentrant {
                    self.reentrant_count += 1;
                    return Ok(true);
                }
                Ok(false)
            },
            LockState::Expired => {
                self.acquire()?;
                Ok(true)
            },
        }
    }

    /// 获取锁
    fn acquire(&mut self) -> Result<()> {
        let now = Utc::now();
        self.state = LockState::Locked;
        self.acquired_at = Some(now);
        self.expires_at = Some(now + chrono::Duration::seconds(self.timeout_secs as i64));
        self.reentrant_count = 1;
        Ok(())
    }

    /// 释放锁
    pub fn release(&mut self) -> Result<bool> {
        match self.state {
            LockState::Locked => {
                if self.lock_type == LockType::Reentrant && self.reentrant_count > 1 {
                    self.reentrant_count -= 1;
                    Ok(false) // 还未完全释放
                } else {
                    self.state = LockState::Unlocked;
                    self.acquired_at = None;
                    self.expires_at = None;
                    self.reentrant_count = 0;
                    Ok(true) // 完全释放
                }
            },
            _ => Err(Error::Lock("锁未被持有，无法释放".to_string())),
        }
    }

    /// 检查是否已过期
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// 检查是否被指定持有者持有
    pub fn is_held_by(&self, owner: &str) -> bool {
        self.state == LockState::Locked && self.owner == owner
    }

    /// 检查是否被锁定
    pub fn is_locked(&self) -> bool {
        matches!(self.state, LockState::Locked) && !self.is_expired()
    }

    /// 获取剩余有效期
    pub fn remaining_ttl(&self) -> Option<Duration> {
        self.expires_at.map(|expires| {
            let now = Utc::now();
            if expires > now {
                Duration::from_secs((expires - now).num_seconds() as u64)
            } else {
                Duration::ZERO
            }
        })
    }

    /// 刷新锁过期时间
    pub fn refresh(&mut self) -> Result<()> {
        if self.state != LockState::Locked {
            return Err(Error::Lock("锁未被持有，无法刷新".to_string()));
        }

        let now = Utc::now();
        self.expires_at = Some(now + chrono::Duration::seconds(self.timeout_secs as i64));
        Ok(())
    }
}

/// 锁类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LockType {
    /// 排他锁
    Exclusive,
    /// 共享锁
    Shared,
    /// 可重入锁
    Reentrant,
}

impl Default for LockType {
    fn default() -> Self {
        LockType::Exclusive
    }
}

/// 锁状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LockState {
    /// 未锁定
    Unlocked,
    /// 已锁定
    Locked,
    /// 已过期
    Expired,
}

impl std::fmt::Display for LockState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LockState::Unlocked => write!(f, "未锁定"),
            LockState::Locked => write!(f, "已锁定"),
            LockState::Expired => write!(f, "已过期"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_creation() {
        let lock = Lock::new("test_key", "owner1");
        assert_eq!(lock.key, "test_key");
        assert_eq!(lock.owner, "owner1");
        assert_eq!(lock.state, LockState::Unlocked);
    }

    #[test]
    fn test_lock_acquire() {
        let mut lock = Lock::new("test_key", "owner1");
        assert!(lock.try_acquire().unwrap());
        assert_eq!(lock.state, LockState::Locked);
        assert!(lock.acquired_at.is_some());
    }

    #[test]
    fn test_lock_release() {
        let mut lock = Lock::new("test_key", "owner1");
        lock.try_acquire().unwrap();
        assert!(lock.release().unwrap());
        assert_eq!(lock.state, LockState::Unlocked);
    }

    #[test]
    fn test_reentrant_lock() {
        let mut lock = Lock::reentrant("test_key", "owner1");

        // 第一次获取
        assert!(lock.try_acquire().unwrap());
        assert_eq!(lock.reentrant_count, 1);

        // 第二次获取（重入）
        assert!(lock.try_acquire().unwrap());
        assert_eq!(lock.reentrant_count, 2);

        // 第一次释放
        assert!(!lock.release().unwrap()); // 未完全释放
        assert_eq!(lock.reentrant_count, 1);

        // 第二次释放
        assert!(lock.release().unwrap()); // 完全释放
        assert_eq!(lock.state, LockState::Unlocked);
    }

    #[test]
    fn test_lock_expiry() {
        let mut lock = Lock::new("test_key", "owner1").with_timeout(1);
        lock.try_acquire().unwrap();

        assert!(!lock.is_expired());

        // 模拟过期
        lock.expires_at = Some(Utc::now() - chrono::Duration::seconds(1));
        assert!(lock.is_expired());
    }

    #[test]
    fn test_lock_refresh() {
        let mut lock = Lock::new("test_key", "owner1").with_timeout(10);
        lock.try_acquire().unwrap();

        let original_expires = lock.expires_at;
        lock.refresh().unwrap();

        assert!(lock.expires_at > original_expires);
    }

    #[test]
    fn test_shared_lock() {
        let lock = Lock::shared("test_key", "owner1");
        assert_eq!(lock.lock_type, LockType::Shared);
    }

    #[test]
    fn test_is_held_by() {
        let mut lock = Lock::new("test_key", "owner1");
        lock.try_acquire().unwrap();

        assert!(lock.is_held_by("owner1"));
        assert!(!lock.is_held_by("owner2"));
    }
}
