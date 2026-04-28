use clap::{Parser, Subcommand};

/// Zig/ZLS Version Manager - 专业级的 Zig + ZLS 联合版本管理工具
#[derive(Parser)]
#[command(name = "zzm")]
#[command(
    about = "Zig/ZLS Version Manager",
    long_about = "专业级的 Zig + ZLS 联合版本管理 CLI 工具\n\n管理 Zig 编译器和 ZLS 语言服务器的版本，\n支持智能兼容性检查、项目级配置和 IDE 集成。"
)]
#[command(version)]
#[command(propagate_version = true)]
#[command(
    after_help = "示例:\n  zzm install 0.13.0          安装 Zig 0.13.0\n  zzm install 0.13.0 --with-zls  同时安装匹配的 ZLS\n  zzm list --remote            查看远程可用版本\n  zzm use 0.13.0               切换到 Zig 0.13.0\n  zzm info                     显示当前环境信息"
)]
pub struct Cli {
    /// 禁用彩色输出
    #[arg(long, global = true)]
    pub no_color: bool,

    /// 详细输出模式（显示调试信息）
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// 以 JSON 格式输出
    #[arg(long, global = true)]
    pub json: bool,

    /// 子命令
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 安装指定版本的 Zig（可同时安装 ZLS）
    #[command(disable_version_flag = true)]
    Install {
        /// 版本号 (如 0.13.0, 0.13, master, stable, .13)
        version: String,

        /// 同时安装匹配的 ZLS
        #[arg(long)]
        with_zls: bool,

        /// 从源码编译 ZLS
        #[arg(long)]
        from_source: bool,

        /// 非交互模式，自动确认所有提示
        #[arg(short, long)]
        yes: bool,

        /// 强制重装已存在的版本
        #[arg(short, long)]
        force: bool,
    },

    /// 卸载指定版本
    Uninstall {
        /// 要卸载的版本号
        version: String,

        /// 清除配置和数据
        #[arg(long)]
        purge: bool,
    },

    /// 列出版本信息
    List {
        /// 仅显示已安装的版本
        #[arg(long)]
        installed: bool,

        /// 显示远程可用版本
        #[arg(long)]
        remote: bool,

        /// 以 JSON 格式输出
        #[arg(long)]
        json: bool,
    },

    /// 切换当前使用的版本
    Use {
        /// 目标版本号
        version: String,

        /// 全局切换（默认行为）
        #[arg(short, long)]
        global: bool,

        /// 项目级切换（写入 .zzmrc）
        #[arg(short, long)]
        project: bool,

        /// 设为默认版本
        #[arg(long)]
        default: bool,

        /// 同时指定 ZLS 版本
        #[arg(long)]
        zls: Option<String>,
    },

    /// 显示当前激活的版本
    Current {
        /// 以 JSON 格式输出
        #[arg(long)]
        json: bool,
    },

    /// ZLS 语言服务器管理子命令组
    Zls {
        #[command(subcommand)]
        command: ZlsCommands,
    },

    /// 一键初始化开发环境
    Setup {
        /// 默认安装的 Zig 版本
        version: Option<String>,

        /// 同时安装 ZLS
        #[arg(long)]
        with_zls: bool,

        /// 配置目标编辑器 (vscode, neovim, helix)
        #[arg(long)]
        ide: Option<String>,

        /// 启动交互式向导
        #[arg(long)]
        wizard: bool,
    },

    /// 同步 Zig 和 ZLS 到推荐组合
    Sync {
        /// 仅预览将要执行的操作（不实际执行）
        #[arg(long)]
        dry_run: bool,
    },

    /// 手动绑定 Zig↔ZLS 版本关系
    Pair {
        /// Zig 版本号
        zig_version: String,

        /// ZLS 版本号（可选，不指定则自动推荐）
        #[arg(long)]
        zls: Option<String>,

        /// 兼容性模式: strict, loose, auto（默认 auto）
        #[arg(long)]
        compatibility: Option<String>,

        /// 显示当前项目的版本绑定
        #[arg(long)]
        show: bool,
    },

    /// 从项目 .zzmrc 还原开发环境
    Restore {
        /// 项目目录路径（默认为当前目录）
        dir: Option<String>,
    },

    /// 显示当前环境详细信息
    Info {
        /// 显示更多详细信息
        #[arg(short, long)]
        verbose: bool,
    },

    /// 配置管理
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// IDE 集成管理
    Ide {
        #[command(subcommand)]
        command: IdeCommands,
    },

    /// 移除未使用的旧版本
    Prune {
        /// 仅预览将要清理的版本（不实际执行）
        #[arg(long)]
        dry_run: bool,

        /// 跳过确认提示，直接执行
        #[arg(short, long)]
        confirm: bool,
    },

    /// 清理缓存和旧版本
    Clean {
        /// 清理所有缓存（包括下载缓存）
        #[arg(long)]
        all: bool,

        /// 仅预览将要清理的内容
        #[arg(long)]
        dry_run: bool,
    },

    /// 运行诊断程序，检查环境配置
    Doctor,

    /// 生成 Shell 自动补全脚本
    Completion {
        /// Shell 类型: bash, zsh, powershell, fish
        shell: String,
    },
}

#[derive(Subcommand, Clone)]
pub enum ZlsCommands {
    /// 安装指定版本的 ZLS
    Install {
        /// ZLS 版本号
        version: String,

        /// 从源码编译安装
        #[arg(long)]
        from_source: bool,

        /// 用于编译 ZLS 的 Zig 版本
        #[arg(long)]
        zig_version: Option<String>,

        /// 非交互模式
        #[arg(short, long)]
        yes: bool,
    },

    /// 卸载 ZLS 版本
    Uninstall {
        /// 要卸载的版本号
        version: String,
    },

    /// 列出 ZLS 版本
    List {
        /// 仅显示已安装版本
        #[arg(long)]
        installed: bool,

        /// 显示远程可用版本
        #[arg(long)]
        remote: bool,
    },

    /// 切换当前 ZLS 版本
    Use {
        /// 目标版本号
        version: String,
    },

    /// 显示当前激活的 ZLS 版本
    Current,
}

#[derive(Subcommand, Clone)]
pub enum ConfigCommands {
    /// 列出所有配置项
    List,

    /// 获取指定配置值
    Get {
        /// 配置键名 (如 zig.default, `zls.install_mode`)
        key: String,
    },

    /// 设置配置值
    Set {
        /// 配置键名
        key: String,

        /// 配置值
        value: String,
    },

    /// 在编辑器中打开配置文件
    Edit,
}

#[derive(Subcommand, Clone)]
pub enum IdeCommands {
    /// 生成 IDE 配置文件
    Config {
        /// 编辑器类型: vscode, neovim, helix
        editor: String,
    },

    /// 检测 IDE 配置状态
    Check,

    /// 诊断 IDE 集成常见问题
    Doctor,

    /// 输出当前 zig/zls 可执行文件路径
    Path,
}
