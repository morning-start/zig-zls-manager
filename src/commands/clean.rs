use crate::commands::AppContext;
use crate::output::console_output;
use crate::utils::error::ZzmError;
use crate::utils::format::format_size;

/// 清理缓存
pub async fn cmd_clean(ctx: &AppContext, all: bool, dry_run: bool) -> Result<(), ZzmError> {
    let cache_mgr = ctx.cache_manager();

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
            console_output::print_info(&format!(
                "共 {} 项，总计 {}",
                items.len(),
                format_size(size)
            ));
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
