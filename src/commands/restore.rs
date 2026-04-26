use std::path::Path;

use crate::commands::AppContext;
use crate::core::callbacks::InstallCallbacks;
use crate::core::project::{ProjectManager, RestoreResult};
use crate::core::tool_manager::ToolKind;
use crate::output::console_output;
use crate::utils::error::ZzmError;

/// `zzm restore` 命令
///
/// 读取项目 `.zzmrc` 配置 → 安装缺失的 Zig/ZLS 版本 → 切换到项目指定版本
pub async fn cmd_restore(
    ctx: &AppContext,
    project_dir: Option<String>,
    json: bool,
) -> Result<(), ZzmError> {
    let dir = project_dir.as_deref().unwrap_or(".");

    let pm = ProjectManager::new();
    let (config_path, config) =
        pm.find_project_config(Path::new(dir))
            .ok_or_else(|| ZzmError::ConfigError {
                path: dir.to_string(),
                reason: "未找到项目配置文件 (.zzmrc)".to_string(),
            })?;

    if !json {
        console_output::print_header("还原项目环境");
        console_output::print_info(&format!("配置文件: {}", config_path.display()));
        console_output::print_info(&format!(
            "Zig: {}  ZLS: {}  兼容性: {:?}",
            config.zig,
            config.zls.as_deref().unwrap_or("(未指定)"),
            config.compatibility,
        ));
    }

    // 解析 ZLS 版本（使用 ProjectManager 的统一方法）
    let zls_version = ProjectManager::resolve_zls_version(&config);

    // 检查并安装 Zig
    let index = ctx.path_manager().read_installed_index()?;
    let zig_installed = !ProjectManager::is_version_installed(&index, ToolKind::Zig, &config.zig);

    if zig_installed {
        if !json {
            console_output::print_info(&format!("正在安装 Zig {}...", config.zig));
        }
        let callbacks = if json {
            InstallCallbacks::silent()
        } else {
            InstallCallbacks::console()
        };
        let zig_manager = ctx.zig_manager(callbacks)?;
        let installed = zig_manager.install(&config.zig, false, None).await?;
        if !json {
            console_output::print_success(&format!("Zig {} 安装完成", installed.version));
        }
    } else if !json {
        console_output::print_success(&format!("Zig {} 已安装", config.zig));
    }

    // 切换到项目指定的 Zig 版本
    let callbacks = if json {
        InstallCallbacks::silent()
    } else {
        InstallCallbacks::console()
    };
    let zig_manager = ctx.zig_manager(callbacks)?;
    zig_manager.use_version(&config.zig).await?;

    // 处理 ZLS 版本
    let mut zls_actually_installed = false;
    if let Some(zls_ver) = &zls_version {
        let index = ctx.path_manager().read_installed_index()?;
        let zls_already_installed =
            ProjectManager::is_version_installed(&index, ToolKind::Zls, zls_ver);

        if !zls_already_installed {
            if !json {
                console_output::print_info(&format!("正在安装 ZLS {zls_ver}..."));
            }
            let callbacks = if json {
                InstallCallbacks::silent()
            } else {
                InstallCallbacks::console()
            };
            let zls_manager = ctx.zls_manager(callbacks)?;
            match zls_manager.install(zls_ver, false, Some(&config.zig)).await {
                Ok(_) => {
                    zls_actually_installed = true;
                    if !json {
                        console_output::print_success(&format!("ZLS {zls_ver} 安装完成"));
                    }
                }
                Err(e) => {
                    if !json {
                        console_output::print_warning(&format!(
                            "ZLS {zls_ver} 安装失败: {e}，跳过 ZLS 配置"
                        ));
                    }
                    // ZLS 安装失败不影响 Zig 的还原
                    let result = RestoreResult {
                        zig_version: config.zig.clone(),
                        zls_version: None,
                        zig_installed,
                        zls_installed: false,
                    };
                    if !json {
                        print_restore_summary(&result);
                    }
                    return Ok(());
                }
            }
        } else if !json {
            console_output::print_success(&format!("ZLS {zls_ver} 已安装"));
        }

        // 切换到项目指定的 ZLS 版本
        let callbacks = if json {
            InstallCallbacks::silent()
        } else {
            InstallCallbacks::console()
        };
        let zls_manager = ctx.zls_manager(callbacks)?;
        zls_manager.use_version(zls_ver).await?;
    } else if !json {
        console_output::print_info("项目配置未指定 ZLS 版本，跳过 ZLS 安装");
    }

    // 输出总结
    let result = RestoreResult {
        zig_version: config.zig.clone(),
        zls_version,
        zig_installed,
        zls_installed: zls_actually_installed,
    };

    if !json {
        print_restore_summary(&result);
    }

    Ok(())
}

/// 输出还原结果摘要
fn print_restore_summary(result: &RestoreResult) {
    if result.has_changes() {
        let mut parts = vec![format!("Zig {}", result.zig_version)];
        if let Some(zls) = &result.zls_version {
            parts.push(format!("ZLS {zls}"));
        }
        console_output::print_success(&format!("项目环境已还原: {}", parts.join(" + ")));
    } else {
        console_output::print_success("项目环境已是最新，无需变更");
    }
}
