//! # 角色管理模块
//!
//! 负责角色的创建、管理和权限分配

use crate::error::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 角色ID
pub type RoleId = String;

/// 权限ID
pub type PermissionId = String;

/// 角色状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RoleStatus {
    /// 启用
    Active,
    /// 禁用
    Disabled,
    /// 已删除
    Deleted,
}

/// 角色类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RoleType {
    /// 超级管理员
    SuperAdmin,
    /// 管理员
    Admin,
    /// 普通用户
    User,
    /// 访客
    Guest,
    /// 自定义
    Custom(String),
}

/// 角色
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// 角色ID
    pub id: RoleId,

    /// 角色名称
    pub name: String,

    /// 角色类型
    pub role_type: RoleType,

    /// 角色状态
    pub status: RoleStatus,

    /// 权限列表
    pub permissions: HashSet<PermissionId>,

    /// 描述
    pub description: Option<String>,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,

    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl Role {
    /// 创建新角色
    pub fn new(name: String, role_type: RoleType) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            role_type,
            status: RoleStatus::Active,
            permissions: HashSet::new(),
            description: None,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    /// 设置描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// 添加权限
    pub fn add_permission(&mut self, permission: PermissionId) {
        self.permissions.insert(permission);
        self.updated_at = Utc::now();
    }

    /// 移除权限
    pub fn remove_permission(&mut self, permission: &PermissionId) {
        self.permissions.remove(permission);
        self.updated_at = Utc::now();
    }

    /// 检查是否有权限
    pub fn has_permission(&self, permission: &PermissionId) -> bool {
        self.permissions.contains(permission)
    }

    /// 启用角色
    pub fn enable(&mut self) {
        self.status = RoleStatus::Active;
        self.updated_at = Utc::now();
    }

    /// 禁用角色
    pub fn disable(&mut self) {
        self.status = RoleStatus::Disabled;
        self.updated_at = Utc::now();
    }

    /// 检查是否启用
    pub fn is_active(&self) -> bool {
        self.status == RoleStatus::Active
    }
}

/// 角色管理器
pub struct RoleManager {
    /// 角色存储
    roles: Arc<RwLock<HashMap<RoleId, Role>>>,

    /// 名称到ID的映射
    name_index: Arc<RwLock<HashMap<String, RoleId>>>,

    /// 配置
    #[allow(dead_code)]
    config: crate::config::Config,
}

impl RoleManager {
    /// 创建新的角色管理器
    pub fn new() -> Self {
        Self::with_config(crate::config::Config::default())
    }

    /// 使用配置创建
    pub fn with_config(config: crate::config::Config) -> Self {
        let manager = Self {
            roles: Arc::new(RwLock::new(HashMap::new())),
            name_index: Arc::new(RwLock::new(HashMap::new())),
            config,
        };

        // 初始化默认角色
        manager.init_default_roles();
        manager
    }

    /// 初始化默认角色
    fn init_default_roles(&self) {
        // 注意：这里不能是async，所以我们用blocking方式
        // 实际应用中应该在异步上下文中调用
    }

    /// 创建角色
    pub async fn create_role(&self, name: String, role_type: RoleType) -> Result<RoleId> {
        // 检查名称是否已存在
        {
            let name_index = self.name_index.read().await;
            if name_index.contains_key(&name) {
                return Err(Error::Permission(format!("角色名称 '{}' 已存在", name)));
            }
        }

        let role = Role::new(name.clone(), role_type);
        let id = role.id.clone();

        let mut roles = self.roles.write().await;
        let mut name_index = self.name_index.write().await;

        name_index.insert(name, id.clone());
        roles.insert(id.clone(), role);

        Ok(id)
    }

    /// 获取角色
    pub async fn get_role(&self, id: &RoleId) -> Result<Role> {
        let roles = self.roles.read().await;
        roles.get(id).cloned().ok_or_else(|| Error::Permission(format!("角色 {} 不存在", id)))
    }

    /// 按名称获取角色
    pub async fn get_role_by_name(&self, name: &str) -> Result<Role> {
        let name_index = self.name_index.read().await;
        let id = name_index
            .get(name)
            .ok_or_else(|| Error::Permission(format!("角色名称 '{}' 不存在", name)))?;

        let roles = self.roles.read().await;
        roles.get(id).cloned().ok_or_else(|| Error::Permission(format!("角色 {} 不存在", id)))
    }

    /// 更新角色
    pub async fn update_role(&self, id: &RoleId, updates: RoleUpdate) -> Result<()> {
        // 先检查名称是否已存在
        if let Some(ref name) = updates.name {
            let name_index = self.name_index.read().await;
            if name_index.contains_key(name) && name_index.get(name) != Some(id) {
                return Err(Error::Permission(format!("角色名称 '{}' 已存在", name)));
            }
        }

        // 获取旧名称（用于更新索引）
        let old_name = {
            let roles = self.roles.read().await;
            roles.get(id).map(|r| r.name.clone())
        };

        // 更新角色
        let mut roles = self.roles.write().await;
        let role =
            roles.get_mut(id).ok_or_else(|| Error::Permission(format!("角色 {} 不存在", id)))?;

        if let Some(name) = updates.name {
            // 更新名称索引
            let mut name_index = self.name_index.write().await;
            if let Some(old) = old_name {
                name_index.remove(&old);
            }
            name_index.insert(name.clone(), id.clone());
            role.name = name;
        }

        if let Some(description) = updates.description {
            role.description = Some(description);
        }

        role.updated_at = Utc::now();
        Ok(())
    }

    /// 删除角色
    pub async fn delete_role(&self, id: &RoleId) -> Result<()> {
        let mut roles = self.roles.write().await;
        let role =
            roles.remove(id).ok_or_else(|| Error::Permission(format!("角色 {} 不存在", id)))?;

        let mut name_index = self.name_index.write().await;
        name_index.remove(&role.name);

        Ok(())
    }

    /// 给角色添加权限
    pub async fn add_permission(&self, role_id: &RoleId, permission: PermissionId) -> Result<()> {
        let mut roles = self.roles.write().await;
        let role = roles
            .get_mut(role_id)
            .ok_or_else(|| Error::Permission(format!("角色 {} 不存在", role_id)))?;
        role.add_permission(permission);
        Ok(())
    }

    /// 从角色移除权限
    pub async fn remove_permission(
        &self,
        role_id: &RoleId,
        permission: &PermissionId,
    ) -> Result<()> {
        let mut roles = self.roles.write().await;
        let role = roles
            .get_mut(role_id)
            .ok_or_else(|| Error::Permission(format!("角色 {} 不存在", role_id)))?;
        role.remove_permission(permission);
        Ok(())
    }

    /// 检查角色是否有权限
    pub async fn has_permission(&self, role_id: &RoleId, permission: &PermissionId) -> bool {
        if let Ok(role) = self.get_role(role_id).await {
            role.has_permission(permission) && role.is_active()
        } else {
            false
        }
    }

    /// 启用角色
    pub async fn enable_role(&self, id: &RoleId) -> Result<()> {
        let mut roles = self.roles.write().await;
        let role =
            roles.get_mut(id).ok_or_else(|| Error::Permission(format!("角色 {} 不存在", id)))?;
        role.enable();
        Ok(())
    }

    /// 禁用角色
    pub async fn disable_role(&self, id: &RoleId) -> Result<()> {
        let mut roles = self.roles.write().await;
        let role =
            roles.get_mut(id).ok_or_else(|| Error::Permission(format!("角色 {} 不存在", id)))?;
        role.disable();
        Ok(())
    }

    /// 列出所有角色
    pub async fn list_roles(&self) -> Vec<Role> {
        let roles = self.roles.read().await;
        roles.values().cloned().collect()
    }

    /// 统计角色数量
    pub async fn count(&self) -> usize {
        let roles = self.roles.read().await;
        roles.len()
    }
}

impl Default for RoleManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 角色更新信息
#[derive(Debug, Clone, Default)]
pub struct RoleUpdate {
    /// 新名称
    pub name: Option<String>,
    /// 新描述
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_role() {
        let manager = RoleManager::new();
        let id = manager.create_role("admin".to_string(), RoleType::Admin).await.unwrap();

        assert!(!id.is_empty());
        let role = manager.get_role(&id).await.unwrap();
        assert_eq!(role.name, "admin");
    }

    #[tokio::test]
    async fn test_get_role_by_name() {
        let manager = RoleManager::new();
        manager.create_role("admin".to_string(), RoleType::Admin).await.unwrap();

        let role = manager.get_role_by_name("admin").await.unwrap();
        assert_eq!(role.name, "admin");
    }

    #[tokio::test]
    async fn test_add_permission() {
        let manager = RoleManager::new();
        let id = manager.create_role("user".to_string(), RoleType::User).await.unwrap();

        manager.add_permission(&id, "read".to_string()).await.unwrap();

        let role = manager.get_role(&id).await.unwrap();
        assert!(role.has_permission(&"read".to_string()));
    }

    #[tokio::test]
    async fn test_remove_permission() {
        let manager = RoleManager::new();
        let id = manager.create_role("user".to_string(), RoleType::User).await.unwrap();

        manager.add_permission(&id, "read".to_string()).await.unwrap();
        manager.remove_permission(&id, &"read".to_string()).await.unwrap();

        let role = manager.get_role(&id).await.unwrap();
        assert!(!role.has_permission(&"read".to_string()));
    }

    #[tokio::test]
    async fn test_delete_role() {
        let manager = RoleManager::new();
        let id = manager.create_role("temp".to_string(), RoleType::Guest).await.unwrap();

        manager.delete_role(&id).await.unwrap();
        let result = manager.get_role(&id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_enable_disable_role() {
        let manager = RoleManager::new();
        let id = manager.create_role("user".to_string(), RoleType::User).await.unwrap();

        manager.disable_role(&id).await.unwrap();
        let role = manager.get_role(&id).await.unwrap();
        assert!(!role.is_active());

        manager.enable_role(&id).await.unwrap();
        let role = manager.get_role(&id).await.unwrap();
        assert!(role.is_active());
    }

    #[tokio::test]
    async fn test_duplicate_name() {
        let manager = RoleManager::new();
        manager.create_role("admin".to_string(), RoleType::Admin).await.unwrap();

        let result = manager.create_role("admin".to_string(), RoleType::Admin).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_roles() {
        let manager = RoleManager::new();
        manager.create_role("admin".to_string(), RoleType::Admin).await.unwrap();
        manager.create_role("user".to_string(), RoleType::User).await.unwrap();

        let roles = manager.list_roles().await;
        assert_eq!(roles.len(), 2);
    }

    #[test]
    fn test_role_creation() {
        let role = Role::new("test".to_string(), RoleType::Custom("custom".to_string()));
        assert!(!role.id.is_empty());
        assert_eq!(role.name, "test");
        assert!(role.is_active());
    }

    #[test]
    fn test_role_permissions() {
        let mut role = Role::new("test".to_string(), RoleType::User);
        role.add_permission("read".to_string());
        role.add_permission("write".to_string());

        assert!(role.has_permission(&"read".to_string()));
        assert!(role.has_permission(&"write".to_string()));
        assert!(!role.has_permission(&"delete".to_string()));

        role.remove_permission(&"write".to_string());
        assert!(!role.has_permission(&"write".to_string()));
    }
}
