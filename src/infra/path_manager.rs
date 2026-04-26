use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use crate::core::channel::Channel;
use crate::core::tool_manager::ToolKind;
use crate::platform::PlatformTrait;
use crate::utils::error::ZzmError;

/// 工具特定的额外数据
///
/// 封装 Zig 和 ZLS 不同的元数据字段，
/// 新增工具类型只需添加变体即可扩展
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolExtraData {
    /// Zig 额外数据：版本通道
    Zig { channel: Channel },
    /// ZLS 额外数据：关联的 Zig 版本
    Zls { zig_version: Option<String> },
}

/// 统一的已安装版本条目
///
/// 消除了 `InstalledZigVersion` / `InstalledZlsVersion` 的重复字段，
/// 差异部分通过 `extra` 枚举承载
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolIndexEntry {
    /// 版本号
    pub version: String,
    /// 安装路径
    pub install_path: PathBuf,
    /// 安装时间 (ISO 8601)
    pub installed_at: String,
    /// 工具特定的额外数据
    pub extra: ToolExtraData,
}

impl ToolIndexEntry {
    /// 获取通道信息（仅 Zig 有）
    pub fn channel(&self) -> Option<&Channel> {
        match &self.extra {
            ToolExtraData::Zig { channel } => Some(channel),
            ToolExtraData::Zls { .. } => None,
        }
    }

    /// 获取关联的 Zig 版本（仅 ZLS 有）
    pub fn zig_version(&self) -> Option<&str> {
        match &self.extra {
            ToolExtraData::Zls { zig_version } => zig_version.as_deref(),
            ToolExtraData::Zig { .. } => None,
        }
    }
}

/// 已安装版本索引（存储在 installed.json）
///
/// 使用 `HashMap<ToolKind, _>` 统一数据层，
/// 消除所有 `match self.kind` 索引分支
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstalledIndex {
    /// 已安装的工具版本，按工具类型分组
    #[serde(default)]
    pub tools: HashMap<ToolKind, Vec<ToolIndexEntry>>,
    /// 当前激活的版本，按工具类型分组
    #[serde(default)]
    pub active: HashMap<ToolKind, String>,
}

// ========== 旧格式兼容 ==========

/// 旧格式索引（用于兼容读取已存在的 installed.json）
#[derive(Debug, Deserialize)]
struct LegacyInstalledIndex {
    #[serde(default)]
    zig_versions: Vec<LegacyInstalledVersion>,
    #[serde(default)]
    zls_versions: Vec<LegacyInstalledVersion>,
    #[serde(default)]
    active_zig: Option<String>,
    #[serde(default)]
    active_zls: Option<String>,
}

/// 旧格式版本条目
#[derive(Debug, Deserialize)]
struct LegacyInstalledVersion {
    version: String,
    install_path: PathBuf,
    installed_at: String,
    #[serde(default)]
    channel: Option<Channel>,
    #[serde(default)]
    zig_version: Option<String>,
}

impl InstalledIndex {
    /// 从 JSON 字符串解析索引，自动兼容旧格式
    ///
    /// 新格式使用 `tools` + `active` 字段；
    /// 旧格式使用 `zig_versions` + `zls_versions` + `active_zig` + `active_zls`。
    /// 当检测到旧格式时自动迁移。
    pub fn from_json_str(content: &str) -> Result<Self, ZzmError> {
        // 先尝试旧格式（如果 JSON 包含 zig_versions 或 active_zig 字段）
        let value: serde_json::Value = serde_json::from_str(content)?;
        if (value.get("zig_versions").is_some() || value.get("active_zig").is_some())
            && let Ok(legacy) = serde_json::from_str::<LegacyInstalledIndex>(content)
        {
            return Ok(Self::from_legacy(legacy));
        }

        // 新格式解析
        let index: Self = serde_json::from_value(value)?;
        Ok(index)
    }

    /// 从旧格式迁移
    fn from_legacy(legacy: LegacyInstalledIndex) -> Self {
        let mut tools: HashMap<ToolKind, Vec<ToolIndexEntry>> = HashMap::new();

        let zig_entries: Vec<ToolIndexEntry> = legacy
            .zig_versions
            .into_iter()
            .map(|v| ToolIndexEntry {
                version: v.version,
                install_path: v.install_path,
                installed_at: v.installed_at,
                extra: ToolExtraData::Zig {
                    channel: v.channel.unwrap_or(Channel::Stable),
                },
            })
            .collect();

        let zls_entries: Vec<ToolIndexEntry> = legacy
            .zls_versions
            .into_iter()
            .map(|v| ToolIndexEntry {
                version: v.version,
                install_path: v.install_path,
                installed_at: v.installed_at,
                extra: ToolExtraData::Zls {
                    zig_version: v.zig_version,
                },
            })
            .collect();

        if !zig_entries.is_empty() {
            tools.insert(ToolKind::Zig, zig_entries);
        }
        if !zls_entries.is_empty() {
            tools.insert(ToolKind::Zls, zls_entries);
        }

        let mut active: HashMap<ToolKind, String> = HashMap::new();
        if let Some(zig) = legacy.active_zig {
            active.insert(ToolKind::Zig, zig);
        }
        if let Some(zls) = legacy.active_zls {
            active.insert(ToolKind::Zls, zls);
        }

        Self { tools, active }
    }

    /// 获取指定工具类型的版本列表
    pub fn get_versions(&self, kind: ToolKind) -> &[ToolIndexEntry] {
        self.tools.get(&kind).map_or(&[], |v| v)
    }

    /// 获取指定工具类型的版本列表（可变）
    pub fn get_versions_mut(&mut self, kind: ToolKind) -> &mut Vec<ToolIndexEntry> {
        self.tools.entry(kind).or_default()
    }

    /// 获取指定工具类型的激活版本
    pub fn get_active(&self, kind: ToolKind) -> Option<&str> {
        self.active.get(&kind).map(|s| s.as_str())
    }

    /// 设置指定工具类型的激活版本
    pub fn set_active(&mut self, kind: ToolKind, version: Option<String>) {
        match version {
            Some(v) => {
                self.active.insert(kind, v);
            }
            None => {
                self.active.remove(&kind);
            }
        }
    }
}

/// 路径管理器
///
/// 管理 zzm 的目录结构、符号链接和元数据文件
pub struct PathManager {
    platform: Box<dyn PlatformTrait>,
}

impl PathManager {
    /// 创建新的路径管理器
    pub fn new(platform: Box<dyn PlatformTrait>) -> Self {
        Self { platform }
    }

    /// 获取安装根目录
    #[allow(dead_code)] // 预留: info/status 命令
    pub fn install_dir(&self) -> PathBuf {
        self.platform.default_install_dir()
    }

    /// 获取 default 目录（指向当前激活版本目录的符号链接）
    ///
    /// 例如: ~/.zzm/default -> ~/.zzm/versions/zig/0.13.0
    /// 用法: 设置 `ZIG_HOME`=~/.zzm/default
    pub fn default_dir(&self) -> PathBuf {
        self.platform.default_dir()
    }

    /// 获取 bin 目录
    pub fn bin_dir(&self) -> PathBuf {
        self.platform.bin_dir()
    }

    /// 获取 Zig 版本存储目录
    pub fn zig_versions_dir(&self) -> PathBuf {
        self.platform.versions_dir().join("zig")
    }

    /// 获取 ZLS 版本存储目录
    pub fn zls_versions_dir(&self) -> PathBuf {
        self.platform.versions_dir().join("zls")
    }

    /// 获取缓存目录
    #[allow(dead_code)] // 预留: 缓存清理命令
    pub fn cache_dir(&self) -> PathBuf {
        self.platform.cache_dir()
    }

    /// 获取配置文件路径
    pub fn config_file_path(&self) -> PathBuf {
        self.platform.config_file_path()
    }

    /// 获取已安装版本索引文件路径
    pub fn installed_index_path(&self) -> PathBuf {
        self.platform.installed_index_path()
    }

    /// 获取指定 Zig 版本的安装路径
    ///
    /// 例如: ~/.zzm/versions/zig/0.13.0/
    pub fn zig_version_dir(&self, version: &str) -> PathBuf {
        self.zig_versions_dir().join(version)
    }

    /// 获取指定 Zig 版本的二进制文件路径
    pub fn zig_binary_path(&self, version: &str) -> PathBuf {
        self.zig_version_dir(version)
            .join(self.platform.zig_binary_name())
    }

    /// 获取指定 ZLS 版本的安装路径
    pub fn zls_version_dir(&self, version: &str) -> PathBuf {
        self.zls_versions_dir().join(version)
    }

    /// 获取指定 ZLS 版本的二进制文件路径
    pub fn zls_binary_path(&self, version: &str) -> PathBuf {
        self.zls_version_dir(version)
            .join(self.platform.zls_binary_name())
    }

    /// 获取 bin 目录中 zig 的符号链接/shim 路径
    pub fn zig_symlink_path(&self) -> PathBuf {
        self.bin_dir().join(self.platform.zig_binary_name())
    }

    /// 获取 bin 目录中 zls 的符号链接/shim 路径
    pub fn zls_symlink_path(&self) -> PathBuf {
        self.bin_dir().join(self.platform.zls_binary_name())
    }

    /// 初始化所有必需的目录
    pub fn initialize_dirs(&self) -> Result<(), ZzmError> {
        self.platform.initialize_dirs()
    }

    /// 读取已安装版本索引
    ///
    /// 自动兼容旧格式（zig_versions/zls_versions）和新格式（tools/active）
    pub fn read_installed_index(&self) -> Result<InstalledIndex, ZzmError> {
        let path = self.installed_index_path();
        if !path.exists() {
            return Ok(InstalledIndex::default());
        }

        let content = std::fs::read_to_string(&path)?;
        InstalledIndex::from_json_str(&content)
    }

    /// 写入已安装版本索引
    pub fn write_installed_index(&self, index: &InstalledIndex) -> Result<(), ZzmError> {
        let path = self.installed_index_path();
        let content = serde_json::to_string_pretty(index)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// 创建 Zig 版本的符号链接（bin 目录方式）
    pub fn create_zig_symlink(&self, version: &str) -> Result<(), ZzmError> {
        let target = self.zig_binary_path(version);
        let link = self.zig_symlink_path();

        if !target.exists() {
            return Err(ZzmError::NotInstalled {
                version: version.to_string(),
            });
        }

        self.platform.create_symlink(&target, &link)
    }

    /// 创建 ZLS 版本的符号链接（bin 目录方式）
    pub fn create_zls_symlink(&self, version: &str) -> Result<(), ZzmError> {
        let target = self.zls_binary_path(version);
        let link = self.zls_symlink_path();

        if !target.exists() {
            return Err(ZzmError::NotInstalled {
                version: version.to_string(),
            });
        }

        self.platform.create_symlink(&target, &link)
    }

    /// 创建 default 目录符号链接（指向指定 Zig 版本目录）
    ///
    /// 这是 java-mocha 风格的版本切换方式：
    /// `~/.zzm/default -> ~/.zzm/versions/zig/0.13.0`
    ///
    /// 用户设置 `ZIG_HOME=~/.zzm/default` 即可使用当前版本
    pub fn create_default_zig_symlink(&self, version: &str) -> Result<(), ZzmError> {
        let target = self.zig_version_dir(version);
        let link = self.default_dir();

        if !target.exists() {
            return Err(ZzmError::NotInstalled {
                version: version.to_string(),
            });
        }

        self.platform.create_symlink(&target, &link)
    }

    /// 创建 default-zls 目录符号链接（指向指定 ZLS 版本目录）
    ///
    /// 这是 java-mocha 风格的 ZLS 版本切换方式：
    /// `~/.zzm/default-zls -> ~/.zzm/versions/zls/0.13.0`
    ///
    /// 用户设置 `ZLS_HOME=~/.zzm/default-zls` 即可使用当前版本
    pub fn create_default_zls_symlink(&self, version: &str) -> Result<(), ZzmError> {
        let target = self.zls_version_dir(version);
        let link = self.install_dir().join("default-zls");

        if !target.exists() {
            return Err(ZzmError::NotInstalled {
                version: version.to_string(),
            });
        }

        self.platform.create_symlink(&target, &link)
    }

    /// 删除 default 目录符号链接
    pub fn remove_default_symlink(&self) -> Result<(), ZzmError> {
        let link = self.default_dir();
        self.platform.remove_symlink(&link)
    }

    /// 删除 default-zls 目录符号链接
    pub fn remove_default_zls_symlink(&self) -> Result<(), ZzmError> {
        let link = self.install_dir().join("default-zls");
        self.platform.remove_symlink(&link)
    }

    /// 删除 Zig 的符号链接
    pub fn remove_zig_symlink(&self) -> Result<(), ZzmError> {
        let link = self.zig_symlink_path();
        self.platform.remove_symlink(&link)
    }

    /// 删除 ZLS 的符号链接
    pub fn remove_zls_symlink(&self) -> Result<(), ZzmError> {
        let link = self.zls_symlink_path();
        self.platform.remove_symlink(&link)
    }

    /// 读取当前激活的 zig 符号链接目标，返回版本号
    #[allow(dead_code)] // 预留: info/status 命令增强
    pub fn read_current_zig_version(&self) -> Option<String> {
        let link = self.zig_symlink_path();
        if !link.exists() {
            return None;
        }

        // 尝试读取符号链接目标
        let target = if cfg!(windows) {
            // Windows 上可能是 shim 文件，尝试直接读取 installed index
            None
        } else {
            std::fs::read_link(&link).ok()
        };

        if let Some(target_path) = target {
            // 从路径中提取版本号
            // 例如: ~/.zzm/versions/zig/0.13.0/zig -> 0.13.0
            target_path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        } else {
            // 回退到读取 installed index
            let index = self.read_installed_index().ok()?;
            index.get_active(ToolKind::Zig).map(|s| s.to_string())
        }
    }

    /// 计算缓存目录的总大小（字节）
    pub fn cache_size(&self) -> Result<u64, ZzmError> {
        let cache_dir = self.cache_dir();
        if !cache_dir.exists() {
            return Ok(0);
        }
        Ok(calculate_dir_size(&cache_dir))
    }
}

/// 递归计算目录大小
fn calculate_dir_size(path: &Path) -> u64 {
    let mut total_size = 0u64;

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                total_size += calculate_dir_size(&path);
            } else if let Ok(metadata) = path.metadata() {
                total_size += metadata.len();
            }
        }
    }

    total_size
}

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::tool_manager::ToolKind;

    #[test]
    fn test_installed_index_default() {
        let index = InstalledIndex::default();
        assert!(index.tools.is_empty());
        assert!(index.active.is_empty());
        assert!(index.get_versions(ToolKind::Zig).is_empty());
        assert!(index.get_versions(ToolKind::Zls).is_empty());
        assert!(index.get_active(ToolKind::Zig).is_none());
        assert!(index.get_active(ToolKind::Zls).is_none());
    }

    #[test]
    fn test_installed_index_serde() {
        let mut tools = HashMap::new();
        tools.insert(
            ToolKind::Zig,
            vec![ToolIndexEntry {
                version: "0.13.0".to_string(),
                install_path: PathBuf::from("/home/user/.zzm/versions/zig/0.13.0"),
                installed_at: "2026-04-24T10:00:00Z".to_string(),
                extra: ToolExtraData::Zig {
                    channel: Channel::Stable,
                },
            }],
        );
        let mut active = HashMap::new();
        active.insert(ToolKind::Zig, "0.13.0".to_string());
        let index = InstalledIndex { tools, active };

        let json = serde_json::to_string_pretty(&index).unwrap();
        let deserialized: InstalledIndex = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.get_active(ToolKind::Zig), Some("0.13.0"));
        assert_eq!(deserialized.get_versions(ToolKind::Zig).len(), 1);
    }

    #[test]
    fn test_installed_index_with_zls() {
        let mut tools = HashMap::new();
        tools.insert(
            ToolKind::Zig,
            vec![ToolIndexEntry {
                version: "0.13.0".to_string(),
                install_path: PathBuf::from("/home/user/.zzm/versions/zig/0.13.0"),
                installed_at: "2026-04-24T10:00:00Z".to_string(),
                extra: ToolExtraData::Zig {
                    channel: Channel::Stable,
                },
            }],
        );
        tools.insert(
            ToolKind::Zls,
            vec![ToolIndexEntry {
                version: "0.13.0".to_string(),
                install_path: PathBuf::from("/home/user/.zzm/versions/zls/0.13.0"),
                installed_at: "2026-04-24T10:05:00Z".to_string(),
                extra: ToolExtraData::Zls {
                    zig_version: Some("0.13.0".to_string()),
                },
            }],
        );
        let mut active = HashMap::new();
        active.insert(ToolKind::Zig, "0.13.0".to_string());
        active.insert(ToolKind::Zls, "0.13.0".to_string());
        let index = InstalledIndex { tools, active };

        let json = serde_json::to_string_pretty(&index).unwrap();
        let deserialized: InstalledIndex = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.get_active(ToolKind::Zls), Some("0.13.0"));
        assert_eq!(deserialized.get_versions(ToolKind::Zls).len(), 1);
        assert_eq!(
            deserialized.get_versions(ToolKind::Zls)[0].zig_version(),
            Some("0.13.0")
        );
    }

    #[test]
    fn test_installed_index_multiple_versions() {
        let mut tools = HashMap::new();
        tools.insert(
            ToolKind::Zig,
            vec![
                ToolIndexEntry {
                    version: "0.13.0".to_string(),
                    install_path: PathBuf::from("/home/.zzm/versions/zig/0.13.0"),
                    installed_at: "2026-04-24T10:00:00Z".to_string(),
                    extra: ToolExtraData::Zig {
                        channel: Channel::Stable,
                    },
                },
                ToolIndexEntry {
                    version: "0.12.0".to_string(),
                    install_path: PathBuf::from("/home/.zzm/versions/zig/0.12.0"),
                    installed_at: "2026-04-23T10:00:00Z".to_string(),
                    extra: ToolExtraData::Zig {
                        channel: Channel::Stable,
                    },
                },
            ],
        );
        let mut active = HashMap::new();
        active.insert(ToolKind::Zig, "0.13.0".to_string());
        let index = InstalledIndex { tools, active };

        let json = serde_json::to_string_pretty(&index).unwrap();
        let deserialized: InstalledIndex = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.get_versions(ToolKind::Zig).len(), 2);
    }

    #[test]
    fn test_installed_index_legacy_migration() {
        // 旧格式 JSON 迁移到新格式
        let legacy_json = r#"{
            "zig_versions": [{"version":"0.13.0","install_path":"/home/.zzm/versions/zig/0.13.0","installed_at":"2026-04-24T10:00:00Z","channel":"stable"}],
            "zls_versions": [{"version":"0.13.0","install_path":"/home/.zzm/versions/zls/0.13.0","installed_at":"2026-04-24T10:05:00Z","zig_version":"0.13.0"}],
            "active_zig": "0.13.0",
            "active_zls": "0.13.0"
        }"#;
        let index = InstalledIndex::from_json_str(legacy_json).unwrap();
        assert_eq!(index.get_versions(ToolKind::Zig).len(), 1);
        assert_eq!(index.get_versions(ToolKind::Zls).len(), 1);
        assert_eq!(index.get_active(ToolKind::Zig), Some("0.13.0"));
        assert_eq!(index.get_active(ToolKind::Zls), Some("0.13.0"));
        assert_eq!(
            index.get_versions(ToolKind::Zig)[0].channel(),
            Some(&Channel::Stable)
        );
        assert_eq!(
            index.get_versions(ToolKind::Zls)[0].zig_version(),
            Some("0.13.0")
        );
    }

    #[test]
    fn test_tool_index_entry_zls_no_zig() {
        let entry = ToolIndexEntry {
            version: "0.13.0".to_string(),
            install_path: PathBuf::from("/home/.zzm/versions/zls/0.13.0"),
            installed_at: "2026-04-24T10:00:00Z".to_string(),
            extra: ToolExtraData::Zls { zig_version: None },
        };

        let json = serde_json::to_string_pretty(&entry).unwrap();
        let parsed: ToolIndexEntry = serde_json::from_str(&json).unwrap();
        assert!(parsed.zig_version().is_none());
    }

    #[test]
    fn test_installed_index_get_versions_mut() {
        let mut index = InstalledIndex::default();
        assert!(index.get_versions(ToolKind::Zig).is_empty());

        index.get_versions_mut(ToolKind::Zig).push(ToolIndexEntry {
            version: "0.13.0".to_string(),
            install_path: PathBuf::from("/home/.zzm/versions/zig/0.13.0"),
            installed_at: "2026-04-24T10:00:00Z".to_string(),
            extra: ToolExtraData::Zig {
                channel: Channel::Stable,
            },
        });

        assert_eq!(index.get_versions(ToolKind::Zig).len(), 1);
    }

    #[test]
    fn test_installed_index_set_active() {
        let mut index = InstalledIndex::default();
        index.set_active(ToolKind::Zig, Some("0.13.0".to_string()));
        assert_eq!(index.get_active(ToolKind::Zig), Some("0.13.0"));

        index.set_active(ToolKind::Zig, None);
        assert!(index.get_active(ToolKind::Zig).is_none());
    }

    #[test]
    fn test_path_manager_creation() {
        let platform = crate::platform::detect_platform();
        let pm = PathManager::new(platform);
        let _ = pm;
    }

    #[test]
    fn test_path_manager_zig_version_dir() {
        let platform = crate::platform::detect_platform();
        let pm = PathManager::new(platform);
        let dir = pm.zig_version_dir("0.13.0");
        assert!(dir.to_string_lossy().contains("0.13.0"));
        assert!(dir.to_string_lossy().contains("zig"));
    }

    #[test]
    fn test_path_manager_zls_version_dir() {
        let platform = crate::platform::detect_platform();
        let pm = PathManager::new(platform);
        let dir = pm.zls_version_dir("0.13.0");
        assert!(dir.to_string_lossy().contains("0.13.0"));
        assert!(dir.to_string_lossy().contains("zls"));
    }

    #[test]
    fn test_path_manager_binary_paths() {
        let platform = crate::platform::detect_platform();
        let pm = PathManager::new(platform);
        let zig_bin = pm.zig_binary_path("0.13.0");
        let zls_bin = pm.zls_binary_path("0.13.0");
        assert!(zig_bin.to_string_lossy().contains("0.13.0"));
        assert!(zls_bin.to_string_lossy().contains("0.13.0"));
    }

    #[test]
    fn test_read_installed_index_nonexistent() {
        let index = InstalledIndex::default();
        assert!(index.get_versions(ToolKind::Zig).is_empty());
        assert!(index.get_active(ToolKind::Zig).is_none());
    }

    #[test]
    fn test_calculate_dir_size_empty() {
        let temp_dir = tempfile::tempdir().unwrap();
        let size = calculate_dir_size(temp_dir.path());
        assert_eq!(size, 0);
    }

    #[test]
    fn test_calculate_dir_size_with_files() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::fs::write(temp_dir.path().join("test.txt"), b"hello").unwrap();
        let size = calculate_dir_size(temp_dir.path());
        assert_eq!(size, 5);
    }

    #[test]
    fn test_calculate_dir_size_nested() {
        let temp_dir = tempfile::tempdir().unwrap();
        let sub_dir = temp_dir.path().join("sub");
        std::fs::create_dir_all(&sub_dir).unwrap();
        std::fs::write(temp_dir.path().join("a.txt"), b"12345").unwrap();
        std::fs::write(sub_dir.join("b.txt"), b"123").unwrap();
        let size = calculate_dir_size(temp_dir.path());
        assert_eq!(size, 8);
    }
}
