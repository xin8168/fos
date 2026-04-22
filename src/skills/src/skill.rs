//! 技能实现

use crate::error::Result;
use crate::SkillDefinition;

/// 技能实例
#[derive(Clone)]
pub struct Skill {
    /// 技能定义（公开以支持 Clone）
    pub definition: SkillDefinition,
}

impl Skill {
    pub fn new(definition: SkillDefinition) -> Self {
        Self { definition }
    }

    pub fn id(&self) -> &str {
        &self.definition.id
    }

    pub fn name(&self) -> &str {
        &self.definition.name
    }

    pub fn steps(&self) -> &[String] {
        &self.definition.steps
    }

    pub fn is_supported(&self, device_type: &str) -> bool {
        self.definition.supported_devices.contains(&"*".to_string())
            || self.definition.supported_devices.contains(&device_type.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_creation() {
        let mut def = SkillDefinition::default();
        def.name = "测试技能".to_string();
        def.steps = vec!["步骤1".to_string()];

        let skill = Skill::new(def);
        assert_eq!(skill.name(), "测试技能");
        assert!(skill.is_supported("any"));
    }
}
