//! FOS Audit 集成测试
//!
//! 测试 Audit 模块与其他模块的集成

use fos_audit::{
    AuditLogStatus, AuditLogType, AuditLogger, AuditQuery, AuditQueryParams, AuditStats,
};

/// 测试日志记录与查询集成
#[tokio::test]
async fn test_log_and_query_integration() {
    let logger = AuditLogger::new();
    let query = AuditQuery::new(logger.storage());

    // 记录不同类型的日志
    logger.log_format_blocked("cmd1".to_string(), "格式错误".to_string()).await.unwrap();
    logger.log_rule_blocked("cmd2".to_string(), "规则拦截".to_string()).await.unwrap();
    logger.log_execution_failed("cmd3".to_string(), "执行失败".to_string()).await.unwrap();

    // 查询所有日志
    let params = AuditQueryParams::default();
    let result = query.query(params).await.unwrap();

    assert_eq!(result.logs.len(), 3);
    assert_eq!(result.total, 3);
}

/// 测试状态管理集成
#[tokio::test]
async fn test_status_management() {
    let logger = AuditLogger::new();
    let query = AuditQuery::new(logger.storage());

    // 记录日志
    let id = logger.log_format_blocked("test".to_string(), "reason".to_string()).await.unwrap();

    // 验证初始状态
    let log = query.get(&id).await.unwrap();
    assert_eq!(log.status, AuditLogStatus::Recorded);

    // 标记为已分析
    logger.mark_analyzed(&id).await.unwrap();
    let log = query.get(&id).await.unwrap();
    assert_eq!(log.status, AuditLogStatus::Analyzed);

    // 标记为已归档
    logger.mark_archived(&id).await.unwrap();
    let log = query.get(&id).await.unwrap();
    assert_eq!(log.status, AuditLogStatus::Archived);
}

/// 测试统计功能集成
#[tokio::test]
async fn test_stats_integration() {
    let logger = AuditLogger::new();

    // 记录多种类型日志
    logger.log_format_blocked("cmd".to_string(), "r".to_string()).await.unwrap();
    logger.log_format_blocked("cmd".to_string(), "r".to_string()).await.unwrap();
    logger.log_rule_blocked("cmd".to_string(), "r".to_string()).await.unwrap();
    let id = logger.log_execution_failed("cmd".to_string(), "r".to_string()).await.unwrap();
    logger.mark_analyzed(&id).await.unwrap();

    // 获取统计
    let stats: AuditStats = logger.stats().await;

    assert_eq!(stats.total, 4);
    assert_eq!(stats.format_blocked, 2);
    assert_eq!(stats.rule_blocked, 1);
    assert_eq!(stats.execution_failed, 1);
    assert_eq!(stats.analyzed, 1);
}

/// 测试复杂查询集成
#[tokio::test]
async fn test_complex_query_integration() {
    let logger = AuditLogger::new();
    let query = AuditQuery::new(logger.storage());

    // 记录多种日志
    for i in 0..10 {
        if i < 3 {
            logger.log_format_blocked(format!("cmd{}", i), format!("reason{}", i)).await.unwrap();
        } else if i < 6 {
            logger.log_rule_blocked(format!("cmd{}", i), format!("reason{}", i)).await.unwrap();
        } else {
            logger.log_execution_failed(format!("cmd{}", i), format!("reason{}", i)).await.unwrap();
        }
    }

    // 复杂查询：格式拦截 + 分页
    let params = AuditQueryParams {
        log_type: Some(AuditLogType::FormatBlocked),
        limit: Some(2),
        ..Default::default()
    };

    let result = query.query(params).await.unwrap();
    assert_eq!(result.logs.len(), 2);

    // 验证都是格式拦截
    for log in &result.logs {
        assert_eq!(log.log_type, AuditLogType::FormatBlocked);
    }
}

/// 测试关键词搜索集成
#[tokio::test]
async fn test_keyword_search() {
    let logger = AuditLogger::new();
    let query = AuditQuery::new(logger.storage());

    logger
        .log_format_blocked("important command".to_string(), "格式错误".to_string())
        .await
        .unwrap();
    logger
        .log_rule_blocked("other command".to_string(), "important reason".to_string())
        .await
        .unwrap();
    logger.log_execution_failed("test".to_string(), "test".to_string()).await.unwrap();

    // 搜索关键词
    let params = AuditQueryParams { keyword: Some("important".to_string()), ..Default::default() };

    let result = query.query(params).await.unwrap();
    assert_eq!(result.logs.len(), 2);
}

/// 测试删除与查询集成
#[tokio::test]
async fn test_delete_query_integration() {
    let logger = AuditLogger::new();
    let query = AuditQuery::new(logger.storage());

    // 记录日志
    let id = logger.log_format_blocked("test".to_string(), "reason".to_string()).await.unwrap();

    // 验证存在
    assert!(query.get(&id).await.is_ok());

    // 删除
    logger.delete(&id).await.unwrap();

    // 验证不存在
    assert!(query.get(&id).await.is_err());
}

/// 测试清空功能
#[tokio::test]
async fn test_clear_integration() {
    let logger = AuditLogger::new();
    let query = AuditQuery::new(logger.storage());

    // 记录多条日志
    for i in 0..5 {
        logger.log_format_blocked(format!("cmd{}", i), format!("r{}", i)).await.unwrap();
    }

    assert_eq!(logger.count().await, 5);

    // 清空
    logger.clear().await.unwrap();

    assert_eq!(logger.count().await, 0);
}

/// 测试容量限制
#[tokio::test]
async fn test_capacity_limit() {
    use fos_audit::AuditConfig;

    let config = AuditConfig { retention_days: 365, max_entries: 3 };
    let logger = AuditLogger::with_config(config);

    // 存储到上限
    logger.log_format_blocked("cmd1".to_string(), "r".to_string()).await.unwrap();
    logger.log_format_blocked("cmd2".to_string(), "r".to_string()).await.unwrap();
    logger.log_format_blocked("cmd3".to_string(), "r".to_string()).await.unwrap();

    // 超出上限应该失败
    let result = logger.log_format_blocked("cmd4".to_string(), "r".to_string()).await;
    assert!(result.is_err());
}

/// 测试按状态查询
#[tokio::test]
async fn test_find_by_status() {
    let logger = AuditLogger::new();
    let query = AuditQuery::new(logger.storage());

    // 记录并更新状态
    let id1 = logger.log_format_blocked("cmd".to_string(), "r".to_string()).await.unwrap();
    let id2 = logger.log_format_blocked("cmd".to_string(), "r".to_string()).await.unwrap();
    logger.mark_analyzed(&id1).await.unwrap();
    logger.mark_archived(&id2).await.unwrap();

    // 按状态查询
    let analyzed = query.find_by_status(AuditLogStatus::Analyzed).await.unwrap();
    assert_eq!(analyzed.len(), 1);

    let archived = query.find_by_status(AuditLogStatus::Archived).await.unwrap();
    assert_eq!(archived.len(), 1);
}

/// 测试最近日志查询
#[tokio::test]
async fn test_recent_logs() {
    let logger = AuditLogger::new();
    let query = AuditQuery::new(logger.storage());

    // 记录多条日志
    for i in 0..20 {
        logger.log_format_blocked(format!("cmd{}", i), format!("r{}", i)).await.unwrap();
    }

    // 获取最近5条
    let recent = query.find_recent(5).await.unwrap();
    assert_eq!(recent.len(), 5);
}
