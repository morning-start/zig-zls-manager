use crate::cli;
use crate::commands::AppContext;
use crate::core::config::ConfigManager;
use crate::output::console_output;
use crate::output::table_output::render_kv_table;
use crate::utils::error::ZzmError;

/// 配置管理命令
pub async fn cmd_config(ctx: &AppContext, command: cli::ConfigCommands) -> Result<(), ZzmError> {
    let config_manager = ConfigManager::new(ctx.platform().clone_box());

    match command {
        cli::ConfigCommands::List => {
            let items = config_manager.list_all()?;
            if items.is_empty() {
                console_output::print_info("配置为空（使用默认值）");
            } else {
                let rows: Vec<(&str, String)> =
                    items.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
                render_kv_table("配置项", &rows);
            }
        }
        cli::ConfigCommands::Get { key } => {
            let value = config_manager.get(&key)?;
            match value {
                Some(v) => println!("{key} = {v}"),
                None => console_output::print_info(&format!("配置项 '{key}' 未设置")),
            }
        }
        cli::ConfigCommands::Set { key, value } => {
            config_manager.set(&key, &value)?;
            console_output::print_success(&format!("已设置 {key} = {value}"));
        }
        cli::ConfigCommands::Edit => {
            let config_path = config_manager.config_path();
            let editor = std::env::var("EDITOR")
                .or_else(|_| std::env::var("VISUAL"))
                .unwrap_or_else(|_| {
                    if cfg!(windows) {
                        "notepad".to_string()
                    } else {
                        "vi".to_string()
                    }
                });
            console_output::print_info(&format!("使用 {} 编辑 {}", editor, config_path.display()));
            let status = std::process::Command::new(&editor)
                .arg(&config_path)
                .status();
            match status {
                Ok(s) if s.success() => console_output::print_success("配置已更新"),
                Ok(s) => console_output::print_error(&format!("编辑器退出码: {s}")),
                Err(e) => console_output::print_error(&format!("无法启动编辑器: {e}")),
            }
        }
    }
    Ok(())
}
