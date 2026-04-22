//! # 权限策略模块
//!
//! 负责权限策略的定义和管理

use crate::error::{Error, Result};
use crate::role::RoleId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 策略ID
pub type PolicyId = String;

/// 资源ID
pub type ResourceId = String;

/// 动作ID
pub type ActionId = String;

/// 策略效果
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyEffect {
    /// 允许
    Allow,
    /// 拒绝
    Deny,
}

/// 策略状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyStatus {
    /// 启用
    Active,
    /// 禁用
    Disabled,
}

/// 资源类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    /// 设备
    Device,
    /// 文件
    File,
    /// 系统
    System,
    /// 网络
    Network,
    /// 自定义
    Custom(String),
}

/// 权限策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// 策略ID
    pub id: PolicyId,

    /// 策略名称
    pub name: String,

    /// 关联的角色
    pub role_id: RoleId,

    /// 资源类型
    pub resource_type: ResourceType,

    /// 资源标识（通配符支持）
    pub resource: String,

    /// 允许的动作
    pub actions: HashSet<ActionId>,

    /// 策略效果
    pub effect: PolicyEffect,

    /// 策略状态
    pub status: PolicyStatus,

    /// 优先级（数字越大优先级越高）
    pub priority: i32,

    /// 条件表达式
    pub condition: Option<String>,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl Policy {
    /// 创建新策略
    pub fn new(
        name: String,
        role_id: RoleId,
        resource_type: ResourceType,
        resource: String,
        effect: PolicyEffect,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            role_id,
            resource_type,
            resource,
            actions: HashSet::new(),
            effect,
            status: PolicyStatus::Active,
            priority: 0,
            condition: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// 添加动作
    pub fn add_action(&mut self, action: ActionId) {
        self.actions.insert(action);
        self.updated_at = Utc::now();
    }

    /// 移除动作
    pub fn remove_action(&mut self, action: &ActionId) {
        self.actions.remove(action);
        self.updated_at = Utc::now();
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// 设置条件
    pub fn with_condition(mut self, condition: String) -> Self {
        self.condition = Some(condition);
        self
    }

    /// 检查是否匹配资源
    pub fn matches_resource(&self, resource: &str) -> bool {
        if self.resource == "*" {
            return true;
        }
        self.resource == resource
    }

    /// 检查是否包含动作
    pub fn has_action(&self, action: &ActionId) -> bool {
        self.actions.contains(action)
    }

    /// 启用策略
    pub fn enable(&mut self) {
        self.status = PolicyStatus::Active;
        self.updated_at = Utc::now();
    }

    /// 禁用策略
    pub fn disable(&mut self) {
        self.status = PolicyStatus::Disabled;
        self.updated_at = Utc::now();
    }

    /// 检查是否启用
    pub fn is_active(&self) -> bool {
        self.status == PolicyStatus::Active
    }
}

/// 策略管理器
pub struct PolicyManager {
    /// 策略存储
    policies: Arc<RwLock<HashMap<PolicyId, Policy>>>,

    /// 角色到策略的映射
    role_policies: Arc<RwLock<HashMap<RoleId, Vec<PolicyId>>>>,
}

impl PolicyManager {
    /// 创建新的策略管理器
    pub fn new() -> Self {
        Self {
            policies: Arc::new(RwLock::new(HashMap::new())),
            role_policies: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建策略
    pub async fn create_policy(
        &self,
        name: String,
        role_id: RoleId,
        resource_type: ResourceType,
        resource: String,
        effect: PolicyEffect,
    ) -> Result<PolicyId> {
        let policy = Policy::new(name, role_id.clone(), resource_type, resource, effect);
        let id = policy.id.clone();

        let mut policies = self.policies.write().await;
        let mut role_policies = self.role_policies.write().await;

        policies.insert(id.clone(), policy);
        role_policies.entry(role_id).or_insert_with(Vec::new).push(id.clone());

        Ok(id)
    }

    /// 获取策略
    pub async fn get_policy(&self, id: &PolicyId) -> Result<Policy> {
        let policies = self.policies.read().await;
        policies.get(id).cloned().ok_or_else(|| Error::Permission(format!("策略 {} 不存在", id)))
    }

    /// 获取角色的所有策略
    pub async fn get_role_policies(&self, role_id: &RoleId) -> Result<Vec<Policy>> {
        let role_policies = self.role_policies.read().await;
        let policy_ids = role_policies.get(role_id).cloned().unwrap_or_default();

        let policies = self.policies.read().await;
        let result: Vec<Policy> =
            policy_ids.iter().filter_map(|id| policies.get(id).cloned()).collect();

        Ok(result)
    }

    /// 更新策略
    pub async fn update_policy(&self, id: &PolicyId, updates: PolicyUpdate) -> Result<()> {
        let mut policies = self.policies.write().await;
        let policy =
            policies.get_mut(id).ok_or_else(|| Error::Permission(format!("策略 {} 不存在", id)))?;

        if let Some(name) = updates.name {
            policy.name = name;
        }

        if let Some(priority) = updates.priority {
            policy.priority = priority;
        }

        if let Some(condition) = updates.condition {
            policy.condition = Some(condition);
        }

        policy.updated_at = Utc::now();
        Ok(())
    }

    /// 删除策略
    pub async fn delete_policy(&self, id: &PolicyId) -> Result<()> {
        let mut policies = self.policies.write().await;
        let policy =
            policies.remove(id).ok_or_else(|| Error::Permission(format!("策略 {} 不存在", id)))?;

        let mut role_policies = self.role_policies.write().await;
        if let Some(ids) = role_policies.get_mut(&policy.role_id) {
            ids.retain(|p| p != id);
        }

        Ok(())
    }

    /// 给策略添加动作
    pub async fn add_action(&self, policy_id: &PolicyId, action: ActionId) -> Result<()> {
        let mut policies = self.policies.write().await;
        let policy = policies
            .get_mut(policy_id)
            .ok_or_else(|| Error::Permission(format!("策略 {} 不存在", policy_id)))?;
        policy.add_action(action);
        Ok(())
    }

    /// 从策略移除动作
    pub async fn remove_action(&self, policy_id: &PolicyId, action: &ActionId) -> Result<()> {
        let mut policies = self.policies.write().await;
        let policy = policies
            .get_mut(policy_id)
            .ok_or_else(|| Error::Permission(format!("策略 {} 不存在", policy_id)))?;
        policy.remove_action(action);
        Ok(())
    }

    /// 启用策略
    pub async fn enable_policy(&self, id: &PolicyId) -> Result<()> {
        let mut policies = self.policies.write().await;
        let policy =
            policies.get_mut(id).ok_or_else(|| Error::Permission(format!("策略 {} 不存在", id)))?;
        policy.enable();
        Ok(())
    }

    /// 禁用策略
    pub async fn disable_policy(&self, id: &PolicyId) -> Result<()> {
        let mut policies = self.policies.write().await;
        let policy =
            policies.get_mut(id).ok_or_else(|| Error::Permission(format!("策略 {} 不存在", id)))?;
        policy.disable();
        Ok(())
    }

    /// 列出所有策略
    pub async fn list_policies(&self) -> Vec<Policy> {
        let policies = self.policies.read().await;
        policies.values().cloned().collect()
    }

    /// 统计策略数量
    pub async fn count(&self) -> usize {
        let policies = self.policies.read().await;
        policies.len()
    }
}

impl Default for PolicyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 策略更新信息
#[derive(Debug, Clone, Default)]
pub struct PolicyUpdate {
    /// 新名称
    pub name: Option<String>,
    /// 新优先级
    pub priority: Option<i32>,
    /// 新条件
    pub condition: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_policy() {
        let manager = PolicyManager::new();
        let id = manager
            .create_policy(
                "test-policy".to_string(),
                "role-001".to_string(),
                ResourceType::Device,
                "device:*".to_string(),
                PolicyEffect::Allow,
            )
            .await
            .unwrap();

        assert!(!id.is_empty());
        let policy = manager.get_policy(&id).await.unwrap();
        assert_eq!(policy.name, "test-policy");
    }

    #[tokio::test]
    async fn test_get_role_policies() {
        let manager = PolicyManager::new();
        manager
            .create_policy(
                "p1".to_string(),
                "role-001".to_string(),
                ResourceType::Device,
                "*".to_string(),
                PolicyEffect::Allow,
            )
            .await
            .unwrap();
        manager
            .create_policy(
                "p2".to_string(),
                "role-001".to_string(),
                ResourceType::File,
                "*".to_string(),
                PolicyEffect::Deny,
            )
            .await
            .unwrap();

        let policies = manager.get_role_policies(&"role-001".to_string()).await.unwrap();
        assert_eq!(policies.len(), 2);
    }

    #[tokio::test]
    async fn test_add_remove_action() {
        let manager = PolicyManager::new();
        let id = manager
            .create_policy(
                "test".to_string(),
                "role-001".to_string(),
                ResourceType::Device,
                "*".to_string(),
                PolicyEffect::Allow,
            )
            .await
            .unwrap();

        manager.add_action(&id, "read".to_string()).await.unwrap();
        manager.add_action(&id, "write".to_string()).await.unwrap();

        let policy = manager.get_policy(&id).await.unwrap();
        assert!(policy.has_action(&"read".to_string()));
        assert!(policy.has_action(&"write".to_string()));

        manager.remove_action(&id, &"write".to_string()).await.unwrap();
        let policy = manager.get_policy(&id).await.unwrap();
        assert!(!policy.has_action(&"write".to_string()));
    }

    #[tokio::test]
    async fn test_enable_disable_policy() {
        let manager = PolicyManager::new();
        let id = manager
            .create_policy(
                "test".to_string(),
                "role-001".to_string(),
                ResourceType::Device,
                "*".to_string(),
                PolicyEffect::Allow,
            )
            .await
            .unwrap();

        manager.disable_policy(&id).await.unwrap();
        let policy = manager.get_policy(&id).await.unwrap();
        assert!(!policy.is_active());

        manager.enable_policy(&id).await.unwrap();
        let policy = manager.get_policy(&id).await.unwrap();
        assert!(policy.is_active());
    }

    #[tokio::test]
    async fn test_delete_policy() {
        let manager = PolicyManager::new();
        let id = manager
            .create_policy(
                "test".to_string(),
                "role-001".to_string(),
                ResourceType::Device,
                "*".to_string(),
                PolicyEffect::Allow,
            )
            .await
            .unwrap();

        manager.delete_policy(&id).await.unwrap();
        let result = manager.get_policy(&id).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_policy_matches_resource() {
        let policy = Policy::new(
            "test".to_string(),
            "role-001".to_string(),
            ResourceType::Device,
            "*".to_string(),
            PolicyEffect::Allow,
        );
        assert!(policy.matches_resource("any-resource"));

        let policy = Policy::new(
            "test".to_string(),
            "role-001".to_string(),
            ResourceType::Device,
            "device-001".to_string(),
            PolicyEffect::Allow,
        );
        assert!(policy.matches_resource("device-001"));
        assert!(!policy.matches_resource("device-002"));
    }

    #[test]
    fn test_policy_creation() {
        let policy = Policy::new(
            "test".to_string(),
            "role-001".to_string(),
            ResourceType::Custom("custom".to_string()),
            "resource".to_string(),
            PolicyEffect::Deny,
        )
        .with_priority(10)
        .with_condition("time > 9:00".to_string());

        assert!(!policy.id.is_empty());
        assert_eq!(policy.priority, 10);
        assert!(policy.condition.is_some());
        assert!(policy.is_active());
    }
}
