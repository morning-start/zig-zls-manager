use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::core::channel::Channel;
use crate::infra::checksum;
use crate::infra::downloader::Downloader;
use crate::infra::filesystem;
use crate::infra::path_manager::{
    InstalledIndex, InstalledZigVersion, InstalledZlsVersion, PathManager,
};
use crate::output::console_output;
use crate::platform::PlatformTrait;
use crate::utils::error::ZzmError;
use crate::utils::version::resolve_version;

/// 工具类型标识
///
/// 用于区分 Zig 和 ZLS 的路径计算、索引操作等差异
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolKind {
    Zig,
    Zls,
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
}

impl<T: VersionProvider> ToolManager<T> {
    /// 创建新的工具管理器
    pub fn new(
        kind: ToolKind,
        platform: Box<dyn PlatformTrait>,
        api_client: T,
    ) -> Result<Self, ZzmError> {
        let path_manager = PathManager::new(platform.clone_box());
        let downloader = Downloader::new()?;

        Ok(Self {
            kind,
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

    /// 安装指定版本
    ///
    /// 通用流程：解析版本 → 获取信息 → 检查已安装 → 下载 → 校验 → 解压 → 注册
    pub async fn install(
        &self,
        version: &str,
        force: bool,
        zig_version: Option<&str>,
    ) -> Result<InstalledVersion, ZzmError> {
        self.initialize()?;

        let tool_name = self.tool_name();
        let resolved = resolve_version(version)?;
        console_output::print_step(
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
        let index = self.path_manager.read_installed_index()?;
        let already_installed = self.is_version_installed(&index, &resolved);

        if already_installed && !force {
            return Err(ZzmError::AlreadyInstalled { version: resolved });
        }

        if already_installed && force {
            console_output::print_info(&format!("强制重装 {tool_name} 版本: {resolved}"));
            let _ = self.uninstall(&resolved);
        }

        // 下载
        console_output::print_step(2, 5, &format!("下载 {tool_name} {resolved}"));
        let cache_dir = self.path_manager.cache_dir();
        let archive_path = self
            .downloader
            .download_to_cache(&asset.url, &cache_dir, &asset.filename)
            .await?;

        // 校验（仅 Zig 提供 shasum）
        console_output::print_step(3, 5, "校验文件完整性");
        if !asset.shasum.is_empty() {
            let verified = checksum::verify_checksum_streaming(&archive_path, &asset.shasum)?;
            if !verified {
                let _ = std::fs::remove_file(&archive_path);
                return Err(ZzmError::ChecksumMismatch {
                    expected: asset.shasum.clone(),
                    actual: String::new(), // 流式校验无法返回实际值
                });
            }
            console_output::print_success("校验通过");
        } else {
            console_output::print_warning("未提供校验和，跳过校验");
        }

        // 解压
        console_output::print_step(4, 5, "解压安装");
        let version_dir = self.version_dir(&resolved);

        if version_dir.exists() {
            filesystem::remove_dir_all(&version_dir)?;
        }

        let extracted_root = filesystem::extract_archive(&archive_path, &version_dir)?;
        let tmp_name = format!("tmp_move_{}", self.kind_suffix());
        filesystem::reorganize_extracted_files(&extracted_root, &version_dir, &tmp_name)?;

        // 工具特定的后处理
        self.post_install(&resolved)?;

        // 注册
        console_output::print_step(5, 5, &format!("注册 {tool_name} 版本"));
        let installed = self.create_installed_record(
            &resolved,
            version_dir,
            &version_info.channel,
            zig_version,
        );

        let mut index = self.path_manager.read_installed_index()?;
        self.remove_version_from_index(&mut index, &resolved);
        self.add_version_to_index(&mut index, &installed);
        self.path_manager.write_installed_index(&index)?;

        console_output::print_success(&format!("{tool_name} {resolved} 安装完成"));
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
            self.remove_symlinks()?;
            self.remove_default_symlink()?;
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

        console_output::print_success(&format!("{tool_name} {resolved} 已卸载"));
        Ok(())
    }

    /// 切换到指定版本
    pub async fn use_version(&self, version: &str) -> Result<String, ZzmError> {
        let tool_name = self.tool_name();
        let resolved = resolve_version(version)?;

        let index = self.path_manager.read_installed_index()?;
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

        // 更新 bin 目录符号链接
        self.create_symlink(&resolved)?;

        // 更新 default 目录符号链接
        if let Err(e) = self.create_default_symlink(&resolved) {
            console_output::print_warning(&format!(
                "创建 {} 目录符号链接失败: {e}，不影响使用，但 {} 模式不可用",
                self.default_link_name(),
                self.home_env_name(),
            ));
        }

        // 更新索引中的 active 版本
        let mut index = self.path_manager.read_installed_index()?;
        self.set_active_version(&mut index, Some(resolved.clone()));
        self.path_manager.write_installed_index(&index)?;

        console_output::print_success(&format!("已切换到 {tool_name} {resolved}"));
        console_output::print_info(&format!(
            "提示: 设置 {}={} 即可通过 {} 使用当前版本",
            self.home_env_name(),
            self.default_dir_path().display(),
            self.home_env_name(),
        ));
        console_output::print_info("  或确保 bin 目录在 PATH 中（zzm info 查看详情）");
        Ok(resolved)
    }

    /// 获取当前激活的版本
    pub fn current(&self) -> Result<Option<InstalledVersion>, ZzmError> {
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
    pub fn list_installed(&self) -> Result<Vec<InstalledVersion>, ZzmError> {
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

    /// 安装后处理（设置可执行权限，ZLS 额外查找二进制）
    fn post_install(&self, version: &str) -> Result<(), ZzmError> {
        let binary_path = self.binary_path(version);
        if binary_path.exists() {
            filesystem::set_executable(&binary_path)?;
        } else if self.kind == ToolKind::Zls {
            // ZLS 的二进制文件可能没有版本后缀，需要查找
            self.find_and_link_zls_binary(version)?;
        }
        Ok(())
    }

    /// ZLS 专用：查找并链接二进制文件
    fn find_and_link_zls_binary(&self, version: &str) -> Result<(), ZzmError> {
        let version_dir = self.version_dir(version);

        // 在版本目录中搜索 zls 或 zls.exe
        if let Ok(entries) = std::fs::read_dir(&version_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if name == "zls" || name == "zls.exe" {
                        let dest = self.binary_path(version);
                        if path != dest {
                            std::fs::copy(&path, &dest).map_err(ZzmError::Io)?;
                        }
                        return Ok(());
                    }
                }
            }
        }

        // 在子目录中搜索
        if let Ok(entries) = std::fs::read_dir(&version_dir) {
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
                                let dest = self.binary_path(version);
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

    /// 创建已安装记录
    fn create_installed_record(
        &self,
        version: &str,
        version_dir: std::path::PathBuf,
        channel: &Channel,
        zig_version: Option<&str>,
    ) -> InstalledVersion {
        match self.kind {
            ToolKind::Zig => InstalledVersion::Zig(InstalledZigVersion {
                version: version.to_string(),
                install_path: version_dir,
                installed_at: Utc::now().to_rfc3339(),
                channel: channel.clone(),
            }),
            ToolKind::Zls => InstalledVersion::Zls(InstalledZlsVersion {
                version: version.to_string(),
                install_path: version_dir,
                installed_at: Utc::now().to_rfc3339(),
                zig_version: zig_version.map(|s| s.to_string()),
            }),
        }
    }

    // ========== 索引操作辅助 ==========

    fn is_version_installed(&self, index: &InstalledIndex, version: &str) -> bool {
        match self.kind {
            ToolKind::Zig => index.zig_versions.iter().any(|v| v.version == version),
            ToolKind::Zls => index.zls_versions.iter().any(|v| v.version == version),
        }
    }

    fn is_active_version(&self, index: &InstalledIndex, version: &str) -> bool {
        match self.kind {
            ToolKind::Zig => index.active_zig.as_ref() == Some(&version.to_string()),
            ToolKind::Zls => index.active_zls.as_ref() == Some(&version.to_string()),
        }
    }

    fn set_active_version(&self, index: &mut InstalledIndex, version: Option<String>) {
        match self.kind {
            ToolKind::Zig => index.active_zig = version,
            ToolKind::Zls => index.active_zls = version,
        }
    }

    fn get_active_version(&self, index: &InstalledIndex) -> Option<String> {
        match self.kind {
            ToolKind::Zig => index.active_zig.clone(),
            ToolKind::Zls => index.active_zls.clone(),
        }
    }

    fn find_installed_position(&self, index: &InstalledIndex, version: &str) -> Option<usize> {
        match self.kind {
            ToolKind::Zig => index.zig_versions.iter().position(|v| v.version == version),
            ToolKind::Zls => index.zls_versions.iter().position(|v| v.version == version),
        }
    }

    fn remove_version_from_index(&self, index: &mut InstalledIndex, version: &str) {
        match self.kind {
            ToolKind::Zig => index.zig_versions.retain(|v| v.version != version),
            ToolKind::Zls => index.zls_versions.retain(|v| v.version != version),
        }
    }

    fn remove_version_at(&self, index: &mut InstalledIndex, pos: usize) {
        match self.kind {
            ToolKind::Zig => {
                index.zig_versions.remove(pos);
            }
            ToolKind::Zls => {
                index.zls_versions.remove(pos);
            }
        }
    }

    fn add_version_to_index(&self, index: &mut InstalledIndex, installed: &InstalledVersion) {
        match (self.kind, installed) {
            (ToolKind::Zig, InstalledVersion::Zig(v)) => index.zig_versions.push(v.clone()),
            (ToolKind::Zls, InstalledVersion::Zls(v)) => index.zls_versions.push(v.clone()),
            _ => {}
        }
    }

    fn find_installed(&self, index: &InstalledIndex, version: &str) -> Option<()> {
        match self.kind {
            ToolKind::Zig => index
                .zig_versions
                .iter()
                .find(|v| v.version == version)
                .map(|_| ()),
            ToolKind::Zls => index
                .zls_versions
                .iter()
                .find(|v| v.version == version)
                .map(|_| ()),
        }
    }

    fn find_installed_by_version(
        &self,
        index: &InstalledIndex,
        version: &str,
    ) -> Option<InstalledVersion> {
        match self.kind {
            ToolKind::Zig => index
                .zig_versions
                .iter()
                .find(|v| v.version == version)
                .cloned()
                .map(InstalledVersion::Zig),
            ToolKind::Zls => index
                .zls_versions
                .iter()
                .find(|v| v.version == version)
                .cloned()
                .map(InstalledVersion::Zls),
        }
    }

    fn get_all_installed(&self, index: &InstalledIndex) -> Vec<InstalledVersion> {
        match self.kind {
            ToolKind::Zig => index
                .zig_versions
                .iter()
                .cloned()
                .map(InstalledVersion::Zig)
                .collect(),
            ToolKind::Zls => index
                .zls_versions
                .iter()
                .cloned()
                .map(InstalledVersion::Zls)
                .collect(),
        }
    }

    // ========== 符号链接操作 ==========

    fn create_symlink(&self, version: &str) -> Result<(), ZzmError> {
        match self.kind {
            ToolKind::Zig => self.path_manager.create_zig_symlink(version),
            ToolKind::Zls => self.path_manager.create_zls_symlink(version),
        }
    }

    fn remove_symlinks(&self) -> Result<(), ZzmError> {
        match self.kind {
            ToolKind::Zig => self.path_manager.remove_zig_symlink(),
            ToolKind::Zls => self.path_manager.remove_zls_symlink(),
        }
    }

    fn create_default_symlink(&self, version: &str) -> Result<(), ZzmError> {
        match self.kind {
            ToolKind::Zig => self.path_manager.create_default_zig_symlink(version),
            ToolKind::Zls => self.path_manager.create_default_zls_symlink(version),
        }
    }

    fn remove_default_symlink(&self) -> Result<(), ZzmError> {
        match self.kind {
            ToolKind::Zig => self.path_manager.remove_default_symlink(),
            ToolKind::Zls => self.path_manager.remove_default_zls_symlink(),
        }
    }
}

/// 已安装版本的统一枚举
///
/// 封装 Zig 和 ZLS 不同的已安装版本结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstalledVersion {
    Zig(InstalledZigVersion),
    Zls(InstalledZlsVersion),
}

impl InstalledVersion {
    /// 获取版本号
    pub fn version(&self) -> &str {
        match self {
            InstalledVersion::Zig(v) => &v.version,
            InstalledVersion::Zls(v) => &v.version,
        }
    }

    /// 获取安装路径
    pub fn install_path(&self) -> &std::path::Path {
        match self {
            InstalledVersion::Zig(v) => &v.install_path,
            InstalledVersion::Zls(v) => &v.install_path,
        }
    }

    /// 获取通道信息（Zig 有独立 channel 字段）
    pub fn channel(&self) -> Option<&Channel> {
        match self {
            InstalledVersion::Zig(v) => Some(&v.channel),
            InstalledVersion::Zls(_) => None,
        }
    }

    /// 获取关联的 Zig 版本（仅 ZLS）
    pub fn zig_version(&self) -> Option<&str> {
        match self {
            InstalledVersion::Zls(v) => v.zig_version.as_deref(),
            InstalledVersion::Zig(_) => None,
        }
    }
}
