use tabled::{Table, Tabled, settings::Style};

/// 版本列表行数据
#[derive(Tabled)]
#[allow(dead_code)] // 预留: 版本列表表格渲染
pub struct VersionRow {
    #[tabled(rename = "版本")]
    pub version: String,
    #[tabled(rename = "通道")]
    pub channel: String,
    #[tabled(rename = "状态")]
    pub status: String,
}

/// 已安装版本行数据
#[derive(Tabled)]
pub struct InstalledVersionRow {
    #[tabled(rename = "版本")]
    pub version: String,
    #[tabled(rename = "通道")]
    pub channel: String,
    #[tabled(rename = "安装路径")]
    pub path: String,
    #[tabled(rename = "状态")]
    pub status: String,
}

/// 远程版本行数据
#[derive(Tabled)]
pub struct RemoteVersionRow {
    #[tabled(rename = "版本")]
    pub version: String,
    #[tabled(rename = "通道")]
    pub channel: String,
    #[tabled(rename = "日期")]
    pub date: String,
    #[tabled(rename = "大小")]
    pub size: String,
    #[tabled(rename = "已安装")]
    pub installed: String,
}

/// 渲染版本列表表格
#[allow(dead_code)] // 预留: 版本列表表格渲染
pub fn render_version_table(rows: &[VersionRow]) {
    let mut table = Table::new(rows);
    table.with(Style::rounded());
    println!("{table}");
}

/// 渲染已安装版本表格
pub fn render_installed_table(rows: &[InstalledVersionRow]) {
    let mut table = Table::new(rows);
    table.with(Style::rounded());
    println!("{table}");
}

/// 渲染远程版本表格
pub fn render_remote_table(rows: &[RemoteVersionRow]) {
    let mut table = Table::new(rows);
    table.with(Style::rounded());
    println!("{table}");
}

/// 渲染通用键值对表格
pub fn render_kv_table(title: &str, items: &[(&str, String)]) {
    println!("{title}");
    for (key, value) in items {
        println!("  {key}: {value}");
    }
}
