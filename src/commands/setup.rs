use crate::commands::AppContext;
use crate::core::callbacks::InstallCallbacks;
use crate::core::compatibility;
use crate::output::console_output;
use crate::utils::error::ZzmError;

/// 交互式 Setup Wizard
///
/// 引导用户选择 Zig 版本 → 推荐 ZLS 版本 → 确认安装 → 配置 IDE
pub async fn cmd_setup_wizard(ctx: &AppContext, json: bool) -> Result<(), ZzmError> {
    console_output::print_header("Zig/ZLS 开发环境设置向导");

    // 步骤 1: 选择 Zig 版本
    let zig_version = select_zig_version(ctx, json).await?;
    console_output::print_success(&format!("已选择 Zig {zig_version}"));

    // 步骤 2: 推荐 ZLS 版本
    let zls_version = recommend_zls_version(ctx, &zig_version, json).await?;
    if let Some(ref zv) = zls_version {
        console_output::print_success(&format!("推荐 ZLS {zv}（与 Zig {zig_version} 兼容）"));
    }

    // 步骤 3: 确认安装计划
    print_install_plan(&zig_version, zls_version.as_deref());

    if !confirm_action("确认执行以上安装计划？")? {
        console_output::print_warning("已取消安装");
        return Ok(());
    }

    // 步骤 4: 执行安装
    execute_install_plan(ctx, &zig_version, zls_version.as_deref(), json).await?;

    // 步骤 5: 可选 IDE 配置
    if confirm_action("是否配置 IDE 集成？")? {
        configure_ide_step(ctx)?;
    }

    // 步骤 6: PATH 提示
    print_path_hint(ctx);

    console_output::print_success("设置向导完成！运行 `zzm info` 查看环境详情");
    Ok(())
}

/// 交互式选择 Zig 版本
///
/// 提供远程版本列表供选择，也支持手动输入版本号
async fn select_zig_version(ctx: &AppContext, json: bool) -> Result<String, ZzmError> {
    console_output::print_info("正在获取可用的 Zig 版本列表...");

    let callbacks = if json {
        InstallCallbacks::silent()
    } else {
        InstallCallbacks::console()
    };
    let manager = ctx.zig_manager(callbacks)?;

    let remote_versions = manager.list_remote().await?;

    if remote_versions.is_empty() {
        console_output::print_warning("无法获取远程版本列表，请手动输入版本号");
        return prompt_version_input("请输入要安装的 Zig 版本号（如 0.13.0, master）:");
    }

    // 构建选择列表：显示最新的稳定版 + master
    let mut items: Vec<String> = Vec::new();
    let mut seen_versions = std::collections::HashSet::new();

    // 先添加推荐版本
    for v in &remote_versions {
        if items.len() >= 15 {
            break;
        }
        // 跳过 prerelease，优先展示稳定版
        let is_stable = matches!(v.channel, crate::core::channel::Channel::Stable);
        let is_master = v.version == "master";
        if (is_stable || is_master) && !seen_versions.contains(&v.version) {
            let date_str = v
                .date
                .as_ref()
                .map(|d| format!(" ({d})"))
                .unwrap_or_default();
            let tag = if is_master { " [latest]" } else { "" };
            items.push(format!("{}{date_str}{tag}", v.version));
            seen_versions.insert(v.version.clone());
        }
    }

    // 补充 prerelease 版本
    for v in &remote_versions {
        if items.len() >= 20 {
            break;
        }
        if !seen_versions.contains(&v.version) {
            let date_str = v
                .date
                .as_ref()
                .map(|d| format!(" ({d})"))
                .unwrap_or_default();
            items.push(format!("{}{date_str} [prerelease]", v.version));
            seen_versions.insert(v.version.clone());
        }
    }

    // 添加手动输入选项
    items.push("(手动输入版本号)".to_string());

    let selection = dialoguer::Select::new()
        .with_prompt("选择要安装的 Zig 版本")
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| ZzmError::ConfigError {
            path: "交互式选择".to_string(),
            reason: e.to_string(),
        })?;

    // 最后一项是手动输入
    if selection == items.len() - 1 {
        return prompt_version_input("请输入要安装的 Zig 版本号（如 0.13.0, master）:");
    }

    // 提取版本号（去掉日期和标签部分）
    let selected = &items[selection];
    let version = selected
        .split_whitespace()
        .next()
        .unwrap_or(selected)
        .to_string();

    Ok(version)
}

/// 推荐匹配的 ZLS 版本
///
/// 基于兼容性矩阵推荐与给定 Zig 版本最匹配的 ZLS 版本
async fn recommend_zls_version(
    ctx: &AppContext,
    zig_version: &str,
    json: bool,
) -> Result<Option<String>, ZzmError> {
    let recommended = compatibility::CompatibilityChecker::recommended_zls_version(zig_version);

    let Some(zls_ver) = recommended else {
        console_output::print_warning("无法确定推荐的 ZLS 版本，跳过 ZLS 安装");
        return Ok(None);
    };

    let prompt_msg = format!("是否同时安装推荐的 ZLS {zls_ver}？");
    if confirm_action(&prompt_msg)? {
        // 验证推荐版本是否存在于远程
        let callbacks = if json {
            InstallCallbacks::silent()
        } else {
            InstallCallbacks::console()
        };
        let zls_manager = ctx.zls_manager(callbacks)?;
        match zls_manager
            .api_client()
            .find_compatible_version(zig_version)
            .await
        {
            Ok(info) => Ok(Some(info.version)),
            Err(_) => {
                console_output::print_warning(&format!(
                    "无法找到与 Zig {zig_version} 兼容的 ZLS 版本，跳过 ZLS 安装"
                ));
                Ok(None)
            }
        }
    } else {
        Ok(None)
    }
}

/// 打印安装计划
fn print_install_plan(zig_version: &str, zls_version: Option<&str>) {
    console_output::print_header("安装计划");
    println!("  1. 安装 Zig {zig_version}");
    if let Some(zv) = zls_version {
        println!("  2. 安装 ZLS {zv}（兼容 Zig {zig_version}）");
    }
    println!();
}

/// 执行安装计划
///
/// 当同时安装 Zig 和 ZLS 时，并行下载两者，串行解压注册。
async fn execute_install_plan(
    ctx: &AppContext,
    zig_version: &str,
    zls_version: Option<&str>,
    json: bool,
) -> Result<(), ZzmError> {
    let zig_callbacks = if json {
        InstallCallbacks::silent()
    } else {
        InstallCallbacks::console()
    };
    let zig_manager = ctx.zig_manager(zig_callbacks)?;

    if let Some(zv) = zls_version {
        // 并行下载 Zig 和 ZLS
        let zls_callbacks = if json {
            InstallCallbacks::silent()
        } else {
            InstallCallbacks::console()
        };
        let zls_manager = ctx.zls_manager(zls_callbacks)?;

        let (zig_result, zls_result) = tokio::join!(
            zig_manager.download_only(zig_version, false),
            zls_manager.download_only(zv, false),
        );
        let zig_downloaded = zig_result?;
        let zls_downloaded = zls_result?;

        // 串行解压注册（先 Zig 后 ZLS）
        let installed = zig_manager.install_from_cache(&zig_downloaded, false, None)?;
        zig_manager.use_version(&installed.version).await?;
        zls_manager.install_from_cache(&zls_downloaded, false, Some(&installed.version))?;
    } else {
        // 仅安装 Zig
        let installed = zig_manager.install(zig_version, false, None).await?;
        zig_manager.use_version(&installed.version).await?;
    }

    Ok(())
}

/// 配置 IDE 步骤
fn configure_ide_step(ctx: &AppContext) -> Result<(), ZzmError> {
    let ide_items = vec!["VS Code", "跳过 IDE 配置"];
    let selection = dialoguer::Select::new()
        .with_prompt("选择要配置的编辑器")
        .items(&ide_items)
        .default(0)
        .interact()
        .map_err(|e| ZzmError::ConfigError {
            path: "IDE 选择".to_string(),
            reason: e.to_string(),
        })?;

    if selection == 0 {
        let ide_manager = crate::core::ide::IdeManager::new(ctx.platform().clone_box());
        ide_manager.setup_vscode()?;
    } else {
        console_output::print_info("已跳过 IDE 配置，稍后可通过 `zzm ide config <editor>` 配置");
    }

    Ok(())
}

/// 打印 PATH 提示
fn print_path_hint(ctx: &AppContext) {
    let platform = ctx.platform();
    let bin_dir = platform.bin_dir();
    let default_dir = platform.default_dir();

    if !platform.is_bin_in_path() {
        console_output::print_warning("PATH 环境变量未配置，请选择以下方式之一：");
        console_output::print_info(&format!(
            "  方式 1 (PATH 模式): 将 {} 添加到 PATH",
            bin_dir.to_string_lossy()
        ));
        if cfg!(windows) {
            console_output::print_info("    PowerShell (用户级):");
            console_output::print_info(&format!(
                "      [Environment]::SetEnvironmentVariable(\"Path\", [Environment]::GetEnvironmentVariable(\"Path\", \"User\") + \";{}\", \"User\")",
                bin_dir.to_string_lossy()
            ));
        } else {
            console_output::print_info(&format!(
                "    echo 'export PATH=\"{}:$PATH\"' >> ~/.bashrc",
                bin_dir.to_string_lossy()
            ));
        }
        console_output::print_info(&format!(
            "  方式 2 (ZIG_HOME 模式): 设置环境变量 ZIG_HOME={}",
            default_dir.display()
        ));
    }
}

/// 确认操作
fn confirm_action(prompt: &str) -> Result<bool, ZzmError> {
    dialoguer::Confirm::new()
        .with_prompt(prompt)
        .default(true)
        .interact()
        .map_err(|e| ZzmError::ConfigError {
            path: "交互式确认".to_string(),
            reason: e.to_string(),
        })
}

/// 手动输入版本号
fn prompt_version_input(prompt: &str) -> Result<String, ZzmError> {
    dialoguer::Input::new()
        .with_prompt(prompt)
        .interact_text()
        .map_err(|e| ZzmError::ConfigError {
            path: "版本号输入".to_string(),
            reason: e.to_string(),
        })
}

/// 一键初始化环境（非向导模式）
pub async fn cmd_setup(
    ctx: &AppContext,
    version: Option<String>,
    with_zls: bool,
    wizard: bool,
    json: bool,
) -> Result<(), ZzmError> {
    if wizard {
        return cmd_setup_wizard(ctx, json).await;
    }

    let Some(version) = version else {
        // 无参数时自动启动向导
        return cmd_setup_wizard(ctx, json).await;
    };

    let callbacks = if json {
        InstallCallbacks::silent()
    } else {
        InstallCallbacks::console()
    };

    // 安装 Zig
    let manager = ctx.zig_manager(callbacks)?;
    let installed = manager.install(&version, false, None).await?;

    // 切换到安装的版本
    manager.use_version(&installed.version).await?;

    // 如果指定 --with-zls
    if with_zls {
        let callbacks = if json {
            InstallCallbacks::silent()
        } else {
            InstallCallbacks::console()
        };
        let zls_manager = ctx.zls_manager(callbacks)?;
        let compat_info = zls_manager
            .api_client()
            .find_compatible_version(&installed.version)
            .await?;
        zls_manager
            .install(&compat_info.version, false, Some(&installed.version))
            .await?;
    }

    // PATH 提示（仅控制台模式）
    if !json {
        print_path_hint(ctx);
    }

    Ok(())
}

/// 同步 Zig 和 ZLS 到推荐组合
///
/// 基于兼容性矩阵检查当前 Zig+ZLS 版本组合，自动安装/切换到最优版本：
/// - 两者都兼容 → 无需操作
/// - 不兼容 → 推荐并安装匹配的 ZLS 版本
/// - 仅有 Zig → 安装推荐的 ZLS
/// - 无 Zig → 提示先安装
/// - `--dry-run` 仅展示推荐操作，不实际执行
pub async fn cmd_sync(ctx: &AppContext, dry_run: bool, json: bool) -> Result<(), ZzmError> {
    let callbacks = if json {
        InstallCallbacks::silent()
    } else {
        InstallCallbacks::console()
    };
    let zig_manager = ctx.zig_manager(callbacks)?;
    let zls_manager = ctx.zls_manager(if json {
        InstallCallbacks::silent()
    } else {
        InstallCallbacks::console()
    })?;

    let zig_current = zig_manager.current()?;
    let zls_current = zls_manager.current()?;

    // 消息输出辅助（不依赖 manager 的 callbacks）
    let on_success = |msg: &str| {
        if !json {
            console_output::print_success(msg);
        }
    };
    let on_info = |msg: &str| {
        if !json {
            console_output::print_info(msg);
        }
    };
    let on_warning = |msg: &str| {
        if !json {
            console_output::print_warning(msg);
        }
    };

    match (zig_current, zls_current) {
        (Some(zig), Some(zls)) => {
            let status = compatibility::CompatibilityChecker::check(&zig.version, &zls.version);
            match status {
                compatibility::CompatibilityStatus::Compatible => {
                    on_success(&format!(
                        "Zig {} 与 ZLS {} 已兼容，无需同步",
                        zig.version, zls.version
                    ));
                }
                compatibility::CompatibilityStatus::LikelyCompatible { reason } => {
                    on_info(&format!(
                        "Zig {} 与 ZLS {} 可能兼容: {reason}",
                        zig.version, zls.version
                    ));
                    on_info("当前组合可用，如需精确匹配可使用 `zzm sync` 安装推荐版本");
                }
                _ => {
                    // 不兼容或未知：推荐 ZLS 版本
                    let recommended =
                        compatibility::CompatibilityChecker::recommended_zls_version(&zig.version);
                    if let Some(zls_ver) = recommended {
                        if dry_run {
                            on_info(&format!(
                                "[预览] 将安装 ZLS {zls_ver} 以匹配 Zig {}（当前 ZLS {} 不兼容）",
                                zig.version, zls.version
                            ));
                        } else {
                            on_info(&format!(
                                "正在安装推荐 ZLS 版本 {zls_ver} 以匹配 Zig {}...",
                                zig.version
                            ));
                            let compat_info = zls_manager
                                .api_client()
                                .find_compatible_version(&zig.version)
                                .await?;
                            zls_manager
                                .install(&compat_info.version, false, Some(&zig.version))
                                .await?;
                            on_success("同步完成");
                        }
                    } else {
                        on_warning("无法确定推荐的 ZLS 版本");
                    }
                }
            }
        }
        (Some(zig), None) => {
            let recommended =
                compatibility::CompatibilityChecker::recommended_zls_version(&zig.version);
            if let Some(zls_ver) = recommended {
                if dry_run {
                    on_info(&format!(
                        "[预览] 将安装 ZLS {zls_ver} 以匹配 Zig {}",
                        zig.version
                    ));
                } else {
                    on_info(&format!(
                        "当前 Zig {} 没有 ZLS，正在安装兼容版本 {zls_ver}...",
                        zig.version
                    ));
                    let compat_info = zls_manager
                        .api_client()
                        .find_compatible_version(&zig.version)
                        .await?;
                    zls_manager
                        .install(&compat_info.version, false, Some(&zig.version))
                        .await?;
                    on_success("同步完成");
                }
            } else {
                on_warning(&format!("当前 Zig {} 无法确定推荐的 ZLS 版本", zig.version));
            }
        }
        (None, _) => {
            on_warning("没有激活的 Zig 版本，请先安装并切换 Zig 版本");
        }
    }
    Ok(())
}
