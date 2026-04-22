//! FOS Permission 集成测试
//!
//! 测试 Permission 模块各组件之间的集成

use fos_permission::{
    PermissionChecker, PermissionCheckerBuilder, PermissionRequest, PolicyEffect, PolicyManager,
    ResourceType, RoleManager, RoleType,
};
use std::sync::Arc;

/// 测试角色与策略集成
#[tokio::test]
async fn test_role_policy_integration() {
    let role_manager = Arc::new(RoleManager::new());
    let policy_manager = Arc::new(PolicyManager::new());

    // 创建角色
    let role_id = role_manager.create_role("admin".to_string(), RoleType::Admin).await.unwrap();

    // 创建策略
    let policy_id = policy_manager
        .create_policy(
            "admin-policy".to_string(),
            role_id.clone(),
            ResourceType::Device,
            "*".to_string(),
            PolicyEffect::Allow,
        )
        .await
        .unwrap();

    // 验证策略关联
    let policies = policy_manager.get_role_policies(&role_id).await.unwrap();
    assert_eq!(policies.len(), 1);
    assert_eq!(policies[0].id, policy_id);
}

/// 测试完整权限校验流程
#[tokio::test]
async fn test_full_permission_check() {
    let role_manager = Arc::new(RoleManager::new());
    let policy_manager = Arc::new(PolicyManager::new());
    let checker = PermissionChecker::new(role_manager.clone(), policy_manager.clone());

    // 创建角色
    let role_id = role_manager.create_role("user".to_string(), RoleType::User).await.unwrap();

    // 创建允许策略（使用通配符）
    let policy_id = policy_manager
        .create_policy(
            "user-read".to_string(),
            role_id.clone(),
            ResourceType::File,
            "*".to_string(),
            PolicyEffect::Allow,
        )
        .await
        .unwrap();

    // 添加动作
    policy_manager.add_action(&policy_id, "read".to_string()).await.unwrap();

    // 检查权限
    let request = PermissionRequest {
        role_id: role_id.clone(),
        resource_type: "file".to_string(),
        resource: "/home/user".to_string(),
        action: "read".to_string(),
    };

    let result = checker.check(request).await.unwrap();
    assert!(result.allowed);
}

/// 测试拒绝策略优先级
#[tokio::test]
async fn test_deny_policy_priority() {
    let role_manager = Arc::new(RoleManager::new());
    let policy_manager = Arc::new(PolicyManager::new());
    let checker = PermissionChecker::new(role_manager.clone(), policy_manager.clone());

    // 创建角色
    let role_id = role_manager.create_role("test".to_string(), RoleType::User).await.unwrap();

    // 创建允许策略（低优先级）
    let allow_id = policy_manager
        .create_policy(
            "allow".to_string(),
            role_id.clone(),
            ResourceType::Device,
            "*".to_string(),
            PolicyEffect::Allow,
        )
        .await
        .unwrap();
    policy_manager.add_action(&allow_id, "write".to_string()).await.unwrap();

    // 创建拒绝策略（高优先级）
    let deny_id = policy_manager
        .create_policy(
            "deny".to_string(),
            role_id.clone(),
            ResourceType::Device,
            "*".to_string(),
            PolicyEffect::Deny,
        )
        .await
        .unwrap();
    policy_manager.add_action(&deny_id, "write".to_string()).await.unwrap();

    // 更新优先级
    policy_manager
        .update_policy(
            &deny_id,
            fos_permission::PolicyUpdate { priority: Some(100), ..Default::default() },
        )
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

/// 测试角色状态对权限的影响
#[tokio::test]
async fn test_role_status_effect() {
    let role_manager = Arc::new(RoleManager::new());
    let policy_manager = Arc::new(PolicyManager::new());
    let checker = PermissionChecker::new(role_manager.clone(), policy_manager.clone());

    // 创建角色
    let role_id = role_manager.create_role("test".to_string(), RoleType::User).await.unwrap();

    // 创建策略
    let policy_id = policy_manager
        .create_policy(
            "test".to_string(),
            role_id.clone(),
            ResourceType::Device,
            "*".to_string(),
            PolicyEffect::Allow,
        )
        .await
        .unwrap();
    policy_manager.add_action(&policy_id, "read".to_string()).await.unwrap();

    // 禁用角色
    role_manager.disable_role(&role_id).await.unwrap();

    // 检查权限（应该被拒绝）
    let request = PermissionRequest {
        role_id: role_id.clone(),
        resource_type: "device".to_string(),
        resource: "device-001".to_string(),
        action: "read".to_string(),
    };

    let result = checker.check(request).await.unwrap();
    assert!(!result.allowed);

    // 重新启用角色
    role_manager.enable_role(&role_id).await.unwrap();

    // 再次检查（应该通过）
    let request = PermissionRequest {
        role_id,
        resource_type: "device".to_string(),
        resource: "device-001".to_string(),
        action: "read".to_string(),
    };

    let result = checker.check(request).await.unwrap();
    assert!(result.allowed);
}

/// 测试策略状态对权限的影响
#[tokio::test]
async fn test_policy_status_effect() {
    let role_manager = Arc::new(RoleManager::new());
    let policy_manager = Arc::new(PolicyManager::new());
    let checker = PermissionChecker::new(role_manager.clone(), policy_manager.clone());

    // 创建角色和策略
    let role_id = role_manager.create_role("test".to_string(), RoleType::User).await.unwrap();
    let policy_id = policy_manager
        .create_policy(
            "test".to_string(),
            role_id.clone(),
            ResourceType::Device,
            "*".to_string(),
            PolicyEffect::Allow,
        )
        .await
        .unwrap();
    policy_manager.add_action(&policy_id, "read".to_string()).await.unwrap();

    // 先验证策略启用时可以访问
    let request = PermissionRequest {
        role_id: role_id.clone(),
        resource_type: "device".to_string(),
        resource: "test".to_string(),
        action: "read".to_string(),
    };

    let result = checker.check(request.clone()).await.unwrap();
    assert!(result.allowed, "策略启用时应该允许访问");

    // 禁用策略
    policy_manager.disable_policy(&policy_id).await.unwrap();

    // 检查权限（应该被拒绝因为没有启用的策略）
    let result = checker.check(request).await.unwrap();
    assert!(!result.allowed, "策略禁用时不应该允许访问");
}

/// 测试批量权限校验
#[tokio::test]
async fn test_batch_permission_check() {
    let role_manager = Arc::new(RoleManager::new());
    let policy_manager = Arc::new(PolicyManager::new());
    let checker = PermissionChecker::new(role_manager.clone(), policy_manager.clone());

    // 创建角色和策略
    let role_id = role_manager.create_role("test".to_string(), RoleType::User).await.unwrap();
    let policy_id = policy_manager
        .create_policy(
            "test".to_string(),
            role_id.clone(),
            ResourceType::Device,
            "*".to_string(),
            PolicyEffect::Allow,
        )
        .await
        .unwrap();
    policy_manager.add_action(&policy_id, "read".to_string()).await.unwrap();

    // 批量检查
    let requests = vec![
        PermissionRequest {
            role_id: role_id.clone(),
            resource_type: "device".to_string(),
            resource: "d1".to_string(),
            action: "read".to_string(),
        },
        PermissionRequest {
            role_id: role_id.clone(),
            resource_type: "device".to_string(),
            resource: "d2".to_string(),
            action: "write".to_string(), // 没有write权限
        },
        PermissionRequest {
            role_id: role_id.clone(),
            resource_type: "file".to_string(), // 没有file策略
            resource: "f1".to_string(),
            action: "read".to_string(),
        },
    ];

    let results = checker.check_batch(requests).await.unwrap();
    assert_eq!(results.len(), 3);
    assert!(results[0].allowed, "device read 应该通过");
    assert!(!results[1].allowed, "device write 应该被拒绝（没有write动作）");
    assert!(!results[2].allowed, "file read 应该被拒绝（没有file策略）");
}

/// 测试权限校验器构建器
#[tokio::test]
async fn test_checker_builder() {
    let role_manager = Arc::new(RoleManager::new());
    let policy_manager = Arc::new(PolicyManager::new());

    // 创建角色
    let role_id = role_manager.create_role("test".to_string(), RoleType::User).await.unwrap();

    let checker = PermissionCheckerBuilder::new()
        .with_role_manager(role_manager)
        .with_policy_manager(policy_manager)
        .build();

    let request = PermissionRequest {
        role_id,
        resource_type: "device".to_string(),
        resource: "test".to_string(),
        action: "read".to_string(),
    };

    // 应该能执行（虽然会被拒绝因为没有策略）
    let result = checker.check(request).await;
    assert!(result.is_ok());
}

/// 测试多角色权限管理
#[tokio::test]
async fn test_multi_role_management() {
    let role_manager = Arc::new(RoleManager::new());
    let policy_manager = Arc::new(PolicyManager::new());

    // 创建多个角色
    let admin_id = role_manager.create_role("admin".to_string(), RoleType::Admin).await.unwrap();
    let user_id = role_manager.create_role("user".to_string(), RoleType::User).await.unwrap();

    // 为每个角色创建策略
    policy_manager
        .create_policy(
            "admin-all".to_string(),
            admin_id.clone(),
            ResourceType::Device,
            "*".to_string(),
            PolicyEffect::Allow,
        )
        .await
        .unwrap();

    policy_manager
        .create_policy(
            "user-read".to_string(),
            user_id.clone(),
            ResourceType::Device,
            "device-001".to_string(),
            PolicyEffect::Allow,
        )
        .await
        .unwrap();

    // 验证各自的角色策略
    let admin_policies = policy_manager.get_role_policies(&admin_id).await.unwrap();
    assert_eq!(admin_policies.len(), 1);

    let user_policies = policy_manager.get_role_policies(&user_id).await.unwrap();
    assert_eq!(user_policies.len(), 1);
}
