//! FOS 4要素执行结构

use serde::{Deserialize, Serialize};

/// FOS 4要素执行结构
///
/// 每个执行步骤必须包含的 4 个要素
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct FourElement {
    /// 动作: 具体的操作行为
    pub action: String,

    /// 对象: 操作的目标对象
    pub target: String,

    /// 条件: 执行前提条件
    pub condition: String,

    /// 结果: 预期的执行结果
    pub expected_result: String,
}

impl FourElement {
    /// 创建新的 4 要素
    pub fn new(
        action: impl Into<String>,
        target: impl Into<String>,
        condition: impl Into<String>,
        expected_result: impl Into<String>,
    ) -> Self {
        Self {
            action: action.into(),
            target: target.into(),
            condition: condition.into(),
            expected_result: expected_result.into(),
        }
    }

    /// 验证要素是否有效
    pub fn validate(&self) -> bool {
        !self.action.is_empty()
            && !self.target.is_empty()
            && !self.condition.is_empty()
            && !self.expected_result.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_four_element_creation() {
        let element =
            FourElement::new("列出文件", "桌面文件列表", "桌面存在文件", "获得完整文件列表");

        assert_eq!(element.action, "列出文件");
        assert!(element.validate());
    }

    #[test]
    fn test_four_element_validation() {
        let invalid_element = FourElement::new("", "", "", "");
        assert!(!invalid_element.validate());
    }
}
