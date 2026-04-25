use std::path::{Path, PathBuf};
use std::time::Duration;

use futures_util::StreamExt;
use reqwest::Client;
use tokio::io::AsyncWriteExt;

use crate::output::progress::create_download_bar;
use crate::utils::error::ZzmError;

/// 默认连接超时
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// 默认读取超时
const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(120);

/// 默认最大重试次数
const DEFAULT_MAX_RETRIES: u32 = 3;

/// 部分下载文件后缀
const PARTIAL_SUFFIX: &str = ".part";

/// 下载管理器
pub struct Downloader {
    client: Client,
    max_retries: u32,
}

impl Downloader {
    /// 创建新的下载管理器
    pub fn new() -> Result<Self, ZzmError> {
        let client = Client::builder()
            .user_agent(format!("zzm/{}", env!("CARGO_PKG_VERSION")))
            .connect_timeout(DEFAULT_CONNECT_TIMEOUT)
            .timeout(DEFAULT_READ_TIMEOUT)
            .build()
            .map_err(ZzmError::Network)?;

        Ok(Self {
            client,
            max_retries: DEFAULT_MAX_RETRIES,
        })
    }

    /// 生成部分下载文件路径
    ///
    /// 在目标文件名后追加 `.part` 后缀，例如：
    /// - `zig-0.13.0.tar.xz` → `zig-0.13.0.tar.xz.part`
    /// - `zls.zip` → `zls.zip.part`
    fn part_file_path(dest: &Path) -> PathBuf {
        let mut name = dest
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        name.push_str(PARTIAL_SUFFIX);
        dest.with_file_name(name)
    }

    /// 设置最大重试次数
    #[allow(dead_code)] // 预留: 配置化重试策略
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// 下载文件到指定路径（带进度条、重试和续传）
    pub async fn download_file(&self, url: &str, dest: &Path) -> Result<PathBuf, ZzmError> {
        tracing::debug!("开始下载: {} -> {}", url, dest.display());

        // 确保目标目录存在
        if let Some(parent) = dest.parent()
            && !parent.exists()
        {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(ZzmError::Io)?;
        }

        let mut last_error = None;

        for attempt in 1..=self.max_retries {
            match self.download_single(url, dest).await {
                Ok(path) => return Ok(path),
                Err(e) => {
                    tracing::warn!("下载失败 (尝试 {}/{}): {}", attempt, self.max_retries, e);
                    last_error = Some(e);

                    if attempt < self.max_retries {
                        let delay = Duration::from_millis(100 * 2u64.pow(attempt));
                        tracing::debug!("等待 {:?} 后重试...", delay);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    /// 单次下载尝试（带进度条显示和续传支持）
    ///
    /// 续传逻辑：
    /// 1. 检查是否存在 `.part` 部分文件
    /// 2. 如果存在，获取已下载字节数，发送 Range 请求续传
    /// 3. 服务器不支持 Range 时，回退到完整下载
    /// 4. 下载完成后，将 `.part` 文件重命名为目标文件
    async fn download_single(&self, url: &str, dest: &Path) -> Result<PathBuf, ZzmError> {
        let part_path = Self::part_file_path(dest);

        // 检查部分文件，确定续传起始位置
        let mut downloaded: u64 = 0;
        if part_path.exists()
            && let Ok(metadata) = tokio::fs::metadata(&part_path).await
        {
            downloaded = metadata.len();
            if downloaded > 0 {
                tracing::info!(
                    "发现部分下载文件: {} (已下载 {} 字节)，尝试续传",
                    part_path.display(),
                    downloaded
                );
            }
        }

        // 构建请求，如果有已下载部分则添加 Range 头
        let request = self.client.get(url);
        let request = if downloaded > 0 {
            request.header("Range", format!("bytes={downloaded}-"))
        } else {
            request
        };

        let response = request.send().await.map_err(|e| ZzmError::DownloadFailed {
            url: url.to_string(),
            reason: e.to_string(),
        })?;

        if !response.status().is_success() && response.status().as_u16() != 206 {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            return Err(ZzmError::HttpError {
                status_code: status,
                message,
            });
        }

        // 判断是否为续传响应（HTTP 206 Partial Content）
        let is_resume = response.status().as_u16() == 206;

        let total_size = if is_resume {
            // 续传时，从 Content-Range 头获取总大小
            // 格式: bytes start-end/total
            response
                .headers()
                .get("content-range")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.split('/').next_back())
                .and_then(|v| v.parse::<u64>().ok())
        } else {
            response.content_length()
        };

        // 如果服务器返回 200 但我们发了 Range 请求，说明服务器不支持续传
        // 需要重新从头下载
        if downloaded > 0 && !is_resume {
            tracing::info!("服务器不支持续传，重新完整下载");
            downloaded = 0;
        }

        let filename = dest
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "file".to_string());

        // 创建进度条
        let pb = match total_size {
            Some(size) => create_download_bar(&filename, size),
            None => create_download_bar(&filename, 0),
        };

        // 如果是续传，进度条从已下载位置开始
        if is_resume && downloaded > 0 {
            pb.set_position(downloaded);
        }

        // 打开部分文件：续传时追加，否则创建新文件
        let mut file = if is_resume && downloaded > 0 {
            tokio::fs::OpenOptions::new()
                .append(true)
                .open(&part_path)
                .await
                .map_err(ZzmError::Io)?
        } else {
            // 非续传或服务器不支持续传，创建新文件
            tokio::fs::File::create(&part_path)
                .await
                .map_err(ZzmError::Io)?
        };

        let mut stream = response.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| ZzmError::DownloadFailed {
                url: url.to_string(),
                reason: e.to_string(),
            })?;

            file.write_all(&chunk)
                .await
                .map_err(|e| ZzmError::DownloadFailed {
                    url: url.to_string(),
                    reason: e.to_string(),
                })?;

            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        file.flush().await.map_err(ZzmError::Io)?;
        drop(file);

        // 下载完成：将部分文件重命名为目标文件
        // 如果目标文件已存在（如上次完整下载但重命名失败），先删除
        if dest.exists() {
            let _ = tokio::fs::remove_file(dest).await;
        }
        tokio::fs::rename(&part_path, dest)
            .await
            .map_err(ZzmError::Io)?;

        pb.finish_with_message("完成");

        tracing::debug!("下载完成: {} ({} 字节)", dest.display(), downloaded);

        Ok(dest.to_path_buf())
    }

    /// 下载文件到缓存目录
    ///
    /// 如果完整文件已存在，直接返回路径而不重新下载。
    /// 如果存在 `.part` 部分文件，则尝试续传。
    pub async fn download_to_cache(
        &self,
        url: &str,
        cache_dir: &Path,
        filename: &str,
    ) -> Result<PathBuf, ZzmError> {
        let dest = cache_dir.join(filename);

        // 如果缓存中已存在完整文件，直接返回
        if dest.exists() {
            tracing::debug!("文件已存在于缓存: {}", dest.display());
            return Ok(dest);
        }

        // 确保缓存目录存在
        if !cache_dir.exists() {
            tokio::fs::create_dir_all(cache_dir)
                .await
                .map_err(ZzmError::Io)?;
        }

        // 检查是否有部分下载文件可续传
        let part_path = Self::part_file_path(&dest);
        if part_path.exists()
            && let Ok(metadata) = std::fs::metadata(&part_path)
        {
            tracing::info!(
                "发现未完成的下载: {} (已下载 {} 字节)，将尝试续传",
                part_path.display(),
                metadata.len()
            );
        }

        self.download_file(url, &dest).await
    }
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new().expect("无法创建下载管理器")
    }
}

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_downloader_creation() {
        let downloader = Downloader::new();
        assert!(downloader.is_ok());
    }

    #[test]
    fn test_downloader_with_retries() {
        let downloader = Downloader::new().unwrap().with_max_retries(5);
        assert_eq!(downloader.max_retries, 5);
    }

    #[test]
    fn test_default_downloader() {
        let downloader = Downloader::default();
        assert_eq!(downloader.max_retries, DEFAULT_MAX_RETRIES);
    }

    #[test]
    fn test_downloader_builder_pattern() {
        let downloader = Downloader::new().unwrap().with_max_retries(10);
        assert_eq!(downloader.max_retries, 10);
    }

    #[test]
    fn test_downloader_zero_retries() {
        let downloader = Downloader::new().unwrap().with_max_retries(0);
        assert_eq!(downloader.max_retries, 0);
    }

    #[test]
    fn test_partial_suffix_constant() {
        assert_eq!(PARTIAL_SUFFIX, ".part");
    }

    #[test]
    fn test_part_path_generation() {
        // 测试 .part 文件路径生成逻辑
        let dest = PathBuf::from("/cache/zig-0.13.0.tar.xz");
        let part_path = Downloader::part_file_path(&dest);
        assert_eq!(part_path, PathBuf::from("/cache/zig-0.13.0.tar.xz.part"));

        // 无扩展名的情况
        let dest2 = PathBuf::from("/cache/zig-binary");
        let part_path2 = Downloader::part_file_path(&dest2);
        assert_eq!(part_path2, PathBuf::from("/cache/zig-binary.part"));
    }

    #[test]
    fn test_part_path_zip() {
        let dest = PathBuf::from("/cache/zig-windows-x86_64-0.13.0.zip");
        let part_path = Downloader::part_file_path(&dest);
        assert_eq!(
            part_path,
            PathBuf::from("/cache/zig-windows-x86_64-0.13.0.zip.part")
        );
    }
}
