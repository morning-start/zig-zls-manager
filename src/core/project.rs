use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::tool_manager::ToolKind;
use crate::infra::path_manager::InstalledIndex;
use crate::utils::error::ZzmError;

/// 项目级配置文件名
const PROJECT_CONFIG_FILE: &str = ".zzmrc";

/// 项目级配置（.zzmrc）
///
/// 存储项目锁定的 Zig/ZLS 版本组合，
/// 团队成员通过 `zzm restore` 还原一致的开发环境
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Zig 版本号
    pub zig: String,
    /// ZLS 版本号（可选）
    #[serde(default)]
    pub zls: Option<String>,
    /// 兼容性模式
    #[serde(default = "default_compatibility")]
    pub compatibility: CompatibilityMode,
    /// 备注
    #[serde(default)]
    pub notes: Option<String>,
}

/// 兼容性模式
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CompatibilityMode {
    /// 严格模式：版本必须精确匹配
    #[default]
    Strict,
    /// 宽松模式：次版本号匹配即可
    Loose,
    /// 自动模式：根据兼容性矩阵自动选择
    Auto,
}

fn default_compatibility() -> CompatibilityMode {
    CompatibilityMode::Strict
}

/// restore 操作结果
#[derive(Debug)]
pub struct RestoreResult {
    /// Zig 版本（安装的或已有的）
    pub zig_version: String,
    /// ZLS 版本（安装的或已有的，或 None）
    pub zls_version: Option<String>,
    /// 是否新安装了 Zig
    pub zig_installed: bool,
    /// 是否新安装了 ZLS
    pub zls_installed: bool,
}

impl RestoreResult {
    /// 是否有任何变更（新安装）
    pub fn has_changes(&self) -> bool {
        self.zig_installed || self.zls_installed
    }
}

/// 项目管理器
///
/// 管理项目级 `.zzmrc` 配置文件的读取、写入和版本还原
pub struct ProjectManager;

impl ProjectManager {
    /// 创建新的项目管理器
    pub fn new() -> Self {
        Self
    }

    /// 从指定目录向上递归查找项目配置文件
    ///
    /// 查找顺序：
    /// 1. `{dir}/.zzmrc`
    /// 2. `{dir}/.zzm/config.toml`
    /// 3. 递归向上查找父目录
    pub fn find_project_config(&self, from_dir: &Path) -> Option<(PathBuf, ProjectConfig)> {
        let mut current = if from_dir.is_absolute() {
            from_dir.to_path_buf()
        } else {
            std::env::current_dir().ok()?.join(from_dir)
        };

        loop {
            // 尝试 .zzmrc
            let zzmrc_path = current.join(PROJECT_CONFIG_FILE);
            if zzmrc_path.is_file()
                && let Ok(config) = Self::read_config_file(&zzmrc_path)
            {
                return Some((zzmrc_path, config));
            }

            // 尝试 .zzm/config.toml
            let zzm_dir_config = current.join(".zzm").join("config.toml");
            if zzm_dir_config.is_file()
                && let Ok(config) = Self::read_config_file(&zzm_dir_config)
            {
                return Some((zzm_dir_config, config));
            }

            // 向上查找
            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => return None,
            }
        }
    }

    /// 读取配置文件（支持 JSON 和 TOML 格式）
    fn read_config_file(path: &Path) -> Result<ProjectConfig, ZzmError> {
        let content = std::fs::read_to_string(path).map_err(ZzmError::Io)?;

        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let config = if extension == "toml" {
            toml::from_str(&content)?
        } else {
            // 默认尝试 JSON
            serde_json::from_str(&content)?
        };

        Ok(config)
    }

    /// 在指定目录初始化项目配置文件
    ///
    /// 如果已存在则返回错误
    #[allow(dead_code)] // T-067/T-068: sync/pair 命令将使用
    pub fn init(&self, dir: &Path, config: &ProjectConfig) -> Result<PathBuf, ZzmError> {
        let config_path = dir.join(PROJECT_CONFIG_FILE);

        if config_path.exists() {
            return Err(ZzmError::ConfigError {
                path: config_path.display().to_string(),
                reason: "配置文件已存在".to_string(),
            });
        }

        // 确保目录存在
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).map_err(ZzmError::Io)?;
        }

        let content = serde_json::to_string_pretty(config)?;
        std::fs::write(&config_path, content).map_err(ZzmError::Io)?;

        Ok(config_path)
    }

    /// 保存（覆盖写入）项目配置文件
    ///
    /// 如果文件不存在会自动创建
    #[allow(dead_code)] // T-067/T-068: sync/pair 命令将使用
    pub fn save(&self, dir: &Path, config: &ProjectConfig) -> Result<PathBuf, ZzmError> {
        let config_path = dir.join(PROJECT_CONFIG_FILE);

        // 确保目录存在
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).map_err(ZzmError::Io)?;
        }

        let content = serde_json::to_string_pretty(config)?;
        std::fs::write(&config_path, content).map_err(ZzmError::Io)?;

        Ok(config_path)
    }

    /// 更新项目配置中的 Zig 版本
    ///
    /// 如果找不到项目配置文件则返回错误
    #[allow(dead_code)] // T-068: pair 命令将使用
    pub fn set_zig_version(&self, dir: &Path, version: &str) -> Result<(), ZzmError> {
        let (config_path, mut config) =
            self.find_project_config(dir)
                .ok_or_else(|| ZzmError::ConfigError {
                    path: dir.display().to_string(),
                    reason: "未找到项目配置文件 (.zzmrc)".to_string(),
                })?;

        config.zig = version.to_string();
        let content = serde_json::to_string_pretty(&config)?;
        std::fs::write(&config_path, content).map_err(ZzmError::Io)?;

        Ok(())
    }

    /// 更新项目配置中的 ZLS 版本
    ///
    /// 如果找不到项目配置文件则返回错误
    #[allow(dead_code)] // T-068: pair 命令将使用
    pub fn set_zls_version(&self, dir: &Path, version: Option<&str>) -> Result<(), ZzmError> {
        let (config_path, mut config) =
            self.find_project_config(dir)
                .ok_or_else(|| ZzmError::ConfigError {
                    path: dir.display().to_string(),
                    reason: "未找到项目配置文件 (.zzmrc)".to_string(),
                })?;

        config.zls = version.map(|v| v.to_string());
        let content = serde_json::to_string_pretty(&config)?;
        std::fs::write(&config_path, content).map_err(ZzmError::Io)?;

        Ok(())
    }

    /// 检查版本是否已安装
    pub fn is_version_installed(index: &InstalledIndex, kind: ToolKind, version: &str) -> bool {
        index
            .get_versions(kind)
            .iter()
            .any(|v| v.version == version)
    }

    /// 根据 Auto 模式确定 ZLS 版本
    ///
    /// 优先使用配置文件中的版本，Auto 模式下通过兼容性矩阵推荐
    pub fn resolve_zls_version(config: &ProjectConfig) -> Option<String> {
        // 配置中明确指定了 ZLS 版本
        if let Some(zls) = &config.zls {
            return Some(zls.clone());
        }

        // Auto 模式：根据兼容性矩阵推荐
        if config.compatibility == CompatibilityMode::Auto {
            return crate::core::compatibility::CompatibilityChecker::recommended_zls_version(
                &config.zig,
            );
        }

        None
    }
}

impl Default for ProjectManager {
    fn default() -> Self {
        Self::new()
    }
}

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_project_config_serde_json() {
        let config = ProjectConfig {
            zig: "0.13.0".to_string(),
            zls: Some("0.13.0".to_string()),
            compatibility: CompatibilityMode::Strict,
            notes: Some("项目测试".to_string()),
        };
        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed: ProjectConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.zig, "0.13.0");
        assert_eq!(parsed.zls, Some("0.13.0".to_string()));
        assert!(matches!(parsed.compatibility, CompatibilityMode::Strict));
    }

    #[test]
    fn test_project_config_serde_toml() {
        let toml_str = r#"
zig = "0.13.0"
zls = "0.13.0"
compatibility = "loose"
notes = "测试项目"
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.zig, "0.13.0");
        assert_eq!(config.zls, Some("0.13.0".to_string()));
        assert!(matches!(config.compatibility, CompatibilityMode::Loose));
    }

    #[test]
    fn test_project_config_minimal() {
        let json = r#"{"zig": "0.14.0"}"#;
        let config: ProjectConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.zig, "0.14.0");
        assert_eq!(config.zls, None);
        assert!(matches!(config.compatibility, CompatibilityMode::Strict));
    }

    #[test]
    fn test_find_project_config_not_found() {
        let tmp = TempDir::new().unwrap();
        let pm = ProjectManager::new();
        let result = pm.find_project_config(tmp.path());
        assert!(result.is_none());
    }

    #[test]
    fn test_find_project_config_json() {
        let tmp = TempDir::new().unwrap();
        let config = ProjectConfig {
            zig: "0.13.0".to_string(),
            zls: None,
            compatibility: CompatibilityMode::Auto,
            notes: None,
        };
        let json = serde_json::to_string_pretty(&config).unwrap();
        let config_path = tmp.path().join(".zzmrc");
        std::fs::write(&config_path, json).unwrap();

        let pm = ProjectManager::new();
        let (found_path, found_config) = pm.find_project_config(tmp.path()).unwrap();
        assert_eq!(found_path, config_path);
        assert_eq!(found_config.zig, "0.13.0");
        assert!(matches!(
            found_config.compatibility,
            CompatibilityMode::Auto
        ));
    }

    #[test]
    fn test_find_project_config_toml() {
        let tmp = TempDir::new().unwrap();
        let zzm_dir = tmp.path().join(".zzm");
        std::fs::create_dir_all(&zzm_dir).unwrap();
        let toml_str = r#"zig = "0.15.0""#;
        std::fs::write(zzm_dir.join("config.toml"), toml_str).unwrap();

        let pm = ProjectManager::new();
        let (_, config) = pm.find_project_config(tmp.path()).unwrap();
        assert_eq!(config.zig, "0.15.0");
    }

    #[test]
    fn test_find_project_config_parent_dir() {
        let tmp = TempDir::new().unwrap();
        let config = ProjectConfig {
            zig: "0.13.0".to_string(),
            zls: Some("0.13.0".to_string()),
            compatibility: CompatibilityMode::Strict,
            notes: None,
        };
        let json = serde_json::to_string_pretty(&config).unwrap();
        std::fs::write(tmp.path().join(".zzmrc"), json).unwrap();

        // 子目录中查找，应该找到父目录的 .zzmrc
        let child_dir = tmp.path().join("subdir");
        std::fs::create_dir_all(&child_dir).unwrap();

        let pm = ProjectManager::new();
        let (_, found_config) = pm.find_project_config(&child_dir).unwrap();
        assert_eq!(found_config.zig, "0.13.0");
    }

    #[test]
    fn test_init_project_config() {
        let tmp = TempDir::new().unwrap();
        let config = ProjectConfig {
            zig: "0.13.0".to_string(),
            zls: None,
            compatibility: CompatibilityMode::Strict,
            notes: None,
        };

        let pm = ProjectManager::new();
        let path = pm.init(tmp.path(), &config).unwrap();
        assert!(path.exists());

        // 重复初始化应失败
        let result = pm.init(tmp.path(), &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_save_project_config() {
        let tmp = TempDir::new().unwrap();
        let config = ProjectConfig {
            zig: "0.13.0".to_string(),
            zls: None,
            compatibility: CompatibilityMode::Strict,
            notes: None,
        };

        let pm = ProjectManager::new();

        // 首次保存（文件不存在）
        let path = pm.save(tmp.path(), &config).unwrap();
        assert!(path.exists());

        // 修改后再次保存（覆盖）
        let updated = ProjectConfig {
            zig: "0.14.0".to_string(),
            ..config
        };
        pm.save(tmp.path(), &updated).unwrap();

        // 验证内容已更新
        let (_, loaded) = pm.find_project_config(tmp.path()).unwrap();
        assert_eq!(loaded.zig, "0.14.0");
    }

    #[test]
    fn test_set_zig_version() {
        let tmp = TempDir::new().unwrap();
        let config = ProjectConfig {
            zig: "0.13.0".to_string(),
            zls: Some("0.13.0".to_string()),
            compatibility: CompatibilityMode::Strict,
            notes: None,
        };

        let pm = ProjectManager::new();
        pm.save(tmp.path(), &config).unwrap();

        // 更新 Zig 版本
        pm.set_zig_version(tmp.path(), "0.14.0").unwrap();

        let (_, loaded) = pm.find_project_config(tmp.path()).unwrap();
        assert_eq!(loaded.zig, "0.14.0");
        // ZLS 版本应保持不变
        assert_eq!(loaded.zls, Some("0.13.0".to_string()));
    }

    #[test]
    fn test_set_zls_version() {
        let tmp = TempDir::new().unwrap();
        let config = ProjectConfig {
            zig: "0.13.0".to_string(),
            zls: None,
            compatibility: CompatibilityMode::Auto,
            notes: None,
        };

        let pm = ProjectManager::new();
        pm.save(tmp.path(), &config).unwrap();

        // 设置 ZLS 版本
        pm.set_zls_version(tmp.path(), Some("0.13.0")).unwrap();

        let (_, loaded) = pm.find_project_config(tmp.path()).unwrap();
        assert_eq!(loaded.zls, Some("0.13.0".to_string()));

        // 清除 ZLS 版本
        pm.set_zls_version(tmp.path(), None).unwrap();

        let (_, loaded) = pm.find_project_config(tmp.path()).unwrap();
        assert_eq!(loaded.zls, None);
    }

    #[test]
    fn test_resolve_zls_version_explicit() {
        let config = ProjectConfig {
            zig: "0.13.0".to_string(),
            zls: Some("0.13.0".to_string()),
            compatibility: CompatibilityMode::Strict,
            notes: None,
        };
        assert_eq!(
            ProjectManager::resolve_zls_version(&config),
            Some("0.13.0".to_string())
        );
    }

    #[test]
    fn test_resolve_zls_version_auto() {
        let config = ProjectConfig {
            zig: "0.13.0".to_string(),
            zls: None,
            compatibility: CompatibilityMode::Auto,
            notes: None,
        };
        // Auto 模式应该根据兼容性矩阵推荐
        assert_eq!(
            ProjectManager::resolve_zls_version(&config),
            Some("0.13.0".to_string())
        );
    }

    #[test]
    fn test_resolve_zls_version_strict_no_zls() {
        let config = ProjectConfig {
            zig: "0.13.0".to_string(),
            zls: None,
            compatibility: CompatibilityMode::Strict,
            notes: None,
        };
        // Strict 模式且未指定 ZLS → None
        assert_eq!(ProjectManager::resolve_zls_version(&config), None);
    }

    #[test]
    fn test_restore_result_has_changes() {
        let result = RestoreResult {
            zig_version: "0.13.0".to_string(),
            zls_version: Some("0.13.0".to_string()),
            zig_installed: false,
            zls_installed: true,
        };
        assert!(result.has_changes());

        let no_changes = RestoreResult {
            zig_version: "0.13.0".to_string(),
            zls_version: None,
            zig_installed: false,
            zls_installed: false,
        };
        assert!(!no_changes.has_changes());
    }

    #[test]
    fn test_compatibility_mode_equality() {
        assert_eq!(CompatibilityMode::Strict, CompatibilityMode::Strict);
        assert_ne!(CompatibilityMode::Strict, CompatibilityMode::Auto);
    }

    #[test]
    fn test_compatibility_mode_serde_roundtrip() {
        let modes = vec![
            CompatibilityMode::Strict,
            CompatibilityMode::Loose,
            CompatibilityMode::Auto,
        ];
        for mode in modes {
            let json = serde_json::to_string(&mode).unwrap();
            let parsed: CompatibilityMode = serde_json::from_str(&json).unwrap();
            assert_eq!(mode, parsed);
        }
    }
}
