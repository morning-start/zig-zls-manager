use serde::Serialize;

use crate::core::tool_manager::{DownloadAsset, ToolKind, VersionInfo};
use crate::infra::path_manager::ToolIndexEntry;
use crate::output::dispatcher::OutputRow;

/// 远程版本输出行
///
/// 统一 Zig/ZLS 远程版本的输出格式
#[derive(Debug, Clone, Serialize)]
pub struct RemoteVersionOutput {
    /// 版本号
    pub version: String,
    /// 通道
    pub channel: String,
    /// 发布日期
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// 下载资源信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<DownloadAsset>,
    /// 是否已安装标记
    #[serde(skip_serializing_if = "str::is_empty")]
    pub installed: String,
}

impl From<&VersionInfo> for RemoteVersionOutput {
    fn from(v: &VersionInfo) -> Self {
        Self {
            version: v.version.clone(),
            channel: v.channel.to_string(),
            date: v.date.clone(),
            asset: v.asset.clone(),
            installed: String::new(),
        }
    }
}

impl OutputRow for RemoteVersionOutput {
    fn to_table_row(&self) -> Vec<String> {
        vec![
            self.version.clone(),
            self.channel.clone(),
            self.date.clone().unwrap_or_default(),
            self.asset
                .as_ref()
                .map(|a| a.size.clone())
                .unwrap_or_default(),
            self.installed.clone(),
        ]
    }

    fn table_headers() -> Vec<&'static str> {
        vec!["版本", "通道", "日期", "大小", "已安装"]
    }
}

/// 已安装版本输出行
///
/// 统一 Zig/ZLS 已安装版本的输出格式
#[derive(Debug, Clone, Serialize)]
pub struct InstalledVersionOutput {
    /// 版本号
    pub version: String,
    /// 通道（Zig）或 Zig 版本（ZLS）
    pub channel_or_zig_ver: String,
    /// 安装路径
    pub path: String,
    /// 状态（当前激活标记）
    pub status: String,
}

impl OutputRow for InstalledVersionOutput {
    fn to_table_row(&self) -> Vec<String> {
        vec![
            self.version.clone(),
            self.channel_or_zig_ver.clone(),
            self.path.clone(),
            self.status.clone(),
        ]
    }

    fn table_headers() -> Vec<&'static str> {
        vec!["版本", "通道", "安装路径", "状态"]
    }
}

/// 从 ToolIndexEntry 列表构建已安装版本输出行
pub fn build_installed_rows(
    entries: &[ToolIndexEntry],
    kind: ToolKind,
    active_version: Option<&str>,
) -> Vec<InstalledVersionOutput> {
    entries
        .iter()
        .map(|v| {
            let is_active = active_version == Some(v.version.as_str());
            let channel_or_zig_ver = match kind {
                ToolKind::Zig => v.channel().map(|c| c.to_string()).unwrap_or_default(),
                ToolKind::Zls => v.zig_version().unwrap_or_default().to_string(),
            };
            InstalledVersionOutput {
                version: v.version.clone(),
                channel_or_zig_ver,
                path: v.install_path.to_string_lossy().to_string(),
                status: if is_active {
                    "=> 当前".to_string()
                } else {
                    String::new()
                },
            }
        })
        .collect()
}

/// 渲染通用键值对表格
pub fn render_kv_table(title: &str, items: &[(&str, String)]) {
    println!("{title}");
    for (key, value) in items {
        println!("  {key}: {value}");
    }
}
