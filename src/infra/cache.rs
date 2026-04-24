use std::fs;
use std::path::Path;

use crate::utils::error::ZzmError;

/// 缓存管理器
///
/// 管理下载缓存、API 响应缓存等
pub struct CacheManager {
    cache_dir: std::path::PathBuf,
}

impl CacheManager {
    /// 创建新的缓存管理器
    pub fn new(cache_dir: std::path::PathBuf) -> Self {
        Self { cache_dir }
    }

    /// 获取缓存目录路径
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// 计算缓存总大小（字节）
    pub fn total_size(&self) -> Result<u64, ZzmError> {
        if !self.cache_dir.exists() {
            return Ok(0);
        }
        Ok(calculate_dir_size(&self.cache_dir))
    }

    /// 清理所有缓存
    pub fn clean_all(&self) -> Result<u64, ZzmError> {
        let size = self.total_size()?;
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)?;
            fs::create_dir_all(&self.cache_dir)?;
        }
        Ok(size)
    }

    /// 清理过期缓存文件
    ///
    /// 删除修改时间超过 TTL 的文件
    pub fn clean_expired(&self, ttl_secs: u64) -> Result<u64, ZzmError> {
        if !self.cache_dir.exists() {
            return Ok(0);
        }

        let mut cleaned_size = 0u64;
        let ttl = std::time::Duration::from_secs(ttl_secs);

        clean_dir_expired(&self.cache_dir, ttl, &mut cleaned_size)?;

        Ok(cleaned_size)
    }

    /// 预览将要清理的内容（不实际删除）
    pub fn preview_clean(&self) -> Result<Vec<String>, ZzmError> {
        let mut items = Vec::new();
        if !self.cache_dir.exists() {
            return Ok(items);
        }

        collect_dir_items(&self.cache_dir, &mut items)?;
        Ok(items)
    }
}

/// 递归计算目录大小
fn calculate_dir_size(path: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                total += calculate_dir_size(&path);
            } else if let Ok(meta) = path.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

/// 递归清理过期文件
fn clean_dir_expired(
    dir: &Path,
    ttl: std::time::Duration,
    cleaned_size: &mut u64,
) -> Result<(), ZzmError> {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                clean_dir_expired(&path, ttl, cleaned_size)?;
                // 如果目录为空，删除之
                if fs::read_dir(&path).map_or(true, |mut d| d.next().is_none()) {
                    let _ = fs::remove_dir(&path);
                }
            } else if let Ok(meta) = path.metadata()
                && let Ok(modified) = meta.modified()
                    && let Ok(elapsed) = modified.elapsed()
                        && elapsed > ttl {
                            *cleaned_size += meta.len();
                            let _ = fs::remove_file(&path);
                        }
        }
    }
    Ok(())
}

/// 收集目录中的文件列表
fn collect_dir_items(dir: &Path, items: &mut Vec<String>) -> Result<(), ZzmError> {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_dir_items(&path, items)?;
            } else if let Ok(meta) = path.metadata() {
                let size = format_size(meta.len());
                items.push(format!("{} ({})", path.display(), size));
            }
        }
    }
    Ok(())
}

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

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_cache_manager_nonexistent_dir() {
        let cm = CacheManager::new(std::path::PathBuf::from("/nonexistent/cache"));
        assert_eq!(cm.total_size().unwrap(), 0);
    }
}