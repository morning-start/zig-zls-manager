use std::path::{Path, PathBuf};

use super::trait_def::PlatformTrait;
use crate::utils::error::ZzmError;

/// macOS 平台适配器
pub struct MacOSPlatform;

#[cfg(target_os = "macos")]
impl PlatformTrait for MacOSPlatform {
    fn name(&self) -> &'static str {
        "macos"
    }

    fn default_install_dir(&self) -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/"))
            .join(".zzm")
    }

    fn create_symlink(&self, original: &Path, link: &Path) -> Result<(), ZzmError> {
        if link.exists() || link.is_symlink() {
            std::fs::remove_file(link)?;
        }
        std::os::unix::fs::symlink(original, link).map_err(|e: std::io::Error| ZzmError::SymlinkFailed {
            from: link.to_string_lossy().to_string(),
            to: original.to_string_lossy().to_string(),
            reason: e.to_string(),
        })
    }

    fn remove_symlink(&self, link: &Path) -> Result<(), ZzmError> {
        if link.exists() || link.is_symlink() {
            std::fs::remove_file(link)?;
        }
        Ok(())
    }

    fn shell_config_files(&self) -> Vec<PathBuf> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        vec![
            home.join(".zshrc"),
            home.join(".bash_profile"),
            home.join(".bashrc"),
            home.join(".profile"),
        ]
    }

    fn zig_binary_name(&self) -> &'static str {
        "zig"
    }

    fn zls_binary_name(&self) -> &'static str {
        "zls"
    }

    fn is_admin(&self) -> bool {
        std::env::var("USER").map(|u| u == "root").unwrap_or(false)
    }
}

#[cfg(not(target_os = "macos"))]
impl PlatformTrait for MacOSPlatform {
    fn name(&self) -> &'static str { "macos" }
    fn default_install_dir(&self) -> PathBuf { PathBuf::from("/.zzm") }
    fn create_symlink(&self, _original: &Path, _link: &Path) -> Result<(), ZzmError> {
        Err(ZzmError::UnsupportedPlatform { platform: "macos".to_string() })
    }
    fn remove_symlink(&self, _link: &Path) -> Result<(), ZzmError> { Ok(()) }
    fn shell_config_files(&self) -> Vec<PathBuf> { vec![] }
    fn zig_binary_name(&self) -> &'static str { "zig" }
    fn zls_binary_name(&self) -> &'static str { "zls" }
    fn is_admin(&self) -> bool { false }
}