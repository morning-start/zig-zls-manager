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
    InsufficientDiskSpace {
        needed: u64,
        available: u64,
    },

    #[error("操作被用户取消")]
    Cancelled,

    #[error("HTTP 错误: {status_code} - {message}")]
    HttpError {
        status_code: u16,
        message: String,
    },

    #[error("速率限制: 请等待 {retry_after} 秒后重试")]
    RateLimited { retry_after: u64 },
}

#[allow(dead_code)] // 预留: 跨模块统一 Result 类型
pub type Result<T> = std::result::Result<T, ZzmError>;
