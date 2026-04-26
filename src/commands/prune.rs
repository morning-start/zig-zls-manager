use crate::commands::AppContext;
use crate::core::callbacks::InstallCallbacks;
use crate::core::tool_manager::ToolKind;
use crate::output::console_output;
use crate::output::dispatcher::{OutputRow, output_list};
use crate::utils::error::ZzmError;
use serde::Serialize;

/// 可清理版本信息（用于表格和 JSON 输出）
#[derive(Debug, Clone, Serialize)]
struct PrunableVersion {
    /// 工具类型
    tool: String,
    /// 版本号
    version: String,
    /// 是否为当前激活版本
    active: bool,
}

impl OutputRow for PrunableVersion {
    fn to_table_row(&self) -> Vec<String> {
        vec![
            self.tool.clone(),
            self.version.clone(),
            if self.active {
                "是".to_string()
            } else {
                "否".to_string()
            },
        ]
    }

    fn table_headers() -> Vec<&'static str> {
        vec!["工具", "版本", "当前激活"]
    }
}

/// 列出可清理的版本
///
/// 返回所有非激活版本的列表，按工具类型分组
fn list_prunable_versions(ctx: &AppContext) -> Result<Vec<PrunableVersion>, ZzmError> {
    let path_mgr = ctx.path_manager();
    let index = path_mgr.read_installed_index()?;

    let mut prunable = Vec::new();

    for kind in [ToolKind::Zig, ToolKind::Zls] {
        let active = index.get_active(kind);
        let versions = index.get_versions(kind);

        for entry in versions {
            let is_active = active == Some(&entry.version);
            prunable.push(PrunableVersion {
                tool: match kind {
                    ToolKind::Zig => "Zig".to_string(),
                    ToolKind::Zls => "ZLS".to_string(),
                },
                version: entry.version.clone(),
                active: is_active,
            });
        }
    }

    Ok(prunable)
}

/// 批量卸载指定版本
async fn batch_uninstall(
    ctx: &AppContext,
    versions: &[(ToolKind, String)],
    json: bool,
) -> Result<PruneSummary, ZzmError> {
    let mut summary = PruneSummary::default();

    for (kind, version) in versions {
        // 卸载前估算大小（卸载后目录已删除）
        let size = estimate_version_size(ctx, *kind, version);

        // 每次循环创建新的 callbacks（InstallCallbacks 含 Box<dyn Fn> 不可 Clone）
        let callbacks = if json {
            InstallCallbacks::silent()
        } else {
            InstallCallbacks::console()
        };

        let result = match kind {
            ToolKind::Zig => {
                let mgr = ctx.zig_manager(callbacks)?;
                mgr.uninstall(version)
            }
            ToolKind::Zls => {
                let mgr = ctx.zls_manager(callbacks)?;
                mgr.uninstall(version)
            }
        };

        match result {
            Ok(()) => {
                summary.removed += 1;
                summary.freed_bytes += size;
            }
            Err(e) => {
                summary.failed += 1;
                summary.errors.push(format!(
                    "{} {}: {e}",
                    match kind {
                        ToolKind::Zig => "Zig",
                        ToolKind::Zls => "ZLS",
                    },
                    version
                ));
            }
        }
    }

    Ok(summary)
}

/// 估算版本目录大小（粗略）
fn estimate_version_size(ctx: &AppContext, kind: ToolKind, version: &str) -> u64 {
    let path_mgr = ctx.path_manager();
    let version_dir = match kind {
        ToolKind::Zig => path_mgr.zig_version_dir(version),
        ToolKind::Zls => path_mgr.zls_version_dir(version),
    };

    dir_size(&version_dir)
}

/// 递归计算目录大小
fn dir_size(path: &std::path::Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total += dir_size(&p);
            } else if let Ok(meta) = p.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

/// 清理摘要
#[derive(Default)]
struct PruneSummary {
    removed: usize,
    failed: usize,
    freed_bytes: u64,
    errors: Vec<String>,
}

/// 执行 prune 命令
pub async fn cmd_prune(ctx: &AppContext, dry_run: bool, json: bool) -> Result<(), ZzmError> {
    let prunable = list_prunable_versions(ctx)?;

    if prunable.is_empty() {
        output_list::<PrunableVersion>(&[], json, Some("没有已安装的版本"));
        return Ok(());
    }

    // 分离非激活版本（可清理）和激活版本（不可清理）
    let (prunable_inactive, _active): (Vec<_>, Vec<_>) =
        prunable.into_iter().partition(|v| !v.active);

    if prunable_inactive.is_empty() {
        output_list::<PrunableVersion>(&[], json, Some("没有可清理的版本（仅剩当前激活版本）"));
        return Ok(());
    }

    // 显示可清理版本
    output_list(&prunable_inactive, json, None);

    if dry_run {
        console_output::print_info(&format!(
            "共 {} 个版本可清理（使用 --confirm 执行清理）",
            prunable_inactive.len()
        ));
        return Ok(());
    }

    // 交互确认
    if !json {
        console_output::print_warning(&format!(
            "即将卸载 {} 个非激活版本",
            prunable_inactive.len()
        ));
        let confirm = dialoguer::Confirm::new()
            .with_prompt("确认清理？")
            .default(false)
            .interact()
            .map_err(|e| ZzmError::ConfigError {
                path: "prune".to_string(),
                reason: format!("交互确认失败: {e}"),
            })?;

        if !confirm {
            console_output::print_info("已取消清理");
            return Ok(());
        }
    }

    // 收集要卸载的版本
    let to_remove: Vec<(ToolKind, String)> = prunable_inactive
        .iter()
        .map(|v| {
            let kind = if v.tool == "Zig" {
                ToolKind::Zig
            } else {
                ToolKind::Zls
            };
            (kind, v.version.clone())
        })
        .collect();

    // 批量卸载
    let summary = batch_uninstall(ctx, &to_remove, json).await?;

    // 输出结果
    if summary.failed > 0 {
        console_output::print_warning(&format!(
            "清理完成: 移除 {} 个版本，{} 个失败，释放 {}",
            summary.removed,
            summary.failed,
            crate::utils::format::format_size(summary.freed_bytes)
        ));
        for err in &summary.errors {
            console_output::print_error(err);
        }
    } else {
        console_output::print_success(&format!(
            "清理完成: 移除 {} 个版本，释放 {}",
            summary.removed,
            crate::utils::format::format_size(summary.freed_bytes)
        ));
    }

    Ok(())
}
