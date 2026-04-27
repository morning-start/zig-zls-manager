#![cfg(target_os = "windows")]

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

    fn platform_default_dir(&self) -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from(r"C:\Users\Default"))
            .join(".zzm")
    }

    fn create_symlink(&self, original: &Path, link: &Path) -> Result<(), ZzmError> {
        // 如果目标是目录，使用目录符号链接；否则使用文件符号链接
        let is_dir = original.is_dir();
        let symlink_result = if is_dir {
            std::os::windows::fs::symlink_dir(original, link)
        } else {
            std::os::windows::fs::symlink_file(original, link)
        };

        // 创建符号链接失败时，根据目标类型回退
        if symlink_result.is_err() {
            if is_dir {
                // 目录链接无法用 shim 替代，创建 junction（目录联接）作为回退
                self.create_junction(original, link)?;
            } else {
                // 文件链接回退到 shim
                self.create_shim(original, link)?;
            }
        }
        Ok(())
    }

    fn remove_symlink(&self, link: &Path) -> Result<(), ZzmError> {
        if link.is_symlink() || link.exists() {
            // 符号链接和 junction 需要用不同的方式删除：
            // - 文件符号链接/shim: remove_file
            // - 目录符号链接/junction: remove_dir（不递归，只删除链接本身）
            if link.is_dir() {
                std::fs::remove_dir(link)?;
            } else {
                std::fs::remove_file(link)?;
            }
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
    /// 创建 Windows shim（批处理文件）作为文件符号链接的替代方案
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

    /// 创建 Windows junction（目录联接）作为目录符号链接的替代方案
    ///
    /// Junction 不需要管理员权限，可以跨驱动器链接本地目录，
    /// 是 Windows 上 `default` 目录链接的最佳回退方案。
    fn create_junction(&self, target: &Path, link: &Path) -> Result<(), ZzmError> {
        // 如果 link 已存在，先删除
        if link.exists() || link.is_symlink() {
            // junction 是目录类型，需要用 remove_dir 而不是 remove_file
            std::fs::remove_dir(link).map_err(|e| ZzmError::SymlinkFailed {
                from: link.to_string_lossy().to_string(),
                to: target.to_string_lossy().to_string(),
                reason: format!("删除已有 junction 失败: {e}"),
            })?;
        }

        // 使用 mklink /J 创建 junction
        let target_str = target.to_string_lossy();
        let link_str = link.to_string_lossy();
        let output = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J", &link_str, &target_str])
            .output()
            .map_err(|e| ZzmError::SymlinkFailed {
                from: link.to_string_lossy().to_string(),
                to: target.to_string_lossy().to_string(),
                reason: format!("执行 mklink 失败: {e}"),
            })?;

        if !output.status.success() {
            return Err(ZzmError::SymlinkFailed {
                from: link.to_string_lossy().to_string(),
                to: target.to_string_lossy().to_string(),
                reason: format!(
                    "mklink /J 失败: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }
        Ok(())
    }
}
