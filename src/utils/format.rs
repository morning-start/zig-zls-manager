/// 格式化文件大小为人类可读的字符串
///
/// # 示例
/// ```
/// use zzm::utils::format::format_size;
/// assert_eq!(format_size(512), "512 B");
/// assert_eq!(format_size(1024), "1.0 KB");
/// assert_eq!(format_size(1024 * 1024), "1.0 MB");
/// assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
/// ```
#[allow(clippy::cast_precision_loss)]
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        let size = bytes as f64 / GB as f64;
        format!("{size:.1} GB")
    } else if bytes >= MB {
        let size = bytes as f64 / MB as f64;
        format!("{size:.1} MB")
    } else if bytes >= KB {
        let size = bytes as f64 / KB as f64;
        format!("{size:.1} KB")
    } else {
        format!("{bytes} B")
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
    fn test_format_size_boundary() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1), "1 B");
        assert_eq!(format_size(1023), "1023 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1024 * 1024 - 1), "1024.0 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
    }
}
