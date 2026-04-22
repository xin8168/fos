//! 备份数据结构

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// 备份类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackupType {
    /// 全量备份
    Full,
    /// 增量备份
    Incremental,
    /// 差异备份
    Differential,
}

/// 备份状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackupStatus {
    /// 创建中
    Creating,
    /// 创建成功
    Created,
    /// 验证中
    Verifying,
    /// 备份成功
    Completed,
    /// 验证失败
    Failed,
    /// 已过期
    Expired,
}

impl std::fmt::Display for BackupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Creating => write!(f, "Creating"),
            Self::Created => write!(f, "Created"),
            Self::Verifying => write!(f, "Verifying"),
            Self::Completed => write!(f, "Completed"),
            Self::Failed => write!(f, "Failed"),
            Self::Expired => write!(f, "Expired"),
        }
    }
}

/// 备份项元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupItem {
    /// 唯一ID
    pub id: String,
    /// 备份类型
    pub backup_type: BackupType,
    /// 备份路径
    pub path: PathBuf,
    //    /// 备份大小（字节）
    pub size: u64,
    /// 文件数量
    pub file_count: usize,
    /// 备份包含的设备列表
    pub devices: Vec<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 完成时间
    pub completed_at: Option<DateTime<Utc>>,
    /// 状态
    pub status: BackupStatus,
    /// 错误信息
    pub error_message: Option<String>,
    /// 校验和
    pub checksum: Option<String>,
    /// 元数据
    pub metadata: BackupMetadata,
}

/// 备份元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    /// 备份版本
    pub version: String,
    /// 备份描述
    pub description: String,
    /// 备份目标标识
    pub target_id: String,
    /// 备份标签
    pub tags: Vec<String>,
    /// 环境信息
    pub environment: BackupEnvironment,
}

/// 备份环境信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEnvironment {
    /// 主机名
    pub hostname: String,
    /// 操作系统
    pub os: String,
    /// 框架版本
    pub framework_version: String,
    /// 备份工具版本
    pub tool_version: String,
}

/// 备份计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupPlan {
    /// 计划ID
    pub id: String,
    /// 计划名称
    pub name: String,
    /// 备份类型
    pub backup_type: BackupType,
    /// 备份目标
    pub targets: Vec<BackupTarget>,
    /// 调度策略
    pub schedule: BackupSchedule,
    /// 保留策略
    pub retention: BackupRetention,
    /// 是否启用
    pub enabled: bool,
}

/// 备份目标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupTarget {
    /// 目标ID
    pub id: String,
    /// 目标类型：device, database, config, logs
    pub target_type: BackupTargetType,
    /// 目标路径
    pub path: PathBuf,
    /// 包含模式
    pub include_patterns: Vec<String>,
    /// 排除模式
    pub exclude_patterns: Vec<String>,
}

/// 备份目标类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackupTargetType {
    /// 设备数据
    Device,
    /// 数据库
    Database,
    /// 配置文件
    Config,
    /// 日志文件
    Logs,
}

/// 备份调度策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupSchedule {
    /// 立即执行（一次性）
    Once { at: DateTime<Utc> },
    /// 定时执行（Cron表达式）
    Cron(String),
    /// 固定间隔
    Interval { seconds: u64 },
    /// 每天
    Daily { hour: u32, minute: u32, second: u32 },
    /// 每周
    Weekly { day_of_week: u32, hour: u32, minute: u32, second: u32 },
    /// 每月
    Monthly { day: u32, hour: u32, minute: u32, second: u32 },
}

/// 备份保留策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRetention {
    /// 最大备份数
    pub max_count: usize,
    /// 保留天数
    pub max_age_days: u32,
    /// 最大存储大小（字节）
    pub max_size_bytes: u64,
    /// 保留模式
    pub mode: RetentionMode,
}

impl Default for BackupRetention {
    fn default() -> Self {
        Self {
            max_count: 7,
            max_age_days: 30,
            max_size_bytes: 10 * 1024 * 1024 * 1024, // 10GB
            mode: RetentionMode::KeepLatest,
        }
    }
}

/// 保留模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetentionMode {
    /// 保留最新的N个
    KeepLatest,
    /// 每天保留
    Daily,
    /// 每周保留
    Weekly,
    /// 每月保留
    Monthly,
}

impl BackupItem {
    /// 创建新备份项
    pub fn new(
        backup_type: BackupType,
        path: PathBuf,
        targets: Vec<String>,
        description: String,
        _author: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            backup_type,
            path,
            size: 0,
            file_count: 0,
            devices: targets,
            created_at: Utc::now(),
            completed_at: None,
            status: BackupStatus::Creating,
            error_message: None,
            checksum: None,
            metadata: BackupMetadata {
                version: "1.0.0".to_string(),
                description,
                target_id: uuid::Uuid::new_v4().to_string(),
                tags: vec![],
                environment: BackupEnvironment {
                    hostname: gethostname::gethostname().to_str().unwrap_or("unknown").to_string(),
                    os: std::env::consts::OS.to_string(),
                    framework_version: env!("CARGO_PKG_VERSION").to_string(),
                    tool_version: env!("CARGO_PKG_VERSION").to_string(),
                },
            },
        }
    }

    /// 标记为创建成功
    pub fn mark_created(&mut self) {
        self.status = BackupStatus::Created;
    }

    /// 标记为验证中
    pub fn mark_verifying(&mut self) {
        self.status = BackupStatus::Verifying;
    }

    /// 标记为完成
    pub fn mark_completed(&mut self, checksum: String) {
        self.status = BackupStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.checksum = Some(checksum);
    }

    /// 标记为失败
    pub fn mark_failed(&mut self, error: String) {
        self.status = BackupStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.error_message = Some(error);
    }

    /// 标记为已过期
    pub fn mark_expired(&mut self) {
        self.status = BackupStatus::Expired;
    }

    /// 更新大小和文件数
    pub fn update_stats(&mut self, size: u64, file_count: usize) {
        self.size = size;
        self.file_count = file_count;
    }

    /// 检查是否已过期
    pub fn is_expired(&self, max_age_days: u32) -> bool {
        let age_days = Utc::now().signed_duration_since(self.created_at).num_days();
        age_days > max_age_days as i64
    }

    /// 标记为已验证（快捷方法：验证中 -> 完成并设置校验和）
    pub fn mark_verified(&mut self, checksum: String) {
        self.mark_verifying();
        self.mark_completed(checksum);
    }

    /// 获取校验和
    pub fn get_checksum(&self) -> Result<&str> {
        self.checksum
            .as_deref()
            .ok_or_else(|| Error::Internal("备份未完成验证，无校验和".to_string()))
    }

    /// 获取备份路径
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 转换为 JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| Error::Internal(format!("序列化失败: {}", e)))
    }

    /// 从 JSON 创建
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| Error::Internal(format!("反序列化失败: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_item_creation() {
        let item = BackupItem::new(
            BackupType::Full,
            PathBuf::from("/backup/test.zip"),
            vec!["device-001".to_string()],
            "Test backup".to_string(),
            "Dev".to_string(),
        );

        assert_eq!(item.status, BackupStatus::Creating);
        assert_eq!(item.devices.len(), 1);
    }

    #[test]
    fn test_backup_status_transitions() {
        let mut item = BackupItem::new(
            BackupType::Incremental,
            PathBuf::from("/backup/incremental.zip"),
            vec![],
            "Test".to_string(),
            "Dev".to_string(),
        );

        assert!(matches!(item.status, BackupStatus::Creating));

        item.mark_created();
        assert!(matches!(item.status, BackupStatus::Created));

        item.mark_verifying();
        assert!(matches!(item.status, BackupStatus::Verifying));

        item.mark_completed("abc123".to_string());
        assert_eq!(item.status, BackupStatus::Completed);
        assert_eq!(item.checksum, Some("abc123".to_string()));
    }

    #[test]
    fn test_backup_expiration() {
        let mut item = BackupItem::new(
            BackupType::Full,
            PathBuf::from("/backup/test.zip"),
            vec![],
            "Test".to_string(),
            "Dev".to_string(),
        );

        // 新创建的不算过期
        assert!(!item.is_expired(30));

        // 标记为过去时间会过期
        item.created_at = Utc::now() - chrono::Duration::days(35);
        assert!(item.is_expired(30));
    }
}
