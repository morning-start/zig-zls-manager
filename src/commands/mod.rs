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

use crate::core::tool_manager::{ToolKind, ToolManager};
use crate::infra::cache::CacheManager;
use crate::infra::path_manager::PathManager;
use crate::infra::zig_api::ZigApiClient;
use crate::infra::zls_api::ZlsApiClient;
use crate::platform::PlatformTrait;
use crate::utils::error::ZzmError;

/// 应用上下文，持有平台引用和各管理器的创建方法
///
/// 使用 `ToolManager` 泛型抽象统一 Zig 和 ZLS 的管理器创建。
/// 每次调用创建新实例，但 `reqwest::Client` 内部有连接池复用，
/// 实际性能影响可忽略。
pub struct AppContext {
    platform: Box<dyn PlatformTrait>,
}

impl AppContext {
    /// 从平台适配器创建应用上下文
    pub fn new(platform: Box<dyn PlatformTrait>) -> Self {
        Self { platform }
    }

    /// 创建 Zig 版本管理器
    pub fn zig_manager(&self) -> Result<ToolManager<ZigApiClient>, ZzmError> {
        let api_client = ZigApiClient::new(self.path_manager().cache_dir())?;
        ToolManager::new(ToolKind::Zig, self.platform.clone_box(), api_client)
    }

    /// 创建 ZLS 版本管理器
    pub fn zls_manager(&self) -> Result<ToolManager<ZlsApiClient>, ZzmError> {
        let api_client = ZlsApiClient::new(self.path_manager().cache_dir())?;
        ToolManager::new(ToolKind::Zls, self.platform.clone_box(), api_client)
    }

    /// 创建 `PathManager`
    pub fn path_manager(&self) -> PathManager {
        PathManager::new(self.platform.clone_box())
    }

    /// 创建 `CacheManager`
    pub fn cache_manager(&self) -> CacheManager {
        let path_mgr = self.path_manager();
        CacheManager::new(path_mgr.cache_dir())
    }

    /// 获取平台引用
    pub fn platform(&self) -> &dyn PlatformTrait {
        self.platform.as_ref()
    }
}
