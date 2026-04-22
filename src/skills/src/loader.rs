//! 技能加载器

use crate::error::Result;
use crate::SkillDefinition;

pub struct SkillLoader;

impl SkillLoader {
    pub fn from_yaml(content: &str) -> Result<SkillDefinition> {
        let skill: SkillDefinition = serde_yaml::from_str(content)?;
        Ok(skill)
    }

    pub fn from_json(content: &str) -> Result<SkillDefinition> {
        let skill: SkillDefinition = serde_json::from_str(content)?;
        Ok(skill)
    }
}
