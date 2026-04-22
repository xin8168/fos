//! Audit 查询模块
//!
//! 提供审计日志的查询功能

use crate::error::{AuditError, Result};
use crate::{AuditLog, AuditLogStatus, AuditLogType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 审计查询条件
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditQueryParams {
    /// 日志类型过滤
    pub log_type: Option<AuditLogType>,

    /// 状态过滤
    pub status: Option<AuditLogStatus>,

    /// 开始时间
    pub start_time: Option<DateTime<Utc>>,

    /// 结束时间
    pub end_time: Option<DateTime<Utc>>,

    /// 关键词搜索（在命令或原因中）
    pub keyword: Option<String>,

    /// 分页偏移
    pub offset: Option<usize>,

    /// 分页限制
    pub limit: Option<usize>,
}

/// 审计查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQueryResult {
    /// 日志列表
    pub logs: Vec<AuditLog>,

    /// 总数
    pub total: usize,

    /// 是否有更多
    pub has_more: bool,
}

/// 审计查询器
pub struct AuditQuery {
    logs: Arc<RwLock<HashMap<String, AuditLog>>>,
}

impl AuditQuery {
    /// 创建新的查询器
    pub fn new(logs: Arc<RwLock<HashMap<String, AuditLog>>>) -> Self {
        Self { logs }
    }

    /// 根据ID获取日志
    pub async fn get(&self, id: &str) -> Result<AuditLog> {
        let logs = self.logs.read().await;
        logs.get(id).cloned().ok_or_else(|| AuditError::NotFound(id.to_string()))
    }

    /// 查询日志
    pub async fn query(&self, params: AuditQueryParams) -> Result<AuditQueryResult> {
        let logs = self.logs.read().await;

        // 过滤
        let mut filtered: Vec<AuditLog> = logs
            .values()
            .filter(|log| {
                // 类型过滤
                if let Some(ref log_type) = params.log_type {
                    if log.log_type != *log_type {
                        return false;
                    }
                }

                // 状态过滤
                if let Some(ref status) = params.status {
                    if log.status != *status {
                        return false;
                    }
                }

                // 时间范围
                if let Some(start_time) = params.start_time {
                    if log.created_at < start_time {
                        return false;
                    }
                }

                if let Some(end_time) = params.end_time {
                    if log.created_at > end_time {
                        return false;
                    }
                }

                // 关键词搜索
                if let Some(ref keyword) = params.keyword {
                    if !log.original_command.contains(keyword) && !log.reason.contains(keyword) {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        // 按时间降序排序
        filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let total = filtered.len();

        // 分页
        if let Some(offset) = params.offset {
            filtered = filtered.into_iter().skip(offset).collect();
        }

        if let Some(limit) = params.limit {
            filtered = filtered.into_iter().take(limit).collect();
        }

        Ok(AuditQueryResult {
            logs: filtered,
            total,
            has_more: params.limit.map(|l| total > l + params.offset.unwrap_or(0)).unwrap_or(false),
        })
    }

    /// 按类型查询
    pub async fn find_by_type(&self, log_type: AuditLogType) -> Result<Vec<AuditLog>> {
        let params = AuditQueryParams { log_type: Some(log_type), ..Default::default() };
        Ok(self.query(params).await?.logs)
    }

    /// 按状态查询
    pub async fn find_by_status(&self, status: AuditLogStatus) -> Result<Vec<AuditLog>> {
        let params = AuditQueryParams { status: Some(status), ..Default::default() };
        Ok(self.query(params).await?.logs)
    }

    /// 搜索关键词
    pub async fn search(&self, keyword: &str) -> Result<Vec<AuditLog>> {
        let params = AuditQueryParams { keyword: Some(keyword.to_string()), ..Default::default() };
        Ok(self.query(params).await?.logs)
    }

    /// 获取最近的日志
    pub async fn find_recent(&self, limit: usize) -> Result<Vec<AuditLog>> {
        let params = AuditQueryParams { limit: Some(limit), ..Default::default() };
        Ok(self.query(params).await?.logs)
    }

    /// 获取时间范围内的日志
    pub async fn find_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AuditLog>> {
        let params =
            AuditQueryParams { start_time: Some(start), end_time: Some(end), ..Default::default() };
        Ok(self.query(params).await?.logs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AuditLogger;

    #[tokio::test]
    async fn test_query_by_type() {
        let logger = AuditLogger::new();
        logger.log_format_blocked("test1".to_string(), "r1".to_string()).await.unwrap();
        logger.log_rule_blocked("test2".to_string(), "r2".to_string()).await.unwrap();

        let query = AuditQuery::new(logger.storage());
        let format_blocked = query.find_by_type(AuditLogType::FormatBlocked).await.unwrap();
        assert_eq!(format_blocked.len(), 1);
    }

    #[tokio::test]
    async fn test_query_by_status() {
        let logger = AuditLogger::new();
        let id = logger.log_format_blocked("test".to_string(), "r".to_string()).await.unwrap();
        logger.mark_analyzed(&id).await.unwrap();
        logger.log_rule_blocked("test2".to_string(), "r".to_string()).await.unwrap();

        let query = AuditQuery::new(logger.storage());
        let analyzed = query.find_by_status(AuditLogStatus::Analyzed).await.unwrap();
        assert_eq!(analyzed.len(), 1);
    }

    #[tokio::test]
    async fn test_search() {
        let logger = AuditLogger::new();
        logger
            .log_format_blocked("important command".to_string(), "格式错误".to_string())
            .await
            .unwrap();
        logger.log_rule_blocked("other command".to_string(), "规则拦截".to_string()).await.unwrap();

        let query = AuditQuery::new(logger.storage());
        let results = query.search("important").await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_find_recent() {
        let logger = AuditLogger::new();
        for i in 0..10 {
            logger.log_format_blocked(format!("cmd{}", i), format!("r{}", i)).await.unwrap();
        }

        let query = AuditQuery::new(logger.storage());
        let recent = query.find_recent(3).await.unwrap();
        assert_eq!(recent.len(), 3);
    }

    #[tokio::test]
    async fn test_complex_query() {
        let logger = AuditLogger::new();
        logger.log_format_blocked("test".to_string(), "reason".to_string()).await.unwrap();
        logger.log_rule_blocked("test".to_string(), "other".to_string()).await.unwrap();
        logger.log_execution_failed("test".to_string(), "reason".to_string()).await.unwrap();

        let query = AuditQuery::new(logger.storage());
        let params = AuditQueryParams {
            log_type: Some(AuditLogType::FormatBlocked),
            keyword: Some("reason".to_string()),
            ..Default::default()
        };

        let result = query.query(params).await.unwrap();
        assert_eq!(result.logs.len(), 1);
    }

    #[tokio::test]
    async fn test_pagination() {
        let logger = AuditLogger::new();
        for i in 0..20 {
            logger.log_format_blocked(format!("cmd{}", i), format!("r{}", i)).await.unwrap();
        }

        let query = AuditQuery::new(logger.storage());
        let params = AuditQueryParams { offset: Some(5), limit: Some(10), ..Default::default() };

        let result = query.query(params).await.unwrap();
        assert_eq!(result.logs.len(), 10);
        assert_eq!(result.total, 20);
    }
}
