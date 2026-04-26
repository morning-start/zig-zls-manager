use crate::cli;
use crate::commands::AppContext;
use crate::core::callbacks::InstallCallbacks;
use crate::core::tool_manager::ToolKind;
use crate::output::console_output;
use crate::output::dispatcher::{output_json_if, output_list};
use crate::output::table_output::{RemoteVersionOutput, build_installed_rows};
use crate::utils::error::ZzmError;

/// ZLS 子命令处理
pub async fn cmd_zls(
    ctx: &AppContext,
    command: cli::ZlsCommands,
    json: bool,
) -> Result<(), ZzmError> {
    let callbacks = if json {
        InstallCallbacks::silent()
    } else {
        InstallCallbacks::console()
    };
    let manager = ctx.zls_manager(callbacks)?;

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
                let rows: Vec<RemoteVersionOutput> =
                    versions.iter().map(RemoteVersionOutput::from).collect();
                output_list(&rows, json, None);
            } else {
                let versions = manager.list_installed()?;
                let path_mgr = ctx.path_manager();
                let index = path_mgr.read_installed_index()?;
                let active = index.get_active(ToolKind::Zls);
                let rows = build_installed_rows(&versions, ToolKind::Zls, active);

                if json {
                    output_json_if(&rows, json)?;
                } else if rows.is_empty() {
                    console_output::print_info("没有已安装的 ZLS 版本");
                } else {
                    output_list(&rows, false, None);
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
