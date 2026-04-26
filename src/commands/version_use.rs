use crate::commands::AppContext;
use crate::core::callbacks::InstallCallbacks;
use crate::utils::error::ZzmError;

/// 切换当前使用的版本
pub async fn cmd_use(
    ctx: &AppContext,
    version: &str,
    zls: Option<String>,
    json: bool,
) -> Result<(), ZzmError> {
    let callbacks = if json {
        InstallCallbacks::silent()
    } else {
        InstallCallbacks::console()
    };
    let manager = ctx.zig_manager(callbacks)?;
    manager.use_version(version).await?;

    // 如果指定了 ZLS 版本，同时切换
    if let Some(zls_version) = zls {
        let callbacks = if json {
            InstallCallbacks::silent()
        } else {
            InstallCallbacks::console()
        };
        let zls_manager = ctx.zls_manager(callbacks)?;
        zls_manager.use_version(&zls_version).await?;
    }

    Ok(())
}
