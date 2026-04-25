use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)] // 部分变体预留给未来功能
pub enum ZzmError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("网络请求失败: {0}")]
    Network(#[from] reqwest::Error),

    #[error("JSON 解析错误: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML 解析错误: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("下载失败: {url} ({reason})")]
    DownloadFailed { url: String, reason: String },

    #[error("下载中断: 已下载 {downloaded}/{total} 字节")]
    DownloadInterrupted { downloaded: u64, total: u64 },

    #[error("无效的版本号: {version}")]
    InvalidVersion { version: String },

    #[error("版本 '{version}' 未找到")]
    VersionNotFound { version: String },

    #[error("版本 '{version}' 已安装，使用 --force 强制重装")]
    AlreadyInstalled { version: String },

    #[error("版本 '{version}' 未安装")]
    NotInstalled { version: String },

    #[error("校验和不匹配\n  期望: {expected}\n  实际: {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("配置文件错误: {path}\n  {reason}")]
    ConfigError { path: String, reason: String },

    #[error("不支持的平台: {platform}")]
    UnsupportedPlatform { platform: String },

    #[error("权限不足: 需要管理员/root 权限执行此操作")]
    PermissionDenied,

    #[error("Zig {zig} 与 ZLS {zls} 不兼容: {reason}")]
    IncompatibleVersions {
        zig: String,
        zls: String,
        reason: String,
    },

    #[error("文件解压失败: {path}\n  {reason}")]
    ExtractionFailed { path: String, reason: String },

    #[error("符号链接创建失败: {from} -> {to}\n  {reason}")]
    SymlinkFailed {
        from: String,
        to: String,
        reason: String,
    },

    #[error("缓存目录创建失败: {path}")]
    CacheDirCreationFailed { path: String },

    #[error("磁盘空间不足: 需要 {needed}, 可用 {available}")]
    InsufficientDiskSpace { needed: u64, available: u64 },

    #[error("操作被用户取消")]
    Cancelled,

    #[error("HTTP 错误: {status_code} - {message}")]
    HttpError { status_code: u16, message: String },

    #[error("速率限制: 请等待 {retry_after} 秒后重试")]
    RateLimited { retry_after: u64 },
}

#[allow(dead_code)] // 预留: 跨模块统一 Result 类型
pub type Result<T> = std::result::Result<T, ZzmError>;

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let zzm_err: ZzmError = io_err.into();
        assert!(matches!(zzm_err, ZzmError::Io(_)));
    }

    #[test]
    fn test_error_display_invalid_version() {
        let err = ZzmError::InvalidVersion {
            version: "abc".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("abc"));
        assert!(msg.contains("无效的版本号"));
    }

    #[test]
    fn test_error_display_version_not_found() {
        let err = ZzmError::VersionNotFound {
            version: "0.99.0".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("0.99.0"));
        assert!(msg.contains("未找到"));
    }

    #[test]
    fn test_error_display_already_installed() {
        let err = ZzmError::AlreadyInstalled {
            version: "0.13.0".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("0.13.0"));
        assert!(msg.contains("--force"));
    }

    #[test]
    fn test_error_display_not_installed() {
        let err = ZzmError::NotInstalled {
            version: "0.13.0".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("0.13.0"));
        assert!(msg.contains("未安装"));
    }

    #[test]
    fn test_error_display_checksum_mismatch() {
        let err = ZzmError::ChecksumMismatch {
            expected: "abc123".to_string(),
            actual: "def456".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("abc123"));
        assert!(msg.contains("def456"));
        assert!(msg.contains("校验和不匹配"));
    }

    #[test]
    fn test_error_display_incompatible_versions() {
        let err = ZzmError::IncompatibleVersions {
            zig: "0.13.0".to_string(),
            zls: "0.12.0".to_string(),
            reason: "版本号不匹配".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("0.13.0"));
        assert!(msg.contains("0.12.0"));
        assert!(msg.contains("不兼容"));
    }

    #[test]
    fn test_error_display_permission_denied() {
        let err = ZzmError::PermissionDenied;
        let msg = err.to_string();
        assert!(msg.contains("权限不足"));
    }

    #[test]
    fn test_error_display_cancelled() {
        let err = ZzmError::Cancelled;
        let msg = err.to_string();
        assert!(msg.contains("取消"));
    }

    #[test]
    fn test_error_display_rate_limited() {
        let err = ZzmError::RateLimited { retry_after: 60 };
        let msg = err.to_string();
        assert!(msg.contains("60"));
        assert!(msg.contains("速率限制"));
    }

    #[test]
    fn test_error_display_http_error() {
        let err = ZzmError::HttpError {
            status_code: 404,
            message: "Not Found".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("404"));
        assert!(msg.contains("Not Found"));
    }

    #[test]
    fn test_error_display_download_failed() {
        let err = ZzmError::DownloadFailed {
            url: "https://example.com/file".to_string(),
            reason: "timeout".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("https://example.com/file"));
        assert!(msg.contains("timeout"));
    }

    #[test]
    fn test_error_display_download_interrupted() {
        let err = ZzmError::DownloadInterrupted {
            downloaded: 1024,
            total: 2048,
        };
        let msg = err.to_string();
        assert!(msg.contains("1024"));
        assert!(msg.contains("2048"));
    }

    #[test]
    fn test_result_type_alias() -> Result<()> {
        let ok: Result<i32> = Ok(42);
        assert_eq!(ok?, 42);

        let err: Result<i32> = Err(ZzmError::Cancelled);
        assert!(err.is_err());

        Ok(())
    }
}
