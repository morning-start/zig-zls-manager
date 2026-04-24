use std::path::PathBuf;
use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};

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
    pub channel: ZigChannel,
    /// 构建日期
    pub date: String,
    /// 当前平台匹配的下载资源
    pub asset: Option<ZigPlatformAsset>,
}

/// Zig 版本通道
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZigChannel {
    /// 稳定发布版
    Stable,
    /// 开发版 (master/nightly)
    Nightly,
}

// ========== API 客户端 ==========

/// Zig API 客户端
pub struct ZigApiClient {
    client: Client,
    cache_dir: PathBuf,
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

        Ok(Self { client, cache_dir })
    }

    /// 获取缓存文件路径
    fn cache_path(&self) -> PathBuf {
        self.cache_dir.join(CACHE_FILENAME)
    }

    /// 从缓存加载索引数据（如果未过期）
    fn load_from_cache(&self) -> Option<ZigDownloadIndex> {
        let path = self.cache_path();
        if !path.exists() {
            return None;
        }

        // 检查缓存文件修改时间
        let metadata = std::fs::metadata(&path).ok()?;
        let modified = metadata.modified().ok()?;
        let elapsed = modified.elapsed().ok()?;

        if elapsed > CACHE_TTL {
            tracing::debug!("Zig API 缓存已过期");
            return None;
        }

        let content = std::fs::read_to_string(&path).ok()?;
        let index: ZigDownloadIndex = serde_json::from_str(&content).ok()?;
        tracing::debug!("从缓存加载 Zig 索引数据");
        Some(index)
    }

    /// 将索引数据写入缓存
    fn save_to_cache(&self, index: &ZigDownloadIndex) -> Result<(), ZzmError> {
        let path = self.cache_path();
        let content = serde_json::to_string_pretty(index)?;
        std::fs::write(&path, content)?;
        tracing::debug!("Zig 索引数据已缓存");
        Ok(())
    }

    /// 从远程 API 获取 Zig 下载索引（带缓存）
    pub async fn fetch_index(&self) -> Result<ZigDownloadIndex, ZzmError> {
        // 先尝试从缓存读取
        if let Some(cached) = self.load_from_cache() {
            return Ok(cached);
        }

        tracing::debug!("从远程获取 Zig 索引数据: {}", ZIG_INDEX_URL);

        let response = self
            .client
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
        self.save_to_cache(&index)?;

        Ok(index)
    }

    /// 获取所有远程可用的 Zig 版本列表
    pub async fn list_remote_versions(&self) -> Result<Vec<ZigVersionInfo>, ZzmError> {
        let index = self.fetch_index().await?;
        let target_triple = crate::platform::current_target_triple();

        let mut versions = Vec::new();

        for (version_str, entry) in &index.0 {
            let channel = if version_str == "master" {
                ZigChannel::Nightly
            } else {
                ZigChannel::Stable
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
                (ZigChannel::Nightly, ZigChannel::Nightly) => std::cmp::Ordering::Equal,
                (ZigChannel::Nightly, ZigChannel::Stable) => std::cmp::Ordering::Greater,
                (ZigChannel::Stable, ZigChannel::Nightly) => std::cmp::Ordering::Less,
                (ZigChannel::Stable, ZigChannel::Stable) => {
                    // 尝试按语义版本比较
                    let va: std::result::Result<crate::utils::version::Version, _> = a.version.parse();
                    let vb: std::result::Result<crate::utils::version::Version, _> = b.version.parse();
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

        let entry = index.0.get(&resolved).ok_or_else(|| ZzmError::VersionNotFound {
            version: version.to_string(),
        })?;

        let channel = if resolved == "master" {
            ZigChannel::Nightly
        } else {
            ZigChannel::Stable
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
    pub async fn get_latest_stable(&self) -> Result<ZigVersionInfo, ZzmError> {
        let versions = self.list_remote_versions().await?;
        versions
            .into_iter()
            .find(|v| v.channel == ZigChannel::Stable)
            .ok_or_else(|| ZzmError::VersionNotFound {
                version: "stable".to_string(),
            })
    }

    /// 获取 master (nightly) 版本
    pub async fn get_master(&self) -> Result<ZigVersionInfo, ZzmError> {
        let versions = self.list_remote_versions().await?;
        versions
            .into_iter()
            .find(|v| v.channel == ZigChannel::Nightly)
            .ok_or_else(|| ZzmError::VersionNotFound {
                version: "master".to_string(),
            })
    }
}

// ========== 辅助函数 ==========

/// 在平台列表中查找匹配当前目标三元组的资源
fn find_matching_asset(
    platforms: &ZigPlatforms,
    target_triple: &str,
) -> Option<ZigPlatformAsset> {
    let (os_name, arch_name) = parse_target_triple(target_triple)?;

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

/// 解析目标三元组为 (os, arch)
fn parse_target_triple(triple: &str) -> Option<(&str, &str)> {
    match triple {
        "x86_64-windows" => Some(("windows", "x86_64")),
        "aarch64-windows" => Some(("windows", "aarch64")),
        "x86_64-macos" => Some(("macos", "x86_64")),
        "aarch64-macos" => Some(("macos", "aarch64")),
        "x86_64-linux" => Some(("linux", "x86_64")),
        "aarch64-linux" => Some(("linux", "aarch64")),
        _ => None,
    }
}

/// 解析人类可读的文件大小字符串为字节数
///
/// 例如: "53MiB" -> 55574528
pub fn parse_size_to_bytes(size_str: &str) -> u64 {
    let size_str = size_str.trim();
    let num_part: String = size_str.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
    let unit_part: String = size_str.chars().skip_while(|c| c.is_ascii_digit() || *c == '.').collect();

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
        assert_eq!(parse_target_triple("x86_64-windows"), Some(("windows", "x86_64")));
        assert_eq!(parse_target_triple("aarch64-macos"), Some(("macos", "aarch64")));
        assert_eq!(parse_target_triple("x86_64-linux"), Some(("linux", "x86_64")));
        assert_eq!(parse_target_triple("unknown"), None);
    }

    #[test]
    fn test_zig_channel_serde() {
        let stable = ZigChannel::Stable;
        let json = serde_json::to_string(&stable).unwrap();
        assert!(json.contains("Stable"));

        let nightly = ZigChannel::Nightly;
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
            windows: vec![asset.clone()],
            macos: vec![],
            linux: vec![],
        };

        let result = find_matching_asset(&platforms, "x86_64-windows");
        assert!(result.is_some());
        assert_eq!(result.unwrap().filename, "zig-x86_64-windows-0.13.0.zip");
    }
}