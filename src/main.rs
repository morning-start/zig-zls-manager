use clap::Parser;
use tracing_subscriber::{EnvFilter, fmt};

mod cli;
mod commands;
mod core;
mod infra;
mod output;
mod platform;
mod utils;

use crate::cli::Cli;
use crate::commands::AppContext;
use crate::platform::detect_platform;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // 初始化日志
    init_logging(cli.verbose);

    // 设置输出模式
    output::console_output::set_no_color(cli.no_color);

    // 运行命令
    if let Err(e) = run(cli).await {
        output::console_output::print_error(&e.to_string());
        std::process::exit(1);
    }

    Ok(())
}

fn init_logging(verbose: bool) {
    let filter_level = if verbose { "debug" } else { "warn" };

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter_level));

    fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();
}

async fn run(cli: Cli) -> Result<(), utils::error::ZzmError> {
    tracing::debug!("zzm v{} 启动", env!("CARGO_PKG_VERSION"));

    let ctx = AppContext::new(detect_platform());

    match cli.command {
        cli::Commands::Install {
            version,
            with_zls,
            from_source: _,
            yes: _,
            force,
        } => commands::install::cmd_install(&ctx, &version, with_zls, force, cli.json).await,
        cli::Commands::Uninstall { version, purge: _ } => {
            commands::install::cmd_uninstall(&ctx, &version, cli.json).await
        }
        cli::Commands::List {
            installed,
            remote,
            json,
        } => commands::list::cmd_list(&ctx, installed, remote, json || cli.json).await,
        cli::Commands::Use {
            version,
            global: _,
            project: _,
            default: _,
            zls,
        } => commands::version_use::cmd_use(&ctx, &version, zls, cli.json).await,
        cli::Commands::Current { json } => {
            commands::list::cmd_current(&ctx, json || cli.json).await
        }
        cli::Commands::Zls { command } => commands::zls::cmd_zls(&ctx, command, cli.json).await,
        cli::Commands::Setup {
            version,
            with_zls,
            ide: _,
            wizard,
        } => commands::setup::cmd_setup(&ctx, version, with_zls, wizard, cli.json).await,
        cli::Commands::Sync { dry_run } => commands::setup::cmd_sync(&ctx, dry_run, cli.json).await,
        cli::Commands::Info { verbose } => commands::info::cmd_info(&ctx, verbose).await,
        cli::Commands::Config { command } => commands::config::cmd_config(&ctx, command).await,
        cli::Commands::Ide { command } => commands::ide::cmd_ide(&ctx, command).await,
        cli::Commands::Clean { all, dry_run } => {
            commands::clean::cmd_clean(&ctx, all, dry_run).await
        }
        cli::Commands::Doctor => commands::info::cmd_doctor(&ctx).await,
        cli::Commands::Completion { shell } => commands::completion::cmd_completion(&shell),
    }
}
