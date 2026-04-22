//! # Plugin模块 - 核心类型定义
//!
//! 定义插件系统的核心数据结构和枚举类型

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 插件元数据
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginMetadata {
    /// 插件唯一标识符
    pub id: String,
    /// 插件名称
    pub name: String,
    /// 插件版本
    pub version: String,
    /// 插件描述
    pub description: String,
    /// 插件作者
    pub author: Option<String>,
    /// 插件类型
    pub plugin_type: PluginType,
    /// 最小FOS版本要求
    pub min_fos_version: Option<String>,
    /// 依赖的其他插件
    pub dependencies: Vec<String>,
    /// 自定义配置项
    pub custom_config: HashMap<String, String>,
}

/// 插件类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PluginType {
    /// 存储插件
    Storage,
    /// 监控插件
    Monitor,
    /// 通知插件
    Notifier,
    /// 自定义插件
    Custom,
}

/// 插件状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PluginState {
    /// 未加载
    Unloaded,
    /// 已加载
    Loaded,
    /// 已初始化
    Initialized,
    /// 运行中
    Running,
    /// 已暂停
    Paused,
    /// 错误状态
    Error,
}

/// 插件状态详情
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginStatus {
    /// 插件ID
    pub plugin_id: String,
    /// 当前状态
    pub state: PluginState,
    /// 最后更新时间
    pub last_updated: i64,
    /// 错误信息（如果有）
    pub error_message: Option<String>,
    /// 统计信息
    pub stats: PluginStats,
}

/// 插件统计信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginStats {
    /// 加载时间（毫秒）
    pub load_time_ms: u64,
    /// 初始化时间（毫秒）
    pub init_time_ms: u64,
    /// 执行次数
    pub execution_count: u64,
    /// 最后执行时间
    pub last_execution_time: Option<i64>,
    /// 错误次数
    pub error_count: u64,
}

impl Default for PluginStats {
    fn default() -> Self {
        Self {
            load_time_ms: 0,
            init_time_ms: 0,
            execution_count: 0,
            last_execution_time: None,
            error_count: 0,
        }
    }
}

impl PluginStatus {
    /// 创建新的插件状态
    pub fn new(plugin_id: String) -> Self {
        Self {
            plugin_id,
            state: PluginState::Unloaded,
            last_updated: chrono::Utc::now().timestamp_millis(),
            error_message: None,
            stats: PluginStats::default(),
        }
    }

    /// 更新状态
    pub fn update_state(&mut self, state: PluginState) {
        self.state = state;
        self.last_updated = chrono::Utc::now().timestamp_millis();
    }

    /// 记录错误
    pub fn record_error(&mut self, error: String) {
        self.error_message = Some(error);
        self.state = PluginState::Error;
        self.stats.error_count += 1;
        self.last_updated = chrono::Utc::now().timestamp_millis();
    }

    /// 记录执行
    pub fn record_execution(&mut self) {
        self.stats.execution_count += 1;
        self.stats.last_execution_time = Some(chrono::Utc::now().timestamp_millis());
        self.last_updated = chrono::Utc::now().timestamp_millis();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_metadata_creation() {
        let metadata = PluginMetadata {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: Some("Test Author".to_string()),
            plugin_type: PluginType::Custom,
            min_fos_version: Some("1.0.0".to_string()),
            dependencies: vec![],
            custom_config: HashMap::new(),
        };

        assert_eq!(metadata.id, "test-plugin");
        assert_eq!(metadata.plugin_type, PluginType::Custom);
    }

    #[test]
    fn test_plugin_type_serialization() {
        let plugin_type = PluginType::Storage;
        let serialized = serde_json::to_string(&plugin_type).unwrap();
        assert_eq!(serialized, r#""storage""#);

        let deserialized: PluginType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, PluginType::Storage);
    }

    #[test]
    fn test_plugin_state_transitions() {
        let mut status = PluginStatus::new("test-plugin".to_string());
        assert_eq!(status.state, PluginState::Unloaded);

        status.update_state(PluginState::Loaded);
        assert_eq!(status.state, PluginState::Loaded);

        status.update_state(PluginState::Initialized);
        assert_eq!(status.state, PluginState::Initialized);

        status.update_state(PluginState::Running);
        assert_eq!(status.state, PluginState::Running);
    }

    #[test]
    fn test_plugin_error_handling() {
        let mut status = PluginStatus::new("test-plugin".to_string());
        status.update_state(PluginState::Running);

        status.record_error("Test error".to_string());
        assert_eq!(status.state, PluginState::Error);
        assert_eq!(status.error_message, Some("Test error".to_string()));
        assert_eq!(status.stats.error_count, 1);
    }

    #[test]
    fn test_plugin_execution_stats() {
        let mut status = PluginStatus::new("test-plugin".to_string());
        status.stats.execution_count = 0;

        status.record_execution();
        assert_eq!(status.stats.execution_count, 1);
        assert!(status.stats.last_execution_time.is_some());
    }

    #[test]
    fn test_plugin_stats_default() {
        let stats = PluginStats::default();
        assert_eq!(stats.load_time_ms, 0);
        assert_eq!(stats.init_time_ms, 0);
        assert_eq!(stats.execution_count, 0);
        assert!(stats.last_execution_time.is_none());
        assert_eq!(stats.error_count, 0);
    }

    #[test]
    fn test_plugin_status_serialization() {
        let status = PluginStatus::new("test-plugin".to_string());
        let serialized = serde_json::to_string(&status).unwrap();
        assert!(serialized.contains("test-plugin"));

        let deserialized: PluginStatus = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.plugin_id, "test-plugin");
    }

    #[test]
    fn test_plugin_metadata_with_dependencies() {
        let metadata = PluginMetadata {
            id: "plugin-a".to_string(),
            name: "Plugin A".to_string(),
            version: "1.0.0".to_string(),
            description: "Plugin with dependencies".to_string(),
            author: None,
            plugin_type: PluginType::Storage,
            min_fos_version: None,
            dependencies: vec!["plugin-b".to_string(), "plugin-c".to_string()],
            custom_config: HashMap::new(),
        };

        assert_eq!(metadata.dependencies.len(), 2);
        assert!(metadata.dependencies.contains(&"plugin-b".to_string()));
    }

    #[test]
    fn test_plugin_metadata_with_custom_config() {
        let mut custom_config = HashMap::new();
        custom_config.insert("timeout".to_string(), "30".to_string());
        custom_config.insert("retry".to_string(), "3".to_string());

        let metadata = PluginMetadata {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Plugin with custom config".to_string(),
            author: None,
            plugin_type: PluginType::Monitor,
            min_fos_version: None,
            dependencies: vec![],
            custom_config,
        };

        assert_eq!(metadata.custom_config.len(), 2);
        assert_eq!(metadata.custom_config.get("timeout"), Some(&"30".to_string()));
    }

    #[test]
    fn test_multiple_errors_accumulate() {
        let mut status = PluginStatus::new("test-plugin".to_string());

        status.record_error("Error 1".to_string());
        assert_eq!(status.stats.error_count, 1);

        status.update_state(PluginState::Running);
        status.record_error("Error 2".to_string());
        assert_eq!(status.stats.error_count, 2);
        assert_eq!(status.error_message, Some("Error 2".to_string()));
    }

    #[test]
    fn test_multiple_executions() {
        let mut status = PluginStatus::new("test-plugin".to_string());

        for i in 0..5 {
            status.record_execution();
            assert_eq!(status.stats.execution_count, i + 1);
        }

        assert_eq!(status.stats.execution_count, 5);
    }
}
