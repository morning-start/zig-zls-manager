use crate::commands::AppContext;
use crate::core::callbacks::InstallCallbacks;
use crate::output::console_output;
use crate::output::json_output;
use crate::output::table_output::{
    InstalledVersionRow, RemoteVersionRow, render_installed_table, render_remote_table,
};
use crate::utils::error::ZzmError;

/// 列出版本信息
pub async fn cmd_list(
    ctx: &AppContext,
    _installed: bool,
    remote: bool,
    json: bool,
) -> Result<(), ZzmError> {
    let callbacks = if json {
        InstallCallbacks::silent()
    } else {
        InstallCallbacks::console()
    };

    if remote {
        let manager = ctx.zig_manager(callbacks)?;
        let versions = manager.list_remote().await?;

        if json {
            json_output::print_json(&versions)?;
        } else {
            let rows: Vec<RemoteVersionRow> = versions
                .iter()
                .map(|v| RemoteVersionRow {
                    version: v.version.clone(),
                    channel: v.channel.to_string(),
                    date: v.date.clone().unwrap_or_default(),
                    size: v.asset.as_ref().map(|a| a.size.clone()).unwrap_or_default(),
                    installed: String::new(), // TODO: 交叉检查
                })
                .collect();
            render_remote_table(&rows);
        }
    } else {
        // 默认显示已安装版本
        let manager = ctx.zig_manager(callbacks)?;
        let versions = manager.list_installed()?;
        let path_mgr = ctx.path_manager();
        let index = path_mgr.read_installed_index()?;

        if json {
            json_output::print_json(&versions)?;
        } else if versions.is_empty() {
            console_output::print_info("没有已安装的 Zig 版本");
            console_output::print_info("使用 zzm install <version> 安装版本");
        } else {
            let rows: Vec<InstalledVersionRow> = versions
                .iter()
                .map(|v| {
                    let is_active = index.active_zig.as_ref() == Some(&v.version().to_string());
                    InstalledVersionRow {
                        version: v.version().to_string(),
                        channel: v.channel().map(|c| c.to_string()).unwrap_or_default(),
                        path: v.install_path().to_string_lossy().to_string(),
                        status: if is_active {
                            "=> 当前".to_string()
                        } else {
                            String::new()
                        },
                    }
                })
                .collect();
            render_installed_table(&rows);
        }
    }
    Ok(())
}

/// 显示当前版本
pub async fn cmd_current(ctx: &AppContext, json: bool) -> Result<(), ZzmError> {
    let callbacks = if json {
        InstallCallbacks::silent()
    } else {
        InstallCallbacks::console()
    };
    let zig_manager = ctx.zig_manager(callbacks)?;
    let zls_manager = ctx.zls_manager(InstallCallbacks::console())?;

    let zig_current = zig_manager.current()?;
    let zls_current = zls_manager.current()?;

    if json {
        let result = serde_json::json!({
            "zig": zig_current.as_ref().map(|v| serde_json::json!({
                "version": v.version(),
                "path": v.install_path().to_string_lossy(),
                "channel": v.channel().map(|c| c.to_string()),
            })),
            "zls": zls_current.as_ref().map(|v| serde_json::json!({
                "version": v.version(),
                "path": v.install_path().to_string_lossy(),
                "zig_version": v.zig_version(),
            })),
        });
        json_output::print_json(&result)?;
    } else {
        match &zig_current {
            Some(v) => console_output::print_success(&format!(
                "Zig {} ({})",
                v.version(),
                v.install_path().to_string_lossy()
            )),
            None => console_output::print_info("当前没有激活的 Zig 版本"),
        }

        match &zls_current {
            Some(v) => console_output::print_success(&format!(
                "ZLS {} ({})",
                v.version(),
                v.install_path().to_string_lossy()
            )),
            None => console_output::print_info("当前没有激活的 ZLS 版本"),
        }
    }

    Ok(())
}
