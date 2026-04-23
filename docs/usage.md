# Zig/ZLS 版本管理器 - 用户使用指南

## 📋 文档信息

- **版本**: v1.0.0
- **创建日期**: 2026-04-23
- **适用版本**: zig-zls-manager v0.1.0+
- **关联文档**: [spec.md](./spec.md) | [architecture.md](./architecture.md)

---

## 1. 快速开始

### 1.1 安装 zzm

#### Windows

```powershell
# 方式 1: 使用 Scoop（推荐）
scoop bucket add zzm https://github.com/your-org/zzm-scoop
scoop install zzm

# 方式 2: 使用 WinGet
winget install zzm.zzm

# 方式 3: 手动下载
# 访问 https://github.com/your-org/zzm/releases/latest
# 下载 zzm-x86_64-windows.zip，解压后将 zzm.exe 放入 PATH
```

#### macOS

```bash
# 方式 1: 使用 Homebrew（推荐）
brew tap your-org/zzm
brew install zzm

# 方式 2: 手动下载
curl -fsSL https://github.com/your-org/zzm/releases/latest/download/zzm-x86_64-macos.tar.gz | tar xz
sudo mv zzm /usr/local/bin/
```

#### Linux

```bash
# 方式 1: 使用安装脚本（推荐）
curl -fsSL https://raw.githubusercontent.com/your-org/zzm/main/install.sh | bash

# 方式 2: 手动下载
curl -fsSL https://github.com/your-org/zzm/releases/latest/download/zzm-x86_64-linux.tar.gz | tar xz
sudo mv zzm /usr/local/bin/

# 方式 3: 使用 Cargo（开发者）
cargo install zzm --locked
```

### 1.2 验证安装

```bash
zzm --version
# 输出: zzm 0.1.0 (x86_64-pc-windows-msvc)

zzm --help
# 显示完整的帮助信息
```

### 1.3 首次运行向导

```bash
zzm setup --wizard
```

**交互式向导示例**：

```
🚀 欢迎使用 Zig/ZLS 版本管理器！
本向导将帮助你快速初始化开发环境。

? 选择要安装的默认 Zig 版本:
  ❯ 0.13.0 (推荐)
    0.12.0
    0.11.0
    master (nightly)

? 是否同时安装 ZLS 语言服务器？ (Y/n)
> Y

? 选择要配置的编辑器:
  ❯ VS Code
    Neovim
    Helix
    跳过

⠋ 正在下载 Zig 0.13.0...
████████████████████████████████████████  100%  45.2 MB/45.2 MB

⠋ 正在下载 ZLS 0.13.0...
████████████████████████████████████████  100%  12.8 MB/12.8 MB

✅ Zig 0.13.0 安装完成
✅ ZLS 0.13.0 安装完成
✅ VS Code 配置已生成

🎉 初始化完成！
   运行 'zzm info' 查看当前环境状态
   运行 'zzm --help' 查看所有可用命令
```

---

## 2. 核心功能使用指南

### 2.1 Zig 版本管理

#### 2.1.1 查看可用版本

```bash
# 查看本地已安装的版本
zzm list --installed
```

**输出示例**：

```
╭─────────────────────────────────────────────────────────────╮
│ 已安装的 Zig 版本                                            │
├──────────┬─────────────┬───────────────┬──────────┬─────────┤
│ 版本     │ 通道        │ 安装路径                │ 状态    │
├──────────┼─────────────┼───────────────┼──────────┼─────────┤
│ 0.13.0   │ Stable      │ ~/.zzm/versions/zig/0.13.0 │ ✓ 默认 │
│ 0.12.0   │ Stable      │ ~/.zzm/versions/zig/0.12.0 │         │
│ master   │ Nightly     │ ~/.zzm/versions/zig/master │         │
╰──────────┴─────────────┴───────────────┴──────────┴─────────╯
```

```bash
# 查看远程可用的版本
zzm list --remote
```

**输出示例**：

```
╭─────────────────────────────────────────────────────────────╮
│ 可用的 Zig 版本（远程）                                      │
├──────────┬─────────────┬──────────────┬──────────┬──────────┤
│ 版本     │ 通道        │ 发布日期      │ 大小     │ 已安装   │
├──────────┼─────────────┼──────────────┼──────────┼──────────┤
│ 0.13.0   │ Stable      │ 2024-06-01   │ 45.2 MB  │ ✓        │
│ 0.12.0   │ Stable      │ 2023-12-01   │ 43.8 MB  │ ✓        │
│ 0.11.0   │ Stable      │ 2023-06-01   │ 42.1 MB  │          │
│ master   │ Nightly     │ 2026-04-23   │ 46.5 MB  │ ✓        │
╰──────────┴─────────────┴──────────────┴──────────┴──────────╯
```

#### 2.1.2 安装 Zig 版本

```bash
# 安装特定版本
zzm install 0.13.0

# 安装最新稳定版
zzm install stable

# 安装最新开发版
zzm install master

# 使用简写版本号（自动解析为最新的补丁版本）
zzm install 0.13    # 等同于 0.13.0
zzm install .14     # 等同于 0.14.x 的最新版本

# 安装并同时安装匹配的 ZLS
zzm install 0.13.0 --with-zls

# 强制重新安装（覆盖已存在的版本）
zzm install 0.13.0 --force

# 非交互模式（适用于 CI/CD）
zzm install 0.13.0 --yes
```

**安装过程示例**：

```
⠋ 正在查询 Zig 0.13.0 的下载信息...
✓ 找到版本: 0.13.0 (Stable)

⠋ 正在下载 zig-0.13.0-x86_64-windows.tar.xz...
[████████████████████████████████████] 45.2 MB/45.2 MB (100%)

⠋ 正在校验文件完整性...
✓ SHA256 校验通过: a1b2c3d4e5f6...

⠋ 正在解压文件...
✓ 解压完成

⠋ 正在配置环境...
✓ 已创建符号链接: ~/.zzm/bin/zig -> ~/.zzm/versions/zig/0.13.0/zig.exe

✅ Zig 0.13.0 安装成功！
   可执行文件: C:\Users\user\.zzm\bin\zig.exe
   版本信息: 0.13.0

💡 提示: 运行 'zzm use 0.13.0' 将其设为默认版本
```

#### 2.1.3 切换 Zig 版本

```bash
# 全局切换（影响所有终端会话）
zzm use 0.13.0

# 项目级切换（仅在当前项目目录生效）
zzm use 0.12.0 --project

# 切换到最新开发版
zzm use master

# 切换并设为默认
zzm use 0.13.0 --default

# 同时切换 Zig 和 ZLS
zzm use 0.13.0 --zls 0.13.0
```

**切换过程示例**：

```
⠋ 正在切换 Zig 版本到 0.13.0...
✓ 已更新符号链接: ~/.zzm/bin/zig
✓ 已更新配置文件

✅ 已切换到 Zig 0.13.0

当前环境:
  Zig: 0.13.0 (Stable)
  ZLS: 0.12.0 (⚠️ 版本不匹配)

💡 建议: 运行 'zzm sync' 同步到推荐的 Zig/ZLS 组合
```

#### 2.1.4 卸载 Zig 版本

```bash
# 卸载特定版本
zzm uninstall 0.11.0

# 卸载并清除相关配置
zzm uninstall 0.11.0 --purge

# 清理所有未标记保留的旧版本
zzm clean
```

### 2.2 ZLS 版本管理

#### 2.2.1 独立管理 ZLS

```bash
# 查看已安装的 ZLS 版本
zzm zls list

# 安装特定版本的 ZLS
zzm zls install 0.13.0

# 安装预编译二进制（默认）
zzm zls install 0.13.0 --mode prebuilt

# 从源码编译安装（需要 Zig 环境）
zzm zls install 0.13.0 --mode source

# 切换 ZLS 版本
zzm zls use 0.13.0

# 查看当前使用的 ZLS 版本
zzm zls current
```

#### 2.2.2 ZLS 与 Zig 的关联

```bash
# 安装 Zig 时自动安装匹配的 ZLS
zzm install 0.13.0 --with-zls

# 根据 Zig 版本自动选择推荐的 ZLS
zzm zls install --match-zig

# 查看当前 Zig 对应的推荐 ZLS 版本
zzm zls recommend
```

**输出示例**：

```
当前 Zig 版本: 0.13.0
推荐 ZLS 版本: 0.13.0

兼容性状态: ✓ 完全兼容

已安装的 ZLS 版本:
  - 0.13.0 ✓ (当前)
  - 0.12.0

💡 运行 'zzm zls use 0.13.0' 切换到推荐版本
```

### 2.3 联合管理功能

#### 2.3.1 一键初始化环境

```bash
# 快速安装 Zig + ZLS
zzm setup 0.13.0 --with-zls

# 完整初始化（包括 IDE 配置）
zzm setup 0.13.0 --with-zls --ide vscode

# 交互式向导
zzm setup --wizard
```

#### 2.3.2 版本同步

```bash
# 同步 Zig 和 ZLS 到最佳匹配组合
zzm sync

# 预览将要执行的操作（不实际执行）
zzm sync --dry-run
```

**输出示例**：

```
⠋ 正在分析当前环境...

当前状态:
  Zig: 0.13.0 (Stable)
  ZLS: 0.12.0 (Stable)

兼容性分析:
  ⚠️ ZLS 版本与 Zig 不匹配
  💡 推荐: ZLS 0.13.0

计划执行的操作:
  1. 安装 ZLS 0.13.0（如果未安装）
  2. 切换到 ZLS 0.13.0

? 是否继续？ (Y/n)
> Y

⠋ 正在安装 ZLS 0.13.0...
████████████████████████████████████████  100%  12.8 MB/12.8 MB

⠋ 正在切换 ZLS 版本...
✓ 已更新符号链接

✅ 同步完成！

当前环境:
  Zig: 0.13.0 (Stable)
  ZLS: 0.13.0 (Stable)
  兼容性: ✓ 完全匹配
```

#### 2.3.3 环境信息查询

```bash
# 显示详细的当前环境信息
zzm info
```

**输出示例**：

```
╭═══════════════════════════════════════════════════════════╮
│              Zig/ZLS Version Manager v0.1.0               │
╰═══════════════════════════════════════════════════════════╯

📍 当前环境
  Zig 编译器:
    版本:    0.13.0 (Stable)
    路径:    C:\Users\user\.zzm\versions\zig\0.13.0\zig.exe
    状态:    ✓ 活动中

  ZLS 语言服务器:
    版本:    0.13.0 (Stable)
    路径:    C:\Users\user\.zzm\versions\zls\0.13.0\zls.exe
    状态:    ✓ 活动中

  兼容性:    ✓ 完全匹配（推荐组合）

📦 已安装版本
  Zig:
    • 0.13.0 ✓ (默认)
    • 0.12.0
    • master

  ZLS:
    • 0.13.0 ✓ (默认)
    • 0.12.0

📂 目录信息
  安装根目录:  C:\Users\user\.zzm
  二进制目录:  C:\Users\user\.zzm\bin
  缓存目录:    C:\Users\user\.zzm\cache
  配置文件:    C:\Users\user\.zzm\config.toml

⚙️  配置
  默认编辑器:  vscode
  兼容性模式:  warn
  下载源:      official

💡 提示: 运行 'zzm --help' 查看所有可用命令
```

### 2.4 项目级版本管理

#### 2.4.1 初始化项目配置

```bash
# 在项目根目录创建 .zzmrc 配置文件
cd /path/to/my-project
zzm init

# 指定初始版本
zzm init --zig 0.13.0 --zls 0.13.0
```

**生成的 `.zzmrc` 文件示例**：

```json
{
  "zig": "0.13.0",
  "zls": "0.13.0",
  "compatibility": "strict",
  "ide": "vscode",
  "notes": "项目需要 Zig 0.13.0 的新特性"
}
```

#### 2.4.2 项目级版本切换

```bash
# 在项目目录下设置版本（自动更新 .zzmrc）
zzm use 0.13.0 --project

# 项目级切换会自动创建/更新 .zzmrc
```

**工作原理**：

```
项目目录结构:
my-project/
├── .zzmrc           # 项目配置文件
├── src/
│   └── main.zig
└── build.zig

当进入 my-project 目录时:
  1. zzm 自动检测 .zzmrc 文件
  2. 读取项目指定的版本
  3. 自动切换到项目版本（临时）
  4. 离开目录后恢复全局版本
```

#### 2.4.3 还原项目环境

```bash
# 新成员克隆项目后，还原项目依赖的版本
git clone https://github.com/user/my-project.git
cd my-project
zzm restore
```

**还原过程示例**：

```
⠋ 正在读取项目配置...
✓ 找到 .zzmrc 文件

项目要求:
  Zig: 0.13.0
  ZLS: 0.13.0

⠋ 正在检查已安装版本...
  Zig 0.13.0: ✓ 已安装
  ZLS 0.13.0: ✗ 未安装

⠋ 正在安装 ZLS 0.13.0...
████████████████████████████████████████  100%  12.8 MB/12.8 MB

✅ 项目环境还原完成！

当前环境:
  Zig: 0.13.0
  ZLS: 0.13.0
  兼容性: ✓ 完全匹配

💡 提示: 项目版本仅在当前目录生效，离开后恢复全局设置
```

### 2.5 IDE 集成

#### 2.5.1 VS Code 集成

```bash
# 生成 VS Code 配置
zzm ide config vscode
```

**生成的 `.vscode/settings.json`**：

```json
{
  "zig.path": "C:\\Users\\user\\.zzm\\bin\\zig.exe",
  "zig.zls.path": "C:\\Users\\user\\.zzm\\bin\\zls.exe",
  "zig.zls.enableAutofix": true,
  "[zig]": {
    "editor.defaultFormatter": "ziglang.vscode-zig",
    "editor.formatOnSave": true,
    "editor.codeActionsOnSave": {
      "source.fixAll": "explicit"
    }
  },
  "files.associations": {
    "*.zig": "zig"
  }
}
```

**手动配置步骤**：

1. 安装 VS Code 扩展：`Zig Language` (ziglang.vscode-zig)
2. 运行 `zzm ide config vscode`
3. 重启 VS Code 或重新加载窗口
4. 打开 `.zig` 文件，验证 ZLS 是否工作

#### 2.5.2 Neovim 集成

```bash
# 生成 Neovim LSP 配置片段
zzm ide config neovim
```

**输出的配置片段**：

```lua
-- 将以下内容添加到你的 Neovim 配置中
local zzm_zig = "C:\\Users\\user\\.zzm\\bin\\zig.exe"
local zzm_zls = "C:\\Users\\user\\.zzm\\bin\\zls.exe"

require('lspconfig').zls.setup({
  cmd = { zzm_zls },
  settings = {
    zls = {
      zig_exe_path = zzm_zig,
      enable_inlay_hints = true,
      enable_snippets = true,
      enable_ast_check_diagnostics = true,
      enable_autofix = true,
    }
  }
})

-- 可选: 自动格式化
vim.api.nvim_create_autocmd("BufWritePre", {
  pattern = "*.zig",
  callback = function()
    vim.lsp.buf.format({ async = false })
  end,
})
```

#### 2.5.3 Helix 集成

Helix 会自动检测 PATH 中的 `zig` 和 `zls`，无需额外配置。

```bash
# 确保 zzm 的 bin 目录在 PATH 中
zzm ide path
# 输出: C:\Users\user\.zzm\bin

# 添加到 PATH（如果尚未添加）
zzm ide path --add-to-path
```

#### 2.5.4 IDE 诊断工具

```bash
# 检查 IDE 配置状态
zzm ide check
```

**输出示例**：

```
⠋ 正在检查 IDE 配置...

VS Code:
  ✓ 找到 .vscode/settings.json
  ✓ Zig 路径配置正确
  ✓ ZLS 路径配置正确
  ✓ 扩展已安装: ziglang.vscode-zig
  ✓ ZLS 服务运行正常

Neovim:
  ⚠️ 未找到 LSP 配置
  💡 运行 'zzm ide config neovim' 生成配置

Helix:
  ✓ 自动检测到 zig 和 zls
  ✓ 无需额外配置

✅ IDE 配置检查完成
```

```bash
# 运行完整诊断
zzm ide doctor
```

**诊断输出示例**：

```
╭═══════════════════════════════════════════════════════════╮
│                    zzm 环境诊断报告                       │
╰═══════════════════════════════════════════════════════════╯

✓ zzm 版本: 0.1.0
✓ 安装目录: C:\Users\user\.zzm
✓ 配置文件: 存在且格式正确

✓ Zig:
  版本: 0.13.0
  路径: C:\Users\user\.zzm\bin\zig.exe
  可执行: 是
  运行测试: ✓ 通过

✓ ZLS:
  版本: 0.13.0
  路径: C:\Users\user\.zzm\bin\zls.exe
  可执行: 是
  LSP 测试: ✓ 通过

✓ 兼容性:
  Zig 0.13.0 + ZLS 0.13.0: ✓ 完全兼容

✓ PATH 环境变量:
  C:\Users\user\.zzm\bin 已在 PATH 中

✓ IDE 集成:
  VS Code: ✓ 配置正确
  Neovim: ⚠️ 未配置

✓ 网络连接:
  ziglang.org: ✓ 可访问
  github.com: ✓ 可访问

╭───────────────────────────────────────────────────────────╮
│ 诊断结果: 所有检查通过 ✓                                  │
╰───────────────────────────────────────────────────────────╯

💡 建议:
  • 运行 'zzm update' 检查 zzm 本身的更新
  • 定期运行 'zzm sync' 保持 Zig/ZLS 版本同步
```

---

## 3. 常见使用场景

### 3.1 场景一：新项目初始化

**需求**：创建一个新的 Zig 项目，配置开发环境

```bash
# 步骤 1: 创建项目目录
mkdir my-zig-project
cd my-zig-project

# 步骤 2: 初始化 Zig 项目（假设已有 build.zig）
# 或者使用 zig init-exe

# 步骤 3: 使用 zzm 初始化环境
zzm init --zig 0.13.0 --zls 0.13.0

# 步骤 4: 安装指定版本
zzm restore

# 步骤 5: 配置 IDE（VS Code）
zzm ide config vscode

# 步骤 6: 打开编辑器开始编码
code .
```

### 3.2 场景二：多项目并行开发

**需求**：同时维护两个使用不同 Zig 版本的项目

```bash
# 项目 A: 使用 Zig 0.12.0
cd ~/projects/project-a
zzm use 0.12.0 --project
# 自动创建 .zzmrc，指定版本为 0.12.0

# 项目 B: 使用 Zig 0.13.0
cd ~/projects/project-b
zzm use 0.13.0 --project
# 自动创建 .zzmrc，指定版本为 0.13.0

# 在项目 A 中工作
cd ~/projects/project-a
zig version  # 输出: 0.12.0
zzm current  # 输出: Zig 0.12.0 (项目级)

# 切换到项目 B
cd ~/projects/project-b
zig version  # 输出: 0.13.0
zzm current  # 输出: Zig 0.13.0 (项目级)

# 回到全局环境
cd ~
zig version  # 输出: 0.13.0 (全局默认)
```

### 3.3 场景三：团队协作

**需求**：团队成员统一开发环境

**团队负责人操作**：

```bash
# 1. 在项目中锁定版本
cd my-team-project
zzm use 0.13.0 --project

# 2. 提交 .zzmrc 到版本控制
git add .zzmrc
git commit -m "chore: 锁定 Zig 版本为 0.13.0"
git push
```

**团队成员操作**：

```bash
# 1. 克隆项目
git clone https://github.com/team/my-team-project.git
cd my-team-project

# 2. 还原项目环境
zzm restore
# 自动安装 .zzmrc 中指定的版本

# 3. 验证环境
zzm info
# 确认 Zig 和 ZLS 版本与团队一致

# 4. 开始开发
code .
```

### 3.4 场景四：尝试新版本特性

**需求**：在稳定版和开发版之间切换测试

```bash
# 安装最新稳定版和开发版
zzm install stable
zzm install master

# 在稳定版上开发
zzm use stable
zig build run

# 切换到开发版测试新特性
zzm use master
zig build run

# 发现问题，切回稳定版
zzm use stable

# 查看所有已安装版本
zzm list --installed
```

### 3.5 场景五：CI/CD 集成

**需求**：在 GitHub Actions 中使用 zzm

```yaml
# .github/workflows/test.yml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install zzm
        run: |
          curl -fsSL https://raw.githubusercontent.com/your-org/zzm/main/install.sh | bash
          echo "$HOME/.zzm/bin" >> $GITHUB_PATH

      - name: Restore project environment
        run: zzm restore --yes

      - name: Run tests
        run: zig build test

      - name: Build release
        run: zig build -Doptimize=ReleaseFast
```

**关键点**：
- 使用 `--yes` 标志启用非交互模式
- 将 `~/.zzm/bin` 添加到 `GITHUB_PATH`
- 使用 `.zzmrc` 锁定版本，确保 CI 与本地环境一致

---

## 4. 高级功能

### 4.1 配置管理

#### 4.1.1 查看和修改配置

```bash
# 列出所有配置项
zzm config list
```

**输出示例**：

```
配置文件: C:\Users\user\.zzm\config.toml

[general]
install_dir = "C:\\Users\\user\\.zzm"
download_mirror = "official"
no_color = false
verbose = false

[zig]
default = "0.13.0"
auto_update = false

[zls]
default = "0.13.0"
install_mode = "prebuilt"
build_optimize = "ReleaseSafe"

[ide]
default_editor = "vscode"
auto_config = true

[compatibility]
mode = "warn"
allow_mismatch = false
```

```bash
# 获取单个配置项
zzm config get zig.default
# 输出: 0.13.0

# 设置配置项
zzm config set zig.default 0.12.0
zzm config set zls.install_mode source

# 打开编辑器编辑配置文件
zzm config edit
```

#### 4.1.2 使用镜像源

```bash
# 设置国内镜像源（加速下载）
zzm config set general.download_mirror "https://mirror.example.com/zig"

# 或使用环境变量
export ZVM_DOWNLOAD_MIRROR="https://mirror.example.com/zig"
zzm install 0.13.0
```

### 4.2 缓存管理

```bash
# 查看缓存信息
zzm cache info
```

**输出示例**：

```
缓存目录: C:\Users\user\.zzm\cache

缓存文件:
  zig-0.13.0-x86_64-windows.tar.xz      45.2 MB  2026-04-20
  zls-0.13.0-x86_64-windows.zip         12.8 MB  2026-04-20
  zig-0.12.0-x86_64-windows.tar.xz      43.8 MB  2026-04-15

总大小: 101.8 MB
```

```bash
# 清空下载缓存
zzm cache clear

# 清理旧版本（保留默认和标记的版本）
zzm clean

# 清理特定版本
zzm uninstall 0.11.0
```

### 4.3 自动补全

#### Bash

```bash
# 生成 Bash 补全脚本
zzm completion bash > ~/.zzm/completion.bash

# 添加到 .bashrc
echo 'source ~/.zzm/completion.bash' >> ~/.bashrc
source ~/.bashrc
```

#### Zsh

```bash
# 生成 Zsh 补全脚本
zzm completion zsh > "${fpath[1]}/_zzm"

# 重新加载 shell
exec zsh
```

#### PowerShell

```powershell
# 生成 PowerShell 补全脚本
zzm completion powershell > $HOME\.zzm\completion.ps1

# 添加到 $PROFILE
Add-Content $PROFILE ". $HOME\.zzm\completion.ps1"
. $PROFILE
```

### 4.4 Shell 钩子（自动切换版本）

**需求**：进入项目目录时自动切换版本

#### Bash/Zsh

```bash
# 添加到 .bashrc 或 .zshrc
zzm_hook() {
  if [[ -f .zzmrc ]]; then
    zzm use --project --quiet
  fi
}
chpwd_functions+=(zzm_hook)
```

#### Fish

```fish
# 添加到 ~/.config/fish/config.fish
function zzm_hook --on-variable PWD
  if test -f .zzmrc
    zzm use --project --quiet
  end
end
```

---

## 5. 故障排除

### 5.1 常见问题

#### 问题 1: 命令未找到

**症状**：

```bash
zzm: command not found
```

**解决方案**：

```bash
# 检查 zzm 是否在 PATH 中
which zzm  # Linux/macOS
where zzm  # Windows

# 如果未找到，手动添加到 PATH
export PATH="$HOME/.zzm/bin:$PATH"  # Linux/macOS
$env:PATH += ";C:\Users\user\.zzm\bin"  # Windows PowerShell

# 永久添加（Linux/macOS）
echo 'export PATH="$HOME/.zzm/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

#### 问题 2: 下载失败

**症状**：

```
❌ 下载失败: Connection timeout
```

**解决方案**：

```bash
# 方案 1: 使用镜像源
zzm config set general.download_mirror "https://mirror.example.com/zig"

# 方案 2: 手动下载后安装
# 1. 从浏览器下载 zig-0.13.0-x86_64-windows.tar.xz
# 2. 将文件放入缓存目录
mv ~/Downloads/zig-0.13.0-x86_64-windows.tar.xz ~/.zzm/cache/

# 3. 再次运行安装命令（会使用缓存）
zzm install 0.13.0

# 方案 3: 使用代理
export https_proxy=http://proxy.example.com:8080
zzm install 0.13.0
```

#### 问题 3: 权限不足

**症状**：

```
❌ 权限不足: 无法创建符号链接
```

**解决方案**：

```bash
# Windows: 以管理员身份运行
# 或使用 shim 模式（zzm 会自动降级）

# Linux/macOS: 检查目录权限
ls -la ~/.zzm
sudo chown -R $(whoami) ~/.zzm
```

#### 问题 4: ZLS 无法启动

**症状**：

```
ZLS exited with status 1
```

**解决方案**：

```bash
# 1. 检查 ZLS 版本兼容性
zzm info
# 查看 "兼容性" 状态

# 2. 同步到推荐组合
zzm sync

# 3. 重新安装 ZLS
zzm zls uninstall 0.13.0
zzm zls install 0.13.0

# 4. 从源码编译（如果预编译版本有问题）
zzm zls install 0.13.0 --mode source

# 5. 运行诊断
zzm ide doctor
```

#### 问题 5: 版本切换不生效

**症状**：

```bash
zzm use 0.13.0
zig version  # 仍然显示旧版本
```

**解决方案**：

```bash
# 1. 检查 PATH 中是否有其他 zig
which zig  # Linux/macOS
where zig  # Windows

# 2. 确保 zzm 的 bin 目录在最前面
echo $PATH

# 3. 重新打开终端或重新加载 shell 配置
exec bash  # 或 exec zsh

# 4. 检查符号链接
ls -la ~/.zzm/bin/zig

# 5. 强制重建符号链接
zzm use 0.13.0 --force
```

### 5.2 日志和调试

```bash
# 启用详细日志
zzm --verbose install 0.13.0

# 查看日志文件
cat ~/.zzm/logs/zzm.log

# 运行诊断
zzm doctor
```

---

## 6. FAQ (常见问题解答)

### Q1: zzm 与其他 Zig 版本管理器有什么区别？

**A**:

| 特性 | zzm | zigup | zvm |
|-----|-----|-------|-----|
| Zig 版本管理 | ✅ | ✅ | ✅ |
| **独立 ZLS 管理** | ✅ **核心特性** | ❌ | ⚠️ 附属功能 |
| **智能兼容性检查** | ✅ | ❌ | ⚠️ 基础 |
| **项目级配置** | ✅ | ❌ | ✅ |
| **IDE 自动配置** | ✅ | ❌ | ⚠️ 手动 |
| 实现语言 | Rust | Zig | Go |

**zzm 的核心优势**：专注于 Zig + ZLS 的联合管理，提供智能的版本兼容性保证和无缝的 IDE 集成。

### Q2: zzm 会修改系统 PATH 吗？

**A**: 是的，但仅限于用户级别。zzm 会将 `~/.zzm/bin` 添加到用户的 PATH 环境变量中，不会修改系统级的 PATH。在 Windows 上，这通过修改用户环境变量实现；在 Linux/macOS 上，通过修改 shell 配置文件（如 `.bashrc`）实现。

### Q3: 如何卸载 zzm？

**A**:

```bash
# 1. 删除安装目录
rm -rf ~/.zzm  # Linux/macOS
rmdir /s C:\Users\user\.zzm  # Windows

# 2. 从 PATH 中移除
# 编辑 ~/.bashrc 或 ~/.zshrc，删除相关行
# 或在 Windows 中编辑用户环境变量

# 3. 删除二进制文件
sudo rm /usr/local/bin/zzm  # Linux/macOS
del C:\Users\user\AppData\Local\Microsoft\WindowsApps\zzm.exe  # Windows
```

### Q4: zzm 支持哪些 Zig 版本？

**A**: zzm 支持所有官方发布的 Zig 版本，包括：
- 稳定版（0.11.0, 0.12.0, 0.13.0 等）
- 开发版（master/nightly）
- 历史版本（0.8.0 及以后）

### Q5: 如何在多台机器间同步配置？

**A**: 有两种方式：

**方式 1**: 同步配置文件

```bash
# 将 ~/.zzm/config.toml 放入云同步或版本控制
ln -s ~/Dropbox/dotfiles/zzm-config.toml ~/.zzm/config.toml
```

**方式 2**: 使用项目级配置

```bash
# 在项目中使用 .zzmrc，提交到版本控制
git add .zzmrc
# 其他机器克隆后运行 zzm restore
```

### Q6: zzm 会自动更新 Zig 吗？

**A**: 默认不会。zzm 遵循"显式优于隐式"原则，需要用户手动执行更新：

```bash
# 更新到最新的稳定版
zzm install stable --force

# 更新到最新的开发版
zzm install master --force
```

但你可以通过配置启用自动检查更新提示：

```bash
zzm config set zig.auto_update_check true
```

### Q7: 如何从其他版本管理器迁移到 zzm？

**A**:

```bash
# 从 zigup 迁移
# 1. 卸载 zigup
# 2. 安装 zzm
# 3. zzm 会自动检测已有的 Zig 安装（如果路径兼容）

# 从 zvm 迁移
# 1. 导出当前版本信息
zvm current > /tmp/zvm-version.txt

# 2. 卸载 zvm

# 3. 安装 zzm 并还原版本
zzm install $(cat /tmp/zvm-version.txt)
```

### Q8: zzm 支持离线安装吗？

**A**: 支持。你可以：

1. 提前下载好 Zig 和 ZLS 的压缩包
2. 将文件放入 `~/.zzm/cache/` 目录
3. 运行 `zzm install <version>`，zzm 会使用缓存文件

---

## 7. 附录

### 7.1 命令速查表

| 命令 | 说明 | 示例 |
|------|------|------|
| `zzm install <ver>` | 安装 Zig 版本 | `zzm install 0.13.0` |
| `zzm uninstall <ver>` | 卸载版本 | `zzm uninstall 0.12.0` |
| `zzm list` | 列出版本 | `zzm list --installed` |
| `zzm use <ver>` | 切换版本 | `zzm use 0.13.0 --project` |
| `zzm current` | 显示当前版本 | `zzm current` |
| `zzm zls install <ver>` | 安装 ZLS | `zzm zls install 0.13.0` |
| `zzm zls use <ver>` | 切换 ZLS | `zzm zls use 0.13.0` |
| `zzm setup <ver>` | 一键初始化 | `zzm setup 0.13.0 --with-zls` |
| `zzm sync` | 同步版本 | `zzm sync` |
| `zzm info` | 显示详细信息 | `zzm info` |
| `zzm config` | 配置管理 | `zzm config list` |
| `zzm ide config` | IDE 配置 | `zzm ide config vscode` |
| `zzm doctor` | 诊断程序 | `zzm doctor` |
| `zzm clean` | 清理工具 | `zzm clean` |

### 7.2 环境变量

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| `ZZM_HOME` | zzm 安装目录 | `~/.zzm` |
| `ZZM_CACHE_DIR` | 缓存目录 | `~/.zzm/cache` |
| `ZZM_CONFIG_FILE` | 配置文件路径 | `~/.zzm/config.toml` |
| `ZZM_DOWNLOAD_MIRROR` | 下载镜像源 | 官方源 |
| `ZZM_NO_COLOR` | 禁用彩色输出 | `false` |
| `ZZM_VERBOSE` | 详细输出 | `false` |

### 7.3 相关资源

- **官方文档**: https://github.com/your-org/zzm/docs
- **GitHub 仓库**: https://github.com/your-org/zzm
- **问题反馈**: https://github.com/your-org/zzm/issues
- **Zig 官网**: https://ziglang.org/
- **ZLS 仓库**: https://github.com/zigtools/zls
- **Zig 语言圣经**: https://course.ziglang.cc/

---

*本文档持续更新中，如有疑问请提交 Issue 或 Pull Request。*
