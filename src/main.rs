use clap::Parser;
use tracing_subscriber::{EnvFilter, fmt};

use zzm::cli::Cli;
use zzm::commands::AppContext;
use zzm::platform::detect_platform;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // 初始化日志
    init_logging(cli.verbose);

    // 设置输出模式
    zzm::output::console_output::set_no_color(cli.no_color);

    // 运行命令
    if let Err(e) = run(cli).await {
        zzm::output::console_output::print_error(&e.to_string());
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

async fn run(cli: Cli) -> Result<(), zzm::utils::error::ZzmError> {
    tracing::debug!("zzm v{} 启动", env!("CARGO_PKG_VERSION"));

    let ctx = AppContext::new(detect_platform());

    match cli.command {
        zzm::cli::Commands::Install {
            version,
            with_zls,
            from_source: _,
            yes: _,
            force,
        } => zzm::commands::install::cmd_install(&ctx, &version, with_zls, force, cli.json).await,
        zzm::cli::Commands::Uninstall { version, purge: _ } => {
            zzm::commands::install::cmd_uninstall(&ctx, &version, cli.json).await
        }
        zzm::cli::Commands::List {
            installed,
            remote,
            json,
        } => zzm::commands::list::cmd_list(&ctx, installed, remote, json || cli.json).await,
        zzm::cli::Commands::Use {
            version,
            global: _,
            project: _,
            default: _,
            zls,
        } => zzm::commands::version_use::cmd_use(&ctx, &version, zls, cli.json).await,
        zzm::cli::Commands::Current { json } => {
            zzm::commands::list::cmd_current(&ctx, json || cli.json).await
        }
        zzm::cli::Commands::Zls { command } => zzm::commands::zls::cmd_zls(&ctx, command, cli.json).await,
        zzm::cli::Commands::Setup {
            version,
            with_zls,
            ide: _,
            wizard,
        } => zzm::commands::setup::cmd_setup(&ctx, version, with_zls, wizard, cli.json).await,
        zzm::cli::Commands::Sync { dry_run } => zzm::commands::setup::cmd_sync(&ctx, dry_run, cli.json).await,
        zzm::cli::Commands::Pair {
            zig_version,
            zls,
            compatibility,
            show,
        } => {
            if show {
                zzm::commands::pair::cmd_pair_show(&ctx, cli.json).await
            } else {
                zzm::commands::pair::cmd_pair(
                    &ctx,
                    &zig_version,
                    zls.as_deref(),
                    compatibility.as_deref(),
                    cli.json,
                )
                .await
            }
        }
        zzm::cli::Commands::Restore { dir } => zzm::commands::restore::cmd_restore(&ctx, dir, cli.json).await,
        zzm::cli::Commands::Info { verbose } => zzm::commands::info::cmd_info(&ctx, verbose).await,
        zzm::cli::Commands::Config { command } => zzm::commands::config::cmd_config(&ctx, command).await,
        zzm::cli::Commands::Ide { command } => zzm::commands::ide::cmd_ide(&ctx, command).await,
        zzm::cli::Commands::Prune {
            dry_run,
            confirm: _,
        } => zzm::commands::prune::cmd_prune(&ctx, dry_run, cli.json).await,
        zzm::cli::Commands::Clean { all, dry_run } => {
            zzm::commands::clean::cmd_clean(&ctx, all, dry_run).await
        }
        zzm::cli::Commands::Doctor => zzm::commands::info::cmd_doctor(&ctx).await,
        zzm::cli::Commands::Completion { shell } => zzm::commands::completion::cmd_completion(&shell),
    }
}
