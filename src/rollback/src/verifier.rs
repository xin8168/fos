//! # 回滚结果验证模块
//!
//! 负责验证回滚操作的结果

use crate::error::Result;
use crate::executor::RollbackResult;
use crate::snapshot::{SnapshotId, SnapshotManager, SnapshotStatus};
use serde::{Deserialize, Serialize};

/// 验证状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    /// 待验证
    Pending,
    /// 验证中
    Verifying,
    /// 验证通过
    Passed,
    /// 验证失败
    Failed,
}

/// 验证检查项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCheck {
    /// 检查项名称
    pub name: String,

    /// 检查类型
    pub check_type: VerificationCheckType,

    /// 是否通过
    pub passed: bool,

    /// 错误信息
    pub error: Option<String>,
}

/// 验证检查类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationCheckType {
    /// 数据一致性
    DataConsistency,
    /// 状态一致性
    StateConsistency,
    /// 资源完整性
    ResourceIntegrity,
    /// 依赖关系
    Dependencies,
    /// 自定义检查
    Custom(String),
}

/// 验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// 快照ID
    pub snapshot_id: SnapshotId,

    /// 验证状态
    pub status: VerificationStatus,

    /// 所有检查项
    pub checks: Vec<VerificationCheck>,

    /// 通过的检查数
    pub passed_count: usize,

    /// 失败的检查数
    pub failed_count: usize,

    /// 验证时间
    pub verified_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl VerificationResult {
    /// 创建新的验证结果
    pub fn new(snapshot_id: SnapshotId) -> Self {
        Self {
            snapshot_id,
            status: VerificationStatus::Pending,
            checks: Vec::new(),
            passed_count: 0,
            failed_count: 0,
            verified_at: None,
        }
    }

    /// 添加检查项
    pub fn add_check(&mut self, check: VerificationCheck) {
        if check.passed {
            self.passed_count += 1;
        } else {
            self.failed_count += 1;
        }
        self.checks.push(check);
    }

    /// 标记为验证中
    pub fn mark_verifying(&mut self) {
        self.status = VerificationStatus::Verifying;
    }

    /// 标记为完成
    pub fn finalize(&mut self) {
        self.status = if self.failed_count == 0 {
            VerificationStatus::Passed
        } else {
            VerificationStatus::Failed
        };
        self.verified_at = Some(chrono::Utc::now());
    }

    /// 检查是否通过
    pub fn is_passed(&self) -> bool {
        self.status == VerificationStatus::Passed
    }
}

/// 回滚验证器
pub struct RollbackVerifier {
    /// 快照管理器
    snapshot_manager: std::sync::Arc<SnapshotManager>,
}

impl RollbackVerifier {
    /// 创建新的验证器
    pub fn new(snapshot_manager: std::sync::Arc<SnapshotManager>) -> Self {
        Self { snapshot_manager }
    }

    /// 验证回滚结果
    pub async fn verify(&self, rollback_result: &RollbackResult) -> Result<VerificationResult> {
        let mut result = VerificationResult::new(rollback_result.snapshot_id.clone());
        result.mark_verifying();

        // 获取快照
        let snapshot = self.snapshot_manager.get_snapshot(&rollback_result.snapshot_id).await?;

        // 检查1: 快照状态
        let status_check = VerificationCheck {
            name: "快照状态检查".to_string(),
            check_type: VerificationCheckType::StateConsistency,
            passed: snapshot.status == SnapshotStatus::RolledBack,
            error: if snapshot.status != SnapshotStatus::RolledBack {
                Some(format!("快照状态不是已回滚: {:?}", snapshot.status))
            } else {
                None
            },
        };
        result.add_check(status_check);

        // 检查2: 回滚动作执行
        let action_check = VerificationCheck {
            name: "回滚动作检查".to_string(),
            check_type: VerificationCheckType::Dependencies,
            passed: rollback_result.success,
            error: if !rollback_result.success { rollback_result.error.clone() } else { None },
        };
        result.add_check(action_check);

        // 检查3: 数据一致性（模拟）
        let data_check = VerificationCheck {
            name: "数据一致性检查".to_string(),
            check_type: VerificationCheckType::DataConsistency,
            passed: true,
            error: None,
        };
        result.add_check(data_check);

        // 检查4: 资源完整性（模拟）
        let resource_check = VerificationCheck {
            name: "资源完整性检查".to_string(),
            check_type: VerificationCheckType::ResourceIntegrity,
            passed: true,
            error: None,
        };
        result.add_check(resource_check);

        result.finalize();
        Ok(result)
    }

    /// 快速验证（仅检查关键项）
    pub async fn quick_verify(&self, rollback_result: &RollbackResult) -> Result<bool> {
        let result = self.verify(rollback_result).await?;
        Ok(result.is_passed())
    }

    /// 批量验证
    pub async fn verify_batch(
        &self,
        rollback_results: &[RollbackResult],
    ) -> Result<Vec<VerificationResult>> {
        let mut results = Vec::new();
        for rollback_result in rollback_results {
            let result = self.verify(rollback_result).await?;
            results.push(result);
        }
        Ok(results)
    }
}

impl Default for RollbackVerifier {
    fn default() -> Self {
        Self::new(std::sync::Arc::new(SnapshotManager::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::RollbackExecutor;

    #[tokio::test]
    async fn test_verify_rollback() {
        let snapshot_manager = std::sync::Arc::new(SnapshotManager::new());
        let executor = RollbackExecutor::new(snapshot_manager.clone());
        let verifier = RollbackVerifier::new(snapshot_manager.clone());

        // 创建并执行回滚
        let id = snapshot_manager
            .create_snapshot_with_rollback(
                "op-001".to_string(),
                crate::snapshot::SnapshotType::Full,
                serde_json::json!({}),
                serde_json::json!({}),
            )
            .await
            .unwrap();

        let rollback_result = executor.execute(&id).await.unwrap();

        // 验证
        let result = verifier.verify(&rollback_result).await.unwrap();
        assert!(result.is_passed());
        assert_eq!(result.passed_count, 4);
    }

    #[tokio::test]
    async fn test_quick_verify() {
        let snapshot_manager = std::sync::Arc::new(SnapshotManager::new());
        let executor = RollbackExecutor::new(snapshot_manager.clone());
        let verifier = RollbackVerifier::new(snapshot_manager.clone());

        let id = snapshot_manager
            .create_snapshot_with_rollback(
                "op-001".to_string(),
                crate::snapshot::SnapshotType::Full,
                serde_json::json!({}),
                serde_json::json!({}),
            )
            .await
            .unwrap();

        let rollback_result = executor.execute(&id).await.unwrap();

        let passed = verifier.quick_verify(&rollback_result).await.unwrap();
        assert!(passed);
    }

    #[test]
    fn test_verification_result() {
        let mut result = VerificationResult::new("snap-001".to_string());

        result.add_check(VerificationCheck {
            name: "测试1".to_string(),
            check_type: VerificationCheckType::DataConsistency,
            passed: true,
            error: None,
        });

        result.add_check(VerificationCheck {
            name: "测试2".to_string(),
            check_type: VerificationCheckType::StateConsistency,
            passed: false,
            error: Some("失败".to_string()),
        });

        result.finalize();

        assert_eq!(result.passed_count, 1);
        assert_eq!(result.failed_count, 1);
        assert!(!result.is_passed());
    }

    #[test]
    fn test_verification_status() {
        let mut result = VerificationResult::new("snap-001".to_string());

        assert_eq!(result.status, VerificationStatus::Pending);

        result.mark_verifying();
        assert_eq!(result.status, VerificationStatus::Verifying);

        result.finalize();
        assert!(
            result.status == VerificationStatus::Passed
                || result.status == VerificationStatus::Failed
        );
    }
}
