use std::path::{Path, PathBuf};

use crate::utils::error::ZzmError;

/// 配置文件部分结构体（仅读取 install_dir，避免循环依赖）
#[derive(Debug, serde::Deserialize)]
struct ZZMConfigPartial {
    #[serde(default)]
    install_dir: Option<String>,
}

/// 平台抽象 trait，封装平台特定操作
#[allow(dead_code)] // trait 部分方法预留: shell_config_files, is_admin
pub trait PlatformTrait: Send + Sync {
    /// 克隆为 Box<dyn PlatformTrait>
    fn clone_box(&self) -> Box<dyn PlatformTrait>;
    /// 获取平台名称
    fn name(&self) -> &'static str;

    /// 获取平台特定的默认安装目录（不考虑环境变量覆盖）
    ///
    /// 各平台实现此方法返回各自的默认路径：
    /// - Windows: `%LOCALAPPDATA%\zzm`
    /// - macOS: `~/.zzm`
    /// - Linux: `$XDG_DATA_HOME/zzm` 或 `~/.zzm`
    fn platform_default_dir(&self) -> PathBuf;

    /// 获取安装根目录（支持环境变量覆盖）
    ///
    /// 优先级：`ZZM_ROOT` 环境变量 > 配置文件 `install_dir` > 平台默认路径
    fn default_install_dir(&self) -> PathBuf {
        // 优先级 1: ZZM_ROOT 环境变量
        if let Ok(root) = std::env::var("ZZM_ROOT")
            && !root.is_empty()
        {
            return PathBuf::from(root);
        }

        // 优先级 2: 配置文件中的 install_dir
        // 延迟加载配置，避免循环依赖：只有在配置文件已存在时才读取
        let config_path = self.platform_default_dir().join("config.toml");
        if config_path.exists()
            && let Ok(content) = std::fs::read_to_string(&config_path)
            && let Ok(config) = toml::from_str::<ZZMConfigPartial>(&content)
            && let Some(ref install_dir) = config.install_dir
            && !install_dir.is_empty()
        {
            return PathBuf::from(install_dir);
        }

        // 优先级 3: 平台默认路径
        self.platform_default_dir()
    }

    /// 创建符号链接（跨平台适配）
    fn create_symlink(&self, original: &Path, link: &Path) -> Result<(), ZzmError>;

    /// 删除符号链接
    fn remove_symlink(&self, link: &Path) -> Result<(), ZzmError>;

    /// 获取 shell 配置文件路径列表
    fn shell_config_files(&self) -> Vec<PathBuf>;

    /// 获取 bin 目录
    fn bin_dir(&self) -> PathBuf {
        self.default_install_dir().join("bin")
    }

    /// 获取版本存储目录
    fn versions_dir(&self) -> PathBuf {
        self.default_install_dir().join("versions")
    }

    /// 获取 default 目录（指向当前版本的符号链接）
    ///
    /// 用法: 设置 ZIG_HOME=<zzm_root>/default 或将 <zzm_root>/default/bin 加入 PATH
    fn default_dir(&self) -> PathBuf {
        self.default_install_dir().join("default")
    }

    /// 获取缓存目录
    fn cache_dir(&self) -> PathBuf {
        self.default_install_dir().join("cache")
    }

    /// 获取配置文件路径
    fn config_file_path(&self) -> PathBuf {
        self.default_install_dir().join("config.toml")
    }

    /// 获取已安装版本索引文件路径
    fn installed_index_path(&self) -> PathBuf {
        self.default_install_dir().join("installed.json")
    }

    /// 初始化目录结构
    fn initialize_dirs(&self) -> Result<(), ZzmError> {
        let dirs = [
            self.default_install_dir(),
            self.bin_dir(),
            self.versions_dir().join("zig"),
            self.versions_dir().join("zls"),
            self.cache_dir(),
        ];

        for dir in dirs {
            if !dir.exists() {
                std::fs::create_dir_all(&dir).map_err(ZzmError::Io)?;
            }
        }

        Ok(())
    }

    /// 检查 bin 目录是否在 PATH 中
    fn is_bin_in_path(&self) -> bool {
        if let Ok(path_var) = std::env::var("PATH") {
            let bin_dir = self.bin_dir();
            let separator = if cfg!(windows) { ';' } else { ':' };
            path_var.split(separator).any(|p| {
                let p_path = Path::new(p.trim());
                if cfg!(windows) {
                    // Windows 路径比较不区分大小写，且忽略尾部反斜杠
                    let p_str = p_path
                        .to_string_lossy()
                        .trim_end_matches('\\')
                        .to_lowercase();
                    let bin_str = bin_dir
                        .to_string_lossy()
                        .trim_end_matches('\\')
                        .to_lowercase();
                    p_str == bin_str
                } else {
                    p_path == bin_dir
                }
            })
        } else {
            false
        }
    }

    /// 获取当前平台的二进制文件名（zig 或 zig.exe）
    fn zig_binary_name(&self) -> &'static str;

    /// 获取当前平台的 ZLS 二进制文件名
    fn zls_binary_name(&self) -> &'static str;

    /// 检查是否具有管理员/root 权限
    fn is_admin(&self) -> bool;
}

/// 运行时平台检测，返回当前平台的适配器
pub fn detect_platform() -> Box<dyn PlatformTrait> {
    if cfg!(target_os = "windows") {
        Box::new(super::windows::WindowsPlatform)
    } else if cfg!(target_os = "macos") {
        Box::new(super::macos::MacOSPlatform)
    } else {
        Box::new(super::linux::LinuxPlatform)
    }
}

/// 获取当前平台的目标三元组标识（用于下载匹配）
pub fn current_target_triple() -> &'static str {
    if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
        "x86_64-windows"
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        "aarch64-macos"
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
        "x86_64-macos"
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        "x86_64-linux"
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "aarch64") {
        "aarch64-linux"
    } else {
        "unknown"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_platform() {
        let platform = detect_platform();
        // 在 Windows 上应该返回 "windows"
        if cfg!(target_os = "windows") {
            assert_eq!(platform.name(), "windows");
        } else if cfg!(target_os = "macos") {
            assert_eq!(platform.name(), "macos");
        } else {
            assert_eq!(platform.name(), "linux");
        }
    }

    #[test]
    fn test_target_triple_not_unknown() {
        // 在支持的平台上，不应返回 "unknown"
        let triple = current_target_triple();
        assert_ne!(triple, "unknown", "当前平台不受支持");
    }
}
