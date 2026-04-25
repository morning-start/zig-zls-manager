use crate::commands::AppContext;
use crate::utils::error::ZzmError;

/// 切换当前使用的版本
pub async fn cmd_use(ctx: &AppContext, version: &str, zls: Option<String>) -> Result<(), ZzmError> {
    let manager = ctx.zig_manager()?;
    manager.use_version(version).await?;

    // 如果指定了 ZLS 版本，同时切换
    if let Some(zls_version) = zls {
        let zls_manager = ctx.zls_manager()?;
        zls_manager.use_version(&zls_version).await?;
    }

    Ok(())
}
