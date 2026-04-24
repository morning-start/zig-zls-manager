use clap::Parser;
use tracing_subscriber::{fmt, EnvFilter};

mod cli;
mod core;
mod infra;
mod output;
mod platform;
mod utils;

use crate::cli::Cli;
use crate::core::zig_manager::ZigManager;
use crate::core::zls_manager::ZlsManager;
use crate::infra::cache::CacheManager;
use crate::infra::path_manager::PathManager;
use crate::output::console_output;
use crate::output::json_output;
use crate::output::table_output::{
    render_installed_table, render_kv_table, render_remote_table, InstalledVersionRow,
    RemoteVersionRow,
};
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
            from_source: _,
            yes: _,
            force,
        } => cmd_install(&version, with_zls, force, &*platform).await,

        cli::Commands::Uninstall { version, purge: _ } => {
            cmd_uninstall(&version, &*platform).await
        }

        cli::Commands::List {
            installed,
            remote,
            json,
        } => cmd_list(installed, remote, json || cli.json, &*platform).await,

        cli::Commands::Use {
            version,
            global: _,
            project: _,
            default: _,
            zls,
        } => cmd_use(&version, zls, &*platform).await,

        cli::Commands::Current { json } => cmd_current(json || cli.json, &*platform).await,

        cli::Commands::Zls { command } => cmd_zls(command, &*platform, cli.json).await,

        cli::Commands::Setup {
            version,
            with_zls,
            ide: _,
            wizard,
        } => cmd_setup(version, with_zls, wizard, &*platform).await,

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
    with_zls: bool,
    force: bool,
    platform: &dyn platform::PlatformTrait,
) -> Result<(), utils::error::ZzmError> {
    let manager = ZigManager::new(platform.clone_box())?;

    // 安装 Zig
    let zig_installed = manager.install(version, force).await?;

    // 如果指定 --with-zls，同时安装兼容的 ZLS
    if with_zls {
        let zls_manager = ZlsManager::new(platform.clone_box())?;
        zls_manager
            .install_compatible(&zig_installed.version, force)
            .await?;
    }

    Ok(())
}

async fn cmd_uninstall(
    version: &str,
    platform: &dyn platform::PlatformTrait,
) -> Result<(), utils::error::ZzmError> {
    let manager = ZigManager::new(platform.clone_box())?;
    manager.uninstall(version).await
}

async fn cmd_list(
    _installed: bool,
    remote: bool,
    json: bool,
    platform: &dyn platform::PlatformTrait,
) -> Result<(), utils::error::ZzmError> {
    if remote {
        let manager = ZigManager::new(platform.clone_box())?;
        let versions = manager.list_remote().await?;

        if json {
            json_output::print_json(&versions)?;
        } else {
            let rows: Vec<RemoteVersionRow> = versions
                .iter()
                .map(|v| RemoteVersionRow {
                    version: v.version.clone(),
                    channel: match v.channel {
                        infra::zig_api::ZigChannel::Stable => "stable".to_string(),
                        infra::zig_api::ZigChannel::Nightly => "nightly".to_string(),
                    },
                    size: v.asset.as_ref().map(|a| a.size.clone()).unwrap_or_default(),
                    installed: String::new(), // TODO: 交叉检查
                })
                .collect();
            render_remote_table(&rows);
        }
    } else {
        // 默认显示已安装版本
        let manager = ZigManager::new(platform.clone_box())?;
        let versions = manager.list_installed()?;
        let path_mgr = PathManager::new(platform.clone_box());
        let index = path_mgr.read_installed_index()?;

        if json {
            json_output::print_json(&versions)?;
        } else if versions.is_empty() {
            console_output::print_info("没有已安装的 Zig 版本");
            console_output::print_info("使用 zzm install <version> 安装版本");
        } else {
            let rows: Vec<InstalledVersionRow> = versions
                .iter()
                .map(|v| {
                    let is_active = index.active_zig.as_ref() == Some(&v.version);
                    InstalledVersionRow {
                        version: v.version.clone(),
                        channel: v.channel.clone(),
                        path: v.install_path.to_string_lossy().to_string(),
                        status: if is_active { "=> 当前".to_string() } else { String::new() },
                    }
                })
                .collect();
            render_installed_table(&rows);
        }
    }
    Ok(())
}

async fn cmd_use(
    version: &str,
    zls: Option<String>,
    platform: &dyn platform::PlatformTrait,
) -> Result<(), utils::error::ZzmError> {
    let manager = ZigManager::new(platform.clone_box())?;
    manager.use_version(version).await?;

    // 如果指定了 ZLS 版本，同时切换
    if let Some(zls_version) = zls {
        let zls_manager = ZlsManager::new(platform.clone_box())?;
        zls_manager.use_version(&zls_version).await?;
    }

    Ok(())
}

async fn cmd_current(
    json: bool,
    platform: &dyn platform::PlatformTrait,
) -> Result<(), utils::error::ZzmError> {
    let zig_manager = ZigManager::new(platform.clone_box())?;
    let zls_manager = ZlsManager::new(platform.clone_box())?;

    let zig_current = zig_manager.current()?;
    let zls_current = zls_manager.current()?;

    if json {
        let result = serde_json::json!({
            "zig": zig_current.as_ref().map(|v| serde_json::json!({
                "version": v.version,
                "path": v.install_path.to_string_lossy(),
                "channel": v.channel,
            })),
            "zls": zls_current.as_ref().map(|v| serde_json::json!({
                "version": v.version,
                "path": v.install_path.to_string_lossy(),
                "zig_version": v.zig_version,
            })),
        });
        json_output::print_json(&result)?;
    } else {
        match &zig_current {
            Some(v) => console_output::print_success(&format!(
                "Zig {} ({})",
                v.version,
                v.install_path.to_string_lossy()
            )),
            None => console_output::print_info("当前没有激活的 Zig 版本"),
        }

        match &zls_current {
            Some(v) => console_output::print_success(&format!(
                "ZLS {} ({})",
                v.version,
                v.install_path.to_string_lossy()
            )),
            None => console_output::print_info("当前没有激活的 ZLS 版本"),
        }
    }

    Ok(())
}

async fn cmd_zls(
    command: cli::ZlsCommands,
    platform: &dyn platform::PlatformTrait,
    json: bool,
) -> Result<(), utils::error::ZzmError> {
    let manager = ZlsManager::new(platform.clone_box())?;

    match command {
        cli::ZlsCommands::Install {
            version,
            from_source: _,
            zig_version,
            yes: _,
        } => {
            manager.install(&version, zig_version.as_deref(), false).await?;
        }
        cli::ZlsCommands::Uninstall { version } => {
            manager.uninstall(&version).await?;
        }
        cli::ZlsCommands::List { installed: _, remote } => {
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
                                infra::zls_api::ZlsChannel::Stable => "stable".to_string(),
                                infra::zls_api::ZlsChannel::Prerelease => "prerelease".to_string(),
                            },
                            size: v.asset.as_ref().map(|a| format_size(a.size)).unwrap_or_default(),
                            installed: String::new(),
                        })
                        .collect();
                    render_remote_table(&rows);
                }
            } else {
                let versions = manager.list_installed()?;
                let path_mgr = PathManager::new(platform.clone_box());
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
                                status: if is_active { "=> 当前".to_string() } else { String::new() },
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

async fn cmd_setup(
    version: Option<String>,
    with_zls: bool,
    wizard: bool,
    platform: &dyn platform::PlatformTrait,
) -> Result<(), utils::error::ZzmError> {
    if wizard {
        console_output::print_info("启动交互式设置向导...");
        console_output::print_warning("交互式向导将在后续版本中实现");
        return Ok(());
    }

    let version = match version {
        Some(v) => v,
        None => {
            console_output::print_warning("请指定版本号或使用 --wizard 参数");
            return Ok(());
        }
    };

    // 安装 Zig
    let manager = ZigManager::new(platform.clone_box())?;
    let installed = manager.install(&version, false).await?;

    // 切换到安装的版本
    manager.use_version(&installed.version).await?;

    // 如果指定 --with-zls
    if with_zls {
        let zls_manager = ZlsManager::new(platform.clone_box())?;
        zls_manager.install_compatible(&installed.version, false).await?;
    }

    // PATH 提示
    let bin_dir = platform.bin_dir();
    if !platform.is_bin_in_path() {
        console_output::print_warning(&format!(
            "请将 {} 添加到 PATH 环境变量",
            bin_dir.to_string_lossy()
        ));
    }

    Ok(())
}

async fn cmd_sync(
    _dry_run: bool,
    _platform: &dyn platform::PlatformTrait,
) -> Result<(), utils::error::ZzmError> {
    console_output::print_info("同步 Zig/ZLS 到推荐组合");
    console_output::print_warning("Sync 功能将在后续 Sprint 中实现");
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

    let zig_manager = ZigManager::new(platform.clone_box())?;
    let zls_manager = ZlsManager::new(platform.clone_box())?;
    let path_mgr = PathManager::new(platform.clone_box());

    let zig_current = zig_manager.current()?;
    let zls_current = zls_manager.current()?;
    let installed_zig = zig_manager.list_installed()?;
    let installed_zls = zls_manager.list_installed()?;

    let zig_version = zig_current
        .as_ref()
        .map(|v| v.version.clone())
        .unwrap_or_else(|| "未设置".to_string());
    let zls_version = zls_current
        .as_ref()
        .map(|v| v.version.clone())
        .unwrap_or_else(|| "未设置".to_string());
    let install_dir = platform.default_install_dir().to_string_lossy().to_string();
    let bin_dir = platform.bin_dir().to_string_lossy().to_string();
    let in_path = if platform.is_bin_in_path() { "是" } else { "否" };

    let info_items = [
        ("平台", platform.name().to_string()),
        ("目标架构", platform::current_target_triple().to_string()),
        ("安装目录", install_dir),
        ("bin 目录", bin_dir),
        ("bin 在 PATH 中", in_path.to_string()),
        ("当前 Zig", zig_version),
        ("当前 ZLS", zls_version),
        ("已安装 Zig", format!("{} 个版本", installed_zig.len())),
        ("已安装 ZLS", format!("{} 个版本", installed_zls.len())),
        ("缓存大小", {
            let size = path_mgr.cache_size().unwrap_or(0);
            format_size(size)
        }),
    ];
    render_kv_table("环境信息", &info_items);

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
    all: bool,
    dry_run: bool,
    platform: &dyn platform::PlatformTrait,
) -> Result<(), utils::error::ZzmError> {
    let path_mgr = PathManager::new(platform.clone_box());
    let cache_mgr = CacheManager::new(path_mgr.cache_dir());

    if dry_run {
        let items = cache_mgr.preview_clean()?;
        if items.is_empty() {
            console_output::print_info("缓存目录为空，没有需要清理的内容");
        } else {
            console_output::print_header("将要清理的文件:");
            for item in &items {
                println!("  {}", item);
            }
            let size = cache_mgr.total_size()?;
            console_output::print_info(&format!("共 {} 项，总计 {}", items.len(), format_size(size)));
        }
        return Ok(());
    }

    if all {
        let size = cache_mgr.clean_all()?;
        console_output::print_success(&format!("已清理所有缓存，释放 {}", format_size(size)));
    } else {
        // 清理 7 天前的缓存
        let size = cache_mgr.clean_expired(7 * 24 * 3600)?;
        console_output::print_success(&format!("已清理过期缓存，释放 {}", format_size(size)));
    }

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

    // 检查已安装版本
    if platform.default_install_dir().exists() {
        let path_mgr = PathManager::new(platform.clone_box());
        if let Ok(index) = path_mgr.read_installed_index() {
            println!("  已安装 Zig: {} 个版本", index.zig_versions.len());
            println!("  已安装 ZLS: {} 个版本", index.zls_versions.len());
            if let Some(ref active) = index.active_zig {
                println!("  当前 Zig: {}", active);
            }
            if let Some(ref active) = index.active_zls {
                println!("  当前 ZLS: {}", active);
            }
        }
    }

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

// ========== 辅助函数 ==========

/// 格式化文件大小
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}