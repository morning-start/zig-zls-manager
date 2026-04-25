use crate::cli;
use crate::commands::AppContext;
use crate::infra::zls_api;
use crate::output::console_output;
use crate::output::json_output;
use crate::output::table_output::{
    InstalledVersionRow, RemoteVersionRow, render_installed_table, render_remote_table,
};
use crate::utils::error::ZzmError;
use crate::utils::format::format_size;

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
                .install(&version, zig_version.as_deref(), false)
                .await?;
        }
        cli::ZlsCommands::Uninstall { version } => {
            manager.uninstall(&version).await?;
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
                            channel: match v.channel {
                                zls_api::ZlsChannel::Stable => "stable".to_string(),
                                zls_api::ZlsChannel::Prerelease => "prerelease".to_string(),
                            },
                            size: v
                                .asset
                                .as_ref()
                                .map(|a| format_size(a.size))
                                .unwrap_or_default(),
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
                            let is_active = index.active_zls.as_ref() == Some(&v.version);
                            InstalledVersionRow {
                                version: v.version.clone(),
                                channel: v.zig_version.clone().unwrap_or_default(),
                                path: v.install_path.to_string_lossy().to_string(),
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
                    v.version,
                    v.install_path.to_string_lossy()
                )),
                None => console_output::print_info("当前没有激活的 ZLS 版本"),
            }
        }
    }

    Ok(())
}
