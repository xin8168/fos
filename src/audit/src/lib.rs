//! # FOS Audit - 拦截日志库模块
//!
//! FOS 拦截日志库，负责存储失败和被拦截的事件
//!
//! ## 核心职责
//! - 记录所有拦截事件
//! - 记录所有失败事件
//! - 提供审计查询
//! - 生成审计报告
//!
//! ## 安全铁律
//! - 不存储成功事件
//! - 不修改记录内容
//! - 不做规则判断

pub mod config;
pub mod error;
pub mod logger;
pub mod query;
pub mod report;

pub use config::AuditConfig;
pub use error::{AuditError, Result};
pub use logger::{AuditLogger, AuditStats};
pub use query::{AuditQuery, AuditQueryParams, AuditQueryResult};
pub use report::AuditReport;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 模块版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 模块名称
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// 审计日志类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuditLogType {
    /// 格式拦截
    FormatBlocked,
    /// 规则拦截
    RuleBlocked,
    /// 执行失败
    ExecutionFailed,
    /// 系统异常
    SystemError,
}

/// 审计日志状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuditLogStatus {
    /// 已记录
    Recorded,
    /// 已分析
    Analyzed,
    /// 已归档
    Archived,
}

/// 审计日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    /// 日志ID
    pub id: String,
    /// 日志类型
    pub log_type: AuditLogType,
    /// 日志状态
    pub status: AuditLogStatus,
    /// 原始指令
    pub original_command: String,
    /// 拦截原因
    pub reason: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 元数据
    pub metadata: serde_json::Value,
}

impl AuditLog {
    /// 创建新的审计日志
    pub fn new(log_type: AuditLogType, original_command: String, reason: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            log_type,
            status: AuditLogStatus::Recorded,
            original_command,
            reason,
            created_at: Utc::now(),
            metadata: serde_json::json!({}),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_creation() {
        let log = AuditLog::new(
            AuditLogType::FormatBlocked,
            "test command".to_string(),
            "格式错误".to_string(),
        );

        assert!(!log.id.is_empty());
        assert_eq!(log.log_type, AuditLogType::FormatBlocked);
        assert_eq!(log.status, AuditLogStatus::Recorded);
    }
}
