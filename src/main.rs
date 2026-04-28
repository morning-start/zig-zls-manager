use clap::Parser;
use tracing_subscriber::{EnvFilter, fmt};

use zzm::cli::Cli;
use zzm::commands::{AppContext, Command};
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
    cli.command.execute(&ctx, cli.json).await
}
