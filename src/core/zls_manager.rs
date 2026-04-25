use chrono::Utc;

use crate::infra::downloader::Downloader;
use crate::infra::filesystem;
use crate::infra::path_manager::{InstalledZlsVersion, PathManager};
use crate::infra::zls_api::{ZlsApiClient, ZlsVersionInfo};
use crate::output::console_output;
use crate::platform::PlatformTrait;
use crate::utils::error::ZzmError;
use crate::utils::version::resolve_version;

/// ZLS 版本管理器
///
/// 提供 ZLS 版本的安装、卸载、切换和查询功能
pub struct ZlsManager {
    platform: Box<dyn PlatformTrait>,
    path_manager: PathManager,
    api_client: ZlsApiClient,
    downloader: Downloader,
}

impl ZlsManager {
    /// 创建新的 ZlsManager
    pub fn new(platform: Box<dyn PlatformTrait>) -> Result<Self, ZzmError> {
        let path_manager = PathManager::new(platform.clone_box());
        let cache_dir = path_manager.cache_dir();
        let api_client = ZlsApiClient::new(cache_dir)?;
        let downloader = Downloader::new()?;

        Ok(Self {
            platform,
            path_manager,
            api_client,
            downloader,
        })
    }

    /// 初始化目录结构
    pub fn initialize(&self) -> Result<(), ZzmError> {
        self.path_manager.initialize_dirs()
    }

    /// 安装指定版本的 ZLS
    pub async fn install(
        &self,
        version: &str,
        zig_version: Option<&str>,
        force: bool,
    ) -> Result<InstalledZlsVersion, ZzmError> {
        self.initialize()?;

        let resolved = resolve_version(version)?;
        console_output::print_step(1, 5, &format!("解析 ZLS 版本: {} → {}", version, resolved));

        // 获取版本信息
        let version_info = self.api_client.get_version_info(&resolved).await?;

        let asset = version_info
            .asset
            .as_ref()
            .ok_or_else(|| ZzmError::VersionNotFound {
                version: format!("ZLS {} (当前平台无匹配的二进制)", resolved),
            })?;

        // 检查是否已安装
        let index = self.path_manager.read_installed_index()?;
        let already_installed = index.zls_versions.iter().any(|v| v.version == resolved);

        if already_installed && !force {
            return Err(ZzmError::AlreadyInstalled { version: resolved });
        }

        if already_installed && force {
            console_output::print_info(&format!("强制重装 ZLS 版本: {}", resolved));
            let _ = self.uninstall(&resolved).await;
        }

        // 下载
        console_output::print_step(2, 5, &format!("下载 ZLS {}", resolved));
        let cache_dir = self.path_manager.cache_dir();
        let archive_path = self
            .downloader
            .download_to_cache(&asset.browser_download_url, &cache_dir, &asset.name)
            .await?;

        // 解压
        console_output::print_step(3, 5, "解压安装");
        let version_dir = self.path_manager.zls_version_dir(&resolved);

        if version_dir.exists() {
            filesystem::remove_dir_all(&version_dir)?;
        }

        let extracted_root = filesystem::extract_archive(&archive_path, &version_dir)?;

        // ZLS 的 tar.xz 包通常只包含 zls 二进制文件
        // 但也可能有顶层目录，需要处理
        self.reorganize_extracted_files(&extracted_root, &version_dir)?;

        // 设置可执行权限
        let zls_binary = self.path_manager.zls_binary_path(&resolved);
        if zls_binary.exists() {
            filesystem::set_executable(&zls_binary)?;
        } else {
            // 在 ZLS 压缩包中，二进制文件可能没有版本后缀
            // 尝试在版本目录中查找 zls/zls.exe
            self.find_and_link_zls_binary(&version_dir, &resolved)?;
        }

        // 注册
        console_output::print_step(5, 5, "注册 ZLS 版本");
        let installed = InstalledZlsVersion {
            version: resolved.clone(),
            install_path: version_dir,
            installed_at: Utc::now().to_rfc3339(),
            zig_version: zig_version.map(|s| s.to_string()),
        };

        let mut index = self.path_manager.read_installed_index()?;
        index.zls_versions.retain(|v| v.version != resolved);
        index.zls_versions.push(installed.clone());
        self.path_manager.write_installed_index(&index)?;

        console_output::print_success(&format!("ZLS {} 安装完成", resolved));
        Ok(installed)
    }

    /// 根据兼容性自动安装匹配 Zig 版本的 ZLS
    pub async fn install_compatible(
        &self,
        zig_version: &str,
        force: bool,
    ) -> Result<InstalledZlsVersion, ZzmError> {
        let zls_info = self.api_client.find_compatible_version(zig_version).await?;

        console_output::print_info(&format!(
            "为 Zig {} 找到兼容的 ZLS 版本: {}",
            zig_version, zls_info.version
        ));

        self.install(&zls_info.version, Some(zig_version), force)
            .await
    }

    /// 卸载指定版本
    pub async fn uninstall(&self, version: &str) -> Result<(), ZzmError> {
        let resolved = resolve_version(version)?;

        let mut index = self.path_manager.read_installed_index()?;

        let pos = index
            .zls_versions
            .iter()
            .position(|v| v.version == resolved)
            .ok_or_else(|| ZzmError::NotInstalled {
                version: resolved.clone(),
            })?;

        if index.active_zls.as_ref() == Some(&resolved) {
            self.path_manager.remove_zls_symlink()?;
            let _ = self.path_manager.remove_default_zls_symlink(); // 清理 default-zls 符号链接
            index.active_zls = None;
        }

        let version_dir = self.path_manager.zls_version_dir(&resolved);
        if version_dir.exists() {
            filesystem::remove_dir_all(&version_dir)?;
        }

        index.zls_versions.remove(pos);
        self.path_manager.write_installed_index(&index)?;

        console_output::print_success(&format!("ZLS {} 已卸载", resolved));
        Ok(())
    }

    /// 列出已安装的版本
    pub fn list_installed(&self) -> Result<Vec<InstalledZlsVersion>, ZzmError> {
        let index = self.path_manager.read_installed_index()?;
        Ok(index.zls_versions)
    }

    /// 列出远程可用版本
    pub async fn list_remote(&self) -> Result<Vec<ZlsVersionInfo>, ZzmError> {
        self.api_client.list_remote_versions().await
    }

    /// 切换到指定版本
    pub async fn use_version(&self, version: &str) -> Result<String, ZzmError> {
        let resolved = resolve_version(version)?;

        let index = self.path_manager.read_installed_index()?;
        index
            .zls_versions
            .iter()
            .find(|v| v.version == resolved)
            .ok_or_else(|| ZzmError::NotInstalled {
                version: resolved.clone(),
            })?;

        let zls_binary = self.path_manager.zls_binary_path(&resolved);
        if !zls_binary.exists() {
            return Err(ZzmError::NotInstalled {
                version: format!("ZLS {} (二进制文件缺失)", resolved),
            });
        }

        self.path_manager.create_zls_symlink(&resolved)?;

        // 更新 default-zls 目录符号链接（java-mocha 风格）
        // ~/.zzm/default-zls -> ~/.zzm/versions/zls/0.13.0
        if let Err(e) = self.path_manager.create_default_zls_symlink(&resolved) {
            console_output::print_warning(&format!(
                "创建 default-zls 目录符号链接失败: {}，不影响使用，但 ZLS_HOME 模式不可用",
                e
            ));
        }

        let mut index = self.path_manager.read_installed_index()?;
        index.active_zls = Some(resolved.clone());
        self.path_manager.write_installed_index(&index)?;

        console_output::print_success(&format!("已切换到 ZLS {}", resolved));
        console_output::print_info(&format!(
            "提示: 设置 ZLS_HOME={} 即可通过 ZLS_HOME 使用当前版本",
            self.path_manager.install_dir().join("default-zls").display()
        ));
        console_output::print_info("  或确保 bin 目录在 PATH 中（zzm info 查看详情）");
        Ok(resolved)
    }

    /// 获取当前激活的版本
    pub fn current(&self) -> Result<Option<InstalledZlsVersion>, ZzmError> {
        let index = self.path_manager.read_installed_index()?;

        let active_version = match &index.active_zls {
            Some(v) => v.clone(),
            None => return Ok(None),
        };

        let installed = index
            .zls_versions
            .into_iter()
            .find(|v| v.version == active_version);

        Ok(installed)
    }

    /// 重新组织解压后的文件（同 ZigManager）
    fn reorganize_extracted_files(
        &self,
        extracted_root: &std::path::Path,
        version_dir: &std::path::Path,
    ) -> Result<(), ZzmError> {
        if extracted_root == version_dir {
            return Ok(());
        }

        if extracted_root.starts_with(version_dir) && extracted_root.is_dir() {
            let temp_dir = version_dir.with_extension("tmp_move_zls");
            if temp_dir.exists() {
                std::fs::remove_dir_all(&temp_dir).map_err(ZzmError::Io)?;
            }

            std::fs::rename(extracted_root, &temp_dir).map_err(ZzmError::Io)?;

            for entry in std::fs::read_dir(&temp_dir).map_err(ZzmError::Io)? {
                let entry = entry.map_err(ZzmError::Io)?;
                let dest = version_dir.join(entry.file_name());
                std::fs::rename(entry.path(), dest).map_err(ZzmError::Io)?;
            }

            let _ = std::fs::remove_dir(&temp_dir);
        }

        Ok(())
    }

    /// 在版本目录中查找 ZLS 二进制文件并创建正确名称的链接
    fn find_and_link_zls_binary(
        &self,
        version_dir: &std::path::Path,
        version: &str,
    ) -> Result<(), ZzmError> {
        let _binary_name = self.platform.zls_binary_name();

        // 在版本目录中搜索 zls 或 zls.exe
        if let Ok(entries) = std::fs::read_dir(version_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if name == "zls" || name == "zls.exe" {
                        let dest = self.path_manager.zls_binary_path(version);
                        if path != dest {
                            std::fs::copy(&path, &dest).map_err(ZzmError::Io)?;
                        }
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
                                let dest = self.path_manager.zls_binary_path(version);
                                std::fs::copy(&sub_path, &dest).map_err(ZzmError::Io)?;
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
}

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::path_manager::InstalledZlsVersion;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_zls_manager_creation() {
        let platform = crate::platform::detect_platform();
        let manager = ZlsManager::new(platform);
        assert!(manager.is_ok());
    }

    #[test]
    fn test_installed_zls_version_creation() {
        let temp_dir = TempDir::new().unwrap();
        let version = InstalledZlsVersion {
            version: "0.13.0".to_string(),
            install_path: temp_dir.path().to_path_buf(),
            installed_at: "2026-04-25T10:00:00Z".to_string(),
            zig_version: Some("0.13.0".to_string()),
        };

        assert_eq!(version.version, "0.13.0");
        assert_eq!(version.zig_version, Some("0.13.0".to_string()));
    }

    #[test]
    fn test_installed_zls_version_no_zig() {
        let temp_dir = TempDir::new().unwrap();
        let version = InstalledZlsVersion {
            version: "0.13.0".to_string(),
            install_path: temp_dir.path().to_path_buf(),
            installed_at: "2026-04-25T10:00:00Z".to_string(),
            zig_version: None,
        };

        assert!(version.zig_version.is_none());
    }

    #[test]
    fn test_installed_zls_version_serialization() {
        let temp_dir = TempDir::new().unwrap();
        let version = InstalledZlsVersion {
            version: "0.13.0".to_string(),
            install_path: temp_dir.path().to_path_buf(),
            installed_at: "2026-04-25T10:00:00Z".to_string(),
            zig_version: Some("0.13.0".to_string()),
        };

        let json = serde_json::to_string_pretty(&version).unwrap();
        let parsed: InstalledZlsVersion = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, "0.13.0");
        assert_eq!(parsed.zig_version, Some("0.13.0".to_string()));
    }

    #[test]
    fn test_reorganize_extracted_files_same_dir() {
        let temp_dir = TempDir::new().unwrap();
        let version_dir = temp_dir.path().join("0.13.0");
        fs::create_dir_all(&version_dir).unwrap();

        let platform = crate::platform::detect_platform();
        let manager = ZlsManager::new(platform).unwrap();

        let result = manager.reorganize_extracted_files(&version_dir, &version_dir);
        assert!(result.is_ok());
    }

    #[test]
    fn test_reorganize_extracted_files_subdirectory() {
        let temp_dir = TempDir::new().unwrap();
        let version_dir = temp_dir.path().join("0.13.0");
        fs::create_dir_all(&version_dir).unwrap();

        // 创建子目录（模拟解压后的结构）
        let sub_dir = version_dir.join("zls-x86_64-windows-0.13.0");
        fs::create_dir_all(&sub_dir).unwrap();
        fs::write(sub_dir.join("zls.exe"), "binary").unwrap();

        let platform = crate::platform::detect_platform();
        let manager = ZlsManager::new(platform).unwrap();

        let result = manager.reorganize_extracted_files(&sub_dir, &version_dir);
        assert!(result.is_ok());

        // 验证文件已移到 version_dir 根目录
        assert!(version_dir.join("zls.exe").exists());
    }

    #[test]
    fn test_find_and_link_zls_binary_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let version_dir = temp_dir.path().join("0.13.0");
        fs::create_dir_all(&version_dir).unwrap();

        // 目录为空，应返回 ExtractionFailed
        let platform = crate::platform::detect_platform();
        let manager = ZlsManager::new(platform).unwrap();

        let result = manager.find_and_link_zls_binary(&version_dir, "0.13.0");
        assert!(result.is_err());
        if let Err(ZzmError::ExtractionFailed { reason, .. }) = result {
            assert!(reason.contains("未找到 ZLS 二进制文件"));
        }
    }

    #[test]
    fn test_find_and_link_zls_binary_found() {
        // 测试 find_and_link_zls_binary 的文件搜索逻辑
        // 注意：此方法会将找到的二进制复制到 path_manager 计算的路径，
        // 该路径基于平台默认安装目录，不在 temp_dir 中。
        // 因此这里仅验证方法在目录中有正确命名的文件时不会返回 "未找到" 错误。
        let temp_dir = TempDir::new().unwrap();
        let version_dir = temp_dir.path().join("0.13.0");
        fs::create_dir_all(&version_dir).unwrap();

        // 创建一个 zls 二进制文件（名称不带版本后缀）
        #[cfg(target_os = "windows")]
        let zls_name = "zls.exe";
        #[cfg(not(target_os = "windows"))]
        let zls_name = "zls";

        fs::write(version_dir.join(zls_name), "binary").unwrap();

        let platform = crate::platform::detect_platform();
        let manager = ZlsManager::new(platform).unwrap();

        let result = manager.find_and_link_zls_binary(&version_dir, "0.13.0");
        // 该方法会尝试 copy 到 path_manager 计算的 zls_binary_path，
        // 可能因目标目录不存在而失败。我们只验证文件搜索逻辑正确：
        // 如果返回 ExtractionFailed 且原因包含"未找到"，则说明搜索逻辑有误
        if let Err(ZzmError::ExtractionFailed { ref reason, .. }) = result {
            assert!(
                !reason.contains("未找到 ZLS 二进制文件"),
                "应能找到 ZLS 二进制文件"
            );
        }
        // 如果因为 IO 错误（目标目录不存在）导致失败，这是预期的
    }

    #[test]
    fn test_zls_manager_list_installed_empty() {
        let platform = crate::platform::detect_platform();
        let manager = ZlsManager::new(platform).unwrap();
        let result = manager.list_installed();
        assert!(result.is_ok());
    }
}
