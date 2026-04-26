use crate::commands::AppContext;
use crate::core::callbacks::InstallCallbacks;
use crate::output::console_output;
use crate::output::table_output::render_kv_table;
use crate::platform;
use crate::utils::error::ZzmError;
use crate::utils::format::format_size;

/// 显示环境信息
pub async fn cmd_info(ctx: &AppContext, _verbose: bool) -> Result<(), ZzmError> {
    console_output::print_header(&format!(
        "Zig/ZLS Version Manager v{}",
        env!("CARGO_PKG_VERSION")
    ));

    let zig_manager = ctx.zig_manager(InstallCallbacks::console())?;
    let zls_manager = ctx.zls_manager(InstallCallbacks::console())?;
    let path_mgr = ctx.path_manager();
    let platform = ctx.platform();

    let zig_current = zig_manager.current()?;
    let zls_current = zls_manager.current()?;
    let installed_zig = zig_manager.list_installed()?;
    let installed_zls = zls_manager.list_installed()?;

    let zig_version = zig_current
        .as_ref()
        .map_or_else(|| "未设置".to_string(), |v| v.version().to_string());
    let zls_version = zls_current
        .as_ref()
        .map_or_else(|| "未设置".to_string(), |v| v.version().to_string());
    let install_dir = platform.default_install_dir().to_string_lossy().to_string();
    let bin_dir = platform.bin_dir().to_string_lossy().to_string();
    let in_path = if platform.is_bin_in_path() {
        "是"
    } else {
        "否"
    };

    let info_items = [
        ("平台", platform.name().to_string()),
        ("目标架构", platform::current_target_triple().to_string()),
        ("安装目录", install_dir),
        (
            "default 目录",
            platform.default_dir().to_string_lossy().to_string(),
        ),
        ("bin 目录", bin_dir),
        ("bin 在 PATH 中", in_path.to_string()),
        (
            "ZZM_ROOT",
            std::env::var("ZZM_ROOT").unwrap_or_else(|_| "(未设置，使用默认路径)".to_string()),
        ),
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

    // 显示环境变量提示
    if zig_current.is_some() {
        console_output::print_info(&format!(
            "提示: 设置 ZIG_HOME={} 即可通过 ZIG_HOME 使用当前 Zig 版本",
            platform.default_dir().display()
        ));
    }
    if zls_current.is_some() {
        console_output::print_info(&format!(
            "提示: 设置 ZLS_HOME={} 即可通过 ZLS_HOME 使用当前 ZLS 版本",
            platform.default_install_dir().join("default-zls").display()
        ));
    }

    Ok(())
}

/// 诊断程序
pub async fn cmd_doctor(ctx: &AppContext) -> Result<(), ZzmError> {
    console_output::print_header(&format!("zzm v{} 诊断报告", env!("CARGO_PKG_VERSION")));

    let platform = ctx.platform();

    check_basic_status(platform);
    check_installed_versions(ctx, platform);
    check_windows_specific();
    check_recommended_config(ctx, platform);

    Ok(())
}

/// 检查基础状态（平台、目录、PATH 等）
fn check_basic_status(platform: &dyn crate::platform::PlatformTrait) {
    let checks = [
        (
            "平台",
            format!(
                "{} ({})",
                platform.name(),
                platform::current_target_triple()
            ),
        ),
        (
            "安装目录",
            platform.default_install_dir().to_string_lossy().to_string(),
        ),
        (
            "目录初始化",
            if platform.default_install_dir().exists() {
                "✓ 已初始化".to_string()
            } else {
                "✗ 未初始化".to_string()
            },
        ),
        (
            "bin 在 PATH",
            if platform.is_bin_in_path() {
                "✓ 已配置".to_string()
            } else {
                "✗ 未配置，请将 bin 目录加入 PATH".to_string()
            },
        ),
        (
            "ZZM_ROOT",
            std::env::var("ZZM_ROOT").unwrap_or_else(|_| "(未设置)".to_string()),
        ),
        ("default 链接", {
            let default_dir = platform.default_dir();
            if default_dir.exists() {
                if cfg!(windows) {
                    match std::fs::read_link(&default_dir) {
                        Ok(target) => format!("✓ -> {}", target.display()),
                        Err(_) => "✓ 已配置（junction 或真实目录）".to_string(),
                    }
                } else {
                    match std::fs::read_link(&default_dir) {
                        Ok(target) => format!("✓ -> {}", target.display()),
                        Err(_) => "✓ 已存在".to_string(),
                    }
                }
            } else {
                "✗ 未配置".to_string()
            }
        }),
    ];

    for (key, value) in &checks {
        println!("  {key}: {value}");
    }
}

/// 检查已安装版本状态
fn check_installed_versions(ctx: &AppContext, platform: &dyn crate::platform::PlatformTrait) {
    if !platform.default_install_dir().exists() {
        return;
    }
    let path_mgr = ctx.path_manager();
    if let Ok(index) = path_mgr.read_installed_index() {
        println!("  已安装 Zig: {} 个版本", index.zig_versions.len());
        println!("  已安装 ZLS: {} 个版本", index.zls_versions.len());
        if let Some(ref active) = index.active_zig {
            println!("  当前 Zig: {active}");
        }
        if let Some(ref active) = index.active_zls {
            println!("  当前 ZLS: {active}");
        }
    }
}

/// Windows 特定检查：UTF-8 编码
fn check_windows_specific() {
    if !cfg!(windows) {
        return;
    }
    println!();
    println!("  --- Windows 专项检查 ---");
    let codepage = std::process::Command::new("chcp")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    let is_utf8 = codepage.contains("65001");
    if is_utf8 {
        println!("  UTF-8 代码页: ✓ 已启用 (65001)");
    } else {
        println!("  UTF-8 代码页: ✗ 未启用（建议开启以避免中文乱码）");
        println!(
            "    设置方式: Windows 设置 → 时间和语言 → 语言和区域 → 管理语言设置 → 更改系统区域设置 → 勾选 \"Beta: 使用 Unicode UTF-8 提供全球语言支持\""
        );
    }
}

/// 检查并输出推荐的环境变量配置
fn check_recommended_config(ctx: &AppContext, platform: &dyn crate::platform::PlatformTrait) {
    println!();
    println!("  --- 推荐配置 ---");
    let default_dir = platform.default_dir();
    println!(
        "  ZIG_HOME={} (设置后 zig 将自动使用当前版本)",
        default_dir.display()
    );
    let path_mgr = ctx.path_manager();
    if let Ok(index) = path_mgr.read_installed_index()
        && index.active_zls.is_some()
    {
        println!(
            "  ZLS_HOME={} (设置后 zls 将自动使用当前版本)",
            platform.default_install_dir().join("default-zls").display()
        );
    }
}
