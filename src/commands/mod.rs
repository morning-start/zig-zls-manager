pub mod clean;
pub mod completion;
pub mod config;
pub mod ide;
pub mod info;
pub mod install;
pub mod list;
pub mod pair;
pub mod restore;
pub mod setup;
pub mod version_use;
pub mod zls;

use std::sync::OnceLock;

use crate::core::callbacks::InstallCallbacks;
use crate::core::tool_manager::{ToolKind, ToolManager};
use crate::infra::cache::CacheManager;
use crate::infra::path_manager::PathManager;
use crate::infra::zig_api::ZigApiClient;
use crate::infra::zls_api::ZlsApiClient;
use crate::platform::PlatformTrait;
use crate::utils::error::ZzmError;

/// 应用上下文，持有平台引用和各管理器的懒加载实例
///
/// 使用 `OnceLock` 实现 `PathManager` 单例复用，避免重复创建。
/// `ToolManager` 因 `InstallCallbacks` 差异按需创建，
/// 但 `reqwest::Client` 内部有连接池复用，实际性能影响可忽略。
pub struct AppContext {
    platform: Box<dyn PlatformTrait>,
    path_manager: OnceLock<PathManager>,
}

impl AppContext {
    /// 从平台适配器创建应用上下文
    pub fn new(platform: Box<dyn PlatformTrait>) -> Self {
        Self {
            platform,
            path_manager: OnceLock::new(),
        }
    }

    /// 创建 Zig 版本管理器
    pub fn zig_manager(
        &self,
        callbacks: InstallCallbacks,
    ) -> Result<ToolManager<ZigApiClient>, ZzmError> {
        let api_client = ZigApiClient::new(self.path_manager().cache_dir())?;
        ToolManager::new(
            ToolKind::Zig,
            self.platform.clone_box(),
            api_client,
            callbacks,
        )
    }

    /// 创建 ZLS 版本管理器
    pub fn zls_manager(
        &self,
        callbacks: InstallCallbacks,
    ) -> Result<ToolManager<ZlsApiClient>, ZzmError> {
        let api_client = ZlsApiClient::new(self.path_manager().cache_dir())?;
        ToolManager::new(
            ToolKind::Zls,
            self.platform.clone_box(),
            api_client,
            callbacks,
        )
    }

    /// 获取 `PathManager`（懒加载单例）
    ///
    /// 首次调用时创建，后续复用同一实例
    pub fn path_manager(&self) -> &PathManager {
        self.path_manager
            .get_or_init(|| PathManager::new(self.platform.clone_box()))
    }

    /// 创建 `CacheManager`
    pub fn cache_manager(&self) -> CacheManager {
        CacheManager::new(self.path_manager().cache_dir())
    }

    /// 获取平台引用
    pub fn platform(&self) -> &dyn PlatformTrait {
        self.platform.as_ref()
    }
}
