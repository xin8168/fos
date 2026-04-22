//! 设备适配模块
//!
//! 提供技能与设备之间的适配能力

use crate::definition::{EnhancedSkillDefinition, ParamDefinition, ParamType};
use crate::error::{Result, SkillsError};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;

/// 设备ID
pub type DeviceId = String;

/// 设备类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    /// 传感器
    Sensor,
    /// 执行器
    Actuator,
    /// 控制器
    Controller,
    /// 网关
    Gateway,
    /// 显示器
    Display,
    /// 存储设备
    Storage,
    /// 网络设备
    Network,
    /// 自定义
    Custom(String),
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::Sensor => write!(f, "sensor"),
            DeviceType::Actuator => write!(f, "actuator"),
            DeviceType::Controller => write!(f, "controller"),
            DeviceType::Gateway => write!(f, "gateway"),
            DeviceType::Display => write!(f, "display"),
            DeviceType::Storage => write!(f, "storage"),
            DeviceType::Network => write!(f, "network"),
            DeviceType::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}

/// 设备能力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapability {
    /// 能力名称
    pub name: String,
    /// 能力版本
    pub version: String,
    /// 参数定义
    pub parameters: Vec<ParamDefinition>,
    /// 输出定义
    pub outputs: Vec<String>,
}

impl DeviceCapability {
    /// 创建新能力
    pub fn new(name: String, version: String) -> Self {
        Self { name, version, parameters: Vec::new(), outputs: Vec::new() }
    }

    /// 添加参数
    pub fn add_parameter(&mut self, param: ParamDefinition) {
        self.parameters.push(param);
    }
}

/// 设备配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    /// 设备ID
    pub id: DeviceId,
    /// 设备名称
    pub name: String,
    /// 设备类型
    pub device_type: DeviceType,
    /// 固件版本
    pub firmware_version: String,
    /// 硬件版本
    pub hardware_version: String,
    /// 支持的能力
    pub capabilities: Vec<String>,
    /// 设备参数
    pub parameters: HashMap<String, serde_json::Value>,
    /// 连接状态
    pub connected: bool,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl DeviceConfig {
    /// 创建新设备配置
    pub fn new(id: DeviceId, name: String, device_type: DeviceType) -> Self {
        Self {
            id,
            name,
            device_type,
            firmware_version: "1.0.0".to_string(),
            hardware_version: "1.0".to_string(),
            capabilities: Vec::new(),
            parameters: HashMap::new(),
            connected: false,
            metadata: HashMap::new(),
        }
    }

    /// 设置固件版本
    pub fn with_firmware(mut self, version: String) -> Self {
        self.firmware_version = version;
        self
    }

    /// 添加能力
    pub fn add_capability(&mut self, capability: String) {
        self.capabilities.push(capability);
    }

    /// 设置参数
    pub fn set_parameter(&mut self, key: String, value: serde_json::Value) {
        self.parameters.insert(key, value);
    }

    /// 检查是否支持能力
    pub fn supports_capability(&self, capability: &str) -> bool {
        self.capabilities.contains(&capability.to_string())
    }
}

/// 适配规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationRule {
    /// 规则ID
    pub id: String,
    /// 规则名称
    pub name: String,
    /// 源设备类型
    pub source_device_type: DeviceType,
    /// 目标设备类型
    pub target_device_type: DeviceType,
    /// 参数映射
    pub parameter_mapping: HashMap<String, String>,
    /// 命令映射
    pub command_mapping: HashMap<String, String>,
    /// 条件表达式
    pub condition: Option<String>,
    /// 优先级
    pub priority: i32,
}

impl AdaptationRule {
    /// 创建新规则
    pub fn new(name: String, source: DeviceType, target: DeviceType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            source_device_type: source,
            target_device_type: target,
            parameter_mapping: HashMap::new(),
            command_mapping: HashMap::new(),
            condition: None,
            priority: 0,
        }
    }

    /// 添加参数映射
    pub fn add_parameter_mapping(&mut self, source: String, target: String) {
        self.parameter_mapping.insert(source, target);
    }

    /// 添加命令映射
    pub fn add_command_mapping(&mut self, source: String, target: String) {
        self.command_mapping.insert(source, target);
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// 应用参数映射
    pub fn apply_parameter_mapping(
        &self,
        params: &HashMap<String, serde_json::Value>,
    ) -> HashMap<String, serde_json::Value> {
        let mut result = HashMap::new();
        for (key, value) in params {
            let mapped_key =
                self.parameter_mapping.get(key).cloned().unwrap_or_else(|| key.clone());
            result.insert(mapped_key, value.clone());
        }
        result
    }

    /// 应用命令映射
    pub fn apply_command_mapping(&self, command: &str) -> String {
        self.command_mapping.get(command).cloned().unwrap_or_else(|| command.to_string())
    }
}

/// 设备适配器
pub struct DeviceAdapter {
    /// 适配规则列表
    rules: Vec<AdaptationRule>,
}

impl DeviceAdapter {
    /// 创建新适配器
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// 添加适配规则
    pub fn add_rule(&mut self, rule: AdaptationRule) {
        self.rules.push(rule);
    }

    /// 查找适配规则
    pub fn find_rule(&self, source: &DeviceType, target: &DeviceType) -> Option<&AdaptationRule> {
        self.rules
            .iter()
            .filter(|r| r.source_device_type == *source && r.target_device_type == *target)
            .max_by_key(|r| r.priority)
    }

    /// 适配参数
    pub fn adapt_parameters(
        &self,
        source: &DeviceType,
        target: &DeviceType,
        params: &HashMap<String, serde_json::Value>,
    ) -> HashMap<String, serde_json::Value> {
        if let Some(rule) = self.find_rule(source, target) {
            rule.apply_parameter_mapping(params)
        } else {
            params.clone()
        }
    }

    /// 适配命令
    pub fn adapt_command(&self, source: &DeviceType, target: &DeviceType, command: &str) -> String {
        if let Some(rule) = self.find_rule(source, target) {
            rule.apply_command_mapping(command)
        } else {
            command.to_string()
        }
    }

    /// 检查设备兼容性
    pub fn check_compatibility(
        &self,
        skill: &EnhancedSkillDefinition,
        device: &DeviceConfig,
    ) -> Result<bool> {
        // 检查设备约束
        let compatible = skill.is_compatible_with(
            &device.device_type.to_string(),
            &device.firmware_version,
            &device.capabilities,
        );

        if !compatible {
            return Err(SkillsError::DeviceIncompatible(format!(
                "设备 {} 不满足技能 {} 的约束",
                device.name, skill.name
            )));
        }

        Ok(true)
    }

    /// 获取所有规则
    pub fn get_rules(&self) -> &[AdaptationRule] {
        &self.rules
    }
}

impl Default for DeviceAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// 设备注册表
pub struct DeviceRegistry {
    /// 设备列表
    devices: HashMap<DeviceId, DeviceConfig>,
}

impl DeviceRegistry {
    /// 创建新注册表
    pub fn new() -> Self {
        Self { devices: HashMap::new() }
    }

    /// 注册设备
    pub fn register(&mut self, device: DeviceConfig) -> DeviceId {
        let id = device.id.clone();
        self.devices.insert(id.clone(), device);
        id
    }

    /// 获取设备
    pub fn get(&self, id: &DeviceId) -> Option<&DeviceConfig> {
        self.devices.get(id)
    }

    /// 获取设备（可变）
    pub fn get_mut(&mut self, id: &DeviceId) -> Option<&mut DeviceConfig> {
        self.devices.get_mut(id)
    }

    /// 移除设备
    pub fn remove(&mut self, id: &DeviceId) -> Option<DeviceConfig> {
        self.devices.remove(id)
    }

    /// 获取所有设备
    pub fn list(&self) -> Vec<&DeviceConfig> {
        self.devices.values().collect()
    }

    /// 按类型获取设备
    pub fn get_by_type(&self, device_type: &DeviceType) -> Vec<&DeviceConfig> {
        self.devices.values().filter(|d| d.device_type == *device_type).collect()
    }

    /// 统计设备数量
    pub fn count(&self) -> usize {
        self.devices.len()
    }
}

impl Default for DeviceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_type_display() {
        assert_eq!(format!("{}", DeviceType::Sensor), "sensor");
        assert_eq!(format!("{}", DeviceType::Custom("special".to_string())), "custom:special");
    }

    #[test]
    fn test_device_config() {
        let mut config =
            DeviceConfig::new("device-1".to_string(), "温度传感器".to_string(), DeviceType::Sensor);
        config.add_capability("temperature_read".to_string());

        assert!(config.supports_capability("temperature_read"));
        assert!(!config.supports_capability("humidity_read"));
    }

    #[test]
    fn test_adaptation_rule() {
        let mut rule =
            AdaptationRule::new("传感器适配".to_string(), DeviceType::Sensor, DeviceType::Gateway);
        rule.add_parameter_mapping("temp".to_string(), "temperature".to_string());
        rule.add_command_mapping("read".to_string(), "fetch_data".to_string());

        let params = HashMap::from([("temp".to_string(), serde_json::json!(25))]);
        let adapted = rule.apply_parameter_mapping(&params);
        assert!(adapted.contains_key("temperature"));
        assert!(!adapted.contains_key("temp"));

        assert_eq!(rule.apply_command_mapping("read"), "fetch_data");
    }

    #[test]
    fn test_device_adapter() {
        let mut adapter = DeviceAdapter::new();

        let mut rule =
            AdaptationRule::new("适配规则".to_string(), DeviceType::Sensor, DeviceType::Controller);
        rule.add_parameter_mapping("value".to_string(), "reading".to_string());
        adapter.add_rule(rule);

        let params = HashMap::from([("value".to_string(), serde_json::json!(100))]);
        let adapted =
            adapter.adapt_parameters(&DeviceType::Sensor, &DeviceType::Controller, &params);
        assert!(adapted.contains_key("reading"));
    }

    #[test]
    fn test_device_registry() {
        let mut registry = DeviceRegistry::new();

        let device =
            DeviceConfig::new("device-1".to_string(), "测试设备".to_string(), DeviceType::Sensor);

        let id = registry.register(device);
        assert_eq!(registry.count(), 1);

        let retrieved = registry.get(&id);
        assert!(retrieved.is_some());

        registry.remove(&id);
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_device_registry_by_type() {
        let mut registry = DeviceRegistry::new();

        registry.register(DeviceConfig::new(
            "sensor-1".to_string(),
            "传感器1".to_string(),
            DeviceType::Sensor,
        ));
        registry.register(DeviceConfig::new(
            "actuator-1".to_string(),
            "执行器1".to_string(),
            DeviceType::Actuator,
        ));

        let sensors = registry.get_by_type(&DeviceType::Sensor);
        assert_eq!(sensors.len(), 1);

        let actuators = registry.get_by_type(&DeviceType::Actuator);
        assert_eq!(actuators.len(), 1);
    }

    #[test]
    fn test_capability() {
        let mut cap = DeviceCapability::new("temperature_read".to_string(), "1.0.0".to_string());
        cap.add_parameter(ParamDefinition::new("interval".to_string(), ParamType::Integer, false));

        assert_eq!(cap.name, "temperature_read");
        assert_eq!(cap.parameters.len(), 1);
    }
}
