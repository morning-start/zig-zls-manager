use crate::cli;
use crate::utils::error::ZzmError;
use clap::CommandFactory;

/// 生成 Shell 自动补全脚本
pub fn cmd_completion(shell: &str) -> Result<(), ZzmError> {
    let mut cmd = cli::Cli::command();

    match shell.to_lowercase().as_str() {
        "bash" => {
            clap_complete::generate(
                clap_complete::Shell::Bash,
                &mut cmd,
                "zzm",
                &mut std::io::stdout(),
            );
        }
        "zsh" => {
            clap_complete::generate(
                clap_complete::Shell::Zsh,
                &mut cmd,
                "zzm",
                &mut std::io::stdout(),
            );
        }
        "fish" => {
            clap_complete::generate(
                clap_complete::Shell::Fish,
                &mut cmd,
                "zzm",
                &mut std::io::stdout(),
            );
        }
        "powershell" | "ps" => {
            clap_complete::generate(
                clap_complete::Shell::PowerShell,
                &mut cmd,
                "zzm",
                &mut std::io::stdout(),
            );
        }
        _ => {
            return Err(ZzmError::ConfigError {
                path: "completion".to_string(),
                reason: format!("不支持的 shell 类型: {shell}，支持: bash, zsh, fish, powershell"),
            });
        }
    }

    Ok(())
}
