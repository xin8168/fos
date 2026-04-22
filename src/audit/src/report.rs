//! Audit 报告

use crate::AuditLog;
use std::collections::HashMap;

pub struct AuditReport {
    pub total_count: usize,
    pub blocked_count: usize,
    pub failed_count: usize,
}

impl AuditReport {
    pub fn generate(logs: &HashMap<String, AuditLog>) -> Self {
        let total_count = logs.len();
        let blocked_count = logs
            .values()
            .filter(|l| {
                matches!(
                    l.log_type,
                    crate::AuditLogType::FormatBlocked | crate::AuditLogType::RuleBlocked
                )
            })
            .count();
        let failed_count = logs
            .values()
            .filter(|l| matches!(l.log_type, crate::AuditLogType::ExecutionFailed))
            .count();

        Self { total_count, blocked_count, failed_count }
    }
}
