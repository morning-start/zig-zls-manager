use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::core::callbacks::InstallCallbacks;
use crate::core::channel::Channel;
use crate::infra::checksum;
use crate::infra::downloader::Downloader;
use crate::infra::filesystem;
use crate::infra::path_manager::{InstalledIndex, PathManager, ToolExtraData, ToolIndexEntry};
use crate::platform::PlatformTrait;
use crate::utils::error::ZzmError;
use crate::utils::version::resolve_version;

/// 工具类型标识
///
/// 用于区分 Zig 和 ZLS 的路径计算、索引操作等差异
/// 实现 Serialize/Deserialize 以支持 HashMap<ToolKind, _> 的 JSON 序列化
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolKind {
    Zig,
    Zls,
}

/// 下载结果（仅包含下载+校验后的信息，不含安装注册数据）
///
/// 用于并行下载场景：先 `download_only` 获取缓存路径和版本信息，
/// 再 `install_from_cache` 完成解压和注册。
pub struct DownloadedAsset {
    /// 解析后的版本号
    pub resolved: String,
    /// 缓存中的归档文件路径
    pub archive_path: std::path::PathBuf,
    /// 版本通道
    pub channel: Channel,
    /// SHA256 校验和（可能为空，预留字段）
    #[allow(dead_code)] // 预留: 校验和可用于安装后二次验证
    pub shasum: String,
}

/// 下载资源信息（统一抽象）
///
/// 将 Zig 的 `ZigPlatformAsset` 和 ZLS 的 `GithubAsset` 统一为同一结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadAsset {
    /// 下载 URL
    pub url: String,
    /// 文件名
    pub filename: String,
    /// SHA256 校验和（ZLS 可能无此字段）
    pub shasum: String,
    /// 文件大小（人类可读）
    pub size: String,
}

/// 统一的版本信息（供 ToolManager 使用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// 版本号字符串
    pub version: String,
    /// 版本通道
    pub channel: Channel,
    /// 发布日期
    pub date: Option<String>,
    /// 当前平台匹配的下载资源
    pub asset: Option<DownloadAsset>,
}

/// 版本提供者 trait
///
/// 封装不同 API 数据源的差异，使 ToolManager 能统一操作
pub trait VersionProvider: Send + Sync {
    /// 获取指定版本的下载信息
    fn get_version_info(
        &self,
        version: &str,
    ) -> impl std::future::Future<Output = Result<VersionInfo, ZzmError>> + Send;

    /// 列出所有远程可用版本
    fn list_remote_versions(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<VersionInfo>, ZzmError>> + Send;

    /// 根据兼容性规则查找匹配 Zig 版本的 ZLS 版本（仅 ZLS 实现）
    #[allow(dead_code)] // 预留: ZLS 兼容性查找
    fn find_compatible_version(
        &self,
        _zig_version: &str,
    ) -> impl std::future::Future<Output = Result<VersionInfo, ZzmError>> + Send {
        std::future::ready(Err(ZzmError::VersionNotFound {
            version: "兼容性查找仅适用于 ZLS".to_string(),
        }))
    }

    /// 安装后钩子（工具特定的后处理逻辑）
    ///
    /// Zig: 设置可执行权限（默认实现）
    /// ZLS: 查找并链接二进制文件（ZlsApiClient 覆盖实现）
    fn post_install_hook(
        &self,
        _version_dir: &std::path::Path,
        _binary_path: &std::path::Path,
    ) -> Result<(), ZzmError> {
        // 默认实现：设置可执行权限
        if _binary_path.exists() {
            crate::infra::filesystem::set_executable(_binary_path)?;
        }
        Ok(())
    }
}

/// 工具版本管理器（泛型抽象）
///
/// 统一 Zig 和 ZLS 的版本管理逻辑，通过泛型参数 `T: VersionProvider`
/// 封装 API 差异，通过 `ToolKind` 枚举处理路径差异。
pub struct ToolManager<T: VersionProvider> {
    kind: ToolKind,
    #[allow(dead_code)] // 预留: 平台特定操作扩展
    platform: Box<dyn PlatformTrait>,
    path_manager: PathManager,
    api_client: T,
    downloader: Downloader,
    callbacks: InstallCallbacks,
}

impl<T: VersionProvider> ToolManager<T> {
    /// 创建新的工具管理器
    pub fn new(
        kind: ToolKind,
        platform: Box<dyn PlatformTrait>,
        api_client: T,
        callbacks: InstallCallbacks,
    ) -> Result<Self, ZzmError> {
        let path_manager = PathManager::new(platform.clone_box());
        let downloader = Downloader::new()?;

        Ok(Self {
            kind,
            platform,
            path_manager,
            api_client,
            downloader,
            callbacks,
        })
    }

    /// 初始化目录结构
    pub fn initialize(&self) -> Result<(), ZzmError> {
        self.path_manager.initialize_dirs()
    }

    /// 安装指定版本
    ///
    /// 通用流程：解析版本 → 获取信息 → 检查已安装 → 下载 → 校验 → 解压 → 注册
    pub async fn install(
        &self,
        version: &str,
        force: bool,
        zig_version: Option<&str>,
    ) -> Result<ToolIndexEntry, ZzmError> {
        self.initialize()?;

        let tool_name = self.tool_name();
        let resolved = resolve_version(version)?;
        (self.callbacks.on_step)(
            1,
            5,
            &format!("解析 {tool_name} 版本: {version} → {resolved}"),
        );

        // 获取版本信息
        let version_info = self.api_client.get_version_info(&resolved).await?;

        let asset = version_info
            .asset
            .as_ref()
            .ok_or_else(|| ZzmError::VersionNotFound {
                version: format!("{tool_name} {resolved} (当前平台无匹配的二进制)"),
            })?;

        // 检查是否已安装
        let mut index = self.path_manager.read_installed_index()?;
        let already_installed = self.is_version_installed(&index, &resolved);

        if already_installed && !force {
            return Err(ZzmError::AlreadyInstalled { version: resolved });
        }

        if already_installed && force {
            (self.callbacks.on_info)(&format!("强制重装 {tool_name} 版本: {resolved}"));
            // 从索引和磁盘移除旧版本
            self.remove_installed_from_index(&mut index, &resolved)?;
        }

        // 下载
        (self.callbacks.on_step)(2, 5, &format!("下载 {tool_name} {resolved}"));
        let cache_dir = self.path_manager.cache_dir();
        let archive_path = self
            .downloader
            .download_to_cache(&asset.url, &cache_dir, &asset.filename)
            .await?;

        // 校验（仅 Zig 提供 shasum）
        (self.callbacks.on_step)(3, 5, "校验文件完整性");
        if !asset.shasum.is_empty() {
            let verified = checksum::verify_checksum_streaming(&archive_path, &asset.shasum)?;
            if !verified {
                let _ = std::fs::remove_file(&archive_path);
                return Err(ZzmError::ChecksumMismatch {
                    expected: asset.shasum.clone(),
                    actual: String::new(), // 流式校验无法返回实际值
                });
            }
            (self.callbacks.on_success)("校验通过");
        } else {
            (self.callbacks.on_warning)("未提供校验和，跳过校验");
        }

        // 解压
        (self.callbacks.on_step)(4, 5, "解压安装");
        let version_dir = self.version_dir(&resolved);

        if version_dir.exists() {
            filesystem::remove_dir_all(&version_dir)?;
        }

        let extracted_root = filesystem::extract_archive(&archive_path, &version_dir)?;
        let tmp_name = format!("tmp_move_{}", self.kind_suffix());
        filesystem::reorganize_extracted_files(&extracted_root, &version_dir, &tmp_name)?;

        // 工具特定的后处理
        self.post_install(&resolved)?;

        // 注册（复用已有索引，无需重新读取）
        (self.callbacks.on_step)(5, 5, &format!("注册 {tool_name} 版本"));
        let installed = self.create_installed_record(
            &resolved,
            version_dir,
            &version_info.channel,
            zig_version,
        );

        self.remove_version_from_index(&mut index, &resolved);
        self.add_version_to_index(&mut index, &installed);
        self.path_manager.write_installed_index(&index)?;

        (self.callbacks.on_success)(&format!("{tool_name} {resolved} 安装完成"));
        Ok(installed)
    }

    /// 仅下载并校验，不解压和注册
    ///
    /// 用于并行下载场景：先并行下载 Zig 和 ZLS 的归档文件，
    /// 再串行调用 `install_from_cache` 完成解压和注册。
    pub async fn download_only(
        &self,
        version: &str,
        force: bool,
    ) -> Result<DownloadedAsset, ZzmError> {
        self.initialize()?;

        let tool_name = self.tool_name();
        let resolved = resolve_version(version)?;
        (self.callbacks.on_step)(
            1,
            3,
            &format!("解析 {tool_name} 版本: {version} → {resolved}"),
        );

        // 获取版本信息
        let version_info = self.api_client.get_version_info(&resolved).await?;

        let asset = version_info
            .asset
            .as_ref()
            .ok_or_else(|| ZzmError::VersionNotFound {
                version: format!("{tool_name} {resolved} (当前平台无匹配的二进制)"),
            })?;

        // 检查是否已安装
        let index = self.path_manager.read_installed_index()?;
        let already_installed = self.is_version_installed(&index, &resolved);

        if already_installed && !force {
            return Err(ZzmError::AlreadyInstalled { version: resolved });
        }

        if already_installed && force {
            (self.callbacks.on_info)(&format!("强制重装 {tool_name} 版本: {resolved}"));
        }

        // 下载
        (self.callbacks.on_step)(2, 3, &format!("下载 {tool_name} {resolved}"));
        let cache_dir = self.path_manager.cache_dir();
        let archive_path = self
            .downloader
            .download_to_cache(&asset.url, &cache_dir, &asset.filename)
            .await?;

        // 校验
        (self.callbacks.on_step)(3, 3, "校验文件完整性");
        if !asset.shasum.is_empty() {
            let verified = checksum::verify_checksum_streaming(&archive_path, &asset.shasum)?;
            if !verified {
                let _ = std::fs::remove_file(&archive_path);
                return Err(ZzmError::ChecksumMismatch {
                    expected: asset.shasum.clone(),
                    actual: String::new(),
                });
            }
            (self.callbacks.on_success)("校验通过");
        } else {
            (self.callbacks.on_warning)("未提供校验和，跳过校验");
        }

        Ok(DownloadedAsset {
            resolved,
            archive_path,
            channel: version_info.channel,
            shasum: asset.shasum.clone(),
        })
    }

    /// 从已下载的缓存文件安装（解压 + 后处理 + 注册）
    ///
    /// 配合 `download_only` 使用，实现并行下载 + 串行安装。
    pub fn install_from_cache(
        &self,
        downloaded: &DownloadedAsset,
        force: bool,
        zig_version: Option<&str>,
    ) -> Result<ToolIndexEntry, ZzmError> {
        let tool_name = self.tool_name();
        let resolved = &downloaded.resolved;

        // 处理已安装版本（强制重装时移除旧版本）
        let mut index = self.path_manager.read_installed_index()?;
        let already_installed = self.is_version_installed(&index, resolved);

        if already_installed && force {
            self.remove_installed_from_index(&mut index, resolved)?;
        }

        // 解压
        (self.callbacks.on_step)(1, 3, "解压安装");
        let version_dir = self.version_dir(resolved);

        if version_dir.exists() {
            filesystem::remove_dir_all(&version_dir)?;
        }

        let extracted_root = filesystem::extract_archive(&downloaded.archive_path, &version_dir)?;
        let tmp_name = format!("tmp_move_{}", self.kind_suffix());
        filesystem::reorganize_extracted_files(&extracted_root, &version_dir, &tmp_name)?;

        // 工具特定的后处理
        self.post_install(resolved)?;

        // 注册（复用已有索引，无需重新读取）
        (self.callbacks.on_step)(2, 3, &format!("注册 {tool_name} 版本"));
        let installed = self.create_installed_record(
            resolved,
            version_dir,
            &downloaded.channel,
            zig_version,
        );

        self.remove_version_from_index(&mut index, resolved);
        self.add_version_to_index(&mut index, &installed);
        self.path_manager.write_installed_index(&index)?;

        (self.callbacks.on_step)(3, 3, &format!("{tool_name} {resolved} 安装完成"));
        Ok(installed)
    }

    /// 卸载指定版本
    pub fn uninstall(&self, version: &str) -> Result<(), ZzmError> {
        let tool_name = self.tool_name();
        let resolved = resolve_version(version)?;

        let mut index = self.path_manager.read_installed_index()?;

        // 查找已安装版本
        let pos = self
            .find_installed_position(&index, &resolved)
            .ok_or_else(|| ZzmError::NotInstalled {
                version: resolved.clone(),
            })?;

        // 如果是当前激活版本，移除符号链接
        if self.is_active_version(&index, &resolved) {
            self.remove_version_symlinks()?;
            self.set_active_version(&mut index, None);
        }

        // 删除版本目录
        let version_dir = self.version_dir(&resolved);
        if version_dir.exists() {
            filesystem::remove_dir_all(&version_dir)?;
        }

        // 从索引中移除
        self.remove_version_at(&mut index, pos);
        self.path_manager.write_installed_index(&index)?;

        (self.callbacks.on_success)(&format!("{tool_name} {resolved} 已卸载"));
        Ok(())
    }

    /// 切换到指定版本
    pub async fn use_version(&self, version: &str) -> Result<String, ZzmError> {
        let tool_name = self.tool_name();
        let resolved = resolve_version(version)?;

        // 单次读取索引，后续复用
        let mut index = self.path_manager.read_installed_index()?;
        self.find_installed(&index, &resolved)
            .ok_or_else(|| ZzmError::NotInstalled {
                version: resolved.clone(),
            })?;

        // 检查二进制文件
        let binary_path = self.binary_path(&resolved);
        if !binary_path.exists() {
            return Err(ZzmError::NotInstalled {
                version: format!("{tool_name} {resolved} (二进制文件缺失)"),
            });
        }

        // 更新符号链接（bin + default）
        self.update_version_symlinks(&resolved)?;

        // 更新索引中的 active 版本（复用已读取的索引）
        self.set_active_version(&mut index, Some(resolved.clone()));
        self.path_manager.write_installed_index(&index)?;

        (self.callbacks.on_success)(&format!("已切换到 {tool_name} {resolved}"));
        (self.callbacks.on_info)(&format!(
            "提示: 设置 {}={} 即可通过 {} 使用当前版本",
            self.home_env_name(),
            self.default_dir_path().display(),
            self.home_env_name(),
        ));
        (self.callbacks.on_info)("  或确保 bin 目录在 PATH 中（zzm info 查看详情）");
        Ok(resolved)
    }

    /// 获取当前激活的版本
    pub fn current(&self) -> Result<Option<ToolIndexEntry>, ZzmError> {
        let index = self.path_manager.read_installed_index()?;

        let active = self.get_active_version(&index);
        let active_version = match active {
            Some(v) => v,
            None => return Ok(None),
        };

        let installed = self.find_installed_by_version(&index, &active_version);
        Ok(installed)
    }

    /// 列出已安装的版本
    pub fn list_installed(&self) -> Result<Vec<ToolIndexEntry>, ZzmError> {
        let index = self.path_manager.read_installed_index()?;
        Ok(self.get_all_installed(&index))
    }

    /// 列出远程可用版本
    pub async fn list_remote(&self) -> Result<Vec<VersionInfo>, ZzmError> {
        self.api_client.list_remote_versions().await
    }

    /// 获取 API 客户端引用
    pub fn api_client(&self) -> &T {
        &self.api_client
    }

    // ========== 内部辅助方法 ==========

    /// 工具名称（用于显示）
    fn tool_name(&self) -> &'static str {
        match self.kind {
            ToolKind::Zig => "Zig",
            ToolKind::Zls => "ZLS",
        }
    }

    /// 类型后缀（用于临时目录名等）
    fn kind_suffix(&self) -> &'static str {
        match self.kind {
            ToolKind::Zig => "zig",
            ToolKind::Zls => "zls",
        }
    }

    /// 版本安装目录
    fn version_dir(&self, version: &str) -> std::path::PathBuf {
        match self.kind {
            ToolKind::Zig => self.path_manager.zig_version_dir(version),
            ToolKind::Zls => self.path_manager.zls_version_dir(version),
        }
    }

    /// 二进制文件路径
    fn binary_path(&self, version: &str) -> std::path::PathBuf {
        match self.kind {
            ToolKind::Zig => self.path_manager.zig_binary_path(version),
            ToolKind::Zls => self.path_manager.zls_binary_path(version),
        }
    }

    /// HOME 环境变量名
    fn home_env_name(&self) -> &'static str {
        match self.kind {
            ToolKind::Zig => "ZIG_HOME",
            ToolKind::Zls => "ZLS_HOME",
        }
    }

    /// default 符号链接名称
    fn default_link_name(&self) -> &'static str {
        match self.kind {
            ToolKind::Zig => "default",
            ToolKind::Zls => "default-zls",
        }
    }

    /// default 目录路径
    fn default_dir_path(&self) -> std::path::PathBuf {
        match self.kind {
            ToolKind::Zig => self.path_manager.default_dir(),
            ToolKind::Zls => self.path_manager.install_dir().join("default-zls"),
        }
    }

    /// 安装后处理（委托给 VersionProvider::post_install_hook）
    fn post_install(&self, version: &str) -> Result<(), ZzmError> {
        let version_dir = self.version_dir(version);
        let binary_path = self.binary_path(version);
        self.api_client
            .post_install_hook(&version_dir, &binary_path)
    }

    /// 创建已安装记录
    fn create_installed_record(
        &self,
        version: &str,
        version_dir: std::path::PathBuf,
        channel: &Channel,
        zig_version: Option<&str>,
    ) -> ToolIndexEntry {
        let extra = match self.kind {
            ToolKind::Zig => ToolExtraData::Zig {
                channel: channel.clone(),
            },
            ToolKind::Zls => ToolExtraData::Zls {
                zig_version: zig_version.map(|s| s.to_string()),
            },
        };
        ToolIndexEntry {
            version: version.to_string(),
            install_path: version_dir,
            installed_at: Utc::now().to_rfc3339(),
            extra,
        }
    }

    // ========== 索引操作辅助 ==========

    /// 从索引中移除已安装版本（包括磁盘清理和符号链接更新）
    fn remove_installed_from_index(
        &self,
        index: &mut InstalledIndex,
        version: &str,
    ) -> Result<(), ZzmError> {
        if let Some(pos) = self.find_installed_position(index, version) {
            if self.is_active_version(index, version) {
                self.remove_version_symlinks()?;
                self.set_active_version(index, None);
            }
            let version_dir = self.version_dir(version);
            if version_dir.exists() {
                filesystem::remove_dir_all(&version_dir)?;
            }
            self.remove_version_at(index, pos);
            self.path_manager.write_installed_index(index)?;
        }
        Ok(())
    }

    fn is_version_installed(&self, index: &InstalledIndex, version: &str) -> bool {
        index
            .get_versions(self.kind)
            .iter()
            .any(|v| v.version == version)
    }

    fn is_active_version(&self, index: &InstalledIndex, version: &str) -> bool {
        index.get_active(self.kind) == Some(version)
    }

    fn set_active_version(&self, index: &mut InstalledIndex, version: Option<String>) {
        index.set_active(self.kind, version);
    }

    fn get_active_version(&self, index: &InstalledIndex) -> Option<String> {
        index.get_active(self.kind).map(|s| s.to_string())
    }

    fn find_installed_position(&self, index: &InstalledIndex, version: &str) -> Option<usize> {
        index
            .get_versions(self.kind)
            .iter()
            .position(|v| v.version == version)
    }

    fn remove_version_from_index(&self, index: &mut InstalledIndex, version: &str) {
        index
            .get_versions_mut(self.kind)
            .retain(|v| v.version != version);
    }

    fn remove_version_at(&self, index: &mut InstalledIndex, pos: usize) {
        index.get_versions_mut(self.kind).remove(pos);
    }

    fn add_version_to_index(&self, index: &mut InstalledIndex, entry: &ToolIndexEntry) {
        index.get_versions_mut(self.kind).push(entry.clone());
    }

    fn find_installed(&self, index: &InstalledIndex, version: &str) -> Option<()> {
        index
            .get_versions(self.kind)
            .iter()
            .find(|v| v.version == version)
            .map(|_| ())
    }

    fn find_installed_by_version(
        &self,
        index: &InstalledIndex,
        version: &str,
    ) -> Option<ToolIndexEntry> {
        index
            .get_versions(self.kind)
            .iter()
            .find(|v| v.version == version)
            .cloned()
    }

    fn get_all_installed(&self, index: &InstalledIndex) -> Vec<ToolIndexEntry> {
        index.get_versions(self.kind).to_vec()
    }

    // ========== 符号链接操作 ==========

    /// 更新版本符号链接（bin + default）
    fn update_version_symlinks(&self, version: &str) -> Result<(), ZzmError> {
        // 更新 bin 目录符号链接
        match self.kind {
            ToolKind::Zig => self.path_manager.create_zig_symlink(version)?,
            ToolKind::Zls => self.path_manager.create_zls_symlink(version)?,
        }

        // 更新 default 目录符号链接（非致命错误）
        let default_result = match self.kind {
            ToolKind::Zig => self.path_manager.create_default_zig_symlink(version),
            ToolKind::Zls => self.path_manager.create_default_zls_symlink(version),
        };
        if let Err(e) = default_result {
            (self.callbacks.on_warning)(&format!(
                "创建 {} 目录符号链接失败: {e}，不影响使用，但 {} 模式不可用",
                self.default_link_name(),
                self.home_env_name(),
            ));
        }

        Ok(())
    }

    /// 移除版本符号链接（bin + default）
    fn remove_version_symlinks(&self) -> Result<(), ZzmError> {
        match self.kind {
            ToolKind::Zig => {
                self.path_manager.remove_zig_symlink()?;
                self.path_manager.remove_default_symlink()?;
            }
            ToolKind::Zls => {
                self.path_manager.remove_zls_symlink()?;
                self.path_manager.remove_default_zls_symlink()?;
            }
        }
        Ok(())
    }
}

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::channel::Channel;
    use crate::infra::path_manager::{ToolExtraData, ToolIndexEntry};

    // ---- ToolKind 测试 ----

    #[test]
    fn test_tool_kind_equality() {
        assert_eq!(ToolKind::Zig, ToolKind::Zig);
        assert_eq!(ToolKind::Zls, ToolKind::Zls);
        assert_ne!(ToolKind::Zig, ToolKind::Zls);
    }

    #[test]
    fn test_tool_kind_copy() {
        let a = ToolKind::Zig;
        let b = a; // Copy 语义
        assert_eq!(a, b);
    }

    // ---- DownloadAsset 测试 ----

    #[test]
    fn test_download_asset_creation() {
        let asset = DownloadAsset {
            url: "https://example.com/zig.tar.xz".to_string(),
            filename: "zig.tar.xz".to_string(),
            shasum: "abc123".to_string(),
            size: "50 MB".to_string(),
        };
        assert_eq!(asset.url, "https://example.com/zig.tar.xz");
        assert_eq!(asset.filename, "zig.tar.xz");
        assert_eq!(asset.shasum, "abc123");
        assert_eq!(asset.size, "50 MB");
    }

    #[test]
    fn test_download_asset_serde_roundtrip() {
        let asset = DownloadAsset {
            url: "https://example.com/zig.tar.xz".to_string(),
            filename: "zig.tar.xz".to_string(),
            shasum: "abc123".to_string(),
            size: "50 MB".to_string(),
        };
        let json = serde_json::to_string(&asset).unwrap();
        let deserialized: DownloadAsset = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.url, asset.url);
        assert_eq!(deserialized.filename, asset.filename);
        assert_eq!(deserialized.shasum, asset.shasum);
    }

    #[test]
    fn test_download_asset_empty_shasum() {
        let asset = DownloadAsset {
            url: "https://example.com/zls.tar.xz".to_string(),
            filename: "zls.tar.xz".to_string(),
            shasum: String::new(),
            size: "30 MB".to_string(),
        };
        assert!(asset.shasum.is_empty());
    }

    // ---- VersionInfo 测试 ----

    #[test]
    fn test_version_info_with_asset() {
        let info = VersionInfo {
            version: "0.13.0".to_string(),
            channel: Channel::Stable,
            date: Some("2026-04-13".to_string()),
            asset: Some(DownloadAsset {
                url: "https://example.com/zig.tar.xz".to_string(),
                filename: "zig.tar.xz".to_string(),
                shasum: "abc123".to_string(),
                size: "50 MB".to_string(),
            }),
        };
        assert_eq!(info.version, "0.13.0");
        assert_eq!(info.channel, Channel::Stable);
        assert!(info.asset.is_some());
    }

    #[test]
    fn test_version_info_without_asset() {
        let info = VersionInfo {
            version: "0.14.0-dev".to_string(),
            channel: Channel::Nightly,
            date: None,
            asset: None,
        };
        assert!(info.asset.is_none());
    }

    #[test]
    fn test_version_info_serde_roundtrip() {
        let info = VersionInfo {
            version: "0.13.0".to_string(),
            channel: Channel::Stable,
            date: Some("2026-04-13".to_string()),
            asset: Some(DownloadAsset {
                url: "https://example.com/zig.tar.xz".to_string(),
                filename: "zig.tar.xz".to_string(),
                shasum: "abc123".to_string(),
                size: "50 MB".to_string(),
            }),
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: VersionInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.version, "0.13.0");
        assert_eq!(deserialized.channel, Channel::Stable);
        assert!(deserialized.asset.is_some());
    }

    // ---- ToolIndexEntry 测试 ----

    fn make_zig_entry(version: &str, channel: Channel) -> ToolIndexEntry {
        ToolIndexEntry {
            version: version.to_string(),
            install_path: std::path::PathBuf::from(format!("/home/.zzm/versions/zig/{version}")),
            installed_at: "2026-04-25T00:00:00+00:00".to_string(),
            extra: ToolExtraData::Zig { channel },
        }
    }

    fn make_zls_entry(version: &str, zig_version: Option<&str>) -> ToolIndexEntry {
        ToolIndexEntry {
            version: version.to_string(),
            install_path: std::path::PathBuf::from(format!("/home/.zzm/versions/zls/{version}")),
            installed_at: "2026-04-25T00:00:00+00:00".to_string(),
            extra: ToolExtraData::Zls {
                zig_version: zig_version.map(|s| s.to_string()),
            },
        }
    }

    #[test]
    fn test_tool_index_entry_zig_accessors() {
        let v = make_zig_entry("0.13.0", Channel::Stable);
        assert_eq!(v.version, "0.13.0");
        assert!(v.install_path.to_string_lossy().contains("0.13.0"));
        assert_eq!(v.channel(), Some(&Channel::Stable));
        assert_eq!(v.zig_version(), None);
    }

    #[test]
    fn test_tool_index_entry_zls_accessors() {
        let v = make_zls_entry("0.13.0", Some("0.13.0"));
        assert_eq!(v.version, "0.13.0");
        assert!(v.install_path.to_string_lossy().contains("0.13.0"));
        assert_eq!(v.channel(), None);
        assert_eq!(v.zig_version(), Some("0.13.0"));
    }

    #[test]
    fn test_tool_index_entry_zls_no_zig_version() {
        let v = make_zls_entry("0.14.0-dev", None);
        assert_eq!(v.zig_version(), None);
    }

    #[test]
    fn test_tool_index_entry_zig_nightly() {
        let v = make_zig_entry("0.14.0-dev", Channel::Nightly);
        assert_eq!(v.channel(), Some(&Channel::Nightly));
    }

    #[test]
    fn test_tool_index_entry_zig_prerelease() {
        let v = make_zig_entry("0.14.0-rc1", Channel::Prerelease);
        assert_eq!(v.channel(), Some(&Channel::Prerelease));
    }

    #[test]
    fn test_tool_index_entry_serde_roundtrip() {
        let zig = make_zig_entry("0.13.0", Channel::Stable);
        let json = serde_json::to_string(&zig).unwrap();
        let deserialized: ToolIndexEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.version, "0.13.0");

        let zls = make_zls_entry("0.13.0", Some("0.13.0"));
        let json = serde_json::to_string(&zls).unwrap();
        let deserialized: ToolIndexEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.version, "0.13.0");
        assert_eq!(deserialized.zig_version(), Some("0.13.0"));
    }

    #[test]
    fn test_tool_index_entry_clone() {
        let v = make_zig_entry("0.13.0", Channel::Stable);
        let cloned = v.clone();
        assert_eq!(cloned.version, "0.13.0");
        assert_eq!(cloned.channel(), Some(&Channel::Stable));
    }

    // ---- 索引操作辅助方法测试 ----

    fn make_index() -> InstalledIndex {
        let mut tools = std::collections::HashMap::new();
        tools.insert(
            ToolKind::Zig,
            vec![ToolIndexEntry {
                version: "0.13.0".to_string(),
                install_path: std::path::PathBuf::from("/home/.zzm/versions/zig/0.13.0"),
                installed_at: "2026-04-25T00:00:00+00:00".to_string(),
                extra: ToolExtraData::Zig {
                    channel: Channel::Stable,
                },
            }],
        );
        tools.insert(
            ToolKind::Zls,
            vec![ToolIndexEntry {
                version: "0.13.0".to_string(),
                install_path: std::path::PathBuf::from("/home/.zzm/versions/zls/0.13.0"),
                installed_at: "2026-04-25T00:00:00+00:00".to_string(),
                extra: ToolExtraData::Zls {
                    zig_version: Some("0.13.0".to_string()),
                },
            }],
        );
        let mut active = std::collections::HashMap::new();
        active.insert(ToolKind::Zig, "0.13.0".to_string());
        active.insert(ToolKind::Zls, "0.13.0".to_string());
        InstalledIndex { tools, active }
    }

    /// 模拟 VersionProvider（测试用，不依赖文件系统）
    #[allow(dead_code)]
    struct MockVersionProvider;

    impl VersionProvider for MockVersionProvider {
        async fn get_version_info(&self, _version: &str) -> Result<VersionInfo, ZzmError> {
            Err(ZzmError::VersionNotFound {
                version: "mock".to_string(),
            })
        }

        async fn list_remote_versions(&self) -> Result<Vec<VersionInfo>, ZzmError> {
            Ok(vec![])
        }
    }

    #[test]
    fn test_installed_index_default() {
        let index = InstalledIndex::default();
        assert!(index.tools.is_empty());
        assert!(index.active.is_empty());
    }

    #[test]
    fn test_installed_index_serde_roundtrip() {
        let index = make_index();
        let json = serde_json::to_string(&index).unwrap();
        let deserialized: InstalledIndex = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.get_versions(ToolKind::Zig).len(), 1);
        assert_eq!(deserialized.get_versions(ToolKind::Zls).len(), 1);
        assert_eq!(deserialized.get_active(ToolKind::Zig), Some("0.13.0"));
        assert_eq!(deserialized.get_active(ToolKind::Zls), Some("0.13.0"));
    }

    #[test]
    fn test_installed_index_empty_deserialization() {
        let json = r#"{}"#;
        let index: InstalledIndex = serde_json::from_str(json).unwrap();
        assert!(index.tools.is_empty());
        assert!(index.active.is_empty());
    }

    #[test]
    fn test_installed_index_legacy_migration() {
        // 旧格式 JSON
        let legacy_json = r#"{
            "zig_versions": [{"version":"0.13.0","install_path":"/home/.zzm/versions/zig/0.13.0","installed_at":"2026-04-25T00:00:00+00:00","channel":"stable"}],
            "zls_versions": [{"version":"0.13.0","install_path":"/home/.zzm/versions/zls/0.13.0","installed_at":"2026-04-25T00:00:00+00:00","zig_version":"0.13.0"}],
            "active_zig": "0.13.0",
            "active_zls": "0.13.0"
        }"#;
        let index = InstalledIndex::from_json_str(legacy_json).unwrap();
        assert_eq!(index.get_versions(ToolKind::Zig).len(), 1);
        assert_eq!(index.get_versions(ToolKind::Zls).len(), 1);
        assert_eq!(index.get_active(ToolKind::Zig), Some("0.13.0"));
        assert_eq!(index.get_active(ToolKind::Zls), Some("0.13.0"));
    }

    #[test]
    fn test_tool_index_entry_zig_channel_serde() {
        let v = ToolIndexEntry {
            version: "0.14.0-dev".to_string(),
            install_path: std::path::PathBuf::from("/home/.zzm/versions/zig/0.14.0-dev"),
            installed_at: "2026-04-25T00:00:00+00:00".to_string(),
            extra: ToolExtraData::Zig {
                channel: Channel::Nightly,
            },
        };
        let json = serde_json::to_string(&v).unwrap();
        let deserialized: ToolIndexEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.channel(), Some(&Channel::Nightly));
    }

    #[test]
    fn test_tool_index_entry_zls_with_zig_version() {
        let v = ToolIndexEntry {
            version: "0.13.0".to_string(),
            install_path: std::path::PathBuf::from("/home/.zzm/versions/zls/0.13.0"),
            installed_at: "2026-04-25T00:00:00+00:00".to_string(),
            extra: ToolExtraData::Zls {
                zig_version: Some("0.13.0".to_string()),
            },
        };
        let json = serde_json::to_string(&v).unwrap();
        let deserialized: ToolIndexEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.zig_version(), Some("0.13.0"));
    }

    #[test]
    fn test_tool_index_entry_zls_without_zig_version() {
        let v = ToolIndexEntry {
            version: "0.14.0-dev".to_string(),
            install_path: std::path::PathBuf::from("/home/.zzm/versions/zls/0.14.0-dev"),
            installed_at: "2026-04-25T00:00:00+00:00".to_string(),
            extra: ToolExtraData::Zls { zig_version: None },
        };
        let json = serde_json::to_string(&v).unwrap();
        let deserialized: ToolIndexEntry = serde_json::from_str(&json).unwrap();
        assert!(deserialized.zig_version().is_none());
    }

    // ---- 流式校验测试 ----

    #[test]
    fn test_verify_checksum_streaming_with_temp_file() {
        use std::io::Write;
        let temp_dir = std::env::temp_dir().join("zzm_test_streaming");
        let _ = std::fs::create_dir_all(&temp_dir);
        let file_path = temp_dir.join("test_checksum.bin");

        let data = b"hello world for streaming test";
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(data).unwrap();
        drop(file);

        // 用内存校验得到期望值
        let expected = crate::infra::checksum::calculate_sha256(data);
        let result = crate::infra::checksum::verify_checksum_streaming(&file_path, &expected);
        assert!(result.unwrap());

        // 清理
        let _ = std::fs::remove_file(&file_path);
    }

    #[test]
    fn test_verify_checksum_streaming_mismatch() {
        use std::io::Write;
        let temp_dir = std::env::temp_dir().join("zzm_test_streaming_mismatch");
        let _ = std::fs::create_dir_all(&temp_dir);
        let file_path = temp_dir.join("test_mismatch.bin");

        let data = b"some data";
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(data).unwrap();
        drop(file);

        let result = crate::infra::checksum::verify_checksum_streaming(&file_path, "0000deadbeef");
        assert!(!result.unwrap());

        let _ = std::fs::remove_file(&file_path);
    }

    #[test]
    fn test_verify_checksum_streaming_file_not_found() {
        let result = crate::infra::checksum::verify_checksum_streaming(
            std::path::Path::new("/nonexistent/file.bin"),
            "abc",
        );
        assert!(result.is_err());
    }
}
