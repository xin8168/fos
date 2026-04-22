//! # 权限校验模块
//!
//! 负责执行权限检查和访问控制决策

use crate::error::Result;
use crate::policy::{Policy, PolicyEffect, PolicyManager};
use crate::role::RoleManager;
use std::sync::Arc;

/// 权限校验请求
#[derive(Debug, Clone)]
pub struct PermissionRequest {
    /// 角色ID
    pub role_id: String,

    /// 资源类型
    pub resource_type: String,

    /// 资源标识
    pub resource: String,

    /// 动作
    pub action: String,
}

/// 权限校验结果
#[derive(Debug, Clone)]
pub struct PermissionResult {
    /// 是否允许
    pub allowed: bool,

    /// 匹配的策略
    pub matched_policies: Vec<String>,

    /// 拒绝原因
    pub deny_reason: Option<String>,
}

impl PermissionResult {
    /// 创建允许结果
    pub fn allowed() -> Self {
        Self { allowed: true, matched_policies: Vec::new(), deny_reason: None }
    }

    /// 创建拒绝结果
    pub fn denied(reason: String) -> Self {
        Self { allowed: false, matched_policies: Vec::new(), deny_reason: Some(reason) }
    }

    /// 添加匹配的策略
    pub fn add_matched_policy(&mut self, policy_id: String) {
        self.matched_policies.push(policy_id);
    }
}

/// 权限校验器
pub struct PermissionChecker {
    /// 角色管理器
    role_manager: Arc<RoleManager>,

    /// 策略管理器
    policy_manager: Arc<PolicyManager>,
}

impl PermissionChecker {
    /// 创建新的权限校验器
    pub fn new(role_manager: Arc<RoleManager>, policy_manager: Arc<PolicyManager>) -> Self {
        Self { role_manager, policy_manager }
    }

    /// 检查权限
    pub async fn check(&self, request: PermissionRequest) -> Result<PermissionResult> {
        // 1. 检查角色是否存在且启用
        let role = match self.role_manager.get_role(&request.role_id).await {
            Ok(r) => r,
            Err(_) => {
                return Ok(PermissionResult::denied(format!("角色 {} 不存在", request.role_id)))
            },
        };

        if !role.is_active() {
            return Ok(PermissionResult::denied(format!("角色 {} 未启用", request.role_id)));
        }

        // 2. 获取角色的所有策略
        let policies = self.policy_manager.get_role_policies(&request.role_id).await?;

        if policies.is_empty() {
            return Ok(PermissionResult::denied(format!(
                "角色 {} 没有定义任何策略",
                request.role_id
            )));
        }

        // 3. 按优先级排序
        let mut policies = policies;
        policies.sort_by(|a, b| b.priority.cmp(&a.priority));

        // 4. 评估策略
        let mut result = PermissionResult::denied("没有匹配的允许策略".to_string());
        let mut has_matching_policy = false;

        for policy in policies {
            if self.policy_matches(&policy, &request) {
                has_matching_policy = true;
                result.add_matched_policy(policy.id.clone());

                match policy.effect {
                    PolicyEffect::Allow => {
                        // 允许策略，继续检查是否有更高优先级的拒绝策略
                        result.allowed = true;
                        result.deny_reason = None;
                    },
                    PolicyEffect::Deny => {
                        // 拒绝策略，立即返回
                        result.allowed = false;
                        result.deny_reason = Some(format!("策略 {} 拒绝访问", policy.name));
                        return Ok(result);
                    },
                }
            }
        }

        if !has_matching_policy {
            // 没有匹配的策略，默认拒绝
            return Ok(PermissionResult::denied("没有匹配的访问策略".to_string()));
        }

        Ok(result)
    }

    /// 快速检查权限
    pub async fn is_allowed(&self, request: PermissionRequest) -> bool {
        match self.check(request).await {
            Ok(result) => result.allowed,
            Err(_) => false,
        }
    }

    /// 批量检查权限
    pub async fn check_batch(
        &self,
        requests: Vec<PermissionRequest>,
    ) -> Result<Vec<PermissionResult>> {
        let mut results = Vec::new();
        for request in requests {
            let result = self.check(request).await?;
            results.push(result);
        }
        Ok(results)
    }

    /// 检查策略是否匹配请求
    fn policy_matches(&self, policy: &Policy, request: &PermissionRequest) -> bool {
        // 检查状态
        if !policy.is_active() {
            return false;
        }

        // 检查资源类型
        let resource_type_matches = match &policy.resource_type {
            crate::policy::ResourceType::Device => request.resource_type == "device",
            crate::policy::ResourceType::File => request.resource_type == "file",
            crate::policy::ResourceType::System => request.resource_type == "system",
            crate::policy::ResourceType::Network => request.resource_type == "network",
            crate::policy::ResourceType::Custom(t) => request.resource_type == *t,
        };

        if !resource_type_matches {
            return false;
        }

        // 检查资源
        if !policy.matches_resource(&request.resource) {
            return false;
        }

        // 检查动作
        if !policy.has_action(&request.action) {
            return false;
        }

        true
    }
}

/// 权限校验器构建器
pub struct PermissionCheckerBuilder {
    role_manager: Option<Arc<RoleManager>>,
    policy_manager: Option<Arc<PolicyManager>>,
}

impl PermissionCheckerBuilder {
    /// 创建构建器
    pub fn new() -> Self {
        Self { role_manager: None, policy_manager: None }
    }

    /// 设置角色管理器
    pub fn with_role_manager(mut self, manager: Arc<RoleManager>) -> Self {
        self.role_manager = Some(manager);
        self
    }

    /// 设置策略管理器
    pub fn with_policy_manager(mut self, manager: Arc<PolicyManager>) -> Self {
        self.policy_manager = Some(manager);
        self
    }

    /// 构建校验器
    pub fn build(self) -> PermissionChecker {
        let role_manager = self.role_manager.unwrap_or_else(|| Arc::new(RoleManager::new()));
        let policy_manager = self.policy_manager.unwrap_or_else(|| Arc::new(PolicyManager::new()));
        PermissionChecker::new(role_manager, policy_manager)
    }
}

impl Default for PermissionCheckerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_permission_allowed() {
        let role_manager = Arc::new(RoleManager::new());
        let policy_manager = Arc::new(PolicyManager::new());
        let checker = PermissionChecker::new(role_manager.clone(), policy_manager.clone());

        // 创建角色
        let role_id = role_manager
            .create_role("admin".to_string(), crate::role::RoleType::Admin)
            .await
            .unwrap();

        // 创建允许策略
        policy_manager
            .create_policy(
                "allow-device".to_string(),
                role_id.clone(),
                crate::policy::ResourceType::Device,
                "*".to_string(),
                PolicyEffect::Allow,
            )
            .await
            .unwrap();
        policy_manager
            .add_action(&policy_manager.list_policies().await[0].id.clone(), "read".to_string())
            .await
            .unwrap();

        // 检查权限
        let request = PermissionRequest {
            role_id,
            resource_type: "device".to_string(),
            resource: "device-001".to_string(),
            action: "read".to_string(),
        };

        let result = checker.check(request).await.unwrap();
        assert!(result.allowed);
    }

    #[tokio::test]
    async fn test_check_permission_denied() {
        let role_manager = Arc::new(RoleManager::new());
        let policy_manager = Arc::new(PolicyManager::new());
        let checker = PermissionChecker::new(role_manager.clone(), policy_manager.clone());

        // 创建角色
        let role_id = role_manager
            .create_role("user".to_string(), crate::role::RoleType::User)
            .await
            .unwrap();

        // 创建拒绝策略
        policy_manager
            .create_policy(
                "deny-device".to_string(),
                role_id.clone(),
                crate::policy::ResourceType::Device,
                "*".to_string(),
                PolicyEffect::Deny,
            )
            .await
            .unwrap();
        policy_manager
            .add_action(&policy_manager.list_policies().await[0].id.clone(), "write".to_string())
            .await
            .unwrap();

        // 检查权限
        let request = PermissionRequest {
            role_id,
            resource_type: "device".to_string(),
            resource: "device-001".to_string(),
            action: "write".to_string(),
        };

        let result = checker.check(request).await.unwrap();
        assert!(!result.allowed);
    }

    #[tokio::test]
    async fn test_check_nonexistent_role() {
        let checker = PermissionCheckerBuilder::new().build();

        let request = PermissionRequest {
            role_id: "nonexistent".to_string(),
            resource_type: "device".to_string(),
            resource: "device-001".to_string(),
            action: "read".to_string(),
        };

        let result = checker.check(request).await.unwrap();
        assert!(!result.allowed);
    }

    #[tokio::test]
    async fn test_is_allowed() {
        let role_manager = Arc::new(RoleManager::new());
        let policy_manager = Arc::new(PolicyManager::new());
        let checker = PermissionChecker::new(role_manager.clone(), policy_manager.clone());

        let role_id = role_manager
            .create_role("test".to_string(), crate::role::RoleType::User)
            .await
            .unwrap();
        policy_manager
            .create_policy(
                "test".to_string(),
                role_id.clone(),
                crate::policy::ResourceType::Device,
                "*".to_string(),
                PolicyEffect::Allow,
            )
            .await
            .unwrap();
        policy_manager
            .add_action(&policy_manager.list_policies().await[0].id.clone(), "read".to_string())
            .await
            .unwrap();

        let request = PermissionRequest {
            role_id,
            resource_type: "device".to_string(),
            resource: "test".to_string(),
            action: "read".to_string(),
        };

        assert!(checker.is_allowed(request).await);
    }

    #[tokio::test]
    async fn test_batch_check() {
        let role_manager = Arc::new(RoleManager::new());
        let policy_manager = Arc::new(PolicyManager::new());
        let checker = PermissionChecker::new(role_manager.clone(), policy_manager.clone());

        let role_id = role_manager
            .create_role("test".to_string(), crate::role::RoleType::User)
            .await
            .unwrap();
        policy_manager
            .create_policy(
                "test".to_string(),
                role_id.clone(),
                crate::policy::ResourceType::Device,
                "*".to_string(),
                PolicyEffect::Allow,
            )
            .await
            .unwrap();
        policy_manager
            .add_action(&policy_manager.list_policies().await[0].id.clone(), "read".to_string())
            .await
            .unwrap();

        let requests = vec![
            PermissionRequest {
                role_id: role_id.clone(),
                resource_type: "device".to_string(),
                resource: "d1".to_string(),
                action: "read".to_string(),
            },
            PermissionRequest {
                role_id: role_id.clone(),
                resource_type: "file".to_string(),
                resource: "f1".to_string(),
                action: "read".to_string(),
            },
        ];

        let results = checker.check_batch(requests).await.unwrap();
        assert_eq!(results.len(), 2);
    }
}
