use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::infra::path_manager::PathManager;
use crate::platform::PlatformTrait;
use crate::utils::error::ZzmError;

/// zzm 配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZzmConfig {
    /// 默认 Zig 版本通道 (stable/nightly)
    #[serde(default)]
    pub default_channel: Option<String>,

    /// 安装 Zig 时自动安装对应 ZLS
    #[serde(default)]
    pub auto_install_zls: bool,

    /// 安装后自动设置全局默认版本
    #[serde(default)]
    pub auto_use: bool,

    /// 下载时使用的镜像源 URL 前缀
    #[serde(default)]
    pub mirror_url: Option<String>,

    /// 自定义安装目录（覆盖平台默认值）
    #[serde(default)]
    pub install_dir: Option<String>,

    /// 并行下载数
    #[serde(default = "default_parallel")]
    pub parallel_downloads: u32,

    /// 是否验证 SSL 证书
    #[serde(default = "default_true")]
    pub verify_ssl: bool,

    /// IDE 集成配置
    #[serde(default)]
    pub ide: IdeConfig,
}

const fn default_parallel() -> u32 {
    4
}

const fn default_true() -> bool {
    true
}

impl Default for ZzmConfig {
    fn default() -> Self {
        Self {
            default_channel: None,
            auto_install_zls: false,
            auto_use: false,
            mirror_url: None,
            install_dir: None,
            parallel_downloads: default_parallel(),
            verify_ssl: default_true(),
            ide: IdeConfig::default(),
        }
    }
}

/// VS Code 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsCodeConfig {
    /// 设置自动更新
    #[serde(default)]
    pub auto_update: bool,

    /// 自动添加 zig.zls.path 到 settings.json
    #[serde(default = "default_true")]
    pub set_zls_path: bool,

    /// settings.json 自定义路径
    #[serde(default)]
    pub settings_path: Option<String>,
}

impl Default for VsCodeConfig {
    fn default() -> Self {
        Self {
            auto_update: false,
            set_zls_path: true, // 这是我们想要的默认值
            settings_path: None,
        }
    }
}

/// IDE 集成配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeConfig {
    /// VS Code 配置
    #[serde(default)]
    pub vscode: VsCodeConfig,
}

impl Default for IdeConfig {
    fn default() -> Self {
        Self {
            vscode: VsCodeConfig::default(),
        }
    }
}

/// 配置管理器
///
/// 读取、写入和管理 zzm 的 TOML 配置文件
pub struct ConfigManager {
    path_manager: PathManager,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new(platform: Box<dyn PlatformTrait>) -> Self {
        let path_manager = PathManager::new(platform);
        Self { path_manager }
    }

    /// 获取配置文件路径
    pub fn config_path(&self) -> PathBuf {
        self.path_manager.config_file_path()
    }

    /// 读取配置文件，若不存在则返回默认配置
    pub fn load(&self) -> Result<ZzmConfig, ZzmError> {
        let path = self.config_path();
        if !path.exists() {
            return Ok(ZzmConfig::default());
        }

        let content = std::fs::read_to_string(&path).map_err(|e| ZzmError::ConfigError {
            path: path.display().to_string(),
            reason: format!("无法读取配置文件: {e}"),
        })?;

        let config: ZzmConfig = toml::from_str(&content).map_err(|e| ZzmError::ConfigError {
            path: path.display().to_string(),
            reason: format!("配置文件格式错误: {e}"),
        })?;

        Ok(config)
    }

    /// 保存配置到文件
    pub fn save(&self, config: &ZzmConfig) -> Result<(), ZzmError> {
        let path = self.config_path();

        // 确保父目录存在
        if let Some(parent) = path.parent()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent).map_err(|e| ZzmError::ConfigError {
                path: parent.display().to_string(),
                reason: format!("无法创建配置目录: {e}"),
            })?;
        }

        let content = toml::to_string_pretty(config).map_err(|e| ZzmError::ConfigError {
            path: path.display().to_string(),
            reason: format!("配置序列化失败: {e}"),
        })?;

        std::fs::write(&path, content).map_err(|e| ZzmError::ConfigError {
            path: path.display().to_string(),
            reason: format!("无法写入配置文件: {e}"),
        })?;

        Ok(())
    }

    /// 辅助：从 serde_json::Value 按点分隔路径获取值
    fn get_nested_value<'a>(
        value: &'a serde_json::Value,
        path: &str,
    ) -> Option<&'a serde_json::Value> {
        let mut current = value;
        for key in path.split('.') {
            match current.get(key) {
                Some(next) => current = next,
                None => return None,
            }
        }
        Some(current)
    }

    /// 辅助：在 serde_json::Value 中设置嵌套路径的值
    fn set_nested_value(
        value: &mut serde_json::Value,
        path: &str,
        new_value: serde_json::Value,
    ) -> Result<(), ZzmError> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = value;

        // 遍历除最后一个部分之外的所有部分
        for (_i, &key) in parts.iter().enumerate().take(parts.len() - 1) {
            // 如果当前不是对象，则转换为对象
            if !current.is_object() {
                *current = serde_json::Value::Object(serde_json::Map::new());
            }
            current = current
                .as_object_mut()
                .ok_or_else(|| ZzmError::ConfigError {
                    path: path.to_string(),
                    reason: "无法设置嵌套值".to_string(),
                })?
                .entry(key)
                .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
        }

        // 设置最后一个部分的值
        let last_key = parts.last().ok_or_else(|| ZzmError::ConfigError {
            path: path.to_string(),
            reason: "无效的配置项路径".to_string(),
        })?;

        if !current.is_object() {
            *current = serde_json::Value::Object(serde_json::Map::new());
        }
        current
            .as_object_mut()
            .ok_or_else(|| ZzmError::ConfigError {
                path: path.to_string(),
                reason: "无法设置值".to_string(),
            })?
            .insert(last_key.to_string(), new_value);

        Ok(())
    }

    /// 辅助：递归收集所有配置路径和值
    fn collect_config_entries(
        prefix: &str,
        value: &serde_json::Value,
        entries: &mut Vec<(String, String)>,
    ) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, sub_value) in map {
                    let new_prefix = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };
                    Self::collect_config_entries(&new_prefix, sub_value, entries);
                }
            }
            serde_json::Value::String(s) => entries.push((prefix.to_string(), s.clone())),
            serde_json::Value::Bool(b) => entries.push((prefix.to_string(), b.to_string())),
            serde_json::Value::Number(n) => {
                entries.push((prefix.to_string(), n.to_string()));
            }
            _ => {} // 忽略不支持的类型
        }
    }

    /// 获取单个配置项的值
    ///
    /// 支持点分隔的路径，如 "`ide.vscode.auto_update`"
    pub fn get(&self, key: &str) -> Result<Option<String>, ZzmError> {
        let config = self.load()?;
        // 先序列化为 JSON Value
        let mut config_json = serde_json::to_value(config).map_err(|e| ZzmError::ConfigError {
            path: key.to_string(),
            reason: format!("序列化配置失败: {e}"),
        })?;

        // 合并默认值（因为 serde_json::to_value 可能不包含 serde 的默认值）
        let default_config_json =
            serde_json::to_value(ZzmConfig::default()).map_err(|e| ZzmError::ConfigError {
                path: key.to_string(),
                reason: format!("序列化默认配置失败: {e}"),
            })?;
        Self::merge_json_values(&mut config_json, &default_config_json);

        // 获取嵌套值
        if let Some(value) = Self::get_nested_value(&config_json, key) {
            match value {
                serde_json::Value::String(s) => Ok(Some(s.clone())),
                serde_json::Value::Bool(b) => Ok(Some(b.to_string())),
                serde_json::Value::Number(n) => Ok(Some(n.to_string())),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// 辅助：递归合并两个 JSON 值，第二个值作为默认值
    fn merge_json_values(base: &mut serde_json::Value, defaults: &serde_json::Value) {
        if let (Some(base_obj), Some(default_obj)) =
            (base.as_object_mut(), defaults.as_object())
        {
            for (key, default_value) in default_obj {
                if !base_obj.contains_key(key) {
                    base_obj.insert(key.clone(), default_value.clone());
                } else if let Some(base_value) = base_obj.get_mut(key) {
                    Self::merge_json_values(base_value, default_value);
                }
            }
        }
    }

    /// 设置单个配置项的值
    ///
    /// 支持点分隔的路径，如 "`ide.vscode.auto_update`"
    pub fn set(&self, key: &str, value: &str) -> Result<(), ZzmError> {
        let mut config = self.load()?;
        let mut config_json = serde_json::to_value(&config).map_err(|e| ZzmError::ConfigError {
            path: key.to_string(),
            reason: format!("序列化配置失败: {e}"),
        })?;

        // 尝试解析值的类型（按优先级：bool -> u32 -> String）
        let parsed_value = if let Ok(b) = value.parse::<bool>() {
            serde_json::Value::Bool(b)
        } else if let Ok(n) = value.parse::<u32>() {
            serde_json::Value::Number(n.into())
        } else {
            serde_json::Value::String(value.to_string())
        };

        Self::set_nested_value(&mut config_json, key, parsed_value)?;

        // 反序列化回 ZzmConfig
        config = serde_json::from_value(config_json).map_err(|e| ZzmError::ConfigError {
            path: key.to_string(),
            reason: format!("无法解析配置: {e}"),
        })?;

        self.save(&config)
    }

    /// 列出所有配置项
    pub fn list_all(&self) -> Result<Vec<(String, String)>, ZzmError> {
        let config = self.load()?;
        let mut config_json = serde_json::to_value(config).map_err(|e| ZzmError::ConfigError {
            path: "".to_string(),
            reason: format!("序列化配置失败: {e}"),
        })?;

        let default_config_json =
            serde_json::to_value(ZzmConfig::default()).map_err(|e| ZzmError::ConfigError {
                path: "".to_string(),
                reason: format!("序列化默认配置失败: {e}"),
            })?;
        Self::merge_json_values(&mut config_json, &default_config_json);

        let mut entries = Vec::new();
        Self::collect_config_entries("", &config_json, &mut entries);

        Ok(entries)
    }

    /// 重置配置为默认值
    #[allow(dead_code)] // 预留: zzm config reset 命令
    pub fn reset(&self) -> Result<(), ZzmError> {
        let config = ZzmConfig::default();
        self.save(&config)
    }
}

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ZzmConfig::default();
        assert!(config.default_channel.is_none());
        assert!(!config.auto_install_zls);
        assert!(!config.auto_use);
        assert!(config.mirror_url.is_none());
        assert!(config.install_dir.is_none());
        assert_eq!(config.parallel_downloads, 4);
        assert!(config.verify_ssl);
    }

    #[test]
    fn test_config_serialization() {
        let config = ZzmConfig {
            default_channel: Some("stable".to_string()),
            auto_install_zls: true,
            ..Default::default()
        };

        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("default_channel = \"stable\""));
        assert!(toml_str.contains("auto_install_zls = true"));

        let parsed: ZzmConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.default_channel, Some("stable".to_string()));
        assert!(parsed.auto_install_zls);
    }

    #[test]
    fn test_ide_config_default() {
        let ide = IdeConfig::default();
        assert!(!ide.vscode.auto_update);
        // 现在 Default 实现正确设置了 set_zls_path 为 true
        assert!(ide.vscode.set_zls_path);
        assert!(ide.vscode.settings_path.is_none());
    }

    #[test]
    fn test_ide_config_deserialization_default() {
        // When deserializing from TOML, default_true() should apply
        let toml_str = "";
        let ide: IdeConfig = toml::from_str(toml_str).unwrap();
        assert!(!ide.vscode.auto_update);
        assert!(ide.vscode.set_zls_path); // default_true() takes effect on deserialization
        assert!(ide.vscode.settings_path.is_none());
    }

    #[test]
    fn test_config_with_mirror_url() {
        let config = ZzmConfig {
            mirror_url: Some("https://mirror.example.com/zig".to_string()),
            ..Default::default()
        };

        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("mirror_url = \"https://mirror.example.com/zig\""));

        let parsed: ZzmConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(
            parsed.mirror_url,
            Some("https://mirror.example.com/zig".to_string())
        );
    }

    #[test]
    fn test_config_with_install_dir() {
        let config = ZzmConfig {
            install_dir: Some("/opt/zig".to_string()),
            ..Default::default()
        };

        let parsed: ZzmConfig = toml::from_str(&toml::to_string_pretty(&config).unwrap()).unwrap();
        assert_eq!(parsed.install_dir, Some("/opt/zig".to_string()));
    }

    #[test]
    fn test_config_parallel_downloads_default() {
        let config = ZzmConfig::default();
        assert_eq!(config.parallel_downloads, 4);
    }

    #[test]
    fn test_config_verify_ssl_default() {
        let config = ZzmConfig::default();
        assert!(config.verify_ssl);
    }

    #[test]
    fn test_ide_config_serialization() {
        let ide = IdeConfig {
            vscode: VsCodeConfig {
                auto_update: true,
                settings_path: Some("/custom/path/settings.json".to_string()),
                ..Default::default()
            }
        };

        let json = serde_json::to_string(&ide).unwrap();
        let parsed: IdeConfig = serde_json::from_str(&json).unwrap();
        assert!(parsed.vscode.auto_update);
        assert_eq!(
            parsed.vscode.settings_path,
            Some("/custom/path/settings.json".to_string())
        );
    }

    #[test]
    fn test_config_all_fields_set() {
        let config = ZzmConfig {
            default_channel: Some("nightly".to_string()),
            auto_install_zls: true,
            auto_use: true,
            mirror_url: Some("https://mirror.example.com".to_string()),
            install_dir: Some("/opt/zzm".to_string()),
            parallel_downloads: 8,
            verify_ssl: false,
            ide: IdeConfig {
                vscode: VsCodeConfig {
                    auto_update: true,
                    set_zls_path: false,
                    settings_path: Some("/custom/path".to_string()),
                }
            },
        };

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: ZzmConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(parsed.default_channel, Some("nightly".to_string()));
        assert!(parsed.auto_install_zls);
        assert!(parsed.auto_use);
        assert_eq!(parsed.parallel_downloads, 8);
        assert!(!parsed.verify_ssl);
        assert!(parsed.ide.vscode.auto_update);
        assert!(!parsed.ide.vscode.set_zls_path);
    }

    #[test]
    fn test_config_toml_minimal() {
        // 测试从最小化的 TOML 解析
        let toml_str = "";
        let config: ZzmConfig = toml::from_str(toml_str).unwrap();
        assert!(!config.auto_install_zls);
        assert_eq!(config.parallel_downloads, 4);
    }
}
