//! 版本锁定模块
//!
//! 提供技能版本的锁定和管理功能

use crate::definition::SkillVersion;
use crate::error::{Result, SkillsError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 版本锁ID
pub type LockId = String;

/// 锁定类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LockType {
    /// 精确版本锁定
    Exact,
    /// 范围版本锁定
    Range,
    /// 最低版本锁定
    Minimum,
    /// 兼容版本锁定
    Compatible,
}

/// 版本锁
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionLock {
    /// 锁ID
    pub id: LockId,
    /// 技能ID
    pub skill_id: String,
    /// 锁定类型
    pub lock_type: LockType,
    /// 锁定的版本
    pub locked_version: SkillVersion,
    /// 最小版本（范围锁定）
    pub min_version: Option<SkillVersion>,
    /// 最大版本（范围锁定）
    pub max_version: Option<SkillVersion>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 创建者
    pub created_by: String,
    /// 锁定原因
    pub reason: String,
    /// 是否强制
    pub forced: bool,
}

impl VersionLock {
    /// 创建精确版本锁
    pub fn exact(
        skill_id: String,
        version: SkillVersion,
        created_by: String,
        reason: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            skill_id,
            lock_type: LockType::Exact,
            locked_version: version.clone(),
            min_version: None,
            max_version: None,
            created_at: Utc::now(),
            created_by,
            reason,
            forced: false,
        }
    }

    /// 创建范围版本锁
    pub fn range(
        skill_id: String,
        min_version: SkillVersion,
        max_version: SkillVersion,
        created_by: String,
        reason: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            skill_id,
            lock_type: LockType::Range,
            locked_version: max_version.clone(),
            min_version: Some(min_version),
            max_version: Some(max_version),
            created_at: Utc::now(),
            created_by,
            reason,
            forced: false,
        }
    }

    /// 创建最低版本锁
    pub fn minimum(
        skill_id: String,
        min_version: SkillVersion,
        created_by: String,
        reason: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            skill_id,
            lock_type: LockType::Minimum,
            locked_version: min_version.clone(),
            min_version: Some(min_version),
            max_version: None,
            created_at: Utc::now(),
            created_by,
            reason,
            forced: false,
        }
    }

    /// 创建兼容版本锁
    pub fn compatible(
        skill_id: String,
        version: SkillVersion,
        created_by: String,
        reason: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            skill_id,
            lock_type: LockType::Compatible,
            locked_version: version.clone(),
            min_version: None,
            max_version: None,
            created_at: Utc::now(),
            created_by,
            reason,
            forced: false,
        }
    }

    /// 设置强制锁定
    pub fn with_forced(mut self) -> Self {
        self.forced = true;
        self
    }

    /// 检查版本是否满足锁定
    pub fn is_satisfied(&self, version: &SkillVersion) -> bool {
        match self.lock_type {
            LockType::Exact => version == &self.locked_version,
            LockType::Range => {
                let min_ok = self.min_version.as_ref().map(|min| version >= min).unwrap_or(true);
                let max_ok = self.max_version.as_ref().map(|max| version <= max).unwrap_or(true);
                min_ok && max_ok
            },
            LockType::Minimum => {
                self.min_version.as_ref().map(|min| version >= min).unwrap_or(true)
            },
            LockType::Compatible => version.is_compatible(&self.locked_version),
        }
    }
}

/// 版本管理器
pub struct VersionManager {
    /// 版本锁存储
    locks: Arc<RwLock<HashMap<String, Vec<VersionLock>>>>,
    /// 技能版本索引
    versions: Arc<RwLock<HashMap<String, SkillVersion>>>,
}

impl VersionManager {
    /// 创建新版本管理器
    pub fn new() -> Self {
        Self {
            locks: Arc::new(RwLock::new(HashMap::new())),
            versions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建版本锁
    pub async fn create_lock(&self, lock: VersionLock) -> Result<LockId> {
        let id = lock.id.clone();
        let skill_id = lock.skill_id.clone();

        let mut locks = self.locks.write().await;
        locks.entry(skill_id).or_insert_with(Vec::new).push(lock);

        Ok(id)
    }

    /// 获取技能的所有版本锁
    pub async fn get_locks(&self, skill_id: &str) -> Vec<VersionLock> {
        let locks = self.locks.read().await;
        locks.get(skill_id).cloned().unwrap_or_default()
    }

    /// 检查版本是否满足所有锁定
    pub async fn check_version(&self, skill_id: &str, version: &SkillVersion) -> Result<bool> {
        let locks = self.locks.read().await;
        if let Some(skill_locks) = locks.get(skill_id) {
            for lock in skill_locks {
                if !lock.is_satisfied(version) && lock.forced {
                    return Err(SkillsError::VersionConflict(format!(
                        "版本 {} 不满足强制锁定: {}",
                        version, lock.id
                    )));
                }
            }
        }
        Ok(true)
    }

    /// 注册技能版本
    pub async fn register_version(&self, skill_id: String, version: SkillVersion) -> Result<()> {
        // 检查是否满足锁定
        self.check_version(&skill_id, &version).await?;

        let mut versions = self.versions.write().await;
        versions.insert(skill_id, version);
        Ok(())
    }

    /// 获取技能版本
    pub async fn get_version(&self, skill_id: &str) -> Option<SkillVersion> {
        let versions = self.versions.read().await;
        versions.get(skill_id).cloned()
    }

    /// 移除版本锁
    pub async fn remove_lock(&self, lock_id: &str) -> Result<()> {
        let mut locks = self.locks.write().await;
        for (_, skill_locks) in locks.iter_mut() {
            if let Some(pos) = skill_locks.iter().position(|l| l.id == lock_id) {
                skill_locks.remove(pos);
                return Ok(());
            }
        }
        Err(SkillsError::NotFound(lock_id.to_string()))
    }

    /// 获取所有锁定的技能
    pub async fn get_locked_skills(&self) -> Vec<String> {
        let locks = self.locks.read().await;
        locks.keys().cloned().collect()
    }

    /// 清理过期的锁定（超过指定天数）
    pub async fn cleanup_expired(&self, days: u32) -> Result<usize> {
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);
        let mut locks = self.locks.write().await;
        let mut count = 0;

        for (_, skill_locks) in locks.iter_mut() {
            let before = skill_locks.len();
            skill_locks.retain(|l| l.created_at > cutoff || l.forced);
            count += before - skill_locks.len();
        }

        Ok(count)
    }
}

impl Default for VersionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_lock() {
        let lock = VersionLock::exact(
            "skill-1".to_string(),
            SkillVersion::new(1, 0, 0),
            "user".to_string(),
            "测试锁定".to_string(),
        );

        assert!(lock.is_satisfied(&SkillVersion::new(1, 0, 0)));
        assert!(!lock.is_satisfied(&SkillVersion::new(1, 1, 0)));
    }

    #[test]
    fn test_range_lock() {
        let lock = VersionLock::range(
            "skill-1".to_string(),
            SkillVersion::new(1, 0, 0),
            SkillVersion::new(2, 0, 0),
            "user".to_string(),
            "范围锁定".to_string(),
        );

        assert!(lock.is_satisfied(&SkillVersion::new(1, 5, 0)));
        assert!(lock.is_satisfied(&SkillVersion::new(1, 0, 0)));
        assert!(lock.is_satisfied(&SkillVersion::new(2, 0, 0)));
        assert!(!lock.is_satisfied(&SkillVersion::new(2, 1, 0)));
        assert!(!lock.is_satisfied(&SkillVersion::new(0, 9, 0)));
    }

    #[test]
    fn test_minimum_lock() {
        let lock = VersionLock::minimum(
            "skill-1".to_string(),
            SkillVersion::new(1, 5, 0),
            "user".to_string(),
            "最低版本锁定".to_string(),
        );

        assert!(lock.is_satisfied(&SkillVersion::new(1, 5, 0)));
        assert!(lock.is_satisfied(&SkillVersion::new(2, 0, 0)));
        assert!(!lock.is_satisfied(&SkillVersion::new(1, 4, 9)));
    }

    #[test]
    fn test_compatible_lock() {
        let lock = VersionLock::compatible(
            "skill-1".to_string(),
            SkillVersion::new(1, 2, 0),
            "user".to_string(),
            "兼容锁定".to_string(),
        );

        assert!(lock.is_satisfied(&SkillVersion::new(1, 2, 0)));
        assert!(lock.is_satisfied(&SkillVersion::new(1, 3, 0)));
        assert!(lock.is_satisfied(&SkillVersion::new(1, 2, 5)));
        assert!(!lock.is_satisfied(&SkillVersion::new(2, 0, 0)));
    }

    #[tokio::test]
    async fn test_version_manager() {
        let manager = VersionManager::new();

        let lock = VersionLock::exact(
            "skill-1".to_string(),
            SkillVersion::new(1, 0, 0),
            "user".to_string(),
            "测试锁定".to_string(),
        );

        manager.create_lock(lock).await.unwrap();

        let locks = manager.get_locks("skill-1").await;
        assert_eq!(locks.len(), 1);

        let ok = manager.check_version("skill-1", &SkillVersion::new(1, 0, 0)).await;
        assert!(ok.is_ok());
    }

    #[tokio::test]
    async fn test_register_version() {
        let manager = VersionManager::new();

        // 注册版本
        manager.register_version("skill-1".to_string(), SkillVersion::new(1, 0, 0)).await.unwrap();

        let version = manager.get_version("skill-1").await;
        assert!(version.is_some());
        assert_eq!(version.unwrap().major, 1);
    }

    #[tokio::test]
    async fn test_forced_lock() {
        let manager = VersionManager::new();

        let lock = VersionLock::exact(
            "skill-1".to_string(),
            SkillVersion::new(1, 0, 0),
            "user".to_string(),
            "强制锁定".to_string(),
        )
        .with_forced();

        manager.create_lock(lock).await.unwrap();

        // 强制锁定应该拒绝不满足的版本
        let result = manager.check_version("skill-1", &SkillVersion::new(2, 0, 0)).await;
        assert!(result.is_err());
    }
}
