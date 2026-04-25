use crate::commands::AppContext;
use crate::utils::error::ZzmError;

/// 安装指定版本的 Zig（可同时安装 ZLS）
pub async fn cmd_install(
    ctx: &AppContext,
    version: &str,
    with_zls: bool,
    force: bool,
) -> Result<(), ZzmError> {
    let manager = ctx.zig_manager()?;

    // 安装 Zig
    let zig_installed = manager.install(version, force, None).await?;

    // 如果指定 --with-zls，同时安装兼容的 ZLS
    if with_zls {
        let zls_manager = ctx.zls_manager()?;
        let compat_info = zls_manager
            .api_client()
            .find_compatible_version(zig_installed.version())
            .await?;
        zls_manager
            .install(&compat_info.version, force, Some(zig_installed.version()))
            .await?;
    }

    Ok(())
}

/// 卸载指定版本
pub async fn cmd_uninstall(ctx: &AppContext, version: &str) -> Result<(), ZzmError> {
    let manager = ctx.zig_manager()?;
    manager.uninstall(version)
}
