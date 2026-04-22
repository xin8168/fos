//! FOS 6维锚定结构

use serde::{Deserialize, Serialize};

/// FOS 6维锚定结构
///
/// FOS 的核心设计：每一个执行动作必须有 6 个锚定点
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SixAnchor {
    /// 事件: 用户想要达成的目标
    pub event: String,

    /// 步骤: 明确的执行步骤序列
    pub steps: Vec<String>,

    /// 判断逻辑: 每一步的判断条件
    pub judgment_logic: String,

    /// 校验标准: 最终结果的成功标准
    pub verification_standard: String,

    /// 地点: 执行环境/设备
    pub location: String,

    /// 主体: 执行者身份
    pub subject: String,
}

impl SixAnchor {
    /// 创建新的 6 维锚定
    pub fn new(
        event: impl Into<String>,
        steps: Vec<String>,
        judgment_logic: impl Into<String>,
        verification_standard: impl Into<String>,
        location: impl Into<String>,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            event: event.into(),
            steps,
            judgment_logic: judgment_logic.into(),
            verification_standard: verification_standard.into(),
            location: location.into(),
            subject: subject.into(),
        }
    }

    /// 验证锚定是否有效
    pub fn validate(&self) -> bool {
        !self.event.is_empty()
            && !self.steps.is_empty()
            && !self.judgment_logic.is_empty()
            && !self.verification_standard.is_empty()
            && !self.location.is_empty()
            && !self.subject.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_six_anchor_creation() {
        let anchor = SixAnchor::new(
            "清理桌面文件",
            vec!["列出文件".into(), "筛选文件".into(), "移动文件".into()],
            "文件大小<100MB且7天未修改",
            "归档文件夹出现对应文件",
            "我的Windows电脑",
            "我",
        );

        assert_eq!(anchor.event, "清理桌面文件");
        assert_eq!(anchor.steps.len(), 3);
        assert!(anchor.validate());
    }

    #[test]
    fn test_six_anchor_validation() {
        let invalid_anchor = SixAnchor::new("", vec![], "", "", "", "");
        assert!(!invalid_anchor.validate());
    }
}
