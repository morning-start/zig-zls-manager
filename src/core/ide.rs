use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::core::config::ConfigManager;
use crate::core::tool_manager::ToolKind;
use crate::infra::path_manager::PathManager;
use crate::output::console_output;
use crate::platform::PlatformTrait;
use crate::utils::error::ZzmError;

/// VS Code settings.json 结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VsCodeSettings {
    /// zig.zls.path 配置
    #[serde(rename = "zig.zls.path", skip_serializing_if = "Option::is_none")]
    pub zls_path: Option<String>,

    /// zig.path 配置
    #[serde(rename = "zig.path", skip_serializing_if = "Option::is_none")]
    pub zig_path: Option<String>,

    /// 其他设置（保留不被覆盖）
    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
}

/// IDE 管理器
///
/// 管理 IDE（VS Code 等）的 Zig/ZLS 配置集成
pub struct IdeManager {
    platform: Box<dyn PlatformTrait>,
    path_manager: PathManager,
}

impl IdeManager {
    /// 创建新的 IDE 管理器
    pub fn new(platform: Box<dyn PlatformTrait>) -> Self {
        let path_manager = PathManager::new(platform.clone_box());
        Self {
            platform,
            path_manager,
        }
    }

    /// 获取当前 Zig 可执行文件路径
    pub fn zig_binary_path(&self) -> Result<PathBuf, ZzmError> {
        let index = self.path_manager.read_installed_index()?;
        let active = index
            .get_active(ToolKind::Zig)
            .ok_or_else(|| ZzmError::NotInstalled {
                version: "zig (no active version)".to_string(),
            })?
            .to_string();

        let version = index
            .get_versions(ToolKind::Zig)
            .iter()
            .find(|v| v.version == active)
            .ok_or_else(|| ZzmError::NotInstalled {
                version: active.clone(),
            })?;

        let binary_name = self.platform.zig_binary_name();
        let bin_path = version.install_path.join(binary_name);

        if bin_path.exists() {
            Ok(bin_path)
        } else {
            Err(ZzmError::NotInstalled {
                version: format!("zig {} binary not found at {}", active, bin_path.display()),
            })
        }
    }

    /// 获取当前 ZLS 可执行文件路径
    pub fn zls_binary_path(&self) -> Result<PathBuf, ZzmError> {
        let index = self.path_manager.read_installed_index()?;
        let active = index
            .get_active(ToolKind::Zls)
            .ok_or_else(|| ZzmError::NotInstalled {
                version: "zls (no active version)".to_string(),
            })?
            .to_string();

        let version = index
            .get_versions(ToolKind::Zls)
            .iter()
            .find(|v| v.version == active)
            .ok_or_else(|| ZzmError::NotInstalled {
                version: active.clone(),
            })?;

        let binary_name = self.platform.zls_binary_name();
        let bin_path = version.install_path.join(binary_name);

        if bin_path.exists() {
            Ok(bin_path)
        } else {
            Err(ZzmError::NotInstalled {
                version: format!("zls {} binary not found at {}", active, bin_path.display()),
            })
        }
    }

    /// 获取 VS Code settings.json 路径
    pub fn vscode_settings_path(&self) -> Result<PathBuf, ZzmError> {
        // 检查自定义路径
        let config_manager = ConfigManager::new(self.platform.clone_box());
        let config = config_manager.load()?;

        if let Some(ref custom_path) = config.ide.vscode_settings_path {
            return Ok(PathBuf::from(custom_path));
        }

        // 默认路径
        let vscode_dir = dirs::config_dir()
            .ok_or_else(|| ZzmError::ConfigError {
                path: "VS Code settings".to_string(),
                reason: "无法确定配置目录".to_string(),
            })?
            .join("Code")
            .join("User");

        Ok(vscode_dir.join("settings.json"))
    }

    /// 配置 VS Code 的 Zig/ZLS 路径
    ///
    /// 读取现有 settings.json，更新 zig.path 和 zig.zls.path，保留其他设置
    pub fn setup_vscode(&self) -> Result<(), ZzmError> {
        let zig_path = self.zig_binary_path().ok();
        let zls_path = self.zls_binary_path().ok();

        if zig_path.is_none() && zls_path.is_none() {
            console_output::print_warning("没有安装任何 Zig 或 ZLS 版本，跳过 VS Code 配置");
            return Ok(());
        }

        let settings_path = self.vscode_settings_path()?;

        // 读取现有 settings.json
        let mut settings = if settings_path.exists() {
            let content =
                std::fs::read_to_string(&settings_path).map_err(|e| ZzmError::ConfigError {
                    path: settings_path.display().to_string(),
                    reason: format!("无法读取: {e}"),
                })?;

            // 处理 VS Code settings.json 可能有注释的情况
            let cleaned = clean_jsonc(&content);
            serde_json::from_str::<VsCodeSettings>(&cleaned).unwrap_or_default()
        } else {
            // 确保父目录存在
            if let Some(parent) = settings_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| ZzmError::ConfigError {
                    path: parent.display().to_string(),
                    reason: format!("无法创建目录: {e}"),
                })?;
            }
            VsCodeSettings::default()
        };

        // 更新路径
        if let Some(ref path) = zig_path {
            let path_str = path.to_string_lossy().to_string();
            // Windows 下使用正斜杠（VS Code 兼容）
            let path_str = path_str.replace('\\', "/");
            settings.zig_path = Some(path_str.clone());
            console_output::print_success(&format!("zig.path = {path_str}"));
        } else {
            console_output::print_warning("没有激活的 Zig 版本，跳过 zig.path 配置");
        }

        if let Some(ref path) = zls_path {
            let path_str = path.to_string_lossy().to_string();
            let path_str = path_str.replace('\\', "/");
            settings.zls_path = Some(path_str.clone());
            console_output::print_success(&format!("zig.zls.path = {path_str}"));
        } else {
            console_output::print_warning("没有激活的 ZLS 版本，跳过 zig.zls.path 配置");
        }

        // 写回文件
        let json_content =
            serde_json::to_string_pretty(&settings).map_err(|e| ZzmError::ConfigError {
                path: settings_path.display().to_string(),
                reason: format!("序列化失败: {e}"),
            })?;

        std::fs::write(&settings_path, json_content).map_err(|e| ZzmError::ConfigError {
            path: settings_path.display().to_string(),
            reason: format!("无法写入: {e}"),
        })?;

        console_output::print_success(&format!("VS Code 配置已更新: {}", settings_path.display()));
        Ok(())
    }

    /// 移除 VS Code 中的 Zig/ZLS 配置
    #[allow(dead_code)] // 预留: zzm ide clean 命令
    pub fn remove_vscode_config(&self) -> Result<(), ZzmError> {
        let settings_path = self.vscode_settings_path()?;

        if !settings_path.exists() {
            console_output::print_info("VS Code settings.json 不存在，无需清理");
            return Ok(());
        }

        let content =
            std::fs::read_to_string(&settings_path).map_err(|e| ZzmError::ConfigError {
                path: settings_path.display().to_string(),
                reason: format!("无法读取: {e}"),
            })?;

        let cleaned = clean_jsonc(&content);
        let mut settings: VsCodeSettings = serde_json::from_str(&cleaned).unwrap_or_default();

        let mut changed = false;

        if settings.zig_path.is_some() {
            settings.zig_path = None;
            changed = true;
            console_output::print_success("已移除 zig.path 配置");
        }

        if settings.zls_path.is_some() {
            settings.zls_path = None;
            changed = true;
            console_output::print_success("已移除 zig.zls.path 配置");
        }

        if changed {
            let json_content =
                serde_json::to_string_pretty(&settings).map_err(|e| ZzmError::ConfigError {
                    path: settings_path.display().to_string(),
                    reason: format!("序列化失败: {e}"),
                })?;

            std::fs::write(&settings_path, json_content).map_err(|e| ZzmError::ConfigError {
                path: settings_path.display().to_string(),
                reason: format!("无法写入: {e}"),
            })?;
        } else {
            console_output::print_info("VS Code 中没有 Zig/ZLS 配置需要移除");
        }

        Ok(())
    }
}

/// 简单清理 JSONC（JSON with Comments）中的注释
///
/// 仅处理行注释（//），不处理块注释
fn clean_jsonc(content: &str) -> String {
    content
        .lines()
        .map(|line| {
            // 找到 // 注释开始位置（不在字符串内的）
            let mut in_string = false;
            let mut escape_next = false;
            let mut comment_pos = None;

            for (i, ch) in line.char_indices() {
                if escape_next {
                    escape_next = false;
                    continue;
                }
                if ch == '\\' && in_string {
                    escape_next = true;
                    continue;
                }
                if ch == '"' {
                    in_string = !in_string;
                    continue;
                }
                if !in_string && ch == '/' && line.as_bytes().get(i + 1) == Some(&b'/') {
                    comment_pos = Some(i);
                    break;
                }
            }

            if let Some(pos) = comment_pos {
                line[..pos].to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_jsonc() {
        let content = r#"{
    "zig.path": "/path/to/zig", // 这是注释
    "zig.zls.path": "/path/to/zls"
    // 另一行注释
}"#;

        let cleaned = clean_jsonc(content);
        assert!(!cleaned.contains("//"));
        assert!(cleaned.contains("/path/to/zig"));
        assert!(cleaned.contains("/path/to/zls"));
    }

    #[test]
    fn test_clean_jsonc_with_strings() {
        let content = r#"{
    "description": "This is a // string with comment",
    "zig.path": "/path/to/zig" // real comment
}"#;

        let cleaned = clean_jsonc(content);
        assert!(cleaned.contains("This is a // string with comment"));
        assert!(!cleaned.contains("real comment"));
    }

    #[test]
    fn test_vs_code_settings_serde() {
        let settings = VsCodeSettings {
            zig_path: Some("/path/to/zig".to_string()),
            zls_path: Some("/path/to/zls".to_string()),
            other: serde_json::Map::new(),
        };

        let json = serde_json::to_string_pretty(&settings).unwrap();
        assert!(json.contains("zig.path"));
        assert!(json.contains("zig.zls.path"));
        assert!(json.contains("/path/to/zig"));
        assert!(json.contains("/path/to/zls"));

        let parsed: VsCodeSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.zig_path, Some("/path/to/zig".to_string()));
        assert_eq!(parsed.zls_path, Some("/path/to/zls".to_string()));
    }

    #[test]
    fn test_vs_code_settings_default() {
        let settings = VsCodeSettings::default();
        assert!(settings.zig_path.is_none());
        assert!(settings.zls_path.is_none());
        assert!(settings.other.is_empty());
    }

    #[test]
    fn test_clean_jsonc_empty_lines() {
        let content = r#"{
    "key": "value"

    // comment
}"#;

        let cleaned = clean_jsonc(content);
        assert!(!cleaned.contains("// comment"));
    }

    #[test]
    fn test_clean_jsonc_escaped_quotes() {
        let content = r#"{
    "key": "value with \"escaped\" quotes", // comment
    "other": "value"
}"#;

        let cleaned = clean_jsonc(content);
        assert!(cleaned.contains("value with \\\"escaped\\\" quotes"));
        assert!(!cleaned.contains("comment"));
    }

    #[test]
    fn test_vs_code_settings_with_other_fields() {
        // 测试 VsCodeSettings 保留其他字段
        let json = r#"{
            "zig.path": "/path/to/zig",
            "zig.zls.path": "/path/to/zls",
            "editor.fontSize": 14,
            "workbench.colorTheme": "Dark+"
        }"#;

        let settings: VsCodeSettings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.zig_path, Some("/path/to/zig".to_string()));
        assert_eq!(settings.zls_path, Some("/path/to/zls".to_string()));
        assert_eq!(settings.other.len(), 2);
        assert!(settings.other.contains_key("editor.fontSize"));
        assert!(settings.other.contains_key("workbench.colorTheme"));
    }

    #[test]
    fn test_vs_code_settings_only_other_fields() {
        let json = r#"{
            "editor.fontSize": 14
        }"#;

        let settings: VsCodeSettings = serde_json::from_str(json).unwrap();
        assert!(settings.zig_path.is_none());
        assert!(settings.zls_path.is_none());
        assert_eq!(settings.other.len(), 1);
    }

    #[test]
    fn test_vs_code_settings_roundtrip_preserves_other() {
        let mut other = serde_json::Map::new();
        other.insert(
            "editor.fontSize".to_string(),
            serde_json::Value::Number(14.into()),
        );
        let settings = VsCodeSettings {
            zig_path: Some("/path/to/zig".to_string()),
            zls_path: None,
            other,
        };

        let json = serde_json::to_string_pretty(&settings).unwrap();
        let parsed: VsCodeSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.zig_path, Some("/path/to/zig".to_string()));
        assert_eq!(parsed.other.len(), 1);
    }

    #[test]
    fn test_clean_jsonc_no_comments() {
        let content = r#"{"key": "value"}"#;
        let cleaned = clean_jsonc(content);
        assert_eq!(cleaned, content);
    }

    #[test]
    fn test_clean_jsonc_multiple_comments() {
        let content = r#"{
    // first comment
    "key1": "value1",
    // second comment
    "key2": "value2"
}"#;

        let cleaned = clean_jsonc(content);
        assert!(!cleaned.contains("first comment"));
        assert!(!cleaned.contains("second comment"));
        assert!(cleaned.contains("value1"));
        assert!(cleaned.contains("value2"));
    }

    #[test]
    fn test_clean_jsonc_comment_inside_string_preserved() {
        // 确保字符串内的 // 不被误删
        let content = r#"{
    "url": "https://example.com/path",
    "zig.path": "/usr/bin/zig"
}"#;

        let cleaned = clean_jsonc(content);
        assert!(cleaned.contains("https://example.com/path"));
    }

    #[test]
    fn test_ide_manager_creation() {
        let platform = crate::platform::detect_platform();
        let manager = IdeManager::new(platform);
        // IdeManager 创建应成功（不依赖已安装的版本）
        let _ = manager;
    }

    #[test]
    fn test_vs_code_settings_empty_json() {
        let json = "{}";
        let settings: VsCodeSettings = serde_json::from_str(json).unwrap();
        assert!(settings.zig_path.is_none());
        assert!(settings.zls_path.is_none());
        assert!(settings.other.is_empty());
    }
}
