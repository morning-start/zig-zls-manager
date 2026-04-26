use std::path::Path;

use crate::commands::AppContext;
use crate::core::project::{CompatibilityMode, ProjectConfig, ProjectManager};
use crate::output::console_output;
use crate::utils::error::ZzmError;

/// `zzm pair` 命令
///
/// 手动绑定 Zig↔ZLS 版本关系，写入项目 `.zzmrc` 配置
pub async fn cmd_pair(
    _ctx: &AppContext,
    zig_version: &str,
    zls_version: Option<&str>,
    compatibility: Option<&str>,
    json: bool,
) -> Result<(), ZzmError> {
    let pm = ProjectManager::new();
    let project_dir = Path::new(".");

    // 解析兼容性模式
    let compat_mode = match compatibility {
        Some("strict") => CompatibilityMode::Strict,
        Some("loose") => CompatibilityMode::Loose,
        Some("auto") => CompatibilityMode::Auto,
        None => CompatibilityMode::Auto, // pair 命令默认 Auto
        Some(other) => {
            return Err(ZzmError::ConfigError {
                path: "compatibility".to_string(),
                reason: format!("无效的兼容性模式: {other}（可选: strict, loose, auto）"),
            });
        }
    };

    // 确定 ZLS 版本
    let zls_ver = match zls_version {
        Some(v) => Some(v.to_string()),
        None => {
            // 未指定 ZLS 版本时，根据兼容性矩阵推荐
            let recommended =
                crate::core::compatibility::CompatibilityChecker::recommended_zls_version(
                    zig_version,
                );
            match recommended {
                Some(v) => {
                    if !json {
                        console_output::print_info(&format!(
                            "未指定 ZLS 版本，根据兼容性矩阵推荐: ZLS {v}"
                        ));
                    }
                    Some(v)
                }
                None => {
                    if !json {
                        console_output::print_warning(
                            "未指定 ZLS 版本且无法自动推荐，将仅绑定 Zig 版本",
                        );
                    }
                    None
                }
            }
        }
    };

    // 检查是否已有项目配置
    let existing = pm.find_project_config(project_dir);

    let config = match existing {
        Some((_, mut existing_config)) => {
            // 更新已有配置
            existing_config.zig = zig_version.to_string();
            if zls_ver.is_some() {
                existing_config.zls = zls_ver;
            }
            existing_config.compatibility = compat_mode;
            existing_config
        }
        None => {
            // 创建新配置
            ProjectConfig {
                zig: zig_version.to_string(),
                zls: zls_ver,
                compatibility: compat_mode,
                notes: None,
            }
        }
    };

    // 写入配置
    let config_path = pm.save(project_dir, &config)?;

    if !json {
        console_output::print_success(&format!("版本绑定已写入: {}", config_path.display()));
        console_output::print_info(&format!(
            "  Zig: {}  ZLS: {}  兼容性: {:?}",
            config.zig,
            config.zls.as_deref().unwrap_or("(未指定)"),
            config.compatibility,
        ));
        console_output::print_info("团队成员可通过 `zzm restore` 还原此配置");
    }

    Ok(())
}

/// `zzm pair --show` 显示当前项目绑定
pub async fn cmd_pair_show(_ctx: &AppContext, json: bool) -> Result<(), ZzmError> {
    let pm = ProjectManager::new();
    let project_dir = Path::new(".");

    match pm.find_project_config(project_dir) {
        Some((config_path, config)) => {
            if json {
                let result = serde_json::json!({
                    "config_path": config_path.to_string_lossy().to_string(),
                    "zig": config.zig,
                    "zls": config.zls,
                    "compatibility": format!("{:?}", config.compatibility),
                    "notes": config.notes,
                });
                crate::output::json_output::print_json(&result)?;
            } else {
                console_output::print_header("当前项目版本绑定");
                console_output::print_info(&format!("配置文件: {}", config_path.display()));
                console_output::print_info(&format!("Zig: {}", config.zig));
                console_output::print_info(&format!(
                    "ZLS: {}",
                    config.zls.as_deref().unwrap_or("(未指定)")
                ));
                console_output::print_info(&format!("兼容性: {:?}", config.compatibility));
                if let Some(notes) = &config.notes {
                    console_output::print_info(&format!("备注: {notes}"));
                }
            }
        }
        None => {
            if json {
                crate::output::json_output::print_json(&serde_json::json!({
                    "error": "未找到项目配置文件 (.zzmrc)"
                }))?;
            } else {
                console_output::print_info("当前项目未配置版本绑定");
                console_output::print_info("使用 `zzm pair <zig_version>` 创建绑定");
            }
        }
    }

    Ok(())
}
