use crate::commands::AppContext;
use crate::core::callbacks::InstallCallbacks;
use crate::core::tool_manager::ToolKind;
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
        .map_or_else(|| "未设置".to_string(), |v| v.version.clone());
    let zls_version = zls_current
        .as_ref()
        .map_or_else(|| "未设置".to_string(), |v| v.version.clone());
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
    check_environment_variables();
    check_installed_versions(ctx, platform);
    check_symlink_validity(ctx, platform);
    check_disk_space(ctx, platform);
    check_compatibility(ctx);
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
        println!(
            "  已安装 Zig: {} 个版本",
            index.get_versions(ToolKind::Zig).len()
        );
        println!(
            "  已安装 ZLS: {} 个版本",
            index.get_versions(ToolKind::Zls).len()
        );
        if let Some(active) = index.get_active(ToolKind::Zig) {
            println!("  当前 Zig: {active}");
        }
        if let Some(active) = index.get_active(ToolKind::Zls) {
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
        && index.get_active(ToolKind::Zls).is_some()
    {
        println!(
            "  ZLS_HOME={} (设置后 zls 将自动使用当前版本)",
            platform.default_install_dir().join("default-zls").display()
        );
    }
}

/// 检查环境变量配置
fn check_environment_variables() {
    println!();
    println!("  --- 环境变量 ---");

    let env_vars = [
        ("ZIG_HOME", "Zig 默认版本目录"),
        ("ZLS_HOME", "ZLS 默认版本目录"),
        ("ZZM_ROOT", "zzm 安装根目录"),
        ("GITHUB_TOKEN", "GitHub API 认证（提高 ZLS 速率限制）"),
        ("HTTP_PROXY", "HTTP 代理"),
        ("HTTPS_PROXY", "HTTPS 代理"),
    ];

    for (name, desc) in &env_vars {
        match std::env::var(name) {
            Ok(val) => println!("  {name}: ✓ 已设置 ({desc}) [值: {val}]"),
            Err(_) => println!("  {name}: - 未设置 ({desc})"),
        }
    }
}

/// 检查符号链接有效性
fn check_symlink_validity(_ctx: &AppContext, platform: &dyn crate::platform::PlatformTrait) {
    println!();
    println!("  --- 符号链接有效性 ---");

    // 检查 bin 目录下的 zig/zls 符号链接
    let bin_dir = platform.bin_dir();
    if !bin_dir.exists() {
        println!("  bin 目录不存在，跳过检查");
        return;
    }

    let symlinks = [
        ("zig", bin_dir.join("zig")),
        ("zls", bin_dir.join("zls")),
        ("zig.exe", bin_dir.join("zig.exe")),
        ("zls.exe", bin_dir.join("zls.exe")),
    ];

    for (name, path) in &symlinks {
        if !path.exists() {
            continue; // 不存在的不检查
        }

        // 检查符号链接目标是否有效
        let target_valid = if cfg!(windows) {
            // Windows: junction 或 symlink，尝试读取目标
            match std::fs::read_link(path) {
                Ok(target) => {
                    let resolved = if target.is_relative() {
                        bin_dir.join(&target)
                    } else {
                        target
                    };
                    resolved.exists()
                }
                Err(_) => true, // 非 symlink（可能是硬链接或拷贝），视为有效
            }
        } else {
            match std::fs::read_link(path) {
                Ok(target) => {
                    let resolved = if target.is_relative() {
                        bin_dir.join(&target)
                    } else {
                        target
                    };
                    resolved.exists()
                }
                Err(_) => true, // 非符号链接，视为有效
            }
        };

        if target_valid {
            println!("  {name}: ✓ 目标有效");
        } else {
            println!("  {name}: ✗ 目标无效（悬空符号链接）");
        }
    }

    // 检查 default 目录
    let default_dir = platform.default_dir();
    if default_dir.exists()
        && let Ok(target) = std::fs::read_link(&default_dir)
    {
        let resolved = if target.is_relative() {
            platform.default_install_dir().join(&target)
        } else {
            target
        };
        if resolved.exists() {
            println!("  default: ✓ -> {} (有效)", resolved.display());
        } else {
            println!("  default: ✗ -> {} (目标不存在)", resolved.display());
        }
    }

    // 检查 default-zls 目录
    let default_zls = platform.default_install_dir().join("default-zls");
    if default_zls.exists()
        && let Ok(target) = std::fs::read_link(&default_zls)
    {
        let resolved = if target.is_relative() {
            platform.default_install_dir().join(&target)
        } else {
            target
        };
        if resolved.exists() {
            println!("  default-zls: ✓ -> {} (有效)", resolved.display());
        } else {
            println!("  default-zls: ✗ -> {} (目标不存在)", resolved.display());
        }
    }
}

/// 检查磁盘空间
fn check_disk_space(ctx: &AppContext, platform: &dyn crate::platform::PlatformTrait) {
    println!();
    println!("  --- 磁盘空间 ---");

    let install_dir = platform.default_install_dir();

    if !install_dir.exists() {
        println!("  安装目录不存在，无法检查磁盘空间");
        return;
    }

    // 计算已安装版本数和缓存大小
    let path_mgr = ctx.path_manager();
    if let Ok(index) = path_mgr.read_installed_index() {
        let zig_count = index.get_versions(ToolKind::Zig).len();
        let zls_count = index.get_versions(ToolKind::Zls).len();

        let cache_size = path_mgr.cache_size().unwrap_or(0);

        println!(
            "  已安装: Zig {} 个版本 + ZLS {} 个版本",
            zig_count, zls_count
        );
        println!("  缓存大小: {}", format_size(cache_size));
    }
}

/// 检查当前 Zig/ZLS 版本兼容性
fn check_compatibility(ctx: &AppContext) {
    println!();
    println!("  --- 版本兼容性 ---");

    let path_mgr = ctx.path_manager();
    if let Ok(index) = path_mgr.read_installed_index() {
        let zig_active = index.get_active(ToolKind::Zig);
        let zls_active = index.get_active(ToolKind::Zls);

        match (zig_active, zls_active) {
            (Some(zig), Some(zls)) => {
                // 检查兼容性
                let status = crate::core::compatibility::CompatibilityChecker::check(zig, zls);
                match status {
                    crate::core::compatibility::CompatibilityStatus::Compatible => {
                        println!("  Zig {zig} ↔ ZLS {zls}: ✓ 兼容");
                    }
                    crate::core::compatibility::CompatibilityStatus::LikelyCompatible {
                        reason: _,
                    } => {
                        println!("  Zig {zig} ↔ ZLS {zls}: ⚠ 可能兼容");
                    }
                    crate::core::compatibility::CompatibilityStatus::Incompatible { reason } => {
                        println!("  Zig {zig} ↔ ZLS {zls}: ✗ 不兼容 ({reason})");
                    }
                    crate::core::compatibility::CompatibilityStatus::Unknown { reason } => {
                        println!("  Zig {zig} ↔ ZLS {zls}: ? 未知 ({reason})");
                    }
                }
            }
            (Some(zig), None) => {
                println!("  Zig {zig} 已激活，但未设置 ZLS 版本");
                // 尝试推荐
                if let Some(recommended) =
                    crate::core::compatibility::CompatibilityChecker::recommended_zls_version(zig)
                {
                    println!("    建议: 安装 ZLS {recommended}");
                }
            }
            (None, Some(zls)) => {
                println!("  ZLS {zls} 已激活，但未设置 Zig 版本");
            }
            (None, None) => {
                println!("  未设置任何活跃版本");
            }
        }
    }
}
