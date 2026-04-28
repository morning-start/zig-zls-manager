pub mod clean;
pub mod completion;
pub mod config;
pub mod ide;
pub mod info;
pub mod install;
pub mod list;
pub mod pair;
pub mod prune;
pub mod restore;
pub mod setup;
pub mod version_use;
pub mod zls;

use std::sync::OnceLock;

use crate::cli::{Commands, ConfigCommands, IdeCommands, ZlsCommands};
use crate::core::callbacks::InstallCallbacks;
use crate::core::tool_manager::{ToolKind, ToolManager};
use crate::infra::cache::CacheManager;
use crate::infra::path_manager::PathManager;
use crate::infra::zig_api::ZigApiClient;
use crate::infra::zls_api::ZlsApiClient;
use crate::platform::PlatformTrait;
use crate::utils::error::ZzmError;

/// Command trait for all CLI commands
#[async_trait::async_trait]
pub trait Command {
    async fn execute(&self, ctx: &AppContext, json: bool) -> Result<(), ZzmError>;
}

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

// Implement Command trait for Commands enum
#[async_trait::async_trait]
impl Command for Commands {
    async fn execute(&self, ctx: &AppContext, json: bool) -> Result<(), ZzmError> {
        match self {
            Commands::Install {
                version,
                with_zls,
                from_source: _,
                yes: _,
                force,
            } => install::cmd_install(ctx, version, *with_zls, *force, json).await,
            Commands::Uninstall { version, purge: _ } => install::cmd_uninstall(ctx, version, json).await,
            Commands::List {
                installed,
                remote,
                json: list_json,
            } => list::cmd_list(ctx, *installed, *remote, *list_json || json).await,
            Commands::Use {
                version,
                global: _,
                project: _,
                default: _,
                zls,
            } => version_use::cmd_use(ctx, version, zls.clone(), json).await,
            Commands::Current { json: current_json } => list::cmd_current(ctx, *current_json || json).await,
            Commands::Zls { command } => command.execute(ctx, json).await,
            Commands::Setup {
                version,
                with_zls,
                ide: _,
                wizard,
            } => setup::cmd_setup(ctx, version.clone(), *with_zls, *wizard, json).await,
            Commands::Sync { dry_run } => setup::cmd_sync(ctx, *dry_run, json).await,
            Commands::Pair {
                zig_version,
                zls,
                compatibility,
                show,
            } => {
                if *show {
                    pair::cmd_pair_show(ctx, json).await
                } else {
                    pair::cmd_pair(ctx, zig_version, zls.as_deref(), compatibility.as_deref(), json).await
                }
            }
            Commands::Restore { dir } => restore::cmd_restore(ctx, dir.clone(), json).await,
            Commands::Info { verbose } => info::cmd_info(ctx, *verbose).await,
            Commands::Config { command } => command.execute(ctx, json).await,
            Commands::Ide { command } => command.execute(ctx, json).await,
            Commands::Prune {
                dry_run,
                confirm: _,
            } => prune::cmd_prune(ctx, *dry_run, json).await,
            Commands::Clean { all, dry_run } => clean::cmd_clean(ctx, *all, *dry_run).await,
            Commands::Doctor => info::cmd_doctor(ctx).await,
            Commands::Completion { shell } => {
                completion::cmd_completion(shell)
            }
        }
    }
}

// Implement Command trait for ZlsCommands
#[async_trait::async_trait]
impl Command for ZlsCommands {
    async fn execute(&self, ctx: &AppContext, json: bool) -> Result<(), ZzmError> {
        zls::cmd_zls(ctx, self.clone(), json).await
    }
}

// Implement Command trait for ConfigCommands
#[async_trait::async_trait]
impl Command for ConfigCommands {
    async fn execute(&self, ctx: &AppContext, _json: bool) -> Result<(), ZzmError> {
        config::cmd_config(ctx, self.clone()).await
    }
}

// Implement Command trait for IdeCommands
#[async_trait::async_trait]
impl Command for IdeCommands {
    async fn execute(&self, ctx: &AppContext, _json: bool) -> Result<(), ZzmError> {
        ide::cmd_ide(ctx, self.clone()).await
    }
}
