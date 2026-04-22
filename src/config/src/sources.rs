//! 配置源定义

use crate::error::{ConfigError, Result};
use serde_yaml::Value;
use std::path::PathBuf;

/// 配置源 trait
pub trait ConfigSource {
    /// 加载配置
    fn load(&self) -> Result<Value>;

    /// 获取源名称
    fn name(&self) -> &str;

    /// 获取源优先级（数字越大优先级越高）
    fn priority(&self) -> u8;
}

/// 文件配置源
pub struct FileSource {
    /// 文件路径
    path: PathBuf,
    /// 文件格式
    format: ConfigFileFormat,
}

/// 配置文件格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFileFormat {
    /// YAML格式
    Yaml,
    /// JSON格式
    Json,
    /// TOML格式
    Toml,
}

impl FileSource {
    /// 创建新的文件配置源
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        let path = path.into();
        let format = Self::detect_format(&path);
        Self { path, format }
    }

    /// 指定格式创建
    pub fn with_format<P: Into<PathBuf>>(path: P, format: ConfigFileFormat) -> Self {
        Self { path: path.into(), format }
    }

    /// 检测文件格式
    fn detect_format(path: &PathBuf) -> ConfigFileFormat {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| match ext.to_lowercase().as_str() {
                "yaml" | "yml" => ConfigFileFormat::Yaml,
                "json" => ConfigFileFormat::Json,
                "toml" => ConfigFileFormat::Toml,
                _ => ConfigFileFormat::Yaml,
            })
            .unwrap_or(ConfigFileFormat::Yaml)
    }

    /// 读取文件内容
    fn read_file(&self) -> Result<String> {
        if !self.path.exists() {
            return Err(ConfigError::FileNotFound(self.path.display().to_string()));
        }
        std::fs::read_to_string(&self.path).map_err(|e| ConfigError::IoError(e.to_string()))
    }
}

impl ConfigSource for FileSource {
    fn load(&self) -> Result<Value> {
        let content = self.read_file()?;

        match self.format {
            ConfigFileFormat::Yaml | ConfigFileFormat::Json => {
                serde_yaml::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))
            },
            ConfigFileFormat::Toml => {
                // TOML解析需要额外处理
                let value: toml::Value =
                    toml::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))?;
                // 转换为YAML Value
                let json = serde_json::to_string(&value)
                    .map_err(|e| ConfigError::ParseError(e.to_string()))?;
                serde_yaml::from_str(&json).map_err(|e| ConfigError::ParseError(e.to_string()))
            },
        }
    }

    fn name(&self) -> &str {
        "file"
    }

    fn priority(&self) -> u8 {
        100
    }
}

/// 环境变量配置源
pub struct EnvironmentSource {
    /// 环境变量前缀
    prefix: String,
}

impl EnvironmentSource {
    /// 创建新的环境变量配置源
    pub fn new() -> Self {
        Self { prefix: "FOS_".to_string() }
    }

    /// 使用自定义前缀
    pub fn with_prefix(prefix: &str) -> Self {
        Self { prefix: prefix.to_string() }
    }
}

impl Default for EnvironmentSource {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigSource for EnvironmentSource {
    fn load(&self) -> Result<Value> {
        let mut config = serde_yaml::Mapping::new();

        for (key, value) in std::env::vars() {
            if key.starts_with(&self.prefix) {
                // 将 PREFIX_KEY 格式转换为 key
                let config_key = key.strip_prefix(&self.prefix).unwrap_or(&key).to_lowercase();

                // 尝试解析值
                let parsed_value = self.parse_value(&value);

                // 嵌套键支持 (使用 __ 分隔)
                if config_key.contains("__") {
                    self.insert_nested(&mut config, &config_key, parsed_value);
                } else {
                    config.insert(Value::String(config_key), parsed_value);
                }
            }
        }

        Ok(Value::Mapping(config))
    }

    fn name(&self) -> &str {
        "environment"
    }

    fn priority(&self) -> u8 {
        200 // 环境变量优先级最高
    }
}

impl EnvironmentSource {
    /// 解析值
    fn parse_value(&self, value: &str) -> Value {
        // 布尔值
        if let Ok(b) = value.parse::<bool>() {
            return Value::Bool(b);
        }
        // 整数
        if let Ok(i) = value.parse::<i64>() {
            return Value::Number(i.into());
        }
        // 浮点数
        if let Ok(f) = value.parse::<f64>() {
            if let Some(n) = serde_yaml::Number::try_from(f).ok() {
                return Value::Number(n);
            }
        }
        Value::String(value.to_string())
    }

    /// 插入嵌套值
    fn insert_nested(&self, config: &mut serde_yaml::Mapping, key: &str, value: Value) {
        let parts: Vec<&str> = key.split("__").collect();

        if parts.len() == 1 {
            config.insert(Value::String(parts[0].to_string()), value);
        } else if parts.len() == 2 {
            let inner = config
                .entry(Value::String(parts[0].to_string()))
                .or_insert_with(|| Value::Mapping(serde_yaml::Mapping::new()));

            if let Value::Mapping(ref mut inner_map) = inner {
                inner_map.insert(Value::String(parts[1].to_string()), value);
            }
        }
        // 更深层次的嵌套暂不处理
    }
}

/// 默认值配置源
pub struct DefaultSource;

impl DefaultSource {
    /// 创建新的默认值配置源
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultSource {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigSource for DefaultSource {
    fn load(&self) -> Result<Value> {
        // 返回空配置，由 ConfigManager 的默认值处理
        Ok(Value::Mapping(serde_yaml::Mapping::new()))
    }

    fn name(&self) -> &str {
        "default"
    }

    fn priority(&self) -> u8 {
        0 // 默认值优先级最低
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_source_creation() {
        let source = FileSource::new("config.yaml");
        assert_eq!(source.format, ConfigFileFormat::Yaml);
    }

    #[test]
    fn test_file_source_json() {
        let source = FileSource::new("config.json");
        assert_eq!(source.format, ConfigFileFormat::Json);
    }

    #[test]
    fn test_file_source_priority() {
        let source = FileSource::new("config.yaml");
        assert_eq!(source.priority(), 100);
    }

    #[test]
    fn test_env_source_creation() {
        let source = EnvironmentSource::new();
        assert_eq!(source.prefix, "FOS_");
    }

    #[test]
    fn test_env_source_with_prefix() {
        let source = EnvironmentSource::with_prefix("MYAPP_");
        assert_eq!(source.prefix, "MYAPP_");
    }

    #[test]
    fn test_env_source_priority() {
        let source = EnvironmentSource::new();
        assert_eq!(source.priority(), 200);
    }

    #[test]
    fn test_env_parse_value() {
        let source = EnvironmentSource::new();

        assert!(matches!(source.parse_value("true"), Value::Bool(true)));
        assert!(matches!(source.parse_value("123"), Value::Number(_)));
        assert!(matches!(source.parse_value("hello"), Value::String(_)));
    }

    #[test]
    fn test_default_source() {
        let source = DefaultSource::new();
        assert_eq!(source.priority(), 0);
    }
}
