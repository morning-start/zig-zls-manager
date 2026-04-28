use std::env;
use std::path::{Path, PathBuf};
use std::time::Duration;

use console::style;
use reqwest::Client;
use serde::Deserialize;

use crate::commands::AppContext;
use crate::utils::error::ZzmError;

const GITHUB_API_URL: &str = "https://api.github.com/repos/user/zig-zls-manager/releases/latest";
const GITHUB_RELEASES_URL: &str = "https://github.com/user/zig-zls-manager/releases";

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

pub async fn cmd_update_self(
    _ctx: &AppContext,
    check_only: bool,
    force: bool,
    json: bool,
) -> Result<(), ZzmError> {
    let current_version = env!("CARGO_PKG_VERSION");
    let current_binary = env::current_exe().map_err(ZzmError::Io)?;

    if !json {
        println!(
            "{}",
            style(format!("当前版本: v{}", current_version)).cyan()
        );
    }

    let client = Client::builder()
        .user_agent(format!("zzm/{}", current_version))
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .map_err(ZzmError::Network)?;

    let release = fetch_latest_release(&client).await?;

    let latest_version = release
        .tag_name
        .strip_prefix('v')
        .unwrap_or(&release.tag_name);

    if !json {
        println!("{}", style(format!("最新版本: v{}", latest_version)).cyan());
    }

    let needs_update = force || is_newer_version(latest_version, current_version);

    if !needs_update {
        if !json {
            println!("{}", style("已是最新版本").green());
        }
        return Ok(());
    }

    if check_only {
        if json {
            println!(
                r#"{{"current": "{}", "latest": "{}", "update_available": true}}"#,
                current_version, latest_version
            );
        } else {
            println!(
                "{}",
                style(format!(
                    "发现新版本 v{} (当前 v{})",
                    latest_version, current_version
                ))
                .yellow()
            );
            println!("运行 `zzm update self` 进行更新");
        }
        return Ok(());
    }

    if json {
        println!(
            r#"{{"current": "{}", "latest": "{}", "updating": true}}"#,
            current_version, latest_version
        );
    } else {
        println!(
            "{}",
            style(format!("正在更新到 v{}...", latest_version)).yellow()
        );
    }

    let target_triple = get_target_triple();
    let asset_name = format_asset_name(&target_triple);

    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| ZzmError::VersionNotFound {
            version: asset_name.clone(),
        })?;

    let temp_dir = tempfile::tempdir().map_err(ZzmError::Io)?;
    let downloaded_binary = download_binary(&client, &asset.browser_download_url, temp_dir.path())
        .await?;

    replace_binary(&downloaded_binary, &current_binary)?;

    if !json {
        println!("{}", style("更新完成!").green());
        println!(
            "请运行 `zzm --version` 验证新版本 (可能需要重新打开终端)"
        );
    }

    Ok(())
}

async fn fetch_latest_release(client: &Client) -> Result<GitHubRelease, ZzmError> {
    let response = client
        .get(GITHUB_API_URL)
        .send()
        .await
        .map_err(|e| ZzmError::DownloadFailed {
            url: GITHUB_API_URL.to_string(),
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

    let release: GitHubRelease = response.json().await?;
    Ok(release)
}

fn get_target_triple() -> String {
    #[cfg(target_os = "windows")]
    let os = "windows";
    #[cfg(target_os = "macos")]
    let os = "macos";
    #[cfg(target_os = "linux")]
    let os = "linux";

    #[cfg(target_arch = "x86_64")]
    let arch = "x86_64";
    #[cfg(target_arch = "aarch64")]
    let arch = "aarch64";
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    let arch = "unknown";

    format!("{}-{}", arch, os)
}

fn format_asset_name(target_triple: &str) -> String {
    let ext = if cfg!(target_os = "windows") {
        "zip"
    } else {
        "tar.gz"
    };
    format!("zzm-{}.{}", target_triple, ext)
}

async fn download_binary(
    client: &Client,
    url: &str,
    dest_dir: &Path,
) -> Result<PathBuf, ZzmError> {
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    let filename = url.rsplit('/').next().unwrap_or("zzm");
    let dest_path = dest_dir.join(filename);

    let response = client
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

    let mut file = tokio::fs::File::create(&dest_path)
        .await
        .map_err(ZzmError::Io)?;

    let mut stream = response.bytes_stream();
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| ZzmError::DownloadFailed {
            url: url.to_string(),
            reason: e.to_string(),
        })?;
        file.write_all(&chunk).await.map_err(ZzmError::Io)?;
    }

    file.flush().await.map_err(ZzmError::Io)?;

    if cfg!(target_os = "windows") {
        extract_zip(&dest_path, dest_dir)?;
        let binary_name = if cfg!(target_os = "windows") {
            "zzm.exe"
        } else {
            "zzm"
        };
        Ok(dest_dir.join(binary_name))
    } else {
        let output_path = dest_dir.join("zzm");
        extract_tar_gz(&dest_path, &output_path)?;
        Ok(output_path)
    }
}

fn extract_tar_gz(archive_path: &Path, output_path: &Path) -> Result<(), ZzmError> {
    use flate2::read::GzDecoder;
    use std::fs::File;
    use tar::Archive;

    let file = File::open(archive_path).map_err(ZzmError::Io)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);

    for entry in archive.entries().map_err(ZzmError::Io)? {
        let mut entry = entry.map_err(ZzmError::Io)?;
        let path = entry.path().map_err(ZzmError::Io)?;
        if path.file_name() == Some(std::ffi::OsStr::new("zzm")) {
            entry.unpack(output_path).map_err(ZzmError::Io)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(output_path)
                    .map_err(ZzmError::Io)?
                    .permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(output_path, perms).map_err(ZzmError::Io)?;
            }
            return Ok(());
        }
    }

    Err(ZzmError::ExtractionFailed {
        path: archive_path.display().to_string(),
        reason: "未找到 zzm 二进制文件".to_string(),
    })
}

fn extract_zip(archive_path: &Path, dest_dir: &Path) -> Result<(), ZzmError> {
    use std::fs::File;
    use zip::ZipArchive;

    let file = File::open(archive_path).map_err(ZzmError::Io)?;
    let mut archive = ZipArchive::new(file).map_err(|e| ZzmError::ExtractionFailed {
        path: archive_path.display().to_string(),
        reason: e.to_string(),
    })?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| ZzmError::ExtractionFailed {
            path: archive_path.display().to_string(),
            reason: e.to_string(),
        })?;
        let name = file.name();
        if name.ends_with("zzm.exe") || name.ends_with("zzm") {
            let output_path = dest_dir.join(name);
            let mut output = File::create(&output_path).map_err(ZzmError::Io)?;
            std::io::copy(&mut file, &mut output).map_err(ZzmError::Io)?;
            return Ok(());
        }
    }

    Err(ZzmError::ExtractionFailed {
        path: archive_path.display().to_string(),
        reason: "未找到 zzm 二进制文件".to_string(),
    })
}

fn replace_binary(new_binary: &Path, current_binary: &Path) -> Result<(), ZzmError> {
    #[cfg(target_os = "windows")]
    {
        let old_path = with_suffix(current_binary, ".old");
        if old_path.exists() {
            std::fs::remove_file(&old_path).map_err(ZzmError::Io)?;
        }
        std::fs::rename(current_binary, &old_path).map_err(ZzmError::Io)?;
        std::fs::copy(new_binary, current_binary).map_err(ZzmError::Io)?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::fs::rename(new_binary, current_binary).map_err(ZzmError::Io)?;
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string() + suffix)
        .unwrap_or_else(|| suffix.to_string());
    path.with_file_name(name)
}

fn is_newer_version(latest: &str, current: &str) -> bool {
    let parse_parts = |v: &str| -> Vec<u64> {
        v.split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    };

    let latest_parts = parse_parts(latest);
    let current_parts = parse_parts(current);

    for (l, c) in latest_parts.iter().zip(current_parts.iter()) {
        if l > c {
            return true;
        }
        if l < c {
            return false;
        }
    }

    latest_parts.len() > current_parts.len()
}

pub fn cmd_update_check(json: bool) -> Result<(), ZzmError> {
    let current_version = env!("CARGO_PKG_VERSION");

    if json {
        println!(
            r#"{{"current": "{}", "repository": "{}"}}"#,
            current_version, GITHUB_RELEASES_URL
        );
    } else {
        println!("当前版本: v{}", current_version);
        println!("检查更新: {}", GITHUB_RELEASES_URL);
        println!("运行 `zzm update self` 进行更新");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer_version_major() {
        assert!(is_newer_version("2.0.0", "1.0.0"));
        assert!(!is_newer_version("1.0.0", "2.0.0"));
    }

    #[test]
    fn test_is_newer_version_minor() {
        assert!(is_newer_version("1.1.0", "1.0.0"));
        assert!(!is_newer_version("1.0.0", "1.1.0"));
    }

    #[test]
    fn test_is_newer_version_patch() {
        assert!(is_newer_version("1.0.1", "1.0.0"));
        assert!(!is_newer_version("1.0.0", "1.0.1"));
    }

    #[test]
    fn test_is_newer_version_equal() {
        assert!(!is_newer_version("1.0.0", "1.0.0"));
    }

    #[test]
    fn test_is_newer_version_different_length() {
        assert!(is_newer_version("1.0.0.1", "1.0.0"));
        assert!(!is_newer_version("1.0.0", "1.0.0.1"));
    }

    #[test]
    fn test_format_asset_name() {
        #[cfg(target_os = "windows")]
        {
            assert_eq!(
                format_asset_name("x86_64-windows"),
                "zzm-x86_64-windows.zip"
            );
        }
        #[cfg(not(target_os = "windows"))]
        {
            assert_eq!(
                format_asset_name("x86_64-linux"),
                "zzm-x86_64-linux.tar.gz"
            );
            assert_eq!(
                format_asset_name("aarch64-macos"),
                "zzm-aarch64-macos.tar.gz"
            );
        }
    }

    #[test]
    fn test_get_target_triple() {
        let triple = get_target_triple();
        assert!(triple.contains('-'));
        assert!(
            triple.contains("windows")
                || triple.contains("macos")
                || triple.contains("linux")
        );
        assert!(triple.contains("x86_64") || triple.contains("aarch64"));
    }
}
