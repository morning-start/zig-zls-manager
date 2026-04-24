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

fn default_parallel() -> u32 {
    4
}

fn default_true() -> bool {
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

/// IDE 集成配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IdeConfig {
    /// VS Code 设置自动更新
    #[serde(default)]
    pub vscode_auto_update: bool,

    /// 自动添加 zig.zls.path 到 VS Code settings.json
    #[serde(default = "default_true")]
    pub vscode_set_zls_path: bool,

    /// VS Code settings.json 自定义路径
    #[serde(default)]
    pub vscode_settings_path: Option<String>,
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
            reason: format!("无法读取配置文件: {}", e),
        })?;

        let config: ZzmConfig =
            toml::from_str(&content).map_err(|e| ZzmError::ConfigError {
                path: path.display().to_string(),
                reason: format!("配置文件格式错误: {}", e),
            })?;

        Ok(config)
    }

    /// 保存配置到文件
    pub fn save(&self, config: &ZzmConfig) -> Result<(), ZzmError> {
        let path = self.config_path();

        // 确保父目录存在
        if let Some(parent) = path.parent()
            && !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| ZzmError::ConfigError {
                    path: parent.display().to_string(),
                    reason: format!("无法创建配置目录: {}", e),
                })?;
            }

        let content =
            toml::to_string_pretty(config).map_err(|e| ZzmError::ConfigError {
                path: path.display().to_string(),
                reason: format!("配置序列化失败: {}", e),
            })?;

        std::fs::write(&path, content).map_err(|e| ZzmError::ConfigError {
            path: path.display().to_string(),
            reason: format!("无法写入配置文件: {}", e),
        })?;

        Ok(())
    }

    /// 获取单个配置项的值
    ///
    /// 支持点分隔的路径，如 "ide.vscode_auto_update"
    pub fn get(&self, key: &str) -> Result<Option<String>, ZzmError> {
        let config = self.load()?;
        let value = match key {
            "default_channel" => config.default_channel,
            "auto_install_zls" => Some(config.auto_install_zls.to_string()),
            "auto_use" => Some(config.auto_use.to_string()),
            "mirror_url" => config.mirror_url,
            "install_dir" => config.install_dir,
            "parallel_downloads" => Some(config.parallel_downloads.to_string()),
            "verify_ssl" => Some(config.verify_ssl.to_string()),
            "ide.vscode_auto_update" => Some(config.ide.vscode_auto_update.to_string()),
            "ide.vscode_set_zls_path" => Some(config.ide.vscode_set_zls_path.to_string()),
            "ide.vscode_settings_path" => config.ide.vscode_settings_path,
            _ => return Ok(None),
        };
        Ok(value)
    }

    /// 设置单个配置项的值
    ///
    /// 支持点分隔的路径，如 "ide.vscode_auto_update"
    pub fn set(&self, key: &str, value: &str) -> Result<(), ZzmError> {
        let mut config = self.load()?;

        match key {
            "default_channel" => config.default_channel = Some(value.to_string()),
            "auto_install_zls" => {
                config.auto_install_zls = value
                    .parse()
                    .map_err(|_| ZzmError::ConfigError {
                        path: key.to_string(),
                        reason: format!("期望布尔值，得到 '{}'", value),
                    })?;
            }
            "auto_use" => {
                config.auto_use = value
                    .parse()
                    .map_err(|_| ZzmError::ConfigError {
                        path: key.to_string(),
                        reason: format!("期望布尔值，得到 '{}'", value),
                    })?;
            }
            "mirror_url" => config.mirror_url = Some(value.to_string()),
            "install_dir" => config.install_dir = Some(value.to_string()),
            "parallel_downloads" => {
                config.parallel_downloads = value.parse().map_err(|_| ZzmError::ConfigError {
                    path: key.to_string(),
                    reason: format!("期望正整数，得到 '{}'", value),
                })?;
            }
            "verify_ssl" => {
                config.verify_ssl = value
                    .parse()
                    .map_err(|_| ZzmError::ConfigError {
                        path: key.to_string(),
                        reason: format!("期望布尔值，得到 '{}'", value),
                    })?;
            }
            "ide.vscode_auto_update" => {
                config.ide.vscode_auto_update = value
                    .parse()
                    .map_err(|_| ZzmError::ConfigError {
                        path: key.to_string(),
                        reason: format!("期望布尔值，得到 '{}'", value),
                    })?;
            }
            "ide.vscode_set_zls_path" => {
                config.ide.vscode_set_zls_path = value
                    .parse()
                    .map_err(|_| ZzmError::ConfigError {
                        path: key.to_string(),
                        reason: format!("期望布尔值，得到 '{}'", value),
                    })?;
            }
            "ide.vscode_settings_path" => {
                config.ide.vscode_settings_path = Some(value.to_string())
            }
            _ => {
                return Err(ZzmError::ConfigError {
                    path: key.to_string(),
                    reason: format!("未知的配置项: {}", key),
                })
            }
        }

        self.save(&config)
    }

    /// 列出所有配置项
    pub fn list_all(&self) -> Result<Vec<(String, String)>, ZzmError> {
        let config = self.load()?;
        let mut items = Vec::new();

        if let Some(ref v) = config.default_channel {
            items.push(("default_channel".to_string(), v.clone()));
        }
        items.push((
            "auto_install_zls".to_string(),
            config.auto_install_zls.to_string(),
        ));
        items.push(("auto_use".to_string(), config.auto_use.to_string()));
        if let Some(ref v) = config.mirror_url {
            items.push(("mirror_url".to_string(), v.clone()));
        }
        if let Some(ref v) = config.install_dir {
            items.push(("install_dir".to_string(), v.clone()));
        }
        items.push((
            "parallel_downloads".to_string(),
            config.parallel_downloads.to_string(),
        ));
        items.push(("verify_ssl".to_string(), config.verify_ssl.to_string()));
        items.push((
            "ide.vscode_auto_update".to_string(),
            config.ide.vscode_auto_update.to_string(),
        ));
        items.push((
            "ide.vscode_set_zls_path".to_string(),
            config.ide.vscode_set_zls_path.to_string(),
        ));
        if let Some(ref v) = config.ide.vscode_settings_path {
            items.push(("ide.vscode_settings_path".to_string(), v.clone()));
        }

        Ok(items)
    }

    /// 重置配置为默认值
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
        let mut config = ZzmConfig::default();
        config.default_channel = Some("stable".to_string());
        config.auto_install_zls = true;

        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("default_channel = \"stable\""));
        assert!(toml_str.contains("auto_install_zls = true"));

        let parsed: ZzmConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.default_channel, Some("stable".to_string()));
        assert!(parsed.auto_install_zls);
    }
}