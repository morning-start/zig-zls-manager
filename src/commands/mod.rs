pub mod clean;
pub mod completion;
pub mod config;
pub mod ide;
pub mod info;
pub mod install;
pub mod list;
pub mod setup;
pub mod version_use;
pub mod zls;

use crate::core::zig_manager::ZigManager;
use crate::core::zls_manager::ZlsManager;
use crate::infra::cache::CacheManager;
use crate::infra::path_manager::PathManager;
use crate::platform::PlatformTrait;
use crate::utils::error::ZzmError;

/// 应用上下文，持有平台引用和各管理器的懒加载实例
///
/// 解决之前每个命令处理函数中重复 `platform.clone_box()` 的问题，
/// 统一管理所有管理器的创建和复用。
pub struct AppContext {
    platform: Box<dyn PlatformTrait>,
}

impl AppContext {
    /// 从平台适配器创建应用上下文
    pub fn new(platform: Box<dyn PlatformTrait>) -> Self {
        Self { platform }
    }

    /// 创建 ZigManager
    pub fn zig_manager(&self) -> Result<ZigManager, ZzmError> {
        ZigManager::new(self.platform.clone_box())
    }

    /// 创建 ZlsManager
    pub fn zls_manager(&self) -> Result<ZlsManager, ZzmError> {
        ZlsManager::new(self.platform.clone_box())
    }

    /// 创建 PathManager
    pub fn path_manager(&self) -> PathManager {
        PathManager::new(self.platform.clone_box())
    }

    /// 创建 CacheManager
    pub fn cache_manager(&self) -> CacheManager {
        let path_mgr = self.path_manager();
        CacheManager::new(path_mgr.cache_dir())
    }

    /// 获取平台引用
    pub fn platform(&self) -> &dyn PlatformTrait {
        self.platform.as_ref()
    }
}
