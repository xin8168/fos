//! 配置加载器

use crate::error::{ConfigError, Result};
use serde_yaml::Value;
use std::path::Path;

/// 配置加载器
pub struct ConfigLoader {
    /// 配置文件路径
    path: Option<std::path::PathBuf>,
}

impl ConfigLoader {
    /// 创建新的配置加载器
    pub fn new() -> Self {
        Self { path: None }
    }

    /// 设置配置文件路径
    pub fn with_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.path = Some(path.as_ref().to_path_buf());
        self
    }

    /// 从YAML文件加载
    pub fn from_yaml<P: AsRef<Path>>(path: P) -> Result<Value> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(ConfigError::FileNotFound(path.display().to_string()));
        }

        let content =
            std::fs::read_to_string(path).map_err(|e| ConfigError::IoError(e.to_string()))?;

        let config: Value =
            serde_yaml::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))?;

        Ok(config)
    }

    /// 从JSON文件加载
    pub fn from_json<P: AsRef<Path>>(path: P) -> Result<Value> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(ConfigError::FileNotFound(path.display().to_string()));
        }

        let content =
            std::fs::read_to_string(path).map_err(|e| ConfigError::IoError(e.to_string()))?;

        // JSON也是有效的YAML，直接解析
        let config: Value =
            serde_yaml::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))?;

        Ok(config)
    }

    /// 从字符串加载
    pub fn from_str(content: &str, format: ConfigFormat) -> Result<Value> {
        match format {
            ConfigFormat::Yaml | ConfigFormat::Json => {
                serde_yaml::from_str(content).map_err(|e| ConfigError::ParseError(e.to_string()))
            },
        }
    }

    /// 从环境变量加载
    pub fn from_env(prefix: &str) -> Result<Value> {
        let mut config = serde_yaml::Mapping::new();

        for (key, value) in std::env::vars() {
            if key.starts_with(prefix) {
                // 将 PREFIX_KEY__SUBKEY 格式转换为 key.subkey
                let config_key = key
                    .strip_prefix(prefix)
                    .unwrap_or(&key)
                    .trim_start_matches('_')
                    .replace("__", ".")
                    .to_lowercase();

                // 尝试解析为各种类型
                let parsed_value = Self::parse_env_value(&value);
                config.insert(Value::String(config_key), parsed_value);
            }
        }

        Ok(Value::Mapping(config))
    }

    /// 解析环境变量值
    fn parse_env_value(value: &str) -> Value {
        // 尝试解析为布尔值
        if let Ok(b) = value.parse::<bool>() {
            return Value::Bool(b);
        }
        // 尝试解析为整数
        if let Ok(i) = value.parse::<i64>() {
            return Value::Number(i.into());
        }
        // 尝试解析为浮点数
        if let Ok(f) = value.parse::<f64>() {
            return Value::Number(serde_yaml::Number::from(f));
        }
        // 默认为字符串
        Value::String(value.to_string())
    }

    /// 加载配置
    pub fn load(&self) -> Result<Value> {
        if let Some(ref path) = self.path {
            if path.extension().map_or(false, |ext| ext == "json") {
                Self::from_json(path)
            } else {
                Self::from_yaml(path)
            }
        } else {
            Ok(Value::Null)
        }
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// 配置格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    /// YAML格式
    Yaml,
    /// JSON格式
    Json,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loader_creation() {
        let loader = ConfigLoader::new();
        assert!(loader.path.is_none());
    }

    #[test]
    fn test_from_yaml_string() {
        let yaml = "server:\n  port: 8080\n";
        let config = ConfigLoader::from_str(yaml, ConfigFormat::Yaml).unwrap();
        assert!(config["server"]["port"].as_u64().unwrap() == 8080);
    }

    #[test]
    fn test_from_json_string() {
        let json = r#"{"server": {"port": 8080}}"#;
        let config = ConfigLoader::from_str(json, ConfigFormat::Json).unwrap();
        assert!(config["server"]["port"].as_u64().unwrap() == 8080);
    }

    #[test]
    fn test_file_not_found() {
        let result = ConfigLoader::from_yaml("/nonexistent/config.yaml");
        assert!(matches!(result, Err(ConfigError::FileNotFound(_))));
    }

    #[test]
    fn test_parse_env_value() {
        assert!(ConfigLoader::parse_env_value("true").is_bool());
        assert!(ConfigLoader::parse_env_value("123").is_number());
        assert!(ConfigLoader::parse_env_value("test").is_string());
    }
}
