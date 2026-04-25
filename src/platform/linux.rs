use std::path::{Path, PathBuf};

use super::trait_def::PlatformTrait;
use crate::utils::error::ZzmError;

/// Linux 平台适配器
pub struct LinuxPlatform;

#[cfg(target_os = "linux")]
impl PlatformTrait for LinuxPlatform {
    fn clone_box(&self) -> Box<dyn PlatformTrait> {
        Box::new(LinuxPlatform)
    }

    fn name(&self) -> &'static str {
        "linux"
    }

    fn default_install_dir(&self) -> PathBuf {
        if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
            PathBuf::from(xdg_data).join("zzm")
        } else {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/"))
                .join(".zzm")
        }
    }

    fn create_symlink(&self, original: &Path, link: &Path) -> Result<(), ZzmError> {
        if link.exists() || link.is_symlink() {
            std::fs::remove_file(link)?;
        }
        std::os::unix::fs::symlink(original, link).map_err(|e: std::io::Error| {
            ZzmError::SymlinkFailed {
                from: link.to_string_lossy().to_string(),
                to: original.to_string_lossy().to_string(),
                reason: e.to_string(),
            }
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
            home.join(".bashrc"),
            home.join(".zshrc"),
            home.join(".profile"),
            home.join(".bash_profile"),
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

#[cfg(not(target_os = "linux"))]
impl PlatformTrait for LinuxPlatform {
    fn clone_box(&self) -> Box<dyn PlatformTrait> {
        Box::new(LinuxPlatform)
    }

    fn name(&self) -> &'static str {
        "linux"
    }
    fn default_install_dir(&self) -> PathBuf {
        PathBuf::from("/.zzm")
    }
    fn create_symlink(&self, _original: &Path, _link: &Path) -> Result<(), ZzmError> {
        Err(ZzmError::UnsupportedPlatform {
            platform: "linux".to_string(),
        })
    }
    fn remove_symlink(&self, _link: &Path) -> Result<(), ZzmError> {
        Ok(())
    }
    fn shell_config_files(&self) -> Vec<PathBuf> {
        vec![]
    }
    fn zig_binary_name(&self) -> &'static str {
        "zig"
    }
    fn zls_binary_name(&self) -> &'static str {
        "zls"
    }
    fn is_admin(&self) -> bool {
        false
    }
}
