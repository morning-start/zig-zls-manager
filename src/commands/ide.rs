use crate::cli;
use crate::commands::AppContext;
use crate::core::ide::IdeManager;
use crate::output::console_output;
use crate::utils::error::ZzmError;

/// IDE 集成命令
pub async fn cmd_ide(ctx: &AppContext, command: cli::IdeCommands) -> Result<(), ZzmError> {
    let ide_manager = IdeManager::new(ctx.platform().clone_box());

    match command {
        cli::IdeCommands::Config { editor } => match editor.to_lowercase().as_str() {
            "vscode" | "code" => {
                ide_manager.setup_vscode()?;
            }
            "neovim" | "nvim" => {
                console_output::print_warning("Neovim 集成将在后续版本中实现");
            }
            "helix" => {
                console_output::print_warning("Helix 集成将在后续版本中实现");
            }
            _ => {
                console_output::print_error(&format!("不支持的编辑器: {editor}"));
            }
        },
        cli::IdeCommands::Check => match ide_manager.vscode_settings_path() {
            Ok(path) => {
                if path.exists() {
                    let content = std::fs::read_to_string(&path).unwrap_or_default();
                    let has_zig = content.contains("zig.path");
                    let has_zls = content.contains("zig.zls.path");
                    if has_zig && has_zls {
                        console_output::print_success(&format!(
                            "VS Code 配置正常 ({})",
                            path.display()
                        ));
                    } else if has_zig {
                        console_output::print_warning("VS Code 中缺少 zig.zls.path 配置");
                    } else if has_zls {
                        console_output::print_warning("VS Code 中缺少 zig.path 配置");
                    } else {
                        console_output::print_info("VS Code 中未配置 Zig/ZLS 路径");
                    }
                } else {
                    console_output::print_info("VS Code settings.json 不存在");
                }
            }
            Err(e) => console_output::print_warning(&format!("无法检查 VS Code: {e}")),
        },
        cli::IdeCommands::Doctor => {
            console_output::print_header("IDE 集成诊断");

            match ide_manager.zig_binary_path() {
                Ok(path) => console_output::print_success(&format!("Zig: {}", path.display())),
                Err(_) => console_output::print_warning("没有激活的 Zig 版本"),
            }

            match ide_manager.zls_binary_path() {
                Ok(path) => console_output::print_success(&format!("ZLS: {}", path.display())),
                Err(_) => console_output::print_warning("没有激活的 ZLS 版本"),
            }

            match ide_manager.vscode_settings_path() {
                Ok(path) => {
                    console_output::print_info(&format!("VS Code settings: {}", path.display()));
                }
                Err(e) => console_output::print_warning(&format!("VS Code: {e}")),
            }
        }
        cli::IdeCommands::Path => {
            match ide_manager.zig_binary_path() {
                Ok(path) => println!("zig: {}", path.display()),
                Err(_) => println!("zig: (未安装)"),
            }
            match ide_manager.zls_binary_path() {
                Ok(path) => println!("zls: {}", path.display()),
                Err(_) => println!("zls: (未安装)"),
            }
        }
    }
    Ok(())
}
