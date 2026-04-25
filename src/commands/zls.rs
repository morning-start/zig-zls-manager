use crate::cli;
use crate::commands::AppContext;
use crate::output::console_output;
use crate::output::json_output;
use crate::output::table_output::{
    InstalledVersionRow, RemoteVersionRow, render_installed_table, render_remote_table,
};
use crate::utils::error::ZzmError;

/// ZLS 子命令处理
pub async fn cmd_zls(
    ctx: &AppContext,
    command: cli::ZlsCommands,
    json: bool,
) -> Result<(), ZzmError> {
    let manager = ctx.zls_manager()?;

    match command {
        cli::ZlsCommands::Install {
            version,
            from_source: _,
            zig_version,
            yes: _,
        } => {
            manager
                .install(&version, false, zig_version.as_deref())
                .await?;
        }
        cli::ZlsCommands::Uninstall { version } => {
            manager.uninstall(&version)?;
        }
        cli::ZlsCommands::List {
            installed: _,
            remote,
        } => {
            if remote {
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
                            installed: String::new(),
                        })
                        .collect();
                    render_remote_table(&rows);
                }
            } else {
                let versions = manager.list_installed()?;
                let path_mgr = ctx.path_manager();
                let index = path_mgr.read_installed_index()?;

                if json {
                    json_output::print_json(&versions)?;
                } else if versions.is_empty() {
                    console_output::print_info("没有已安装的 ZLS 版本");
                } else {
                    let rows: Vec<InstalledVersionRow> = versions
                        .iter()
                        .map(|v| {
                            let is_active =
                                index.active_zls.as_ref() == Some(&v.version().to_string());
                            InstalledVersionRow {
                                version: v.version().to_string(),
                                channel: v.zig_version().unwrap_or_default().to_string(),
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
        }
        cli::ZlsCommands::Use { version } => {
            manager.use_version(&version).await?;
        }
        cli::ZlsCommands::Current => {
            let current = manager.current()?;
            match &current {
                Some(v) => console_output::print_success(&format!(
                    "ZLS {} ({})",
                    v.version(),
                    v.install_path().to_string_lossy()
                )),
                None => console_output::print_info("当前没有激活的 ZLS 版本"),
            }
        }
    }

    Ok(())
}
