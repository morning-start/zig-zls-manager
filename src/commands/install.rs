use crate::commands::AppContext;
use crate::core::callbacks::InstallCallbacks;
use crate::utils::error::ZzmError;
use crate::utils::version::resolve_version;

/// 安装指定版本的 Zig（可同时安装 ZLS）
///
/// 当指定 `--with-zls` 时，Zig 和 ZLS 并行下载，串行解压注册。
pub async fn cmd_install(
    ctx: &AppContext,
    version: &str,
    with_zls: bool,
    force: bool,
    json: bool,
) -> Result<(), ZzmError> {
    if !with_zls {
        // 不安装 ZLS，直接串行安装 Zig
        let callbacks = if json {
            InstallCallbacks::silent()
        } else {
            InstallCallbacks::console()
        };
        let manager = ctx.zig_manager(callbacks)?;
        manager.install(version, force, None).await?;
        return Ok(());
    }

    // 并行下载场景：先获取版本信息，再并行下载，最后串行安装
    let zig_callbacks = if json {
        InstallCallbacks::silent()
    } else {
        InstallCallbacks::console()
    };
    let zls_callbacks = if json {
        InstallCallbacks::silent()
    } else {
        InstallCallbacks::console()
    };

    let zig_manager = ctx.zig_manager(zig_callbacks)?;
    let zls_manager = ctx.zls_manager(zls_callbacks)?;

    // 第一步：解析 Zig 版本号 + 获取 ZLS 兼容版本信息
    let resolved = resolve_version(version)?;
    let compat_info = zls_manager
        .api_client()
        .find_compatible_version(&resolved)
        .await?;

    // 第二步：并行下载 Zig 和 ZLS
    let (zig_result, zls_result) = tokio::join!(
        zig_manager.download_only(version, force),
        zls_manager.download_only(&compat_info.version, force),
    );
    let zig_downloaded = zig_result?;
    let zls_downloaded = zls_result?;

    // 第三步：串行解压注册（先 Zig 后 ZLS）
    let zig_installed = zig_manager.install_from_cache(&zig_downloaded, force, None)?;
    zls_manager.install_from_cache(
        &zls_downloaded,
        force,
        Some(&zig_installed.version),
    )?;

    Ok(())
}

/// 卸载指定版本
pub async fn cmd_uninstall(ctx: &AppContext, version: &str, json: bool) -> Result<(), ZzmError> {
    let callbacks = if json {
        InstallCallbacks::silent()
    } else {
        InstallCallbacks::console()
    };
    let manager = ctx.zig_manager(callbacks)?;
    manager.uninstall(version)
}
