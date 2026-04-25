use chrono::Utc;

use crate::infra::checksum;
use crate::infra::downloader::Downloader;
use crate::infra::filesystem;
use crate::infra::path_manager::{InstalledZigVersion, PathManager};
use crate::infra::zig_api::{ZigApiClient, ZigChannel, ZigVersionInfo};
use crate::output::console_output;
use crate::platform::PlatformTrait;
use crate::utils::error::ZzmError;
use crate::utils::version::resolve_version;

/// Zig 版本管理器
///
/// 提供 Zig 版本的安装、卸载、切换和查询功能
pub struct ZigManager {
    #[allow(dead_code)] // 预留: 平台特定操作扩展
    platform: Box<dyn PlatformTrait>,
    path_manager: PathManager,
    api_client: ZigApiClient,
    downloader: Downloader,
}

impl ZigManager {
    /// 创建新的 ZigManager
    pub fn new(platform: Box<dyn PlatformTrait>) -> Result<Self, ZzmError> {
        let path_manager = PathManager::new(platform.clone_box());
        let cache_dir = path_manager.cache_dir();
        let api_client = ZigApiClient::new(cache_dir.clone())?;
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

    /// 安装指定版本的 Zig
    ///
    /// 完整流程：解析版本 → 下载 → 校验 → 解压 → 注册
    pub async fn install(
        &self,
        version: &str,
        force: bool,
    ) -> Result<InstalledZigVersion, ZzmError> {
        self.initialize()?;

        // 解析版本号
        let resolved = resolve_version(version)?;
        console_output::print_step(1, 5, &format!("解析版本: {} → {}", version, resolved));

        // 获取版本信息
        let version_info = self.api_client.get_version_info(&resolved).await?;

        let asset = version_info
            .asset
            .as_ref()
            .ok_or_else(|| ZzmError::VersionNotFound {
                version: format!("{} (当前平台无匹配的二进制)", resolved),
            })?;

        // 检查是否已安装
        let index = self.path_manager.read_installed_index()?;
        let already_installed = index.zig_versions.iter().any(|v| v.version == resolved);

        if already_installed && !force {
            return Err(ZzmError::AlreadyInstalled { version: resolved });
        }

        // 如果强制安装，先卸载旧版本
        if already_installed && force {
            console_output::print_info(&format!("强制重装版本: {}", resolved));
            let _ = self.uninstall(&resolved).await;
        }

        // 下载
        console_output::print_step(2, 5, &format!("下载 Zig {}", resolved));
        let cache_dir = self.path_manager.cache_dir();
        let archive_path = self
            .downloader
            .download_to_cache(&asset.url, &cache_dir, &asset.filename)
            .await?;

        // 校验
        console_output::print_step(3, 5, "校验文件完整性");
        if !asset.shasum.is_empty() {
            let data = std::fs::read(&archive_path)?;
            let verified = checksum::verify_checksum(&data, &asset.shasum)?;
            if !verified {
                // 删除损坏的缓存文件
                let _ = std::fs::remove_file(&archive_path);
                return Err(ZzmError::ChecksumMismatch {
                    expected: asset.shasum.clone(),
                    actual: checksum::calculate_sha256(&data),
                });
            }
            console_output::print_success("校验通过");
        } else {
            console_output::print_warning("未提供校验和，跳过校验");
        }

        // 解压
        console_output::print_step(4, 5, "解压安装");
        let version_dir = self.path_manager.zig_version_dir(&resolved);

        // 如果目标目录已存在，先删除
        if version_dir.exists() {
            filesystem::remove_dir_all(&version_dir)?;
        }

        let extracted_root = filesystem::extract_archive(&archive_path, &version_dir)?;

        // 检查解压后的目录结构
        // Zig 的 zip 包通常包含一个顶层目录 zig-platform-version/
        // 我们需要将内容移动到正确的位置
        filesystem::reorganize_extracted_files(&extracted_root, &version_dir, "tmp_move")?;

        // 设置可执行权限
        let zig_binary = self.path_manager.zig_binary_path(&resolved);
        if zig_binary.exists() {
            filesystem::set_executable(&zig_binary)?;
        }

        // 注册
        console_output::print_step(5, 5, "注册版本");
        let installed = InstalledZigVersion {
            version: resolved.clone(),
            install_path: version_dir,
            installed_at: Utc::now().to_rfc3339(),
            channel: match version_info.channel {
                ZigChannel::Stable => "stable".to_string(),
                ZigChannel::Nightly => "nightly".to_string(),
            },
        };

        let mut index = self.path_manager.read_installed_index()?;
        // 移除同版本旧记录（如果存在）
        index.zig_versions.retain(|v| v.version != resolved);
        index.zig_versions.push(installed.clone());
        self.path_manager.write_installed_index(&index)?;

        console_output::print_success(&format!("Zig {} 安装完成", resolved));
        Ok(installed)
    }

    /// 卸载指定版本
    pub async fn uninstall(&self, version: &str) -> Result<(), ZzmError> {
        let resolved = resolve_version(version)?;

        let mut index = self.path_manager.read_installed_index()?;

        // 查找已安装版本
        let pos = index
            .zig_versions
            .iter()
            .position(|v| v.version == resolved)
            .ok_or_else(|| ZzmError::NotInstalled {
                version: resolved.clone(),
            })?;

        // 如果是当前激活版本，先移除符号链接
        if index.active_zig.as_ref() == Some(&resolved) {
            self.path_manager.remove_zig_symlink()?;
            let _ = self.path_manager.remove_default_symlink(); // 清理 default 符号链接
            index.active_zig = None;
        }

        // 删除版本目录
        let version_dir = self.path_manager.zig_version_dir(&resolved);
        if version_dir.exists() {
            filesystem::remove_dir_all(&version_dir)?;
        }

        // 从索引中移除
        index.zig_versions.remove(pos);
        self.path_manager.write_installed_index(&index)?;

        console_output::print_success(&format!("Zig {} 已卸载", resolved));
        Ok(())
    }

    /// 列出已安装的版本
    pub fn list_installed(&self) -> Result<Vec<InstalledZigVersion>, ZzmError> {
        let index = self.path_manager.read_installed_index()?;
        Ok(index.zig_versions)
    }

    /// 列出远程可用版本
    pub async fn list_remote(&self) -> Result<Vec<ZigVersionInfo>, ZzmError> {
        self.api_client.list_remote_versions().await
    }

    /// 切换到指定版本
    pub async fn use_version(&self, version: &str) -> Result<String, ZzmError> {
        let resolved = resolve_version(version)?;

        // 确认版本已安装
        let index = self.path_manager.read_installed_index()?;
        let _installed = index
            .zig_versions
            .iter()
            .find(|v| v.version == resolved)
            .ok_or_else(|| ZzmError::NotInstalled {
                version: resolved.clone(),
            })?;

        // 检查二进制文件是否存在
        let zig_binary = self.path_manager.zig_binary_path(&resolved);
        if !zig_binary.exists() {
            return Err(ZzmError::NotInstalled {
                version: format!("{} (二进制文件缺失)", resolved),
            });
        }

        // 更新 bin 目录符号链接（向后兼容）
        self.path_manager.create_zig_symlink(&resolved)?;

        // 更新 default 目录符号链接（java-mocha 风格）
        // ~/.zzm/default -> ~/.zzm/versions/zig/0.13.0
        if let Err(e) = self.path_manager.create_default_zig_symlink(&resolved) {
            console_output::print_warning(&format!(
                "创建 default 目录符号链接失败: {}，不影响使用，但 ZIG_HOME 模式不可用",
                e
            ));
        }

        // 更新索引中的 active_zig
        let mut index = self.path_manager.read_installed_index()?;
        index.active_zig = Some(resolved.clone());
        self.path_manager.write_installed_index(&index)?;

        console_output::print_success(&format!("已切换到 Zig {}", resolved));
        console_output::print_info(&format!(
            "提示: 设置 ZIG_HOME={} 即可通过 ZIG_HOME 使用当前版本",
            self.path_manager.default_dir().display()
        ));
        console_output::print_info("  或确保 bin 目录在 PATH 中（zzm info 查看详情）");
        Ok(resolved)
    }

    /// 获取当前激活的版本
    pub fn current(&self) -> Result<Option<InstalledZigVersion>, ZzmError> {
        let index = self.path_manager.read_installed_index()?;

        let active_version = match &index.active_zig {
            Some(v) => v.clone(),
            None => return Ok(None),
        };

        let installed = index
            .zig_versions
            .into_iter()
            .find(|v| v.version == active_version);

        Ok(installed)
    }
}

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::path_manager::InstalledZigVersion;
    use crate::utils::version::Version;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_zig_manager_creation() {
        let platform = crate::platform::detect_platform();
        let manager = ZigManager::new(platform);
        assert!(manager.is_ok());
    }

    #[test]
    fn test_resolve_version() {
        // 测试标准版本解析
        let version: Version = "0.13.0".parse().unwrap();
        assert_eq!(version.major, 0);
        assert_eq!(version.minor, 13);
        assert_eq!(version.patch, 0);

        // 测试简写版本
        let resolved = resolve_version("0.13").unwrap();
        assert_eq!(resolved, "0.13.0");

        // 测试master版本
        let resolved = resolve_version("master").unwrap();
        assert_eq!(resolved, "master");

        // 测试stable版本
        let resolved = resolve_version("stable").unwrap();
        assert_eq!(resolved, "stable");
    }

    #[test]
    fn test_reorganize_extracted_files_same_dir() {
        let temp_dir = TempDir::new().unwrap();
        let version_dir = temp_dir.path().join("0.13.0");
        fs::create_dir_all(&version_dir).unwrap();

        // 测试当 extracted_root 和 version_dir 相同时
        let result = filesystem::reorganize_extracted_files(&version_dir, &version_dir, "tmp_move");
        assert!(result.is_ok());
    }

    #[test]
    fn test_installed_zig_version_creation() {
        let temp_dir = TempDir::new().unwrap();
        let version = InstalledZigVersion {
            version: "0.13.0".to_string(),
            install_path: temp_dir.path().to_path_buf(),
            installed_at: "2024-06-06T10:00:00Z".to_string(),
            channel: "stable".to_string(),
        };

        assert_eq!(version.version, "0.13.0");
        assert_eq!(version.channel, "stable");
        assert_eq!(version.installed_at, "2024-06-06T10:00:00Z");
    }

    #[test]
    fn test_resolve_version_edge_cases() {
        // 测试点号处理
        let resolved = resolve_version(".13").unwrap();
        assert_eq!(resolved, "0.13.0");

        // "0." 不是有效版本号，应返回错误
        let result = resolve_version("0.");
        assert!(result.is_err(), "\"0.\" 应该是无效版本号");

        // 测试无效版本
        let result = resolve_version("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_version_comparison() {
        let v1: Version = "0.13.0".parse().unwrap();
        let v2: Version = "0.13.1".parse().unwrap();
        let v3: Version = "0.12.0".parse().unwrap();
        let v4: Version = "1.0.0".parse().unwrap();

        assert!(v1 < v2);
        assert!(v3 < v1);
        assert!(v1 < v4);
    }

    #[test]
    fn test_reorganize_extracted_files_subdirectory() {
        let temp_dir = TempDir::new().unwrap();
        let version_dir = temp_dir.path().join("0.13.0");
        fs::create_dir_all(&version_dir).unwrap();

        // 创建子目录（模拟解压后的结构）
        let sub_dir = version_dir.join("zig-x86_64-windows-0.13.0");
        fs::create_dir_all(&sub_dir).unwrap();
        fs::write(sub_dir.join("zig.exe"), "binary").unwrap();

        let result = filesystem::reorganize_extracted_files(&sub_dir, &version_dir, "tmp_move");
        assert!(result.is_ok());

        // 验证文件已移到 version_dir 根目录
        assert!(version_dir.join("zig.exe").exists());
    }

    #[test]
    fn test_installed_zig_version_serialization() {
        let temp_dir = TempDir::new().unwrap();
        let version = InstalledZigVersion {
            version: "0.13.0".to_string(),
            install_path: temp_dir.path().to_path_buf(),
            installed_at: "2026-04-25T10:00:00Z".to_string(),
            channel: "stable".to_string(),
        };

        let json = serde_json::to_string_pretty(&version).unwrap();
        let parsed: InstalledZigVersion = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, "0.13.0");
        assert_eq!(parsed.channel, "stable");
    }

    #[test]
    fn test_zig_manager_list_installed_empty() {
        let platform = crate::platform::detect_platform();
        let manager = ZigManager::new(platform).unwrap();
        // 在未初始化的环境中，list_installed 应返回默认空列表
        let result = manager.list_installed();
        assert!(result.is_ok());
    }

    #[test]
    fn test_installed_zig_version_nightly_channel() {
        let temp_dir = TempDir::new().unwrap();
        let version = InstalledZigVersion {
            version: "master".to_string(),
            install_path: temp_dir.path().to_path_buf(),
            installed_at: "2026-04-25T10:00:00Z".to_string(),
            channel: "nightly".to_string(),
        };
        assert_eq!(version.channel, "nightly");
    }
}
