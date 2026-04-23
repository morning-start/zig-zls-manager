# Zig/ZLS 版本管理器 - 技术架构设计文档

## 📋 文档信息

- **版本**: v1.0.0
- **创建日期**: 2026-04-23
- **状态**: 初稿
- **关联文档**: [spec.md](./spec.md) (需求规格说明书)

---

## 1. 架构总览

### 1.1 设计理念

**核心原则**：
- 🎯 **用户优先**: 简洁直观的 CLI 体验，5 分钟上手
- 🔧 **模块化**: 高内聚低耦合，易于扩展和维护
- ⚡ **高性能**: Rust 实现，启动快速，内存安全
- 🔒 **可靠性**: 完善的错误处理和兼容性检查
- 🌍 **跨平台**: Windows/macOS/Linux 统一体验

### 1.2 系统分层架构

```
┌─────────────────────────────────────────────────────────────┐
│                     用户接口层 (CLI)                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │ 命令解析  │  │ 交互式向导│  │ 自动补全  │  │ 输出格式  │    │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘    │
├───────┴─────────────┴─────────────┴─────────────┴───────────┤
│                     业务逻辑层 (Core)                        │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │ Zig 管理  │  │ ZLS 管理  │  │ 联合管理  │  │ IDE 集成  │    │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘    │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │ 版本解析  │  │ 兼容性   │  │ 配置管理  │  │ 项目管理  │    │
│  │ 引擎     │  │ 检查器   │  │ 器      │  │ 器       │    │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘    │
├───────┴─────────────┴─────────────┴─────────────┴───────────┤
│                     基础设施层 (Infrastructure)              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │ 下载管理  │  │ 文件系统  │  │ 路径管理  │  │ 进程管理  │    │
│  │ 器       │  │ 操作     │  │ 器       │  │ 器       │    │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘    │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                  │
│  │ 校验和验证│  │ 缓存管理  │  │ 日志系统  │                  │
│  └──────────┘  └──────────┘  └──────────┘                  │
├─────────────────────────────────────────────────────────────┤
│                     平台抽象层 (Platform)                    │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                  │
│  │ Windows  │  │ macOS    │  │ Linux    │                  │
│  │ 适配器   │  │ 适配器   │  │ 适配器   │                  │
│  └──────────┘  └──────────┘  └──────────┘                  │
└─────────────────────────────────────────────────────────────┘
```

### 1.3 核心数据流

```
用户输入命令
     ↓
[CLI 解析层] → 参数验证 + 子命令路由
     ↓
[业务逻辑层] → 执行具体操作
     ├── Zig 安装/切换/查询
     ├── ZLS 安装/切换/查询
     ├── 兼容性检查
     └── IDE 配置生成
     ↓
[基础设施层] → 底层操作
     ├── HTTP 下载（带进度条）
     ├── 文件解压（tar.gz / zip）
     ├── 符号链接创建（POSIX）或 shim 可执行文件（Windows）
     ├── SHA256 校验
     └── PATH 环境变量更新
     ↓
[结果输出] → 格式化输出（人类可读 / JSON）
```

---

## 2. 模块详细设计

### 2.1 CLI 层模块

#### 2.1.1 命令行解析器 (`cli`)

**职责**：定义、解析和验证所有 CLI 命令和参数

**技术选型**：`clap` v4 (derive mode)

**命令树结构**：

```
zzm <command> [subcommand] [args] [options]

Commands:
├── install <version>          # 安装 Zig
│   ├── --with-zls             # 同时安装匹配的 ZLS
│   ├── --from-source          # 从源码编译
│   ├── --yes, -y              # 非交互模式
│   └── --force, -f            # 强制重装
│
├── uninstall <version>        # 卸载版本
│   └── --purge                # 清除配置和数据
│
├── list                       # 列出版本
│   ├── --installed            # 仅已安装
│   ├── --remote               # 远端可用版本
│   └── --json                 # JSON 格式输出
│
├── use <version>              # 切换版本
│   ├── --global, -g           # 全局切换（默认）
│   ├── --project, -p          # 项目级切换
│   ├── --default              # 设为默认版本
│   └── --zls <version>        # 同时指定 ZLS 版本
│
├── current                    # 显示当前版本
│   └── --json                 # JSON 格式
│
├── zls                        # ZLS 子命令组
│   ├── install <version>
│   ├── uninstall <version>
│   ├── list
│   ├── use <version>
│   └── current
│
├── setup <version>            # 一键初始化环境
│   ├── --with-zls
│   ├── --ide <editor>
│   └── --wizard               # 交互式向导
│
├── sync                       # 同步到推荐组合
│   └── --dry-run              # 预览将要执行的操作
│
├── info                       # 显示详细信息
│   └── --verbose, -v          # 详细模式
│
├── config                     # 配置管理
│   ├── list
│   ├── get <key>
│   ├── set <key> <value>
│   └── edit
│
├── ide                        # IDE 集成
│   ├── config <editor>
│   ├── check
│   ├── doctor
│   └── path
│
├── clean                      # 清理工具
│   ├── --all                 # 清理所有缓存
│   └── --dry-run             # 预览
│
├── doctor                     # 诊断程序
│
├── completion <shell>         # 生成补全脚本
│
└── [--help] [-h] [--version] [-V]
    └── [--no-color]           # 禁用彩色输出
        [--verbose]            # 详细日志
        [--json]               # JSON 输出模式
```

**代码结构示例**：

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "zzm")]
#[command(about = "Zig/ZLS Version Manager", long_about = None)]
#[command(version)]
struct Cli {
    /// 禁用彩色输出
    #[arg(long, global = true)]
    no_color: bool,

    /// 详细输出模式
    #[arg(short, long, global = true)]
    verbose: bool,

    /// 以 JSON 格式输出
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 安装指定版本的 Zig
    Install {
        /// 版本号 (如 0.13.0, master, stable)
        version: String,

        /// 同时安装匹配的 ZLS
        #[arg(long)]
        with_zls: bool,

        /// 从源码编译
        #[arg(long)]
        from_source: bool,

        /// 非交互模式
        #[arg(short, long)]
        yes: bool,
    },

    // ... 其他子命令
}
```

#### 2.1.2 交互式向导 (`wizard`)

**职责**：首次使用引导，交互式配置

**场景**：
1. `zzm setup --wizard`: 引导新用户完成初始设置
2. 检测到未配置时自动触发

**实现方式**：使用 `dialoguer` crate

```rust
use dialoguer::{Select, Confirm, Input};

pub fn run_setup_wizard() -> Result<()> {
    println!("🚀 欢迎使用 Zig/ZLS 版本管理器！\n");

    let default_version = Select::new()
        .with_prompt("选择要安装的默认 Zig 版本")
        .items(&["0.13.0 (推荐)", "0.12.0", "master (nightly)"])
        .default(0)
        .interact()?;

    let install_zls = Confirm::new()
        .with_prompt("是否同时安装 ZLS 语言服务器？")
        .default(true)
        .interact()?;

    let editor = Select::new()
        .with_prompt("选择要配置的编辑器")
        .items(&["VS Code", "Neovim", "Helix", "跳过"])
        .default(0)
        .interact()?;

    // 执行安装...

    println!("\n✅ 初始化完成！运行 'zzm info' 查看当前状态。");
    Ok(())
}
```

#### 2.1.3 输出格式化 (`output`)

**职责**：统一管理终端输出，支持多种格式

**功能**：
- 彩色文本输出（使用 `console` 或 `colored` crate）
- 表格展示（使用 `comfy-table` 或 `tabled` crate）
- 进度条显示（使用 `indicatif` crate）
- JSON 输出模式（使用 `serde_json`）

**示例**：

```rust
use indicatif::{ProgressBar, ProgressStyle};
use console::style;

pub fn print_install_progress(name: &str, total_size: u64) {
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
    );
    pb.set_message(format!("正在下载 {}", name));
    // 更新进度...
}

pub fn print_success(msg: &str) {
    println!("{} {}", style("✅").green(), msg);
}

pub fn print_warning(msg: &str) {
    println!("{} {}", style("⚠️").yellow(), msg);
}

pub fn print_error(msg: &str) {
    eprintln!("{} {}", style("❌").red(), msg);
}
```

---

### 2.2 业务逻辑层模块

#### 2.2.1 Zig 版本管理器 (`zig_manager`)

**职责**：管理 Zig 编译器的完整生命周期

**核心数据结构**：

```rust
use serde::{Deserialize, Serialize};

/// Zig 版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigVersion {
    pub version: String,           // "0.13.0"
    pub channel: Channel,         // Stable | Nightly | Maintenance
    pub release_date: Option<String>,
    pub download_url: String,
    pub checksum_sha256: String,
    pub size: u64,
}

/// 版本通道
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Channel {
    Stable,
    Nightly,  // master
    Maintenance,
}

/// 已安装的 Zig 实例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigInstallation {
    pub version: ZigVersion,
    pub install_path: PathBuf,
    pub installed_at: DateTime<Utc>,
    pub is_default: bool,
    pub is_active: bool,
}
```

**核心方法**：

```rust
impl ZigManager {
    /// 安装指定版本
    pub async fn install(&self, version: &str, options: InstallOptions) -> Result<()>;

    /// 卸载版本
    pub async fn uninstall(&self, version: &str, purge: bool) -> Result<()>;

    /// 列出已安装的版本
    pub fn list_installed(&self) -> Result<Vec<ZigInstallation>>;

    /// 列出远程可用版本
    pub async fn list_remote(&self) -> Result<Vec<ZigVersion>>;

    /// 切换活动版本
    pub async fn use_version(&self, version: &str, scope: Scope) -> Result<()>;

    /// 获取当前活动版本
    pub fn current(&self) -> Result<Option<ZigInstallation>>;

    /// 设置默认版本
    pub fn set_default(&self, version: &str) -> Result<()>;

    /// 解析版本字符串（支持简写）
    pub fn resolve_version(&self, input: &str) -> Result<String>;
}
```

#### 2.2.2 ZLS 版本管理器 (`zls_manager`)

**职责**：独立管理 ZLS 的生命周期，支持与 Zig 的关联

**核心数据结构**：

```rust
/// ZLS 版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZlsVersion {
    pub version: String,           // "0.13.0"
    pub compatible_zig: String,    // 匹配的 Zig 版本
    pub download_url: Option<String>,  // 预编译二进制 URL
    pub source_repo: String,       // GitHub 仓库地址
    pub build_requirements: BuildRequirements,
}

/// 构建需求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildRequirements {
    pub min_zig_version: String,
    pub build_command: String,
    pub output_path: String,
}

/// 已安装的 ZLS 实例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZlsInstallation {
    pub version: ZlsVersion,
    pub install_path: PathBuf,
    pub zig_version: String,       // 用于构建此 ZLS 的 Zig 版本
    pub install_mode: ZlsInstallMode,  // Prebuilt | Source
    pub installed_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZlsInstallMode {
    Prebuilt,
    Source { zig_version_used: String },
}
```

**核心方法**：

```rust
impl ZlsManager {
    /// 安装 ZLS
    pub async fn install(&self, version: &str, options: ZlsInstallOptions) -> Result<()>;

    /// 从源码编译安装 ZLS
    pub async fn install_from_source(&self, zig_version: &str, options: SourceBuildOptions) -> Result<()>;

    /// 卸载 ZLS
    pub async fn uninstall(&self, version: &str) -> Result<()>;

    /// 切换活动版本
    pub async fn use_version(&self, version: &str, scope: Scope) -> Result<()>;

    /// 根据 Zig 版本获取推荐的 ZLS 版本
    pub fn get_recommended_for_zig(&self, zig_version: &str) -> Result<Option<String>>;

    /// 检查与指定 Zig 版本的兼容性
    pub fn check_compatibility(&self, zls_ver: &str, zig_ver: &str) -> CompatibilityStatus;
}
```

#### 2.2.3 兼容性检查器 (`compatibility`)

**职责**：维护和管理 Zig ↔ ZLS 版本兼容性矩阵

**数据结构**：

```rust
/// 兼容性状态
#[derive(Debug, Clone, PartialEq)]
pub enum CompatibilityStatus {
    /// 完全兼容（官方推荐组合）
    FullyCompatible,
    /// 可能兼容（未经充分测试，但通常可用）
    PossiblyCompatible,
    /// 不兼容（已知问题）
    Incompatible(String),  // 包含问题描述
    /// 未知（缺乏足够信息）
    Unknown,
}

/// 兼容性规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityRule {
    pub zig_version_range: VersionRange,
    pub recommended_zls: String,
    pub known_compatible: Vec<String>,   // 已知兼容的 ZLS 版本列表
    pub known_issues: Vec<String>,       // 已知问题
    pub last_verified: DateTime<Utc>,
}
```

**兼容性矩阵示例**（内置或从远程加载）：

```rust
lazy_static! {
    static ref COMPATIBILITY_MATRIX: Vec<CompatibilityRule> = vec![
        CompatibilityRule {
            zig_version_range: VersionRange::from("0.11.*"),
            recommended_zls: "0.11.0".to_string(),
            known_compatible: vec!["0.11.0".to_string()],
            known_issues: vec![],
            last_verified: Utc::now(),
        },
        CompatibilityRule {
            zig_version_range: VersionRange::from("0.13.*"),
            recommended_zls: "0.13.0".to_string(),
            known_compatible: vec!["0.13.0".to_string()],
            known_issues: vec![
                "master 分支可能存在不稳定".to_string()
            ],
            last_verified: Utc::now(),
        },
        // ... 更多规则
    ];
}
```

#### 2.2.4 配置管理器 (`config`)

**职责**：管理系统和项目级别的配置文件

**配置文件位置**：

| 级别 | 路径 | 用途 |
|-----|------|------|
| 全局（系统） | `/etc/zzm/config.toml` (Linux)<br>`C:\ProgramData\zzm\config.toml` (Windows) | 系统级默认配置 |
| 全局（用户） | `~/.zzm/config.toml` | 用户个人偏好 |
| 项目 | `.zzmrc` 或 `.zzm/config.toml` | 项目特定配置 |

**配置覆盖优先级**：
```
项目配置 > 用户全局 > 系统全局 > 内置默认值
```

**配置结构体**：

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub general: GeneralConfig,
    pub zig: ZigConfig,
    pub zls: ZlsConfig,
    pub ide: IdeConfig,
    pub compatibility: CompatibilityConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct GeneralConfig {
    pub install_dir: Option<PathBuf>,
    pub download_mirror: Option<String>,
    pub no_color: Option<bool>,
    pub verbose: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ZigConfig {
    pub default: Option<String>,
    pub auto_update: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ZlsConfig {
    pub default: Option<String>,
    pub install_mode: Option<String>,  // "prebuilt" | "source"
    pub build_optimize: Option<String>,
}

// ... 其他配置项
```

#### 2.2.5 项目管理器 (`project`)

**职责**：管理 `.zzmrc` 项目配置文件

**`.zzmrc` 格式**（JSON 或 TOML）：

```json
{
  "zig": "0.13.0",
  "zls": "0.13.0",
  "compatibility": "strict",
  "ide": "vscode",
  "notes": "项目需要 Zig 0.13.0 特性"
}
```

或 TOML 格式：

```toml
[zig]
version = "0.13.0"

[zls]
version = "0.13.0"

[settings]
compatibility = "strict"
ide = "vscode"
```

**核心功能**：

```rust
impl ProjectManager {
    /// 初始化项目配置文件
    pub fn init(&self, dir: &Path, options: InitOptions) -> Result<()>;

    /// 读取项目配置
    pub fn read_config(&self, dir: &Path) -> Result<ProjectConfig>;

    /// 写入项目配置
    pub fn write_config(&self, dir: &Path, config: &ProjectConfig) -> Result<()>;

    /// 获取当前目录的项目配置（向上递归查找）
    pub fn find_project_config(&self, from_dir: &Path) -> Result<Option<(PathBuf, ProjectConfig)>>;

    /// 还原项目依赖的版本
    pub async fn restore(&self, dir: &Path, options: RestoreOptions) -> Result<()>;
}
```

#### 2.2.6 IDE 集成模块 (`ide`)

**职责**：为各种编辑器生成配置文件

**支持的编辑器和生成的配置**：

##### VS Code

生成 `.vscode/settings.json`:

```json
{
  "zig.path": "C:\\Users\\user\\.zzm\\bin\\zig.exe",
  "zig.zls.path": "C:\\Users\\user\\.zzm\\bin\\zls.exe",
  "[zig]": {
    "editor.defaultFormatter": "ziglang.vscode-zig",
    "editor.formatOnSave": true
  }
}
```

##### Neovim

生成 `lspconfig` 配置片段：

```lua
local zzm_paths = require('zzm').get_paths()

require('lspconfig').zls.setup({
  cmd = { zzm_paths.zls },
  settings = {
    zls = {
      zig_exe_path = zzm_paths.zig,
      enable_inlay_hints = true,
    }
  }
})
```

##### Helix

Helix 自动检测 PATH，无需额外配置，但可以生成 languages.toml 补充：

```toml
[language-server.zls]
command = "zls"

[[language]]
name = "zig"
language-servers = ["zls"]
formatter = { command = "zig", args = ["fmt", "--stdin"] }
```

**IDE 模块接口**：

```rust
pub trait IdeGenerator: Send + Sync {
    fn name(&self) -> &str;
    fn generate_config(&self, paths: &ToolPaths) -> Result<String>;
    fn config_file_path(&self) -> PathBuf;
    fn installation_instructions(&self) -> String;
}

pub struct VsCodeGenerator;
pub struct NeovimGenerator;
pub struct HelixGenerator;
// ...
```

---

### 2.3 基础设施层模块

#### 2.3.1 下载管理器 (`downloader`)

**职责**：处理文件下载、断点续传、校验等

**技术选型**：`reqwest` (异步 HTTP 客户端)

**核心功能**：

```rust
use reqwest::Client;
use tokio::fs::File;
use futures_util::StreamExt;

pub struct Downloader {
    client: Client,
    progress_callback: Option<Box<dyn Fn(u64, u64) + Send + Sync>>,
}

impl Downloader {
    /// 下载文件到本地（带进度回调）
    pub async fn download_to_file(
        &self,
        url: &str,
        dest: &Path,
    ) -> Result<()> {
        let response = self.client.get(url).send().await?;
        let total = response.content_length().unwrap_or(0);

        let mut file = File::create(dest).await?;
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            if let Some(ref cb) = self.progress_callback {
                cb(downloaded, total);
            }
        }

        Ok(())
    }

    /// 下载并验证校验和
    pub async fn download_with_checksum(
        &self,
        url: &str,
        dest: &Path,
        expected_sha256: &str,
    ) -> Result<()> {
        self.download_to_file(url, dest).await?;
        self.verify_checksum(dest, expected_sha256)?;
        Ok(())
    }

    /// 计算 SHA256 校验和
    fn verify_checksum(&self, file: &Path, expected: &str) -> Result<()> {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        let mut file = std::fs::File::open(file)?;
        std::io::copy(&mut file, &mut hasher)?;

        let result = format!("{:x}", hasher.finalize());
        if result != expected {
            return Err(Error::ChecksumMismatch { expected: expected.to_string(), actual: result });
        }
        Ok(())
    }
}
```

#### 2.3.2 文件系统操作 (`filesystem`)

**职责**：跨平台的文件操作封装

**核心功能**：

```rust
use std::path::PathBuf;

pub struct FileSystemManager;

impl FileSystemManager {
    /// 解压归档文件（tar.gz 或 zip）
    pub fn extract_archive(archive: &Path, dest: &Path) -> Result<()> {
        // 自动检测格式并解压
        if archive.extension().map(|e| e == "zip").unwrap_or(false) {
            Self::extract_zip(archive, dest)?;
        } else {
            Self::extract_tar_gz(archive, dest)?;
        }
        Ok(())
    }

    /// 创建符号链接（跨平台）
    pub fn create_symlink(original: &Path, link: &Path) -> Result<()> {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(original, link)?;
        }
        #[cfg(windows)]
        {
            // Windows 下如果需要管理员权限，改用 shim 方式
            if std::symlink_file(original, link).is_err() {
                Self::create_shim(original, link)?;
            }
        }
        Ok(())
    }

    /// Windows: 创建 shim 可执行文件
    #[cfg(windows)]
    pub fn create_shim(target: &Path, shim_path: &Path) -> Result<()> {
        let shim_code = format!(
            r#"@echo off
"{target}" %*
"#,
            target.display()
        );
        std::fs::write(shim_path, shim_code)?;
        Ok(())
    }
}
```

#### 2.3.3 路径管理器 (`path_manager`)

**职责**：管理工具的可执行文件路径和 PATH 环境变量

**目录结构设计**：

```
~/.zzm/
├── bin/                          # 当前活动的符号链接/shim
│   ├── zig -> ../versions/zig/<version>/zig
│   └── zls -> ../versions/zls/<version>/zls
├── versions/
│   ├── zig/
│   │   ├── 0.11.0/
│   │   │   └── zig (或 zig.exe)
│   │   ├── 0.12.0/
│   │   ├── 0.13.0/
│   │   └── master/
│   └── zls/
│       ├── 0.11.0/
│       │   └── zls (或 zls.exe)
│       ├── 0.12.0/
│       ├── 0.13.0/
│       └── master/
├── cache/                        # 下载缓存
│   ├── zig-0.13.0.tar.gz
│   └── zls-0.13.0.zip
├── config.toml                   # 全局配置
├── compatibility.json            # 兼容性矩阵（可更新）
└── logs/                         # 日志文件
```

**路径管理逻辑**：

```rust
pub struct PathManager {
    base_dir: PathBuf,
    bin_dir: PathBuf,
    versions_dir: PathBuf,
    cache_dir: PathBuf,
}

impl PathManager {
    /// 初始化目录结构
    pub fn initialize(&self) -> Result<()> {
        fs::create_dir_all(&self.bin_dir)?;
        fs::create_dir_all(&self.versions_dir.join("zig"))?;
        fs::create_dir_all(&self.versions_dir.join("zls"))?;
        fs::create_dir_all(&self.cache_dir)?;
        Ok(())
    }

    /// 更新符号链接指向新的版本
    pub fn update_symlink(&self, tool: ToolType, version: &str) -> Result<()> {
        let version_path = self.get_version_path(tool, version);
        let link_path = self.bin_dir.join(tool.binary_name());

        // 移除旧链接
        if link_path.exists() {
            fs::remove_file(&link_path)?;
        }

        // 创建新链接
        self.filesystem.create_symlink(&version_path, &link_path)?;
        Ok(())
    }

    /// 将 bin 目录添加到 PATH（通过 shell profile）
    pub fn add_to_path(&self) -> Result<()> {
        let bin_str = self.bin_dir.to_string_lossy().to_string();

        #[cfg(unix)]
        {
            // 追加到 .bashrc/.zshrc 等
            let export_line = format!(r#"export PATH="{}:$PATH""#, bin_str);
            // 写入 shell 配置...
        }

        #[cfg(windows)]
        {
            // 添加到用户环境变量
            Command::new("setx")
                .args(["PATH", &format!("{};%PATH%", bin_str)])
                .output()?;
        }

        Ok(())
    }

    /// 获取工具的实际路径
    pub fn get_tool_path(&self, tool: ToolType) -> Result<PathBuf> {
        let link_path = self.bin_dir.join(tool.binary_name());

        // 解析符号链接获取实际路径
        #[cfg(unix)]
        {
            std::fs::read_link(&link_path)
        }

        #[cfg(windows)]
        {
            // Windows shim 文件，需要读取内容解析目标
            Ok(link_path)  // 简化处理
        }
    }
}
```

#### 2.3.4 进程管理器 (`process`)

**职责**：执行外部命令（用于从源码编译 ZLS）

```rust
use tokio::process::Command;

pub struct ProcessManager;

impl ProcessManager {
    /// 执行命令并返回输出
    pub async fn execute(&self, cmd: &str, args: &[&str], cwd: &Path) -> Result<ProcessOutput> {
        let output = Command::new(cmd)
            .args(args)
            .current_dir(cwd)
            .output()
            .await?;

        if output.status.success() {
            Ok(ProcessOutput {
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                success: true,
            })
        } else {
            Err(Error::CommandFailed {
                cmd: cmd.to_string(),
                code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            })
        }
    }

    /// 编译 ZLS（调用 zig build）
    pub async fn build_zls(&self, zls_source: &Path, zig_path: &Path, optimize: &str) -> Result<PathBuf> {
        self.execute(
            zig_path.to_string_lossy().as_ref(),
            &["build", &format!("-Doptimize={optimize}")],
            zls_source,
        ).await?;

        // 编译产物路径
        Ok(zls_source.join("zig-out/bin/zls"))
    }
}
```

---

### 2.4 平台抽象层 (`platform`)

**职责**：封装平台特定操作，提供统一接口

```rust
pub trait PlatformTrait: Send + Sync {
    /// 获取平台名称
    fn name(&self) -> &'static str;

    /// 获取默认安装目录
    fn default_install_dir(&self) -> PathBuf;

    /// 创建符号链接
    fn create_symlink(&self, original: &Path, link: &Path) -> Result<()>;

    /// 获取 shell 配置文件路径列表
    fn shell_config_files(&self) -> Vec<PathBuf>;

    /// 更新 PATH 环境变量
    fn update_path_env(&self, new_entry: &str) -> Result<()>;

    /// 获取临时目录
    fn temp_dir(&self) -> PathBuf;

    /// 检查是否具有管理员/root 权限
    fn is_admin(&self) -> bool;
}

pub struct WindowsPlatform;
pub struct MacOSPlatform;
pub struct LinuxPlatform;

impl PlatformTrait for WindowsPlatform {
    fn name(&self) -> &'static str { "windows" }

    fn default_install_dir(&self) -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from(r"C:\Users"))
            .join("zzm")
    }

    fn create_symlink(&self, original: &Path, link: &Path) -> Result<()> {
        // Windows 可能需要特殊处理
        std::os::windows::fs::symlink_file(original, link)
            .or_else(|_| Self::create_shim(original, link))
    }

    // ... 其他实现
}

// 类似地实现 MacOS 和 Linux 平台
```

---

## 3. 技术选型

### 3.1 核心依赖库

| 类别 | 库名 | 版本 | 用途 | 选择理由 |
|-----|------|------|------|---------|
| **CLI 框架** | `clap` | ^4.x | 命令行解析 | 功能强大，derive 模式易用，生态成熟 |
| **异步运行时** | `tokio` | ^1.x | 异步 I/O | Rust 异步事实标准，性能优秀 |
| **HTTP 客户端** | `reqwest` | ^0.11 | 文件下载 | 支持 async，API 友好 |
| **JSON 处理** | `serde_json` | ^1.x | JSON 序列化 | Rust 事实标准 |
| **TOML 解析** | `toml` | ^0.8 | 配置文件解析 | 官方维护，轻量 |
| **终端输出** | `console` | ^0.15 | 彩色文本/样式 | 跨平台终端控制 |
| **表格输出** | `tabled` | ^0.15 | 表格渲染 | 功能丰富，自定义性强 |
| **进度条** | `indicatif` | ^0.17 | 进度显示 | 美观且功能完善 |
| **交互输入** | `dialoguer` | ^0.11 | 交互式向导 | 支持多种输入控件 |
| **错误处理** | `anyhow` | ^1.x | 错误处理 | 简化错误类型，便于传播 |
| **日志** | `tracing` | ^0.1 | 结构化日志 | 异步友好，可观测性好 |
| **版本比较** | `semver` | ^1.x | 语义化版本处理 | 官方推荐库 |
| **文件监听** | `notify` | ^6.x | 文件变更监听 | 跨平台文件系统事件 |
| **SHA256** | `sha2` | ^0.10 | 校验和计算 | 纯 Rust 实现 |
| **压缩解压** | `flate2` | ^1.x | gzip 解压 | 性能好，API 简单 |
| **ZIP 处理** | `zip` | ^0.6 | ZIP 文件处理 | 功能全面 |
| **路径处理** | `dirs` | ^5.x | 系统目录查找 | 跨平台标准路径 |

### 3.2 开发依赖

| 类别 | 库名 | 用途 |
|-----|------|------|
| 测试框架 | 内置 + `tempfile` | 单元测试和集成测试 |
| 代码格式化 | `rustfmt` | 代码风格统一 |
| Lint | `clippy` | 代码质量检查 |
| 文档测试 | `doc-comment` | 文档示例测试 |

### 3.3 技术决策记录 (ADR)

#### ADR-001: 选择 Rust 作为实现语言

**状态**: 已采纳

**背景**:
- 需要高性能的单二进制分发
- 跨平台支持（Windows/macOS/Linux）
- 内存安全和并发安全

**选项**:
1. **Rust** ✅ (选中): 无 GC，零成本抽象，优秀的跨平台编译能力
2. Go: 简单易用，但二进制体积较大
3. Zig: 与项目主题契合，但生态尚不成熟
4. Python: 开发快，但性能差，依赖运行时

**决定**: 使用 Rust (Edition 2024)，利用其：
- 出色的 CLI 开发生态（clap, tokio 等）
- 单静态链接二进制，无外部依赖
- 内存安全保证
- 良好的跨平台编译支持（cross, cargo-xbuild）

#### ADR-002: 使用 clap derive 模式

**状态**: 已采纳

**背景**: 需要一个强大且类型安全的 CLI 解析方案

**决定**: 使用 clap v4 的 derive 模式，理由：
- 类型安全，编译时验证
- 自动生成帮助文档
- 支持子命令嵌套
- 社区活跃，文档完善

#### ADR-003: 异步架构设计

**状态**: 已采纳

**背景**: 涉及大量 I/O 操作（网络下载、文件读写）

**决定**: 基于 Tokio 的异步架构：
- 下载操作不阻塞主线程
- 可以并行执行多个独立任务
- 为未来可能的并发扩展预留空间

---

## 4. 数据存储设计

### 4.1 本地文件系统存储

**策略**: 不使用数据库，采用文件系统 + 结构化配置文件

**原因**:
- 数据量小（版本元信息、配置）
- 避免额外依赖
- 用户可直接查看和编辑
- 易于备份和迁移

### 4.2 文件格式规范

| 文件 | 格式 | 用途 |
|------|------|------|
| `config.toml` | TOML | 主配置文件 |
| `.zzmrc` | JSON/TOML | 项目配置 |
| `compatibility.json` | JSON | 兼容性矩阵（可从远程更新） |
| `installed.json` | JSON | 已安装版本索引 |
| `cache-metadata.json` | JSON | 缓存文件的元信息（URL、校验和、时间） |

### 4.3 数据流示意

```
用户执行: zzm install 0.13.0 --with-zls
    ↓
1. 读取 ~/.zzm/config.toml          [配置读取]
2. 查询 https://ziglang.org/download  [远程 API 调用]
3. 下载 zig-0.13.0-x86_64-windows... [文件下载]
4. 校验 SHA256                      [完整性验证]
5. 解压到 ~/.zzm/versions/zig/0.13.0/ [文件解压]
6. 更新 ~/.zzm/bin/zig 符号链接       [路径更新]
7. 重复步骤 2-6 for ZLS             [ZLS 安装]
8. 更新 installed.json              [元数据持久化]
9. 执行兼容性检查                    [业务逻辑]
10. 输出结果                        [用户反馈]
```

---

## 5. 目录结构设计

```
zig-zls-manager/
├── Cargo.toml                      # 项目配置和依赖
├── Cargo.lock                      # 依赖锁定文件
├── README.md                       # 项目说明
├── .gitignore                      # Git 忽略规则
│
├── docs/                           # 项目文档
│   ├── spec.md                     # 需求规格说明书
│   ├── architecture.md             # 架构设计文档（本文档）
│   └── usage.md                    # 使用指南
│
├── src/                            # 源代码根目录
│   ├── main.rs                     # 程序入口
│   ├── cli.rs                      # CLI 定义和解析
│   │
│   ├── core/                       # 核心业务逻辑
│   │   ├── mod.rs
│   │   ├── zig_manager.rs          # Zig 版本管理器
│   │   ├── zls_manager.rs          # ZLS 版本管理器
│   │   ├── compatibility.rs        # 兼容性检查器
│   │   ├── config.rs               # 配置管理器
│   │   ├── project.rs              # 项目管理器
│   │   └── ide.rs                  # IDE 集成模块
│   │
│   ├── infra/                      # 基础设施层
│   │   ├── mod.rs
│   │   ├── downloader.rs           # 下载管理器
│   │   ├── filesystem.rs           # 文件系统操作
│   │   ├── path_manager.rs         # 路径管理器
│   │   ├── process.rs              # 进程管理器
│   │   ├── checksum.rs             # 校验和计算
│   │   └── cache.rs                # 缓存管理
│   │
│   ├── platform/                   # 平台抽象层
│   │   ├── mod.rs
│   │   ├── trait.rs                # Platform trait 定义
│   │   ├── windows.rs              # Windows 实现
│   │   ├── macos.rs                # macOS 实现
│   │   └── linux.rs                # Linux 实现
│   │
│   ├── output/                     # 输出格式化
│   │   ├── mod.rs
│   │   ├── console.rs              # 控制台输出
│   │   ├── table.rs                # 表格输出
│   │   ├── progress.rs             # 进度条
│   │   └── json.rs                 # JSON 输出
│   │
│   ├── utils/                      # 工具函数
│   │   ├── mod.rs
│   │   ├── version.rs              # 版本解析工具
│   │   ├── error.rs                # 错误类型定义
│   │   └── helpers.rs              # 通用辅助函数
│   │
│   └── lib.rs                      # 库入口（可选，供测试用）
│
├── tests/                          # 集成测试
│   ├── integration/
│   │   ├── test_install.rs
│   │   ├── test_switch.rs
│   │   └── test_ide_integration.rs
│   └── fixtures/                   # 测试用固定数据
│       └── sample_configs/
│
└── scripts/                        # 构建和发布脚本
    ├── build.rs                    # 构建脚本
    ├── release.sh                  # 多平台发布脚本
    └── generate_completion.sh      # 生成补全脚本
```

---

## 6. 错误处理策略

### 6.1 错误分类体系

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ZzmError {
    // ========== IO 错误 ==========
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    // ========== 网络错误 ==========
    #[error("网络请求失败: {0}")]
    Network(#[from] reqwest::Error),

    #[error("下载失败: {url} ({reason})")]
    DownloadFailed { url: String, reason: String },

    // ========== 版本错误 ==========
    #[error("无效的版本号: {version}")]
    InvalidVersion { version: String },

    #[error("版本 '{version}' 未找到")]
    VersionNotFound { version: String },

    #[error("版本 '{version}' 已安装，使用 --force 强制重装")]
    AlreadyInstalled { version: String },

    // ========== 兼容性错误 ==========
    #[error("Zig {zig} 与 ZLS {zls} 不兼容: {reason}")]
    IncompatibleVersions { zig: String, zls: String, reason: String },

    // ========== 配置错误 ==========
    #[error("配置文件错误: {path} - {reason}")]
    ConfigError { path: PathBuf, reason: String },

    // ========== 校验错误 ==========
    #[error("校验和不匹配: 期望 {expected}, 实际 {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    // ========== 平台错误 ==========
    #[error("不支持的平台: {platform}")]
    UnsupportedPlatform { platform: String },

    #[error("权限不足: 需要管理员/root 权限执行此操作")]
    PermissionDenied,

    // ========== 用户中断 ==========
    #[error("操作被用户取消")]
    Cancelled,
}
```

### 6.2 错误处理最佳实践

1. **使用 `anyhow` 进行错误传播**：在应用层使用 `Result<T, anyhow::Error>`
2. **使用 `thiserror` 定义领域错误**：在库层使用具体的错误枚举
3. **提供上下文信息**：使用 `.context()` 方法添加错误上下文
4. **用户友好的错误消息**：将技术错误转换为可操作的中文提示
5. **建议修复措施**：每个错误都尽可能提供解决方案提示

**示例**：

```rust
pub async fn install_zig(&self, version: &str) -> Result<()> {
    // 验证版本格式
    let resolved = self.resolve_version(version)
        .context(format!("无法解析版本 '{}'", version))?

    // 检查是否已安装
    if self.is_installed(&resolved)? {
        return Err(ZzmError::AlreadyInstalled {
            version: resolved.clone(),
        }.into());
    }

    // 下载
    self.downloader.download_with_checksum(
        &download_url,
        &temp_path,
        &expected_checksum,
    ).await
    .context(format!(
        "下载 Zig {} 失败，请检查网络连接或尝试使用镜像源",
        resolved
    ))?;

    // ... 后续操作

    Ok(())
}
```

---

## 7. 安全考虑

### 7.1 下载安全

- ✅ 仅从官方源（ziglang.org, github.com）下载
- ✅ 强制 SHA256 校验和验证
- ✅ 支持 HTTPS only，禁止明文 HTTP
- ✅ 缓存文件同样需要校验

### 7.2 执行安全

- ✅ 不执行任意用户提供的代码
- ✅ 从源码编译时使用沙箱（可选）
- ✅ PATH 修改前进行确认提示

### 7.3 权限最小化

- ✅ 默认不需要管理员权限
- ✅ 用户级别安装（~/.zzm）
- ✅ 明确标识需要提升权限的操作

### 7.4 数据隐私

- ✅ 不收集用户数据
- ✅ 不上传任何使用统计（除非用户明确同意）
- ✅ 配置文件包含敏感信息时给出警告

---

## 8. 测试策略

### 8.1 测试层级

| 层级 | 工具 | 覆盖目标 | 示例 |
|------|------|---------|------|
| **单元测试** | 内置 `#[test]` | 单个函数/方法 | 版本解析、校验和计算 |
| **集成测试** | `tests/` 目录 | 模块间交互 | 安装流程、版本切换 |
| **端到端测试** | `assert_cmd` | CLI 命令 | 完整用户场景 |
| **属性测试** | `proptest` | 边界情况 | 各种版本号格式输入 |

### 8.2 关键测试用例

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_install_valid_version() {
        // 测试正常安装流程
    }

    #[test]
    fn test_resolve_version_shorthand() {
        assert_eq!(resolve_version("0.13"), Ok("0.13.0".to_string()));
        assert_eq!(resolve_version(".14"), Ok("0.14.2".to_string()));  // 最新 0.14.x
    }

    #[test]
    fn test_compatibility_check() {
        let status = check_compatibility("0.13.0", "0.13.0");
        assert_eq!(status, CompatibilityStatus::FullyCompatible);

        let status = check_compatibility("0.13.0", "0.11.0");
        assert!(matches!(status, CompatibilityStatus::Incompatible(_)));
    }

    #[test]
    fn test_config_parsing() {
        let config = Config::from_toml("...");
        assert!(config.is_ok());
    }
}
```

### 8.3 Mock 和桩

对于外部依赖（网络、文件系统），使用 mock 进行单元测试：

```rust
#[cfg(test)]
mock! {
    pub struct Downloader {}

    impl Downloader {
        pub async fn download(&self, url: &str, dest: &Path) -> Result<()>;
    }
}
```

---

## 9. 性能优化策略

### 9.1 启动优化

- **延迟加载**：非必需模块按需初始化
- **懒解析**：配置文件仅在需要时读取
- **并行启动**：Tokio runtime 在后台预热的资源

### 9.2 下载优化

- **HTTP/2 多路复用**：复用连接
- **断点续传**：支持 Range 请求
- **并发下载**：Zig 和 ZLS 并行下载
- **智能缓存**：避免重复下载相同文件

### 9.3 存储优化

- **增量更新**：仅下载变更部分（未来支持）
- **压缩存储**：旧版本归档
- **LRU 淘汰**：缓存大小限制

---

## 10. 部署与分发

### 10.1 构建目标平台

```toml
# Cargo.toml 中配置
[target.aarch64-apple-darwin]
[target.x86_64-apple-darwin]
[target.x86_64-pc-windows-msvc]
[target.x86_64-unknown-linux-gnu]
[target.aarch64-unknown-linux-gnu]
```

### 10.2 CI/CD 流水线

```yaml
# .github/workflows/release.yml (示例)
on:
  push:
    tags: ['v*']

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: windows-latest
            target: x86_64-pc-windows-msvc
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - run: cargo build --release --target ${{ matrix.target }}
      - name: Upload Release Asset
        uses: softprops/action-gh-release@v1
        with:
          files: target/${{ matrix.target }}/release/zzm*
```

### 10.3 安装方式

```bash
# 方式 1: 直接下载二进制（推荐）
curl -fsSL https://github.com/user/zzm/releases/latest/download/zzm-x86_64-windows.zip
# 或
winget install zzm.zzm

# 方式 2: 使用包管理器（未来支持）
brew tap zzm/zzm && brew install zzm
scoop bucket add zzm && scoop install zlm

# 方式 3: Cargo 安装（开发者）
cargo install zzm --locked
```

---

## 11. 未来演进路线

### Phase 1: MVP (v0.1.0)
- ✅ 核心 Zig 版本管理（install/uninstall/list/use）
- ✅ 基本 ZLS 管理（跟随 Zig 版本）
- ✅ 简单的兼容性警告
- ✅ VS Code 集成
- ✅ Windows + macOS + Linux 支持

### Phase 2: 增强版 (v0.2.0)
- 🔧 独立的 ZLS 版本管理（不从属于 Zig）
- 🔧 完整的兼容性矩阵（可在线更新）
- 🔧 项目级 `.zzmrc` 配置
- 🔧 更多 IDE 支持（Neovim, Helix, JetBrains）
- 🔧 交互式 setup wizard
- 🔧 Shell 自动补全

### Phase 3: 专业版 (v0.3.0+)
- 🚀 GUI 包装（可选的 TUI/GUI 前端）
- 🚀 插件系统（支持其他语言服务器？）
- 🚀 团队协作功能（锁文件共享）
- 🚀 CI/CD Docker 镜像
- 🚀 性能监控和遥测（可选启用）
- 🚀 多架构交叉编译支持

---

## 12. 附录

### A. 架构决策记录索引

| ID | 标题 | 状态 |
|----|------|------|
| ADR-001 | 选择 Rust 作为实现语言 | ✅ 已采纳 |
| ADR-002 | 使用 clap derive 模式 | ✅ 已采纳 |
| ADR-003 | 异步架构设计 | ✅ 已采纳 |

### B. 参考资源

- [Rust CLI 最佳实践](https://rust-cli.github.io/book/index.html)
- [clap 官方文档](https://docs.rs/clap/)
- [Tokio 官方指南](https://tokio.rs/tokio/tutorial)
- [Zig 官方下载页面](https://ziglang.org/download/)
- [ZLS GitHub 仓库](https://github.com/zigtools/zls)

### C. 术语表

| 术语 | 英文 | 说明 |
|------|------|------|
| Shim | Shim Executable | Windows 下的转发可执行文件 |
| LSP | Language Server Protocol | 语言服务器协议 |
| TUI | Terminal User Interface | 终端用户界面 |
| DERIVE | Derive Macro | Rust 过程宏，用于自动实现 trait |

---

*本文档将随着项目开发持续迭代和完善。*
