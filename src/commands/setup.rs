use crate::commands::AppContext;
use crate::core::compatibility;
use crate::output::console_output;
use crate::utils::error::ZzmError;

/// 一键初始化环境
pub async fn cmd_setup(
    ctx: &AppContext,
    version: Option<String>,
    with_zls: bool,
    wizard: bool,
) -> Result<(), ZzmError> {
    if wizard {
        console_output::print_info("启动交互式设置向导...");
        console_output::print_warning("交互式向导将在后续版本中实现");
        return Ok(());
    }

    let Some(version) = version else {
        console_output::print_warning("请指定版本号或使用 --wizard 参数");
        return Ok(());
    };

    // 安装 Zig
    let manager = ctx.zig_manager()?;
    let installed = manager.install(&version, false, None).await?;

    // 切换到安装的版本
    manager.use_version(&installed.version()).await?;

    // 如果指定 --with-zls
    if with_zls {
        let zls_manager = ctx.zls_manager()?;
        let compat_info = zls_manager.api_client().find_compatible_version(installed.version()).await?;
        zls_manager.install(&compat_info.version, false, Some(installed.version())).await?;
    }

    // PATH 提示
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
        if cfg!(windows) {
            console_output::print_info("    PowerShell (用户级):");
            console_output::print_info(&format!(
                "      [Environment]::SetEnvironmentVariable(\"ZIG_HOME\", \"{}\", \"User\")",
                default_dir.display()
            ));
        } else {
            console_output::print_info(&format!(
                "    echo 'export ZIG_HOME=\"{}\"' >> ~/.bashrc",
                default_dir.display()
            ));
        }
    }

    Ok(())
}

/// 同步 Zig 和 ZLS 到推荐组合
pub async fn cmd_sync(ctx: &AppContext, _dry_run: bool) -> Result<(), ZzmError> {
    let zig_manager = ctx.zig_manager()?;
    let zls_manager = ctx.zls_manager()?;

    let zig_current = zig_manager.current()?;
    let zls_current = zls_manager.current()?;

    match (zig_current, zls_current) {
        (Some(zig), Some(zls)) => {
            let status = compatibility::CompatibilityChecker::check(zig.version(), zls.version());
            match status {
                compatibility::CompatibilityStatus::Compatible => {
                    console_output::print_success(&format!(
                        "Zig {} 与 ZLS {} 已兼容，无需同步",
                        zig.version(), zls.version()
                    ));
                }
                _ => {
                    let recommended =
                        compatibility::CompatibilityChecker::recommended_zls_version(zig.version());
                    if let Some(zls_ver) = recommended {
                        console_output::print_info(&format!(
                            "正在安装推荐 ZLS 版本 {} 以匹配 Zig {}...",
                            zls_ver, zig.version()
                        ));
                        let compat_info = zls_manager.api_client().find_compatible_version(zig.version()).await?;
                        zls_manager.install(&compat_info.version, false, Some(zig.version())).await?;
                        console_output::print_success("同步完成");
                    } else {
                        console_output::print_warning("无法确定推荐的 ZLS 版本");
                    }
                }
            }
        }
        (Some(zig), None) => {
            console_output::print_info(&format!(
                "当前 Zig {} 没有 ZLS，正在安装兼容版本...",
                zig.version()
            ));
            let compat_info = zls_manager.api_client().find_compatible_version(zig.version()).await?;
            zls_manager.install(&compat_info.version, false, Some(zig.version())).await?;
            console_output::print_success("同步完成");
        }
        (None, _) => {
            console_output::print_warning("没有激活的 Zig 版本，请先安装并切换 Zig 版本");
        }
    }
    Ok(())
}
