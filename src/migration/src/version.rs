//! 迁移版本管理

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// 迁移方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MigrationDirection {
    /// 向上迁移
    Up,
    /// 向下迁移
    Down,
}

/// 迁移状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationStatus {
    /// 待执行
    Pending,
    /// 执行中
    Running,
    /// 已完成
    Completed,
    /// 已回滚
    RolledBack,
    /// 失败
    Failed,
}

impl fmt::Display for MigrationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Running => write!(f, "Running"),
            Self::Completed => write!(f, "Completed"),
            Self::RolledBack => write!(f, "RolledBack"),
            Self::Failed => write!(f, "Failed"),
        }
    }
}

/// 迁移版本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationVersion {
    /// 版本号（格式：YYYYMMDDHHMMSS）
    pub version: String,
    /// 版本描述
    pub description: String,
    /// 作者
    pub author: String,
    /// 迁移类型
    pub migration_type: MigrationType,
    /// 是否可回滚
    pub rollback: bool,
    /// 依赖的版本
    pub dependencies: Vec<String>,
}

impl fmt::Display for MigrationVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.version, self.description)
    }
}

impl MigrationVersion {
    /// 创建新迁移版本
    pub fn new(version: String, description: String, author: String) -> Self {
        Self {
            description,
            author,
            migration_type: MigrationType::Schema,
            rollback: true,
            dependencies: vec![],
            version,
        }
    }

    /// 添加依赖
    pub fn with_dependency(mut self, version: String) -> Self {
        self.dependencies.push(version);
        self
    }

    /// 设置回滚支持
    pub fn with_rollback(mut self, rollback: bool) -> Self {
        self.rollback = rollback;
        self
    }

    /// 设置迁移类型
    pub fn with_type(mut self, migration_type: MigrationType) -> Self {
        self.migration_type = migration_type;
        self
    }

    /// 比较版本号
    pub fn compare(&self, other: &Self) -> std::cmp::Ordering {
        self.version.cmp(&other.version)
    }
}

impl PartialEq for MigrationVersion {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
    }
}

impl Eq for MigrationVersion {}

impl PartialOrd for MigrationVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.compare(other))
    }
}

impl Ord for MigrationVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.compare(other)
    }
}

/// 迁移类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MigrationType {
    /// 模式迁移
    Schema,
    /// 数据迁移
    Data,
    /// 配置迁移
    Config,
}

/// 迁移记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRecord {
    /// 记录ID
    pub id: String,
    /// 版本号
    pub version: String,
    /// 迁移方向
    pub direction: MigrationDirection,
    /// 状态
    pub status: MigrationStatus,
    /// 开始时间
    pub started_at: DateTime<Utc>,
    /// 完成时间
    pub completed_at: Option<DateTime<Utc>>,
    /// 错误信息
    pub error_message: Option<String>,
    /// 执行时长（毫秒）
    pub duration_ms: Option<u64>,
}

impl MigrationRecord {
    /// 创建新记录
    pub fn new(version: String, direction: MigrationDirection) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            version,
            direction,
            status: MigrationStatus::Pending,
            started_at: Utc::now(),
            completed_at: None,
            error_message: None,
            duration_ms: None,
        }
    }

    /// 标记为运行中
    pub fn mark_running(&mut self) {
        self.status = MigrationStatus::Running;
        self.started_at = Utc::now();
    }

    /// 标记为完成
    pub fn mark_completed(&mut self) {
        self.status = MigrationStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.duration_ms =
            Some((Utc::now().timestamp_millis() - self.started_at.timestamp_millis()).abs() as u64);
    }

    /// 标记为已回滚
    pub fn mark_rolled_back(&mut self) {
        self.status = MigrationStatus::RolledBack;
        self.completed_at = Some(Utc::now());
        self.duration_ms =
            Some((Utc::now().timestamp_millis() - self.started_at.timestamp_millis()).abs() as u64);
    }

    /// 标记为失败
    pub fn mark_failed(&mut self, error: String) {
        self.status = MigrationStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.error_message = Some(error);
        self.duration_ms =
            Some((Utc::now().timestamp_millis() - self.started_at.timestamp_millis()).abs() as u64);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_version_creation() {
        let version = MigrationVersion::new(
            "20240313120000".to_string(),
            "Initial migration".to_string(),
            "FOS Team".to_string(),
        );

        assert_eq!(version.version, "20240313120000");
        assert_eq!(version.description, "Initial migration");
        assert_eq!(version.author, "FOS Team");
    }

    #[test]
    fn test_migration_version_builder() {
        let version = MigrationVersion::new(
            "20240313120000".to_string(),
            "Add users table".to_string(),
            "FOS Team".to_string(),
        )
        .with_dependency("20240313000000".to_string())
        .with_rollback(true)
        .with_type(MigrationType::Schema);

        assert_eq!(version.dependencies.len(), 1);
        assert_eq!(version.dependencies[0], "20240313000000");
        assert!(version.rollback);
        assert_eq!(version.migration_type, MigrationType::Schema);
    }

    #[test]
    fn test_migration_version_comparison() {
        let v1 = MigrationVersion::new(
            "20240313000000".to_string(),
            "Test1".to_string(),
            "FOS".to_string(),
        );
        let v2 = MigrationVersion::new(
            "20240314000000".to_string(),
            "Test2".to_string(),
            "FOS".to_string(),
        );

        assert!(v1 < v2);
        assert!(v2 > v1);
    }

    #[test]
    fn test_migration_record_creation() {
        let record = MigrationRecord::new("20240313120000".to_string(), MigrationDirection::Up);

        assert_eq!(record.status, MigrationStatus::Pending);
        assert_eq!(record.version, "20240313120000");
        assert_eq!(record.direction, MigrationDirection::Up);
    }

    #[test]
    fn test_migration_record_transitions() {
        let mut record = MigrationRecord::new("20240313120000".to_string(), MigrationDirection::Up);

        // Running
        record.mark_running();
        assert_eq!(record.status, MigrationStatus::Running);

        // Completed
        record.mark_completed();
        assert_eq!(record.status, MigrationStatus::Completed);
        assert!(record.completed_at.is_some());
        assert!(record.duration_ms.is_some());
    }

    #[test]
    fn test_migration_record_failure() {
        let mut record = MigrationRecord::new("20240313120000".to_string(), MigrationDirection::Up);

        record.mark_running();
        record.mark_failed("Test error".to_string());

        assert_eq!(record.status, MigrationStatus::Failed);
        assert_eq!(record.error_message, Some("Test error".to_string()));
    }

    #[test]
    fn test_migration_record_rollback() {
        let mut record =
            MigrationRecord::new("20240313120000".to_string(), MigrationDirection::Down);

        record.mark_running();
        record.mark_completed();
        record.mark_rolled_back();

        assert_eq!(record.status, MigrationStatus::RolledBack);
    }

    #[test]
    fn test_migration_status_display() {
        assert_eq!(MigrationStatus::Pending.to_string(), "Pending");
        assert_eq!(MigrationStatus::Running.to_string(), "Running");
        assert_eq!(MigrationStatus::Completed.to_string(), "Completed");
        assert_eq!(MigrationStatus::RolledBack.to_string(), "RolledBack");
        assert_eq!(MigrationStatus::Failed.to_string(), "Failed");
    }
}
