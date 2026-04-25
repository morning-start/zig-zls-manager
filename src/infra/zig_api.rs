use std::path::PathBuf;
use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::core::channel::Channel;
use crate::utils::error::ZzmError;

/// Zig 官方下载 API 端点
const ZIG_INDEX_URL: &str = "https://ziglang.org/download/index.json";

/// 缓存 TTL: 1 小时
const CACHE_TTL: Duration = Duration::from_secs(3600);

/// 缓存文件名
const CACHE_FILENAME: &str = "zig_index.json";

// ========== API 原始数据结构 ==========

/// Zig 下载索引（顶层 JSON 对象）
///
/// 键为版本号字符串（如 "0.16.0", "0.15.2", "master"），
/// 值为对应版本信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigDownloadIndex(pub std::collections::BTreeMap<String, ZigVersionEntry>);

/// 单个 Zig 版本的详细信息
///
/// 实际 API 响应结构示例：
/// ```json
/// {
///   "0.16.0": {
///     "version": "0.16.0",
///     "date": "2026-04-13",
///     "docs": "https://...",
///     "stdDocs": "https://...",
///     "notes": "https://...",
///     "src": { "tarball": "...", "shasum": "...", "size": "..." },
///     "bootstrap": { "tarball": "...", "shasum": "...", "size": "..." },
///     "x86_64-macos": { "tarball": "...", "shasum": "...", "size": "..." },
///     "x86_64-windows": { "tarball": "...", "shasum": "...", "size": "..." }
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigVersionEntry {
    /// 构建日期 (YYYY-MM-DD)
    pub date: String,
    /// 版本号字符串（如 "0.16.0", "0.17.0-dev.101+4e2147d14"）
    #[serde(default)]
    pub version: String,
    /// 文档链接
    #[serde(default)]
    pub docs: String,
    /// 标准库文档链接
    #[serde(default, rename = "stdDocs")]
    pub std_docs: String,
    /// 发布说明链接
    #[serde(default)]
    pub notes: String,
    /// 源码包信息
    #[serde(default)]
    pub src: Option<ZigPlatformAsset>,
    /// Bootstrap 源码包信息
    #[serde(default)]
    pub bootstrap: Option<ZigPlatformAsset>,
    /// 按平台分类的预编译二进制
    ///
    /// 键为平台标识（如 "x86_64-macos", "aarch64-linux", "x86_64-windows"），
    /// 旧版本键名格式可能为 "macos-x86_64"。
    /// 使用 flatten 捕获所有非已知字段作为平台条目。
    #[serde(flatten)]
    pub platforms: std::collections::BTreeMap<String, ZigPlatformAsset>,
}

/// 平台特定的下载资源
///
/// 实际 API 中每个平台条目只包含 tarball/shasum/size 三个字段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigPlatformAsset {
    /// 下载 URL
    pub tarball: String,
    /// SHA256 校验和
    #[serde(default)]
    pub shasum: String,
    /// 文件大小（字节数字符串，如 "55574528"）
    #[serde(default)]
    pub size: String,
}

// ========== 内部统一版本信息结构 ==========

/// 统一的 Zig 版本信息（供内部使用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigVersionInfo {
    /// 版本号字符串
    pub version: String,
    /// 版本通道
    pub channel: Channel,
    /// 构建日期
    pub date: String,
    /// 当前平台匹配的下载资源
    pub asset: Option<ZigPlatformAsset>,
}

// ========== API 客户端 ==========

/// Zig API 客户端
pub struct ZigApiClient {
    client: Client,
    cache: crate::infra::api_cache::ApiCache<ZigDownloadIndex>,
}

impl ZigApiClient {
    /// 创建新的 API 客户端
    pub fn new(cache_dir: PathBuf) -> Result<Self, ZzmError> {
        let client = Client::builder()
            .user_agent(format!("zzm/{}", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .map_err(ZzmError::Network)?;

        // 确保缓存目录存在
        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir).map_err(ZzmError::Io)?;
        }

        let cache = crate::infra::api_cache::ApiCache::new(cache_dir, CACHE_FILENAME, CACHE_TTL);

        Ok(Self { client, cache })
    }

    /// 从远程 API 获取 Zig 下载索引（带缓存）
    pub async fn fetch_index(&self) -> Result<ZigDownloadIndex, ZzmError> {
        // 先尝试从缓存读取
        if let Some(cached) = self.cache.load() {
            return Ok(cached);
        }

        tracing::debug!("从远程获取 Zig 索引数据: {}", ZIG_INDEX_URL);

        let response =
            self.client
                .get(ZIG_INDEX_URL)
                .send()
                .await
                .map_err(|e| ZzmError::DownloadFailed {
                    url: ZIG_INDEX_URL.to_string(),
                    reason: e.to_string(),
                })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            return Err(ZzmError::HttpError {
                status_code: status,
                message,
            });
        }

        let index: ZigDownloadIndex = response.json().await?;

        // 保存到缓存
        self.cache.save(&index)?;

        Ok(index)
    }

    /// 获取所有远程可用的 Zig 版本列表
    pub async fn list_remote_versions(&self) -> Result<Vec<ZigVersionInfo>, ZzmError> {
        let index = self.fetch_index().await?;
        let target_triple = crate::platform::current_target_triple();

        let mut versions = Vec::new();

        for (version_str, entry) in &index.0 {
            let channel = if version_str == "master" {
                Channel::Nightly
            } else {
                Channel::Stable
            };

            // 查找当前平台匹配的资源
            let asset = find_matching_asset(&entry.platforms, target_triple);

            versions.push(ZigVersionInfo {
                version: version_str.clone(),
                channel,
                date: entry.date.clone(),
                asset,
            });
        }

        // 稳定版按版本号降序排列，master 放最后
        versions.sort_by(|a, b| {
            match (&a.channel, &b.channel) {
                (Channel::Nightly, Channel::Nightly) => std::cmp::Ordering::Equal,
                (Channel::Nightly, _) => std::cmp::Ordering::Greater,
                (Channel::Prerelease, Channel::Prerelease) => std::cmp::Ordering::Equal,
                (Channel::Prerelease, Channel::Nightly) => std::cmp::Ordering::Less,
                (Channel::Prerelease, Channel::Stable) => std::cmp::Ordering::Greater,
                (Channel::Stable, Channel::Nightly) => std::cmp::Ordering::Less,
                (Channel::Stable, Channel::Prerelease) => std::cmp::Ordering::Less,
                (Channel::Stable, Channel::Stable) => {
                    // 尝试按语义版本比较
                    let va: std::result::Result<crate::utils::version::Version, _> =
                        a.version.parse();
                    let vb: std::result::Result<crate::utils::version::Version, _> =
                        b.version.parse();
                    match (va, vb) {
                        (Ok(va), Ok(vb)) => vb.cmp(&va), // 降序
                        _ => b.version.cmp(&a.version),
                    }
                }
            }
        });

        Ok(versions)
    }

    /// 根据版本号获取特定版本的下载信息
    pub async fn get_version_info(&self, version: &str) -> Result<ZigVersionInfo, ZzmError> {
        let index = self.fetch_index().await?;
        let target_triple = crate::platform::current_target_triple();

        // 处理特殊标识符
        let resolved = crate::utils::version::resolve_version(version)?;

        let entry = index
            .0
            .get(&resolved)
            .ok_or_else(|| ZzmError::VersionNotFound {
                version: version.to_string(),
            })?;

        let channel = if resolved == "master" {
            Channel::Nightly
        } else {
            Channel::Stable
        };

        let asset = find_matching_asset(&entry.platforms, target_triple);

        Ok(ZigVersionInfo {
            version: resolved,
            channel,
            date: entry.date.clone(),
            asset,
        })
    }

    /// 获取最新稳定版本
    #[allow(dead_code)] // 预留: zzm install latest 命令
    pub async fn get_latest_stable(&self) -> Result<ZigVersionInfo, ZzmError> {
        let versions = self.list_remote_versions().await?;
        versions
            .into_iter()
            .find(|v| v.channel == Channel::Stable)
            .ok_or_else(|| ZzmError::VersionNotFound {
                version: "stable".to_string(),
            })
    }

    /// 获取 master (nightly) 版本
    #[allow(dead_code)] // 预留: zzm install master 命令
    pub async fn get_master(&self) -> Result<ZigVersionInfo, ZzmError> {
        let versions = self.list_remote_versions().await?;
        versions
            .into_iter()
            .find(|v| v.channel == Channel::Nightly)
            .ok_or_else(|| ZzmError::VersionNotFound {
                version: "master".to_string(),
            })
    }
}

// ========== 辅助函数 ==========

/// 已知的非平台键（ZigVersionEntry 的显式字段）
const KNOWN_NON_PLATFORM_KEYS: &[&str] = &[
    "date",
    "version",
    "docs",
    "stdDocs",
    "notes",
    "src",
    "bootstrap",
];

/// 判断一个键是否为平台标识键
///
/// 平台键格式：
/// - 新版: `x86_64-macos`, `aarch64-linux`, `x86_64-windows`
/// - 旧版: `macos-x86_64`, `linux-aarch64`, `windows-x86_64`
/// - 特殊: `arm-linux`, `riscv64-linux`, `armv7a-linux` 等
fn is_platform_key(key: &str) -> bool {
    // 已知的非平台键
    if KNOWN_NON_PLATFORM_KEYS.contains(&key) {
        return false;
    }

    // 平台键必须包含至少一个已知标识符
    let known_os = ["windows", "macos", "linux", "freebsd", "netbsd", "openbsd"];
    let known_arch = [
        "x86_64",
        "aarch64",
        "arm",
        "armv7a",
        "armv6kz",
        "i386",
        "x86",
        "riscv64",
        "powerpc64le",
        "powerpc64",
        "powerpc",
        "loongarch64",
        "s390x",
    ];

    let key_lower = key.to_lowercase();
    let has_os = known_os.iter().any(|os| key_lower.contains(os));
    let has_arch = known_arch
        .iter()
        .any(|arch| key_lower.contains(arch.to_lowercase().as_str()));

    has_os && has_arch
}

/// 在平台映射中查找匹配当前目标三元组的资源
fn find_matching_asset(
    platforms: &std::collections::BTreeMap<String, ZigPlatformAsset>,
    target_triple: &str,
) -> Option<ZigPlatformAsset> {
    let (os_name, arch_name) = crate::platform::parse_target_triple(target_triple)?;

    // 优先精确匹配：键名包含目标三元组（如 "x86_64-windows"）
    // 新版 API 格式: arch-os (如 "x86_64-windows")
    let exact_key = format!("{arch_name}-{os_name}");
    if let Some(asset) = platforms.get(&exact_key) {
        return Some(asset.clone());
    }

    // 旧版 API 格式: os-arch (如 "windows-x86_64" 或 "macos-x86_64")
    let legacy_key = format!("{os_name}-{arch_name}");
    if let Some(asset) = platforms.get(&legacy_key) {
        return Some(asset.clone());
    }

    // 模糊匹配：遍历所有平台键，查找包含目标 os 和 arch 的条目
    for (key, asset) in platforms {
        if !is_platform_key(key) {
            continue;
        }
        let key_lower = key.to_lowercase();
        if key_lower.contains(os_name) && key_lower.contains(arch_name) {
            return Some(asset.clone());
        }
    }

    None
}

/// 从 tarball URL 中提取文件名
///
/// 例如: "https://ziglang.org/download/0.16.0/zig-x86_64-windows-0.16.0.zip"
///   -> "zig-x86_64-windows-0.16.0.zip"
#[allow(dead_code)]
fn extract_filename_from_url(url: &str) -> String {
    url.rsplit('/').next().unwrap_or("zig-unknown").to_string()
}

/// 解析文件大小字符串为字节数
///
/// 支持两种格式：
/// - 数字字符串（API 返回的精确字节数，如 "55574528"）
/// - 人类可读格式（如 "53MiB"）
#[allow(
    dead_code,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]
pub fn parse_size_to_bytes(size_str: &str) -> u64 {
    let size_str = size_str.trim();

    // 先尝试纯数字解析（API 实际返回格式）
    if let Ok(bytes) = size_str.parse::<u64>() {
        return bytes;
    }

    // 回退到人类可读格式解析
    let num_part: String = size_str
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let unit_part: String = size_str
        .chars()
        .skip_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();

    let num: f64 = num_part.parse().unwrap_or(0.0);

    let multiplier = match unit_part.to_uppercase().as_str() {
        "B" => 1u64,
        "KIB" | "KB" | "K" => 1024,
        "MIB" | "MB" | "M" => 1024 * 1024,
        "GIB" | "GB" | "G" => 1024 * 1024 * 1024,
        _ => 1u64,
    };

    (num * multiplier as f64) as u64
}

// ========== VersionProvider 实现 ==========

impl crate::core::tool_manager::VersionProvider for ZigApiClient {
    async fn get_version_info(
        &self,
        version: &str,
    ) -> Result<crate::core::tool_manager::VersionInfo, ZzmError> {
        let info = self.get_version_info(version).await?;
        Ok(crate::core::tool_manager::VersionInfo {
            version: info.version,
            channel: info.channel,
            asset: info.asset.map(|a| {
                let filename = extract_filename_from_url(&a.tarball);
                crate::core::tool_manager::DownloadAsset {
                    url: a.tarball,
                    filename,
                    shasum: a.shasum,
                    size: a.size,
                }
            }),
        })
    }

    async fn list_remote_versions(
        &self,
    ) -> Result<Vec<crate::core::tool_manager::VersionInfo>, ZzmError> {
        let versions = self.list_remote_versions().await?;
        Ok(versions
            .into_iter()
            .map(|v| crate::core::tool_manager::VersionInfo {
                version: v.version,
                channel: v.channel,
                asset: v.asset.map(|a| {
                    let filename = extract_filename_from_url(&a.tarball);
                    crate::core::tool_manager::DownloadAsset {
                        url: a.tarball,
                        filename,
                        shasum: a.shasum,
                        size: a.size,
                    }
                }),
            })
            .collect())
    }
}

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size_to_bytes_numeric() {
        // API 实际返回的数字字符串
        assert_eq!(parse_size_to_bytes("55574528"), 55_574_528);
        assert_eq!(parse_size_to_bytes("22512620"), 22_512_620);
        assert_eq!(parse_size_to_bytes("0"), 0);
    }

    #[test]
    fn test_parse_size_to_bytes_human_readable() {
        assert_eq!(parse_size_to_bytes("53MiB"), 55_574_528);
        assert_eq!(parse_size_to_bytes("21MiB"), 22_020_096);
        assert_eq!(parse_size_to_bytes("1GiB"), 1_073_741_824);
        assert_eq!(parse_size_to_bytes("100KiB"), 102_400);
        assert_eq!(parse_size_to_bytes("512B"), 512);
        assert_eq!(parse_size_to_bytes("1KB"), 1024);
        assert_eq!(parse_size_to_bytes("1MB"), 1024 * 1024);
        assert_eq!(parse_size_to_bytes("2GiB"), 2 * 1024 * 1024 * 1024);
        assert_eq!(parse_size_to_bytes("1.5MiB"), 1_572_864);
    }

    #[test]
    fn test_parse_size_to_bytes_empty() {
        assert_eq!(parse_size_to_bytes(""), 0);
    }

    #[test]
    fn test_parse_target_triple() {
        assert_eq!(
            crate::platform::parse_target_triple("x86_64-windows"),
            Some(("windows", "x86_64"))
        );
        assert_eq!(
            crate::platform::parse_target_triple("aarch64-macos"),
            Some(("macos", "aarch64"))
        );
        assert_eq!(
            crate::platform::parse_target_triple("x86_64-linux"),
            Some(("linux", "x86_64"))
        );
        assert_eq!(crate::platform::parse_target_triple("unknown"), None);
    }

    #[test]
    fn test_zig_channel_serde() {
        let stable = Channel::Stable;
        let json = serde_json::to_string(&stable).unwrap();
        assert!(json.contains("Stable"));

        let nightly = Channel::Nightly;
        let json = serde_json::to_string(&nightly).unwrap();
        assert!(json.contains("Nightly"));
    }

    #[test]
    fn test_is_platform_key() {
        // 平台键
        assert!(is_platform_key("x86_64-macos"));
        assert!(is_platform_key("aarch64-linux"));
        assert!(is_platform_key("x86_64-windows"));
        assert!(is_platform_key("arm-linux"));
        assert!(is_platform_key("riscv64-linux"));
        assert!(is_platform_key("aarch64-freebsd"));
        assert!(is_platform_key("x86_64-netbsd"));
        assert!(is_platform_key("x86_64-openbsd"));
        assert!(is_platform_key("powerpc64le-linux"));
        assert!(is_platform_key("loongarch64-linux"));
        assert!(is_platform_key("s390x-linux"));
        // 旧版格式
        assert!(is_platform_key("macos-x86_64"));
        assert!(is_platform_key("linux-aarch64"));
        assert!(is_platform_key("windows-x86_64"));

        // 非平台键
        assert!(!is_platform_key("date"));
        assert!(!is_platform_key("version"));
        assert!(!is_platform_key("docs"));
        assert!(!is_platform_key("stdDocs"));
        assert!(!is_platform_key("notes"));
        assert!(!is_platform_key("src"));
        assert!(!is_platform_key("bootstrap"));
    }

    #[test]
    fn test_find_matching_asset_new_format() {
        // 新版 API 格式: arch-os 键名
        let mut platforms = std::collections::BTreeMap::new();
        platforms.insert(
            "x86_64-macos".to_string(),
            ZigPlatformAsset {
                tarball: "https://ziglang.org/download/0.16.0/zig-x86_64-macos-0.16.0.tar.xz"
                    .to_string(),
                shasum: "abc123".to_string(),
                size: "57396836".to_string(),
            },
        );
        platforms.insert(
            "x86_64-windows".to_string(),
            ZigPlatformAsset {
                tarball: "https://ziglang.org/download/0.16.0/zig-x86_64-windows-0.16.0.zip"
                    .to_string(),
                shasum: "def456".to_string(),
                size: "97217739".to_string(),
            },
        );

        let result = find_matching_asset(&platforms, "x86_64-windows");
        assert!(result.is_some());
        assert_eq!(
            result.unwrap().tarball,
            "https://ziglang.org/download/0.16.0/zig-x86_64-windows-0.16.0.zip"
        );
    }

    #[test]
    fn test_find_matching_asset_legacy_format() {
        // 旧版 API 格式: os-arch 键名 (如 0.13.0 及更早版本)
        let mut platforms = std::collections::BTreeMap::new();
        platforms.insert(
            "macos-x86_64".to_string(),
            ZigPlatformAsset {
                tarball: "https://ziglang.org/download/0.13.0/zig-macos-x86_64-0.13.0.tar.xz"
                    .to_string(),
                shasum: "abc123".to_string(),
                size: "48857012".to_string(),
            },
        );
        platforms.insert(
            "windows-x86_64".to_string(),
            ZigPlatformAsset {
                tarball: "https://ziglang.org/download/0.13.0/zig-windows-x86_64-0.13.0.zip"
                    .to_string(),
                shasum: "def456".to_string(),
                size: "79163968".to_string(),
            },
        );

        let result = find_matching_asset(&platforms, "x86_64-windows");
        assert!(result.is_some());
        assert!(result.unwrap().tarball.contains("windows"));
    }

    #[test]
    fn test_find_matching_asset_empty() {
        let platforms = std::collections::BTreeMap::new();
        assert!(find_matching_asset(&platforms, "x86_64-windows").is_none());
    }

    #[test]
    fn test_find_matching_asset_not_found() {
        let mut platforms = std::collections::BTreeMap::new();
        platforms.insert(
            "x86_64-linux".to_string(),
            ZigPlatformAsset {
                tarball: "https://example.com/zig-linux.tar.xz".to_string(),
                shasum: "abc".to_string(),
                size: "50000000".to_string(),
            },
        );

        assert!(find_matching_asset(&platforms, "x86_64-windows").is_none());
    }

    #[test]
    fn test_extract_filename_from_url() {
        assert_eq!(
            extract_filename_from_url(
                "https://ziglang.org/download/0.16.0/zig-x86_64-windows-0.16.0.zip"
            ),
            "zig-x86_64-windows-0.16.0.zip"
        );
        assert_eq!(
            extract_filename_from_url(
                "https://ziglang.org/builds/zig-x86_64-macos-0.17.0-dev.101+4e2147d14.tar.xz"
            ),
            "zig-x86_64-macos-0.17.0-dev.101+4e2147d14.tar.xz"
        );
    }

    #[test]
    fn test_zig_platform_asset_deserialization() {
        let json = r#"{
            "tarball": "https://ziglang.org/download/0.16.0/zig-x86_64-windows-0.16.0.zip",
            "shasum": "68659eb5f1e4eb1437a722f1dd889c5a322c9954607f5edcf337bc3684a75a7e",
            "size": "97217739"
        }"#;

        let asset: ZigPlatformAsset = serde_json::from_str(json).unwrap();
        assert_eq!(
            asset.shasum,
            "68659eb5f1e4eb1437a722f1dd889c5a322c9954607f5edcf337bc3684a75a7e"
        );
        assert_eq!(asset.size, "97217739");
        assert!(asset.tarball.contains("0.16.0"));
    }

    #[test]
    fn test_zig_version_entry_deserialization() {
        let json = r#"{
            "version": "0.16.0",
            "date": "2026-04-13",
            "docs": "https://ziglang.org/documentation/0.16.0/",
            "stdDocs": "https://ziglang.org/documentation/0.16.0/std/",
            "notes": "https://ziglang.org/download/0.16.0/release-notes.html",
            "src": {
                "tarball": "https://ziglang.org/download/0.16.0/zig-0.16.0.tar.xz",
                "shasum": "abc123",
                "size": "22503260"
            },
            "x86_64-macos": {
                "tarball": "https://ziglang.org/download/0.16.0/zig-x86_64-macos-0.16.0.tar.xz",
                "shasum": "def456",
                "size": "57396836"
            },
            "x86_64-windows": {
                "tarball": "https://ziglang.org/download/0.16.0/zig-x86_64-windows-0.16.0.zip",
                "shasum": "ghi789",
                "size": "97217739"
            }
        }"#;

        let entry: ZigVersionEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.version, "0.16.0");
        assert_eq!(entry.date, "2026-04-13");
        assert_eq!(entry.docs, "https://ziglang.org/documentation/0.16.0/");
        assert_eq!(
            entry.std_docs,
            "https://ziglang.org/documentation/0.16.0/std/"
        );
        assert!(entry.src.is_some());
        // platforms 应该包含 x86_64-macos 和 x86_64-windows（不含 src，因为 src 是显式字段）
        // 注意：由于 serde flatten 的行为，src 也会出现在 platforms map 中
        // 这是预期行为 —— find_matching_asset 通过 is_platform_key 过滤
        assert!(entry.platforms.contains_key("x86_64-macos"));
        assert!(entry.platforms.contains_key("x86_64-windows"));
    }

    #[test]
    fn test_zig_version_entry_master() {
        let json = r#"{
            "version": "0.17.0-dev.101+4e2147d14",
            "date": "2026-04-24",
            "docs": "https://ziglang.org/documentation/master/",
            "stdDocs": "https://ziglang.org/documentation/master/std/",
            "x86_64-windows": {
                "tarball": "https://ziglang.org/builds/zig-x86_64-windows-0.17.0-dev.101+4e2147d14.zip",
                "shasum": "abc123",
                "size": "97892217"
            }
        }"#;

        let entry: ZigVersionEntry = serde_json::from_str(json).unwrap();
        assert!(entry.version.contains("0.17.0-dev"));
        assert!(entry.platforms.contains_key("x86_64-windows"));
    }

    #[test]
    fn test_zig_version_entry_old_format() {
        // 0.13.0 及更早版本使用 os-arch 格式的键名
        let json = r#"{
            "date": "2024-06-07",
            "docs": "https://ziglang.org/documentation/0.13.0/",
            "x86_64-macos": {
                "tarball": "https://ziglang.org/download/0.13.0/zig-macos-x86_64-0.13.0.tar.xz",
                "shasum": "8b06ed1091b2269b700b3b07f8e3be3b833000841bae5aa6a09b1a8b4773effd",
                "size": "48857012"
            },
            "x86_64-windows": {
                "tarball": "https://ziglang.org/download/0.13.0/zig-windows-x86_64-0.13.0.zip",
                "shasum": "d859994725ef9402381e557c60bb57497215682e355204d754ee3df75ee3c158",
                "size": "79163968"
            }
        }"#;

        let entry: ZigVersionEntry = serde_json::from_str(json).unwrap();
        assert!(entry.platforms.contains_key("x86_64-macos"));
        assert!(entry.platforms.contains_key("x86_64-windows"));
    }

    #[test]
    fn test_zig_download_index_deserialization_real_data() {
        // 使用模拟的真实 API 响应数据片段
        let json = r#"{
            "0.16.0": {
                "version": "0.16.0",
                "date": "2026-04-13",
                "docs": "https://ziglang.org/documentation/0.16.0/",
                "stdDocs": "https://ziglang.org/documentation/0.16.0/std/",
                "src": {
                    "tarball": "https://ziglang.org/download/0.16.0/zig-0.16.0.tar.xz",
                    "shasum": "43186959edc87d5c7a1be7b7d2a25efffd22ce5807c7af99067f86f99641bfdf",
                    "size": "22503260"
                },
                "x86_64-windows": {
                    "tarball": "https://ziglang.org/download/0.16.0/zig-x86_64-windows-0.16.0.zip",
                    "shasum": "68659eb5f1e4eb1437a722f1dd889c5a322c9954607f5edcf337bc3684a75a7e",
                    "size": "97217739"
                }
            },
            "master": {
                "version": "0.17.0-dev.101+4e2147d14",
                "date": "2026-04-24",
                "docs": "https://ziglang.org/documentation/master/",
                "x86_64-windows": {
                    "tarball": "https://ziglang.org/builds/zig-x86_64-windows-0.17.0-dev.101+4e2147d14.zip",
                    "shasum": "1b65f0cb67b78850ec994f6d993a8e1766a6f91a86482b5a4342f7c07e8dd822",
                    "size": "97892217"
                }
            }
        }"#;

        let index: ZigDownloadIndex = serde_json::from_str(json).unwrap();
        assert!(index.0.contains_key("0.16.0"));
        assert!(index.0.contains_key("master"));

        let v016 = &index.0["0.16.0"];
        assert_eq!(v016.version, "0.16.0");
        assert!(v016.src.is_some());
        assert!(v016.platforms.contains_key("x86_64-windows"));
    }

    #[test]
    fn test_zig_channel_equality() {
        assert_eq!(Channel::Stable, Channel::Stable);
        assert_eq!(Channel::Nightly, Channel::Nightly);
        assert_ne!(Channel::Stable, Channel::Nightly);
    }

    #[test]
    fn test_zig_api_client_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let client = ZigApiClient::new(temp_dir.path().to_path_buf());
        assert!(client.is_ok());
    }

    #[test]
    fn test_zig_version_info_serialization() {
        let info = ZigVersionInfo {
            version: "0.16.0".to_string(),
            channel: Channel::Stable,
            date: "2026-04-13".to_string(),
            asset: None,
        };

        let json = serde_json::to_string(&info).unwrap();
        let parsed: ZigVersionInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, "0.16.0");
        assert_eq!(parsed.channel, Channel::Stable);
    }

    #[test]
    fn test_find_matching_asset_aarch64_macos() {
        let mut platforms = std::collections::BTreeMap::new();
        platforms.insert(
            "aarch64-macos".to_string(),
            ZigPlatformAsset {
                tarball: "https://ziglang.org/download/0.16.0/zig-aarch64-macos-0.16.0.tar.xz"
                    .to_string(),
                shasum: "abc".to_string(),
                size: "52238004".to_string(),
            },
        );

        let result = find_matching_asset(&platforms, "aarch64-macos");
        assert!(result.is_some());
        assert!(result.unwrap().tarball.contains("aarch64-macos"));
    }

    #[test]
    fn test_find_matching_asset_x86_linux() {
        let mut platforms = std::collections::BTreeMap::new();
        platforms.insert(
            "x86-linux".to_string(),
            ZigPlatformAsset {
                tarball: "https://ziglang.org/download/0.16.0/zig-x86-linux-0.16.0.tar.xz"
                    .to_string(),
                shasum: "abc".to_string(),
                size: "58196012".to_string(),
            },
        );

        // x86-linux 不是标准 target_triple，但通过模糊匹配仍可找到
        // 注意：parse_target_triple 不支持 x86-linux，所以这个测试验证模糊匹配
        let result = find_matching_asset(&platforms, "x86_64-linux");
        // x86_64-linux 不匹配 x86-linux（因为 "x86_64" 不包含在 "x86-linux" 中作为子串）
        // 但 "linux" 匹配，"x86_64" 不匹配 "x86"
        assert!(result.is_none()); // 正确行为：x86_64 != x86
    }
}
