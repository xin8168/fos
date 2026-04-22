//! Audit 配置

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub retention_days: u64,
    pub max_entries: usize,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self { retention_days: 365, max_entries: 100000 }
    }
}
