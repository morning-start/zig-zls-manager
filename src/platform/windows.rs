use std::path::{Path, PathBuf};

use super::trait_def::PlatformTrait;
use crate::utils::error::ZzmError;

/// Windows 平台适配器
pub struct WindowsPlatform;

impl PlatformTrait for WindowsPlatform {
    fn clone_box(&self) -> Box<dyn PlatformTrait> {
        Box::new(WindowsPlatform)
    }

    fn name(&self) -> &'static str {
        "windows"
    }

    fn default_install_dir(&self) -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from(r"C:\Users\Default"))
            .join("zzm")
    }

    fn create_symlink(&self, original: &Path, link: &Path) -> Result<(), ZzmError> {
        // Windows 上优先尝试创建符号链接，失败则回退到 shim 文件
        if std::os::windows::fs::symlink_file(original, link).is_err() {
            self.create_shim(original, link)?;
        }
        Ok(())
    }

    fn remove_symlink(&self, link: &Path) -> Result<(), ZzmError> {
        if link.exists() {
            std::fs::remove_file(link)?;
        }
        Ok(())
    }

    fn shell_config_files(&self) -> Vec<PathBuf> {
        // Windows 上主要通过注册表和 setx 管理 PATH
        vec![]
    }

    fn zig_binary_name(&self) -> &'static str {
        "zig.exe"
    }

    fn zls_binary_name(&self) -> &'static str {
        "zls.exe"
    }

    fn is_admin(&self) -> bool {
        // 尝试写入受保护目录来检测管理员权限
        std::fs::write(r"C:\Windows\Temp\.zzm_admin_test", b"test").is_ok()
            && std::fs::remove_file(r"C:\Windows\Temp\.zzm_admin_test").is_ok()
    }
}

impl WindowsPlatform {
    /// 创建 Windows shim（批处理文件）作为符号链接的替代方案
    fn create_shim(&self, target: &Path, shim_path: &Path) -> Result<(), ZzmError> {
        let target_str = target.to_string_lossy();
        let shim_code = format!(
            "@echo off\r\n\"{}\" %*\r\n",
            target_str.replace('\\', "\\\\")
        );
        std::fs::write(shim_path, shim_code).map_err(|e| ZzmError::SymlinkFailed {
            from: shim_path.to_string_lossy().to_string(),
            to: target.to_string_lossy().to_string(),
            reason: e.to_string(),
        })
    }
}