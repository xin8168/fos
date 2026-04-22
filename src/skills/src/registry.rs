//! 技能注册表

use crate::error::{Result, SkillsError};
use crate::{Skill, SkillDefinition};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 技能注册表
pub struct SkillRegistry {
    skills: Arc<RwLock<HashMap<String, Skill>>>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self { skills: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn register(&self, definition: SkillDefinition) -> Result<String> {
        let id = definition.id.clone();
        let skill = Skill::new(definition);

        let mut skills = self.skills.write().await;
        skills.insert(id.clone(), skill);

        Ok(id)
    }

    pub async fn get(&self, id: &str) -> Result<Skill> {
        let skills = self.skills.read().await;
        skills.get(id).cloned().ok_or_else(|| SkillsError::NotFound(id.to_string()))
    }

    pub async fn list(&self) -> Vec<String> {
        let skills = self.skills.read().await;
        skills.keys().cloned().collect()
    }

    pub async fn count(&self) -> usize {
        let skills = self.skills.read().await;
        skills.len()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}
