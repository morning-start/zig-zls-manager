use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::platform::PlatformTrait;
use crate::utils::error::ZzmError;

/// 已安装版本索引（存储在 installed.json）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstalledIndex {
    /// 已安装的 Zig 版本列表
    #[serde(default)]
    pub zig_versions: Vec<InstalledZigVersion>,
    /// 已安装的 ZLS 版本列表
    #[serde(default)]
    pub zls_versions: Vec<InstalledZlsVersion>,
    /// 当前激活的 Zig 版本
    #[serde(default)]
    pub active_zig: Option<String>,
    /// 当前激活的 ZLS 版本
    #[serde(default)]
    pub active_zls: Option<String>,
}

/// 已安装的 Zig 版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledZigVersion {
    /// 版本号
    pub version: String,
    /// 安装路径
    pub install_path: PathBuf,
    /// 安装时间 (ISO 8601)
    pub installed_at: String,
    /// 版本通道
    pub channel: String,
}

/// 已安装的 ZLS 版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledZlsVersion {
    /// 版本号
    pub version: String,
    /// 安装路径
    pub install_path: PathBuf,
    /// 安装时间 (ISO 8601)
    pub installed_at: String,
    /// 关联的 Zig 版本
    pub zig_version: Option<String>,
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
    pub fn install_dir(&self) -> PathBuf {
        self.platform.default_install_dir()
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
        self.zig_version_dir(version).join(self.platform.zig_binary_name())
    }

    /// 获取指定 ZLS 版本的安装路径
    pub fn zls_version_dir(&self, version: &str) -> PathBuf {
        self.zls_versions_dir().join(version)
    }

    /// 获取指定 ZLS 版本的二进制文件路径
    pub fn zls_binary_path(&self, version: &str) -> PathBuf {
        self.zls_version_dir(version).join(self.platform.zls_binary_name())
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
    pub fn read_installed_index(&self) -> Result<InstalledIndex, ZzmError> {
        let path = self.installed_index_path();
        if !path.exists() {
            return Ok(InstalledIndex::default());
        }

        let content = std::fs::read_to_string(&path)?;
        let index: InstalledIndex = serde_json::from_str(&content)?;
        Ok(index)
    }

    /// 写入已安装版本索引
    pub fn write_installed_index(&self, index: &InstalledIndex) -> Result<(), ZzmError> {
        let path = self.installed_index_path();
        let content = serde_json::to_string_pretty(index)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// 创建 Zig 版本的符号链接
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

    /// 创建 ZLS 版本的符号链接
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
            index.active_zig
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

    #[test]
    fn test_installed_index_default() {
        let index = InstalledIndex::default();
        assert!(index.zig_versions.is_empty());
        assert!(index.zls_versions.is_empty());
        assert!(index.active_zig.is_none());
        assert!(index.active_zls.is_none());
    }

    #[test]
    fn test_installed_index_serde() {
        let index = InstalledIndex {
            zig_versions: vec![InstalledZigVersion {
                version: "0.13.0".to_string(),
                install_path: PathBuf::from("/home/user/.zzm/versions/zig/0.13.0"),
                installed_at: "2026-04-24T10:00:00Z".to_string(),
                channel: "stable".to_string(),
            }],
            zls_versions: vec![],
            active_zig: Some("0.13.0".to_string()),
            active_zls: None,
        };

        let json = serde_json::to_string_pretty(&index).unwrap();
        let deserialized: InstalledIndex = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.active_zig, Some("0.13.0".to_string()));
        assert_eq!(deserialized.zig_versions.len(), 1);
    }
}