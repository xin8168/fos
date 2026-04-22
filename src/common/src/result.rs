//! FOS 执行结果和步骤结果

use serde::{Deserialize, Serialize};

/// 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// 是否成功
    pub success: bool,

    /// 输出内容
    pub output: String,

    /// 错误信息
    pub error: Option<String>,

    /// 执行时间（毫秒）
    pub duration_ms: u64,

    /// 步骤结果
    pub step_results: Vec<StepResult>,
}

impl ExecutionResult {
    /// 创建成功的执行结果
    pub fn success(output: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            success: true,
            output: output.into(),
            error: None,
            duration_ms,
            step_results: Vec::new(),
        }
    }

    /// 创建失败的执行结果
    pub fn failure(error: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            success: false,
            output: String::new(),
            error: Some(error.into()),
            duration_ms,
            step_results: Vec::new(),
        }
    }

    /// 添加步骤结果
    pub fn with_step_result(mut self, step: StepResult) -> Self {
        self.step_results.push(step);
        self
    }
}

/// 步骤执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// 步骤索引
    pub step_index: usize,

    /// 是否成功
    pub success: bool,

    /// 输出
    pub output: String,

    /// 错误
    pub error: Option<String>,
}

impl StepResult {
    /// 创建成功的步骤结果
    pub fn success(step_index: usize, output: impl Into<String>) -> Self {
        Self { step_index, success: true, output: output.into(), error: None }
    }

    /// 创建失败的步骤结果
    pub fn failure(step_index: usize, error: impl Into<String>) -> Self {
        Self { step_index, success: false, output: String::new(), error: Some(error.into()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_result_success() {
        let result = ExecutionResult::success("操作完成", 100);
        assert!(result.success);
        assert_eq!(result.output, "操作完成");
        assert!(result.error.is_none());
        assert_eq!(result.duration_ms, 100);
    }

    #[test]
    fn test_execution_result_failure() {
        let result = ExecutionResult::failure("操作失败", 50);
        assert!(!result.success);
        assert_eq!(result.error, Some("操作失败".to_string()));
    }

    #[test]
    fn test_step_result_creation() {
        let success_step = StepResult::success(0, "步骤完成");
        assert!(success_step.success);
        assert_eq!(success_step.step_index, 0);

        let failure_step = StepResult::failure(1, "步骤失败");
        assert!(!failure_step.success);
        assert_eq!(failure_step.step_index, 1);
    }
}
