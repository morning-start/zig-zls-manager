use std::path::PathBuf;
use std::time::Duration;

use reqwest::Client;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};

use crate::core::channel::Channel;
use crate::utils::error::ZzmError;

/// ZLS GitHub Releases API 端点
const ZLS_RELEASES_URL: &str = "https://api.github.com/repos/zigtools/zls/releases";

/// 缓存 TTL: 1 小时
const CACHE_TTL: Duration = Duration::from_secs(3600);

/// 缓存文件名
const CACHE_FILENAME: &str = "zls_releases.json";

/// GitHub Token 环境变量名
const GITHUB_TOKEN_ENV: &str = "GITHUB_TOKEN";

// ========== API 数据结构 ==========

/// GitHub Release 对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubRelease {
    /// Release URL
    pub url: String,
    /// HTML 页面 URL
    pub html_url: String,
    /// Release ID
    pub id: u64,
    /// Git 标签名（即版本号）
    pub tag_name: String,
    /// Release 标题
    pub name: String,
    /// 是否为草稿
    pub draft: bool,
    /// 是否为预发布版本
    pub prerelease: bool,
    /// 创建时间
    pub created_at: String,
    /// 发布时间
    pub published_at: Option<String>,
    /// 附件列表
    pub assets: Vec<GithubAsset>,
    /// Release 说明（Markdown）
    #[serde(default)]
    pub body: Option<String>,
}

/// GitHub Asset 对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubAsset {
    /// Asset ID
    pub id: u64,
    /// 文件名
    pub name: String,
    /// 自定义标签
    pub label: Option<String>,
    /// MIME 类型
    pub content_type: String,
    /// 上传状态
    pub state: String,
    /// 文件大小（字节）
    pub size: u64,
    /// 下载次数
    pub download_count: u64,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
    /// 直接下载链接
    pub browser_download_url: String,
}

// ========== 统一版本信息结构 ==========

/// ZLS 版本信息（供内部使用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZlsVersionInfo {
    /// 版本号字符串（来自 `tag_name`）
    pub version: String,
    /// 版本通道
    pub channel: Channel,
    /// 发布时间
    pub published_at: Option<String>,
    /// 当前平台匹配的下载资源
    pub asset: Option<GithubAsset>,
    /// Release 页面 URL
    pub html_url: String,
}

// ========== API 客户端 ==========

/// ZLS GitHub API 客户端
pub struct ZlsApiClient {
    client: Client,
    cache: crate::infra::api_cache::ApiCache<Vec<GithubRelease>>,
}

impl ZlsApiClient {
    /// 创建新的 API 客户端
    pub fn new(cache_dir: PathBuf) -> Result<Self, ZzmError> {
        let mut headers = HeaderMap::new();

        // 如果有 GitHub Token，添加认证头以提高速率限制
        if let Ok(token) = std::env::var(GITHUB_TOKEN_ENV)
            && !token.is_empty()
        {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {token}"))
                    .unwrap_or_else(|_| HeaderValue::from_static("")),
            );
            tracing::debug!("已配置 GitHub Token 认证");
        }

        let client = Client::builder()
            .user_agent(format!("zzm/{}", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .default_headers(headers)
            .build()
            .map_err(ZzmError::Network)?;

        // 确保缓存目录存在
        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir).map_err(ZzmError::Io)?;
        }

        let cache = crate::infra::api_cache::ApiCache::new(cache_dir, CACHE_FILENAME, CACHE_TTL);

        Ok(Self { client, cache })
    }

    /// 从远程 API 获取 ZLS Releases 列表（带缓存和重试）
    pub async fn fetch_releases(&self) -> Result<Vec<GithubRelease>, ZzmError> {
        // 先尝试从缓存读取
        if let Some(cached) = self.cache.load() {
            return Ok(cached);
        }

        tracing::debug!("从远程获取 ZLS releases: {}", ZLS_RELEASES_URL);

        let releases = self.fetch_with_retry(3).await?;

        // 保存到缓存
        self.cache.save(&releases)?;

        Ok(releases)
    }

    /// 带重试的获取请求
    async fn fetch_with_retry(&self, max_retries: u32) -> Result<Vec<GithubRelease>, ZzmError> {
        let mut last_error = None;

        for attempt in 1..=max_retries {
            match self.fetch_single_page().await {
                Ok(releases) => return Ok(releases),
                Err(e) => {
                    tracing::warn!("ZLS API 请求失败 (尝试 {}/{}): {}", attempt, max_retries, e);
                    last_error = Some(e);

                    // 检查是否为速率限制错误
                    if let ZzmError::RateLimited { retry_after } = last_error.as_ref().unwrap() {
                        tracing::warn!("速率限制，等待 {} 秒后重试", retry_after);
                        tokio::time::sleep(Duration::from_secs(*retry_after)).await;
                        continue;
                    }

                    if attempt < max_retries {
                        let delay = Duration::from_millis(100 * 2u64.pow(attempt));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    /// 获取单页 `releases（per_page=100`）
    async fn fetch_single_page(&self) -> Result<Vec<GithubRelease>, ZzmError> {
        let response = self
            .client
            .get(ZLS_RELEASES_URL)
            .query(&[("per_page", "100")])
            .send()
            .await
            .map_err(|e| ZzmError::DownloadFailed {
                url: ZLS_RELEASES_URL.to_string(),
                reason: e.to_string(),
            })?;

        let status = response.status();

        // 检查速率限制
        if status.as_u16() == 403 {
            let remaining = response
                .headers()
                .get("x-ratelimit-remaining")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok());

            if remaining == Some(0) {
                let reset_time = response
                    .headers()
                    .get("x-ratelimit-reset")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(60);

                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let retry_after = reset_time.saturating_sub(now).max(1);

                return Err(ZzmError::RateLimited { retry_after });
            }
        }

        if !status.is_success() {
            let status_code = status.as_u16();
            let message = response.text().await.unwrap_or_default();
            return Err(ZzmError::HttpError {
                status_code,
                message,
            });
        }

        let releases: Vec<GithubRelease> = response.json().await?;
        Ok(releases)
    }

    /// 获取所有远程可用的 ZLS 版本列表（过滤掉 draft）
    pub async fn list_remote_versions(&self) -> Result<Vec<ZlsVersionInfo>, ZzmError> {
        let releases = self.fetch_releases().await?;
        let target_triple = crate::platform::current_target_triple();

        let mut versions: Vec<ZlsVersionInfo> = releases
            .into_iter()
            .filter(|r| !r.draft) // 过滤掉草稿
            .map(|release| {
                let channel = if release.prerelease {
                    Channel::Prerelease
                } else {
                    Channel::Stable
                };

                let asset = find_matching_zls_asset(&release.assets, target_triple);

                ZlsVersionInfo {
                    version: release.tag_name.clone(),
                    channel,
                    published_at: release.published_at.clone(),
                    asset,
                    html_url: release.html_url,
                }
            })
            .collect();

        // 稳定版在前，按版本号降序排列
        versions.sort_by(|a, b| match (&a.channel, &b.channel) {
            (Channel::Prerelease, Channel::Prerelease) => std::cmp::Ordering::Equal,
            (Channel::Prerelease, _) => std::cmp::Ordering::Greater,
            (Channel::Nightly, Channel::Prerelease) => std::cmp::Ordering::Less,
            (Channel::Nightly, Channel::Nightly) => std::cmp::Ordering::Equal,
            (Channel::Nightly, Channel::Stable) => std::cmp::Ordering::Greater,
            (Channel::Stable, Channel::Prerelease) => std::cmp::Ordering::Less,
            (Channel::Stable, Channel::Nightly) => std::cmp::Ordering::Less,
            (Channel::Stable, Channel::Stable) => {
                let va: std::result::Result<crate::utils::version::Version, _> = a.version.parse();
                let vb: std::result::Result<crate::utils::version::Version, _> = b.version.parse();
                match (va, vb) {
                    (Ok(va), Ok(vb)) => vb.cmp(&va),
                    _ => b.version.cmp(&a.version),
                }
            }
        });

        Ok(versions)
    }

    /// 根据版本号获取特定版本的下载信息
    pub async fn get_version_info(&self, version: &str) -> Result<ZlsVersionInfo, ZzmError> {
        let versions = self.list_remote_versions().await?;
        versions
            .into_iter()
            .find(|v| v.version == version)
            .ok_or_else(|| ZzmError::VersionNotFound {
                version: version.to_string(),
            })
    }

    /// 获取最新稳定版本
    #[allow(dead_code)] // 预留: zzm zls install latest 命令
    pub async fn get_latest_stable(&self) -> Result<ZlsVersionInfo, ZzmError> {
        let versions = self.list_remote_versions().await?;
        versions
            .into_iter()
            .find(|v| v.channel == Channel::Stable)
            .ok_or_else(|| ZzmError::VersionNotFound {
                version: "zls stable".to_string(),
            })
    }

    /// 根据兼容性规则查找匹配 Zig 版本的 ZLS 版本
    ///
    /// 当前使用简单规则：ZLS x.y.z 匹配 Zig x.y.z
    pub async fn find_compatible_version(
        &self,
        zig_version: &str,
    ) -> Result<ZlsVersionInfo, ZzmError> {
        let versions = self.list_remote_versions().await?;

        // 先尝试精确匹配
        let exact_match = versions
            .iter()
            .find(|v| v.version == zig_version && v.channel == Channel::Stable);
        if let Some(m) = exact_match {
            return Ok(m.clone());
        }

        // 尝试主版本号匹配 (如 Zig 0.13.0 匹配 ZLS 0.13.x)
        let zig_parts: Vec<&str> = zig_version.split('.').collect();
        if zig_parts.len() >= 2 {
            let major_minor = format!("{}.{}", zig_parts[0], zig_parts[1]);
            let partial_match = versions
                .iter()
                .find(|v| v.version.starts_with(&major_minor) && v.channel == Channel::Stable);
            if let Some(m) = partial_match {
                return Ok(m.clone());
            }
        }

        // 回退到最新稳定版
        let latest = versions.iter().find(|v| v.channel == Channel::Stable);
        if let Some(m) = latest {
            tracing::warn!(
                "未找到与 Zig {} 精确匹配的 ZLS 版本，使用最新稳定版 {}",
                zig_version,
                m.version
            );
            return Ok(m.clone());
        }

        Err(ZzmError::VersionNotFound {
            version: format!("zls compatible with zig {zig_version}"),
        })
    }
}

// ========== 辅助函数 ==========

/// 在 asset 列表中查找匹配当前平台的 ZLS 二进制文件
fn find_matching_zls_asset(assets: &[GithubAsset], target_triple: &str) -> Option<GithubAsset> {
    let (os_name, arch_name) = crate::platform::parse_target_triple(target_triple)?;

    assets
        .iter()
        .find(|asset| {
            let name_lower = asset.name.to_lowercase();
            // 排除签名文件
            if name_lower.ends_with(".minisig") {
                return false;
            }
            name_lower.contains(os_name) && name_lower.contains(arch_name)
        })
        .cloned()
}

// ========== ZLS 二进制查找辅助 ==========

/// 在版本目录中查找 zls/zls.exe 并复制到预期位置
fn find_and_link_zls_binary(
    version_dir: &std::path::Path,
    binary_path: &std::path::Path,
) -> Result<(), ZzmError> {
    // 在版本目录中搜索 zls 或 zls.exe
    if let Ok(entries) = std::fs::read_dir(version_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name == "zls" || name == "zls.exe" {
                    if path != binary_path {
                        std::fs::copy(&path, binary_path).map_err(ZzmError::Io)?;
                    }
                    crate::infra::filesystem::set_executable(binary_path)?;
                    return Ok(());
                }
            }
        }
    }

    // 在子目录中搜索
    if let Ok(entries) = std::fs::read_dir(version_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir()
                && let Ok(sub_entries) = std::fs::read_dir(&path)
            {
                for sub_entry in sub_entries.flatten() {
                    let sub_path = sub_entry.path();
                    if sub_path.is_file() {
                        let name = sub_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        if name == "zls" || name == "zls.exe" {
                            std::fs::copy(&sub_path, binary_path).map_err(ZzmError::Io)?;
                            crate::infra::filesystem::set_executable(binary_path)?;
                            return Ok(());
                        }
                    }
                }
            }
        }
    }

    Err(ZzmError::ExtractionFailed {
        path: version_dir.to_string_lossy().to_string(),
        reason: "未找到 ZLS 二进制文件".to_string(),
    })
}

// ========== VersionProvider 实现 ==========

impl crate::core::tool_manager::VersionProvider for ZlsApiClient {
    /// ZLS 安装后钩子：设置可执行权限，如果二进制不在预期位置则查找并链接
    fn post_install_hook(
        &self,
        version_dir: &std::path::Path,
        binary_path: &std::path::Path,
    ) -> Result<(), ZzmError> {
        // 如果二进制文件已在预期位置，只需设置权限
        if binary_path.exists() {
            crate::infra::filesystem::set_executable(binary_path)?;
            return Ok(());
        }

        // ZLS 的二进制文件可能没有版本后缀，需要在版本目录中查找
        find_and_link_zls_binary(version_dir, binary_path)
    }

    async fn get_version_info(
        &self,
        version: &str,
    ) -> Result<crate::core::tool_manager::VersionInfo, ZzmError> {
        let info = self.get_version_info(version).await?;
        Ok(crate::core::tool_manager::VersionInfo {
            version: info.version,
            channel: info.channel,
            date: info.published_at,
            asset: info
                .asset
                .map(|a| crate::core::tool_manager::DownloadAsset {
                    url: a.browser_download_url,
                    filename: a.name,
                    shasum: String::new(), // ZLS 不提供 shasum
                    size: crate::utils::format::format_size(a.size),
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
                date: v.published_at,
                asset: v.asset.map(|a| crate::core::tool_manager::DownloadAsset {
                    url: a.browser_download_url,
                    filename: a.name,
                    shasum: String::new(),
                    size: crate::utils::format::format_size(a.size),
                }),
            })
            .collect())
    }

    async fn find_compatible_version(
        &self,
        zig_version: &str,
    ) -> Result<crate::core::tool_manager::VersionInfo, ZzmError> {
        let info = self.find_compatible_version(zig_version).await?;
        Ok(crate::core::tool_manager::VersionInfo {
            version: info.version,
            channel: info.channel,
            date: info.published_at,
            asset: info
                .asset
                .map(|a| crate::core::tool_manager::DownloadAsset {
                    url: a.browser_download_url,
                    filename: a.name,
                    shasum: String::new(),
                    size: crate::utils::format::format_size(a.size),
                }),
        })
    }
}

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_zls_target_triple() {
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
    fn test_find_matching_zls_asset() {
        let assets = vec![
            GithubAsset {
                id: 1,
                name: "zls-x86_64-windows.tar.xz".to_string(),
                label: None,
                content_type: "application/x-xz".to_string(),
                state: "uploaded".to_string(),
                size: 4_200_000,
                download_count: 892,
                created_at: "2026-04-16T20:44:37Z".to_string(),
                updated_at: "2026-04-16T20:46:43Z".to_string(),
                browser_download_url: "https://github.com/zigtools/zls/releases/download/0.16.0/zls-x86_64-windows.tar.xz".to_string(),
            },
            GithubAsset {
                id: 2,
                name: "zls-x86_64-windows.tar.xz.minisig".to_string(),
                label: None,
                content_type: "application/x-xz".to_string(),
                state: "uploaded".to_string(),
                size: 128,
                download_count: 120,
                created_at: "2026-04-16T20:44:37Z".to_string(),
                updated_at: "2026-04-16T20:46:43Z".to_string(),
                browser_download_url: "https://github.com/zigtools/zls/releases/download/0.16.0/zls-x86_64-windows.tar.xz.minisig".to_string(),
            },
            GithubAsset {
                id: 3,
                name: "zls-x86_64-linux.tar.xz".to_string(),
                label: None,
                content_type: "application/x-xz".to_string(),
                state: "uploaded".to_string(),
                size: 4_050_000,
                download_count: 1234,
                created_at: "2026-04-16T20:44:37Z".to_string(),
                updated_at: "2026-04-16T20:46:43Z".to_string(),
                browser_download_url: "https://github.com/zigtools/zls/releases/download/0.16.0/zls-x86_64-linux.tar.xz".to_string(),
            },
        ];

        // 查找 Windows x86_64
        let result = find_matching_zls_asset(&assets, "x86_64-windows");
        assert!(result.is_some());
        let found = result.unwrap();
        assert_eq!(found.name, "zls-x86_64-windows.tar.xz");
        assert!(!found.name.ends_with(".minisig"));

        // 查找 Linux x86_64
        let result = find_matching_zls_asset(&assets, "x86_64-linux");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "zls-x86_64-linux.tar.xz");

        // 查找不存在的平台
        let result = find_matching_zls_asset(&assets, "aarch64-macos");
        assert!(result.is_none());
    }

    #[test]
    fn test_zls_channel_serde() {
        let stable = Channel::Stable;
        let json = serde_json::to_string(&stable).unwrap();
        assert!(json.contains("stable"));
    }

    #[test]
    fn test_zls_channel_equality() {
        assert_eq!(Channel::Stable, Channel::Stable);
        assert_eq!(Channel::Prerelease, Channel::Prerelease);
        assert_ne!(Channel::Stable, Channel::Prerelease);
    }

    #[test]
    fn test_parse_zls_target_triple_all() {
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
    fn test_find_matching_zls_asset_excludes_minisig() {
        let assets = vec![
            GithubAsset {
                id: 1,
                name: "zls-x86_64-windows.tar.xz".to_string(),
                label: None,
                content_type: "application/x-xz".to_string(),
                state: "uploaded".to_string(),
                size: 4_200_000,
                download_count: 892,
                created_at: "2026-04-16T20:44:37Z".to_string(),
                updated_at: "2026-04-16T20:46:43Z".to_string(),
                browser_download_url: "https://example.com/zls.tar.xz".to_string(),
            },
            GithubAsset {
                id: 2,
                name: "zls-x86_64-windows.tar.xz.minisig".to_string(),
                label: None,
                content_type: "application/x-xz".to_string(),
                state: "uploaded".to_string(),
                size: 128,
                download_count: 120,
                created_at: "2026-04-16T20:44:37Z".to_string(),
                updated_at: "2026-04-16T20:46:43Z".to_string(),
                browser_download_url: "https://example.com/zls.tar.xz.minisig".to_string(),
            },
        ];

        let result = find_matching_zls_asset(&assets, "x86_64-windows");
        assert!(result.is_some());
        let found = result.unwrap();
        assert_eq!(found.id, 1); // 应该匹配非签名文件
        assert!(!found.name.ends_with(".minisig"));
    }

    #[test]
    fn test_find_matching_zls_asset_empty() {
        let assets: Vec<GithubAsset> = vec![];
        let result = find_matching_zls_asset(&assets, "x86_64-windows");
        assert!(result.is_none());
    }

    #[test]
    fn test_zls_api_client_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let client = ZlsApiClient::new(temp_dir.path().to_path_buf());
        assert!(client.is_ok());
    }

    #[test]
    fn test_github_release_serialization() {
        let release = GithubRelease {
            url: "https://api.github.com/repos/zigtools/zls/releases/1".to_string(),
            html_url: "https://github.com/zigtools/zls/releases/tag/0.13.0".to_string(),
            id: 12_345,
            tag_name: "0.13.0".to_string(),
            name: "ZLS 0.13.0".to_string(),
            draft: false,
            prerelease: false,
            created_at: "2026-04-16T20:44:37Z".to_string(),
            published_at: Some("2026-04-16T20:46:43Z".to_string()),
            assets: vec![],
            body: Some("Release notes".to_string()),
        };

        let json = serde_json::to_string(&release).unwrap();
        let parsed: GithubRelease = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.tag_name, "0.13.0");
        assert!(!parsed.draft);
        assert!(!parsed.prerelease);
    }

    #[test]
    fn test_zls_version_info_serialization() {
        let info = ZlsVersionInfo {
            version: "0.13.0".to_string(),
            channel: Channel::Stable,
            published_at: Some("2026-04-16T20:46:43Z".to_string()),
            asset: None,
            html_url: "https://github.com/zigtools/zls/releases/tag/0.13.0".to_string(),
        };

        let json = serde_json::to_string(&info).unwrap();
        let parsed: ZlsVersionInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, "0.13.0");
        assert_eq!(parsed.channel, Channel::Stable);
    }

    #[test]
    fn test_github_asset_serialization() {
        let asset = GithubAsset {
            id: 1,
            name: "zls-x86_64-linux.tar.xz".to_string(),
            label: Some("Linux x86_64".to_string()),
            content_type: "application/x-xz".to_string(),
            state: "uploaded".to_string(),
            size: 4_050_000,
            download_count: 1234,
            created_at: "2026-04-16T20:44:37Z".to_string(),
            updated_at: "2026-04-16T20:46:43Z".to_string(),
            browser_download_url:
                "https://github.com/zigtools/zls/releases/download/0.13.0/zls-x86_64-linux.tar.xz"
                    .to_string(),
        };

        let json = serde_json::to_string(&asset).unwrap();
        let parsed: GithubAsset = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "zls-x86_64-linux.tar.xz");
        assert_eq!(parsed.size, 4050000);
    }

    #[test]
    fn test_github_release_draft_filtered() {
        // Draft releases 应该在 list_remote_versions 中被过滤
        let release = GithubRelease {
            url: "https://api.github.com/repos/zigtools/zls/releases/1".to_string(),
            html_url: "https://github.com/zigtools/zls/releases/tag/0.13.0".to_string(),
            id: 1,
            tag_name: "0.13.0".to_string(),
            name: "Draft".to_string(),
            draft: true,
            prerelease: false,
            created_at: "2026-04-16T20:44:37Z".to_string(),
            published_at: None,
            assets: vec![],
            body: None,
        };

        assert!(release.draft);
    }
}
