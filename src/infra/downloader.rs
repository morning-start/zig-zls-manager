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

    /// 设置最大重试次数
    #[allow(dead_code)] // 预留: 配置化重试策略
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// 下载文件到指定路径（带进度条和重试）
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

    /// 单次下载尝试（带进度条显示）
    async fn download_single(&self, url: &str, dest: &Path) -> Result<PathBuf, ZzmError> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| ZzmError::DownloadFailed {
                url: url.to_string(),
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

        let total_size = response.content_length();
        let filename = dest
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "file".to_string());

        // 创建进度条
        let pb = match total_size {
            Some(size) => create_download_bar(&filename, size),
            None => create_download_bar(&filename, 0),
        };

        // 创建临时文件（写入完成后再重命名为目标文件）
        let temp_path = dest.with_extension("tmp");

        let mut file = tokio::fs::File::create(&temp_path)
            .await
            .map_err(ZzmError::Io)?;

        let mut downloaded: u64 = 0;
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

        // 原子重命名：临时文件 -> 目标文件
        tokio::fs::rename(&temp_path, dest)
            .await
            .map_err(ZzmError::Io)?;

        pb.finish_with_message("完成");

        tracing::debug!("下载完成: {} ({} 字节)", dest.display(), downloaded);

        Ok(dest.to_path_buf())
    }

    /// 下载文件到缓存目录
    ///
    /// 如果文件已存在，直接返回路径而不重新下载
    pub async fn download_to_cache(
        &self,
        url: &str,
        cache_dir: &Path,
        filename: &str,
    ) -> Result<PathBuf, ZzmError> {
        let dest = cache_dir.join(filename);

        // 如果缓存中已存在该文件，直接返回
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
}
