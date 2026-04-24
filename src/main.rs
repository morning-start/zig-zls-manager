use clap::Parser;
use tracing_subscriber::{fmt, EnvFilter};

mod cli;
mod core;
mod infra;
mod output;
mod platform;
mod utils;

use crate::cli::Cli;
use crate::output::console_output;
use crate::output::table_output::render_kv_table;
use crate::platform::detect_platform;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // 初始化日志
    init_logging(cli.verbose);

    // 设置输出模式
    console_output::set_no_color(cli.no_color);

    // 运行命令
    if let Err(e) = run(cli).await {
        console_output::print_error(&e.to_string());
        std::process::exit(1);
    }

    Ok(())
}

fn init_logging(verbose: bool) {
    let filter_level = if verbose { "debug" } else { "warn" };

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(filter_level));

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

    let platform = detect_platform();

    match cli.command {
        cli::Commands::Install {
            version,
            with_zls,
            from_source,
            yes,
            force,
        } => {
            cmd_install(&version, with_zls, from_source, yes, force, &*platform).await
        }

        cli::Commands::Uninstall { version, purge } => {
            cmd_uninstall(&version, purge, &*platform).await
        }

        cli::Commands::List {
            installed,
            remote,
            json,
        } => cmd_list(installed, remote, json || cli.json, &*platform).await,

        cli::Commands::Use {
            version,
            global,
            project,
            default,
            zls,
        } => cmd_use(&version, global, project, default, zls, &*platform).await,

        cli::Commands::Current { json } => cmd_current(json || cli.json, &*platform).await,

        cli::Commands::Zls { command } => cmd_zls(command, &*platform, cli.json).await,

        cli::Commands::Setup {
            version,
            with_zls,
            ide,
            wizard,
        } => cmd_setup(version, with_zls, ide, wizard, &*platform).await,

        cli::Commands::Sync { dry_run } => cmd_sync(dry_run, &*platform).await,

        cli::Commands::Info { verbose } => cmd_info(verbose, &*platform).await,

        cli::Commands::Config { command } => cmd_config(command, &*platform).await,

        cli::Commands::Ide { command } => cmd_ide(command, &*platform).await,

        cli::Commands::Clean { all, dry_run } => cmd_clean(all, dry_run, &*platform).await,

        cli::Commands::Doctor => cmd_doctor(&*platform).await,

        cli::Commands::Completion { shell } => cmd_completion(&shell),
    }
}

// ========== 命令处理函数 ==========

async fn cmd_install(
    version: &str,
    _with_zls: bool,
    _from_source: bool,
    _yes: bool,
    _force: bool,
    _platform: &dyn platform::PlatformTrait,
) -> Result<(), utils::error::ZzmError> {
    console_output::print_info(&format!("安装 Zig 版本: {}", version));
    console_output::print_warning("安装功能将在 Sprint 3 中实现");
    Ok(())
}

async fn cmd_uninstall(
    version: &str,
    _purge: bool,
    _platform: &dyn platform::PlatformTrait,
) -> Result<(), utils::error::ZzmError> {
    console_output::print_info(&format!("卸载版本: {}", version));
    console_output::print_warning("卸载功能将在 Sprint 3 中实现");
    Ok(())
}

async fn cmd_list(
    installed: bool,
    remote: bool,
    _json: bool,
    _platform: &dyn platform::PlatformTrait,
) -> Result<(), utils::error::ZzmError> {
    if remote {
        console_output::print_info("查询远程可用版本...");
        console_output::print_warning("远程版本查询将在 Sprint 2 中实现");
    } else if installed {
        console_output::print_info("已安装版本列表:");
        console_output::print_warning("版本列表功能将在 Sprint 3 中实现");
    } else {
        console_output::print_info("默认显示已安装版本:");
        console_output::print_warning("版本列表功能将在 Sprint 3 中实现");
    }
    Ok(())
}

async fn cmd_use(
    version: &str,
    _global: bool,
    _project: bool,
    _default: bool,
    _zls: Option<String>,
    _platform: &dyn platform::PlatformTrait,
) -> Result<(), utils::error::ZzmError> {
    console_output::print_info(&format!("切换到版本: {}", version));
    console_output::print_warning("版本切换功能将在 Sprint 3 中实现");
    Ok(())
}

async fn cmd_current(
    _json: bool,
    _platform: &dyn platform::PlatformTrait,
) -> Result<(), utils::error::ZzmError> {
    console_output::print_info("显示当前版本");
    console_output::print_warning("当前版本查询将在 Sprint 3 中实现");
    Ok(())
}

async fn cmd_zls(
    command: cli::ZlsCommands,
    _platform: &dyn platform::PlatformTrait,
    _json: bool,
) -> Result<(), utils::error::ZzmError> {
    match command {
        cli::ZlsCommands::Install { version, .. } => {
            console_output::print_info(&format!("安装 ZLS 版本: {}", version));
        }
        cli::ZlsCommands::Uninstall { version } => {
            console_output::print_info(&format!("卸载 ZLS 版本: {}", version));
        }
        cli::ZlsCommands::List { .. } => {
            console_output::print_info("列出 ZLS 版本");
        }
        cli::ZlsCommands::Use { version } => {
            console_output::print_info(&format!("切换 ZLS 到版本: {}", version));
        }
        cli::ZlsCommands::Current => {
            console_output::print_info("显示当前 ZLS 版本");
        }
    }
    console_output::print_warning("ZLS 管理功能将在 Sprint 4 中实现");
    Ok(())
}

async fn cmd_setup(
    version: Option<String>,
    _with_zls: bool,
    _ide: Option<String>,
    wizard: bool,
    _platform: &dyn platform::PlatformTrait,
) -> Result<(), utils::error::ZzmError> {
    if wizard {
        console_output::print_info("启动交互式设置向导...");
    } else if let Some(v) = version {
        console_output::print_info(&format!("初始化环境: Zig {}", v));
    } else {
        console_output::print_warning("请指定版本号或使用 --wizard 参数");
    }
    console_output::print_warning("Setup 功能将在后续 Sprint 中实现");
    Ok(())
}

async fn cmd_sync(
    _dry_run: bool,
    _platform: &dyn platform::PlatformTrait,
) -> Result<(), utils::error::ZzmError> {
    console_output::print_info("同步 Zig/ZLS 到推荐组合");
    console_output::print_warning("Sync 功能将在 Sprint 4 中实现");
    Ok(())
}

async fn cmd_info(
    _verbose: bool,
    platform: &dyn platform::PlatformTrait,
) -> Result<(), utils::error::ZzmError> {
    console_output::print_header(&format!(
        "Zig/ZLS Version Manager v{}",
        env!("CARGO_PKG_VERSION")
    ));

    let install_dir = platform.default_install_dir().to_string_lossy().to_string();
    let bin_dir = platform.bin_dir().to_string_lossy().to_string();
    let in_path = if platform.is_bin_in_path() { "是" } else { "否" };
    let info_items = [
        ("平台", platform.name().to_string()),
        ("安装目录", install_dir),
        ("bin 目录", bin_dir),
        ("bin 在 PATH 中", in_path.to_string()),
        ("目标架构", platform::current_target_triple().to_string()),
    ];
    render_kv_table("环境信息", &info_items);

    console_output::print_warning("完整环境信息功能将在 Sprint 5 中实现");
    Ok(())
}

async fn cmd_config(
    command: cli::ConfigCommands,
    _platform: &dyn platform::PlatformTrait,
) -> Result<(), utils::error::ZzmError> {
    match command {
        cli::ConfigCommands::List => console_output::print_info("列出所有配置项"),
        cli::ConfigCommands::Get { key } => {
            console_output::print_info(&format!("获取配置: {}", key))
        }
        cli::ConfigCommands::Set { key, value } => {
            console_output::print_info(&format!("设置配置: {} = {}", key, value))
        }
        cli::ConfigCommands::Edit => console_output::print_info("编辑配置文件"),
    }
    console_output::print_warning("配置管理功能将在 Sprint 5 中实现");
    Ok(())
}

async fn cmd_ide(
    command: cli::IdeCommands,
    _platform: &dyn platform::PlatformTrait,
) -> Result<(), utils::error::ZzmError> {
    match command {
        cli::IdeCommands::Config { editor } => {
            console_output::print_info(&format!("生成 {} IDE 配置", editor));
        }
        cli::IdeCommands::Check => console_output::print_info("检查 IDE 配置状态"),
        cli::IdeCommands::Doctor => console_output::print_info("诊断 IDE 集成问题"),
        cli::IdeCommands::Path => console_output::print_info("输出工具路径"),
    }
    console_output::print_warning("IDE 集成功能将在 Sprint 4 中实现");
    Ok(())
}

async fn cmd_clean(
    _all: bool,
    _dry_run: bool,
    _platform: &dyn platform::PlatformTrait,
) -> Result<(), utils::error::ZzmError> {
    console_output::print_info("清理缓存和旧版本");
    console_output::print_warning("清理功能将在 Sprint 5 中实现");
    Ok(())
}

async fn cmd_doctor(
    platform: &dyn platform::PlatformTrait,
) -> Result<(), utils::error::ZzmError> {
    console_output::print_header(&format!(
        "zzm v{} 诊断报告",
        env!("CARGO_PKG_VERSION")
    ));

    let checks = [
        ("平台", format!("{} ({})", platform.name(), platform::current_target_triple())),
        ("安装目录", platform.default_install_dir().to_string_lossy().to_string()),
        ("目录初始化", if platform.default_install_dir().exists() { "✓ 已初始化".to_string() } else { "✗ 未初始化".to_string() }),
        ("bin 在 PATH", if platform.is_bin_in_path() { "✓ 已配置".to_string() } else { "✗ 未配置，请将 bin 目录加入 PATH".to_string() }),
    ];

    for (key, value) in &checks {
        println!("  {}: {}", key, value);
    }

    console_output::print_warning("完整诊断功能将在后续 Sprint 中实现");
    Ok(())
}

fn cmd_completion(shell: &str) -> Result<(), utils::error::ZzmError> {
    use clap::CommandFactory;
    let mut cmd = cli::Cli::command();

    match shell.to_lowercase().as_str() {
        "bash" => {
            clap_complete::generate(clap_complete::Shell::Bash, &mut cmd, "zzm", &mut std::io::stdout());
        }
        "zsh" => {
            clap_complete::generate(clap_complete::Shell::Zsh, &mut cmd, "zzm", &mut std::io::stdout());
        }
        "fish" => {
            clap_complete::generate(clap_complete::Shell::Fish, &mut cmd, "zzm", &mut std::io::stdout());
        }
        "powershell" | "ps" => {
            clap_complete::generate(clap_complete::Shell::PowerShell, &mut cmd, "zzm", &mut std::io::stdout());
        }
        _ => {
            return Err(utils::error::ZzmError::ConfigError {
                path: "completion".to_string(),
                reason: format!("不支持的 shell 类型: {}，支持: bash, zsh, fish, powershell", shell),
            });
        }
    }

    Ok(())
}