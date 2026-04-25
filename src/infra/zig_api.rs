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
/// 键为版本号字符串（如 "0.13.0", "0.12.1", "master"），
/// 值为对应版本信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigDownloadIndex(pub std::collections::BTreeMap<String, ZigVersionEntry>);

/// 单个 Zig 版本的详细信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigVersionEntry {
    /// 构建日期 (YYYY-MM-DD)
    pub date: String,
    /// 文档链接
    #[serde(default)]
    pub docs: std::collections::BTreeMap<String, String>,
    /// 通用发布文件列表（源码等）
    #[serde(default)]
    pub releases: Vec<ZigReleaseAsset>,
    /// 按平台分类的预编译二进制
    #[serde(default)]
    pub platforms: ZigPlatforms,
}

/// 按平台分类的二进制文件
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ZigPlatforms {
    #[serde(default)]
    pub windows: Vec<ZigPlatformAsset>,
    #[serde(default)]
    pub macos: Vec<ZigPlatformAsset>,
    #[serde(default)]
    pub linux: Vec<ZigPlatformAsset>,
}

/// 通用发布文件（源码包等）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigReleaseAsset {
    /// 文件类型: "Source", "Bootstrap" 等
    #[serde(rename = "type", default)]
    pub asset_type: String,
    /// 目标平台（源码为 null）
    pub target: Option<String>,
    /// 文件名
    pub filename: String,
    /// 文件大小（人类可读，如 "21MiB"）
    #[serde(default)]
    pub size: String,
    /// SHA256 校验和
    #[serde(default)]
    pub shasum: String,
    /// 数字签名
    #[serde(default)]
    pub signature: Option<ZigSignature>,
    /// 下载 URL
    #[serde(default)]
    pub url: String,
}

/// 平台特定的预编译二进制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigPlatformAsset {
    /// 操作系统名称
    pub os: String,
    /// CPU 架构
    pub arch: String,
    /// 文件名
    pub filename: String,
    /// 文件大小（人类可读）
    #[serde(default)]
    pub size: String,
    /// SHA256 校验和
    #[serde(default)]
    pub shasum: String,
    /// 数字签名
    #[serde(default)]
    pub signature: Option<ZigSignature>,
    /// 下载 URL
    pub url: String,
}

/// minisign 签名信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigSignature {
    #[serde(rename = "type", default)]
    pub sig_type: String,
    #[serde(default)]
    pub file: String,
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

/// 在平台列表中查找匹配当前目标三元组的资源
fn find_matching_asset(platforms: &ZigPlatforms, target_triple: &str) -> Option<ZigPlatformAsset> {
    let (os_name, arch_name) = crate::platform::parse_target_triple(target_triple)?;

    let platform_list = match os_name {
        "windows" => &platforms.windows,
        "macos" => &platforms.macos,
        "linux" => &platforms.linux,
        _ => return None,
    };

    platform_list
        .iter()
        .find(|asset| {
            let asset_os = asset.os.to_lowercase();
            let asset_arch = asset.arch.to_lowercase();
            asset_os.contains(os_name) && asset_arch.contains(arch_name)
        })
        .cloned()
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
            asset: info
                .asset
                .map(|a| crate::core::tool_manager::DownloadAsset {
                    url: a.url,
                    filename: a.filename,
                    shasum: a.shasum,
                    size: a.size,
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
                asset: v.asset.map(|a| crate::core::tool_manager::DownloadAsset {
                    url: a.url,
                    filename: a.filename,
                    shasum: a.shasum,
                    size: a.size,
                }),
            })
            .collect())
    }
}

/// 解析人类可读的文件大小字符串为字节数
///
/// 例如: "53MiB" -> 55574528
#[allow(
    dead_code,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]
pub fn parse_size_to_bytes(size_str: &str) -> u64 {
    let size_str = size_str.trim();
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

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size_to_bytes() {
        assert_eq!(parse_size_to_bytes("53MiB"), 55574528);
        assert_eq!(parse_size_to_bytes("21MiB"), 22020096);
        assert_eq!(parse_size_to_bytes("1GiB"), 1073741824);
        assert_eq!(parse_size_to_bytes("100KiB"), 102400);
        assert_eq!(parse_size_to_bytes("512B"), 512);
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
    fn test_find_matching_asset_empty() {
        let platforms = ZigPlatforms {
            windows: vec![],
            macos: vec![],
            linux: vec![],
        };
        assert!(find_matching_asset(&platforms, "x86_64-windows").is_none());
    }

    #[test]
    fn test_find_matching_asset_with_data() {
        let asset = ZigPlatformAsset {
            os: "Windows".to_string(),
            arch: "x86_64".to_string(),
            filename: "zig-x86_64-windows-0.13.0.zip".to_string(),
            size: "93MiB".to_string(),
            shasum: "abc123".to_string(),
            signature: None,
            url: "https://ziglang.org/download/0.13.0/zig-x86_64-windows-0.13.0.zip".to_string(),
        };

        let platforms = ZigPlatforms {
            windows: vec![asset],
            macos: vec![],
            linux: vec![],
        };

        let result = find_matching_asset(&platforms, "x86_64-windows");
        assert!(result.is_some());
        assert_eq!(result.unwrap().filename, "zig-x86_64-windows-0.13.0.zip");
    }

    #[test]
    fn test_parse_size_to_bytes_various() {
        assert_eq!(parse_size_to_bytes("0B"), 0);
        assert_eq!(parse_size_to_bytes("1KB"), 1024);
        assert_eq!(parse_size_to_bytes("1MB"), 1024 * 1024);
        assert_eq!(parse_size_to_bytes("2GiB"), 2 * 1024 * 1024 * 1024);
        assert_eq!(parse_size_to_bytes("1.5MiB"), 1572864);
    }

    #[test]
    fn test_parse_size_to_bytes_empty() {
        assert_eq!(parse_size_to_bytes(""), 0);
    }

    #[test]
    fn test_parse_target_triple_all() {
        assert_eq!(
            crate::platform::parse_target_triple("x86_64-windows"),
            Some(("windows", "x86_64"))
        );
        assert_eq!(
            crate::platform::parse_target_triple("aarch64-windows"),
            Some(("windows", "aarch64"))
        );
        assert_eq!(
            crate::platform::parse_target_triple("x86_64-macos"),
            Some(("macos", "x86_64"))
        );
        assert_eq!(
            crate::platform::parse_target_triple("aarch64-macos"),
            Some(("macos", "aarch64"))
        );
        assert_eq!(
            crate::platform::parse_target_triple("x86_64-linux"),
            Some(("linux", "x86_64"))
        );
        assert_eq!(
            crate::platform::parse_target_triple("aarch64-linux"),
            Some(("linux", "aarch64"))
        );
    }

    #[test]
    fn test_zig_channel_equality() {
        assert_eq!(Channel::Stable, Channel::Stable);
        assert_eq!(Channel::Nightly, Channel::Nightly);
        assert_ne!(Channel::Stable, Channel::Nightly);
    }

    #[test]
    fn test_find_matching_asset_macos() {
        let asset = ZigPlatformAsset {
            os: "macos".to_string(),
            arch: "aarch64".to_string(),
            filename: "zig-aarch64-macos-0.13.0.tar.xz".to_string(),
            size: "42MiB".to_string(),
            shasum: "def456".to_string(),
            signature: None,
            url: "https://ziglang.org/download/0.13.0/zig-aarch64-macos-0.13.0.tar.xz".to_string(),
        };

        let platforms = ZigPlatforms {
            windows: vec![],
            macos: vec![asset],
            linux: vec![],
        };

        let result = find_matching_asset(&platforms, "aarch64-macos");
        assert!(result.is_some());
        assert_eq!(result.unwrap().filename, "zig-aarch64-macos-0.13.0.tar.xz");
    }

    #[test]
    fn test_find_matching_asset_linux() {
        let asset = ZigPlatformAsset {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            filename: "zig-x86_64-linux-0.13.0.tar.xz".to_string(),
            size: "42MiB".to_string(),
            shasum: "ghi789".to_string(),
            signature: None,
            url: "https://ziglang.org/download/0.13.0/zig-x86_64-linux-0.13.0.tar.xz".to_string(),
        };

        let platforms = ZigPlatforms {
            windows: vec![],
            macos: vec![],
            linux: vec![asset],
        };

        let result = find_matching_asset(&platforms, "x86_64-linux");
        assert!(result.is_some());
    }

    #[test]
    fn test_zig_version_info_serialization() {
        let info = ZigVersionInfo {
            version: "0.13.0".to_string(),
            channel: Channel::Stable,
            date: "2024-06-06".to_string(),
            asset: None,
        };

        let json = serde_json::to_string(&info).unwrap();
        let parsed: ZigVersionInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, "0.13.0");
        assert_eq!(parsed.channel, Channel::Stable);
    }

    #[test]
    fn test_zig_platform_asset_serialization() {
        let asset = ZigPlatformAsset {
            os: "windows".to_string(),
            arch: "x86_64".to_string(),
            filename: "zig-x86_64-windows-0.13.0.zip".to_string(),
            size: "93MiB".to_string(),
            shasum: "abc123".to_string(),
            signature: None,
            url: "https://example.com/zig.zip".to_string(),
        };

        let json = serde_json::to_string(&asset).unwrap();
        let parsed: ZigPlatformAsset = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.filename, "zig-x86_64-windows-0.13.0.zip");
        assert_eq!(parsed.shasum, "abc123");
    }

    #[test]
    fn test_zig_api_client_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let client = ZigApiClient::new(temp_dir.path().to_path_buf());
        assert!(client.is_ok());
    }

    #[test]
    fn test_zig_download_index_deserialization() {
        let json = r#"{
            "0.13.0": {
                "date": "2024-06-06",
                "platforms": {
                    "windows": [],
                    "macos": [],
                    "linux": []
                }
            }
        }"#;

        let index: ZigDownloadIndex = serde_json::from_str(json).unwrap();
        assert!(index.0.contains_key("0.13.0"));
    }
}
