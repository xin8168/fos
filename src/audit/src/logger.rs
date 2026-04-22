//! Audit 日志器
//!
//! 负责审计日志的存储和管理

use crate::error::{AuditError, Result};
use crate::{AuditConfig, AuditLog, AuditLogStatus, AuditLogType};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 审计日志存储
pub struct AuditLogger {
    /// 日志存储
    logs: Arc<RwLock<HashMap<String, AuditLog>>>,
    /// 配置
    config: AuditConfig,
}

impl AuditLogger {
    /// 创建新的审计日志器
    pub fn new() -> Self {
        Self::with_config(AuditConfig::default())
    }

    /// 使用配置创建
    pub fn with_config(config: AuditConfig) -> Self {
        Self { logs: Arc::new(RwLock::new(HashMap::new())), config }
    }

    /// 记录格式拦截日志
    pub async fn log_format_blocked(&self, command: String, reason: String) -> Result<String> {
        self.log(AuditLogType::FormatBlocked, command, reason).await
    }

    /// 记录规则拦截日志
    pub async fn log_rule_blocked(&self, command: String, reason: String) -> Result<String> {
        self.log(AuditLogType::RuleBlocked, command, reason).await
    }

    /// 记录执行失败日志
    pub async fn log_execution_failed(&self, command: String, reason: String) -> Result<String> {
        self.log(AuditLogType::ExecutionFailed, command, reason).await
    }

    /// 记录系统异常日志
    pub async fn log_system_error(&self, command: String, reason: String) -> Result<String> {
        self.log(AuditLogType::SystemError, command, reason).await
    }

    /// 通用日志记录
    pub async fn log(
        &self,
        log_type: AuditLogType,
        command: String,
        reason: String,
    ) -> Result<String> {
        // 检查容量限制
        {
            let logs = self.logs.read().await;
            if logs.len() >= self.config.max_entries {
                return Err(AuditError::StorageFailed("日志存储已达上限".to_string()));
            }
        }

        let log = AuditLog::new(log_type, command, reason);
        let id = log.id.clone();
        let mut logs = self.logs.write().await;
        logs.insert(id.clone(), log);
        Ok(id)
    }

    /// 获取日志
    pub async fn get(&self, id: &str) -> Result<AuditLog> {
        let logs = self.logs.read().await;
        logs.get(id).cloned().ok_or_else(|| AuditError::NotFound(id.to_string()))
    }

    /// 更新日志状态
    pub async fn update_status(&self, id: &str, status: AuditLogStatus) -> Result<()> {
        let mut logs = self.logs.write().await;
        if let Some(log) = logs.get_mut(id) {
            log.status = status;
            Ok(())
        } else {
            Err(AuditError::NotFound(id.to_string()))
        }
    }

    /// 标记为已分析
    pub async fn mark_analyzed(&self, id: &str) -> Result<()> {
        self.update_status(id, AuditLogStatus::Analyzed).await
    }

    /// 标记为已归档
    pub async fn mark_archived(&self, id: &str) -> Result<()> {
        self.update_status(id, AuditLogStatus::Archived).await
    }

    /// 删除日志
    pub async fn delete(&self, id: &str) -> Result<()> {
        let mut logs = self.logs.write().await;
        logs.remove(id).ok_or_else(|| AuditError::NotFound(id.to_string()))?;
        Ok(())
    }

    /// 统计日志数量
    pub async fn count(&self) -> usize {
        let logs = self.logs.read().await;
        logs.len()
    }

    /// 按类型统计
    pub async fn count_by_type(&self, log_type: AuditLogType) -> usize {
        let logs = self.logs.read().await;
        logs.values().filter(|l| l.log_type == log_type).count()
    }

    /// 按状态统计
    pub async fn count_by_status(&self, status: AuditLogStatus) -> usize {
        let logs = self.logs.read().await;
        logs.values().filter(|l| l.status == status).count()
    }

    /// 清理过期日志
    pub async fn cleanup_expired(&self) -> Result<usize> {
        let cutoff = Utc::now() - chrono::Duration::days(self.config.retention_days as i64);
        let mut logs = self.logs.write().await;
        let initial_count = logs.len();

        logs.retain(|_, log| log.created_at > cutoff);

        Ok(initial_count - logs.len())
    }

    /// 清空所有日志
    pub async fn clear(&self) -> Result<()> {
        let mut logs = self.logs.write().await;
        logs.clear();
        Ok(())
    }

    /// 获取存储引用（用于查询）
    pub fn storage(&self) -> Arc<RwLock<HashMap<String, AuditLog>>> {
        self.logs.clone()
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// 审计日志条目（带元数据扩展）
impl AuditLog {
    /// 添加元数据
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// 添加单个元数据字段
    pub fn add_metadata(&mut self, key: &str, value: serde_json::Value) {
        if let serde_json::Value::Object(ref mut map) = self.metadata {
            map.insert(key.to_string(), value);
        }
    }
}

/// 审计日志统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    /// 总数
    pub total: usize,
    /// 格式拦截数
    pub format_blocked: usize,
    /// 规则拦截数
    pub rule_blocked: usize,
    /// 执行失败数
    pub execution_failed: usize,
    /// 系统异常数
    pub system_error: usize,
    /// 已分析数
    pub analyzed: usize,
    /// 已归档数
    pub archived: usize,
}

impl AuditLogger {
    /// 获取统计信息
    pub async fn stats(&self) -> AuditStats {
        let logs = self.logs.read().await;

        AuditStats {
            total: logs.len(),
            format_blocked: logs
                .values()
                .filter(|l| l.log_type == AuditLogType::FormatBlocked)
                .count(),
            rule_blocked: logs.values().filter(|l| l.log_type == AuditLogType::RuleBlocked).count(),
            execution_failed: logs
                .values()
                .filter(|l| l.log_type == AuditLogType::ExecutionFailed)
                .count(),
            system_error: logs.values().filter(|l| l.log_type == AuditLogType::SystemError).count(),
            analyzed: logs.values().filter(|l| l.status == AuditLogStatus::Analyzed).count(),
            archived: logs.values().filter(|l| l.status == AuditLogStatus::Archived).count(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_log_format_blocked() {
        let logger = AuditLogger::new();
        let id =
            logger.log_format_blocked("test command".to_string(), "格式错误".to_string()).await;

        assert!(id.is_ok());
        let log = logger.get(&id.unwrap()).await.unwrap();
        assert_eq!(log.log_type, AuditLogType::FormatBlocked);
    }

    #[tokio::test]
    async fn test_log_rule_blocked() {
        let logger = AuditLogger::new();
        let id = logger.log_rule_blocked("test".to_string(), "规则拦截".to_string()).await;

        assert!(id.is_ok());
        let log = logger.get(&id.unwrap()).await.unwrap();
        assert_eq!(log.log_type, AuditLogType::RuleBlocked);
    }

    #[tokio::test]
    async fn test_log_execution_failed() {
        let logger = AuditLogger::new();
        let id = logger.log_execution_failed("test".to_string(), "执行失败".to_string()).await;

        assert!(id.is_ok());
        let log = logger.get(&id.unwrap()).await.unwrap();
        assert_eq!(log.log_type, AuditLogType::ExecutionFailed);
    }

    #[tokio::test]
    async fn test_log_system_error() {
        let logger = AuditLogger::new();
        let id = logger.log_system_error("test".to_string(), "系统错误".to_string()).await;

        assert!(id.is_ok());
        let log = logger.get(&id.unwrap()).await.unwrap();
        assert_eq!(log.log_type, AuditLogType::SystemError);
    }

    #[tokio::test]
    async fn test_mark_analyzed() {
        let logger = AuditLogger::new();
        let id = logger.log_format_blocked("test".to_string(), "reason".to_string()).await.unwrap();

        logger.mark_analyzed(&id).await.unwrap();
        let log = logger.get(&id).await.unwrap();
        assert_eq!(log.status, AuditLogStatus::Analyzed);
    }

    #[tokio::test]
    async fn test_mark_archived() {
        let logger = AuditLogger::new();
        let id = logger.log_format_blocked("test".to_string(), "reason".to_string()).await.unwrap();

        logger.mark_archived(&id).await.unwrap();
        let log = logger.get(&id).await.unwrap();
        assert_eq!(log.status, AuditLogStatus::Archived);
    }

    #[tokio::test]
    async fn test_delete() {
        let logger = AuditLogger::new();
        let id = logger.log_format_blocked("test".to_string(), "reason".to_string()).await.unwrap();

        logger.delete(&id).await.unwrap();
        let result = logger.get(&id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_count_by_type() {
        let logger = AuditLogger::new();
        logger.log_format_blocked("test".to_string(), "r".to_string()).await.unwrap();
        logger.log_format_blocked("test".to_string(), "r".to_string()).await.unwrap();
        logger.log_rule_blocked("test".to_string(), "r".to_string()).await.unwrap();

        assert_eq!(logger.count_by_type(AuditLogType::FormatBlocked).await, 2);
        assert_eq!(logger.count_by_type(AuditLogType::RuleBlocked).await, 1);
    }

    #[tokio::test]
    async fn test_stats() {
        let logger = AuditLogger::new();
        logger.log_format_blocked("test".to_string(), "r".to_string()).await.unwrap();
        logger.log_rule_blocked("test".to_string(), "r".to_string()).await.unwrap();
        let id = logger.log_execution_failed("test".to_string(), "r".to_string()).await.unwrap();
        logger.mark_analyzed(&id).await.unwrap();

        let stats = logger.stats().await;
        assert_eq!(stats.total, 3);
        assert_eq!(stats.format_blocked, 1);
        assert_eq!(stats.rule_blocked, 1);
        assert_eq!(stats.execution_failed, 1);
        assert_eq!(stats.analyzed, 1);
    }

    #[tokio::test]
    async fn test_clear() {
        let logger = AuditLogger::new();
        logger.log_format_blocked("test".to_string(), "r".to_string()).await.unwrap();
        logger.log_rule_blocked("test".to_string(), "r".to_string()).await.unwrap();

        logger.clear().await.unwrap();
        assert_eq!(logger.count().await, 0);
    }

    #[tokio::test]
    async fn test_max_entries_limit() {
        let config = AuditConfig { retention_days: 365, max_entries: 2 };
        let logger = AuditLogger::with_config(config);

        // 存储两条
        logger.log_format_blocked("test1".to_string(), "r".to_string()).await.unwrap();
        logger.log_format_blocked("test2".to_string(), "r".to_string()).await.unwrap();

        // 第三条应该失败
        let result = logger.log_format_blocked("test3".to_string(), "r".to_string()).await;
        assert!(result.is_err());
    }
}
