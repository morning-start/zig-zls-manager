# Zig/ZLS 版本管理器 (zig-zls-manager) 需求规格说明书

## 📋 文档信息

- **版本**: v1.0.0
- **创建日期**: 2026-04-23
- **状态**: 初稿
- **作者**: zig-zls-manager 项目组

---

## 1. 项目概述

### 1.1 项目背景

Zig 语言正在快速发展中（当前版本范围 0.11-0.15+），尚未发布 1.0 正式版。由于：
- **版本迭代频繁**：Zig 大约每 6 个月随 LLVM 新版本发布一次稳定版，同时有频繁的 nightly 构建
- **配套工具依赖**：ZLS（Zig Language Server）作为 Zig 的语言服务器，需要与特定 Zig 版本匹配才能正常工作
- **多项目需求**：开发者可能同时维护多个使用不同 Zig 版本的项目
- **现有工具不足**：
  - `zigup`：只管理 Zig 编译器，不支持 ZLS
  - `zvm`（hendriknielaender）：虽然支持 `--zls` 参数，但 ZLS 管理功能是附属于 Zig 安装的，缺乏独立的 ZLS 版本管理能力
  - 其他通用版本管理器（nvm, rbenv, pyenv）：只针对单一语言，无法处理编译器+语言服务器的组合管理

### 1.2 项目目标

构建一个专业的 **Zig + ZLS 联合版本管理 CLI 工具**，实现：
- ✅ 同时管理 Zig 编译器和 ZLS 语言服务器版本
- ✅ 智能的版本兼容性匹配和校验
- ✅ 灵活的全局/项目级版本切换
- ✅ 无缝的 IDE 集成支持
- ✅ 优雅的开发体验和命令行交互

### 1.3 目标用户

| 用户类型 | 使用场景 | 核心需求 |
|---------|---------|---------|
| **个人开发者** | 多项目并行开发 | 快速切换版本组合 |
| **团队协作者** | 统一开发环境 | 版本一致性保证 |
| **学习者** | 尝试不同版本特性 | 便捷安装和对比 |
| **CI/CD 工程师** | 自动化构建流水线 | 可脚本化的版本管理 |

---

## 2. 功能需求

### 2.1 核心功能模块

#### 2.1.1 Zig 版本管理

**功能描述**：管理 Zig 编译器的完整生命周期

| 命令 | 说明 | 示例 |
|-----|------|------|
| `install` | 安装指定版本的 Zig | `zzm install 0.13.0`<br>`zzm install master` |
| `uninstall` | 卸载指定版本 | `zzm uninstall 0.12.0` |
| `list` | 列出已安装/可用的版本 | `zzm list --installed`<br>`zzm list --remote` |
| `use` | 切换当前使用的版本 | `zzm use 0.13.0`<br>`zzm use 0.13.0 --project` |
| `current` | 显示当前激活的版本 | `zzm current` |

**特殊版本标识符**：
- `master` / `nightly`: 最新开发版
- `stable`: 最新稳定版
- `0.13`: 自动解析为最新的 0.13.x 补丁版本（如 0.13.0）
- `.13`: 同上（简写形式）

#### 2.1.2 ZLS 版本管理

**功能描述**：独立管理 ZLS 语言服务器的版本，支持与 Zig 的关联

| 命令 | 说明 | 示例 |
|-----|------|------|
| `zls install` | 安装指定版本的 ZLS | `zzm zls install 0.13.0`<br>`zzm zls install master` |
| `zls uninstall` | 卸载指定版本 | `zzm zls uninstall 0.13.0` |
| `zls list` | 列出已安装/可用的 ZLS 版本 | `zzm zls list` |
| `zls use` | 切换当前使用的 ZLS 版本 | `zzm zls use 0.13.0` |
| `zls current` | 显示当前激活的 ZLS 版本 | `zzm zls current` |

**智能特性**：
- 🔗 **自动关联**：安装 Zig 时可选择自动安装匹配的 ZLS (`--with-zls`)
- 🔍 **兼容性检查**：切换版本时验证 Zig/ZLS 兼容性
- 📦 **多源安装**：支持预编译二进制下载 或从源码编译 (`--from-source`)

#### 2.1.3 联合管理功能

**功能描述**：Zig + ZLS 协同管理的增强功能

| 命令 | 说明 | 示例 |
|-----|------|------|
| `setup` | 一键初始化开发环境 | `zzm setup 0.13.0 --with-zls` |
| `sync` | 同步 Zig 和 ZLS 到推荐组合 | `zzm sync` |
| `pair` | 手动绑定 Zig/ZLS 版本关系 | `zzm pair 0.13.0 0.13.0-zls` |
| `info` | 显示当前环境完整信息 | `zzm info` |

#### 2.1.4 IDE 集成

**功能描述**：为主流编辑器提供集成支持

**支持的编辑器**：
- ✅ VS Code
- ✅ Neovim
- ✅ Helix
- ✅ JetBrains 系列（通过插件）
- ✅ Emacs
- ✅ Sublime Text

**提供的功能**：
```bash
# 生成 IDE 配置
zzm ide config vscode    # 生成 VS Code settings.json
zzm ide config neovim    # 生成 Neovim lspconfig 配置
zzm ide path             # 输出当前 zig/zls 路径供 IDE 引用

# 自动检测和提示
zzm ide check            # 检测 IDE 配置状态
zzm ide doctor           # 诊断常见问题
```

### 2.2 高级功能

#### 2.2.1 项目级版本管理

```bash
# 在项目目录下创建 .zzmrc 配置文件
zzm init                 # 初始化项目配置文件
zzm use 0.13.0 --project # 设置项目专属版本

# .zzmrc 示例内容
{
  "zig": "0.13.0",
  "zls": "0.13.0",
  "compatibility": "strict"  // strict | loose | auto
}
```

**优先级机制**：
```
项目级 (.zzmrc) > 用户全局 (~/.zzm/config.toml) > 系统默认
```

#### 2.2.2 版本兼容性矩阵

**自动维护 Zig ↔ ZLS 版本兼容性映射表**：

| Zig 版本 | 推荐 ZLS 版本 | 兼容性说明 |
|---------|--------------|-----------|
| 0.11.x | zls 0.11.x | 完全兼容 |
| 0.12.x | zls 0.12.x | 完全兼容 |
| 0.13.x | zls 0.13.x | 完全兼容 |
| 0.14.x | zls 0.14.x | 完全兼容 |
| master | zls master | 可能存在不稳定 |

**警告机制**：
- ⚠️ 版本不匹配时发出警告
- ❌ 严重不兼容时阻止操作并给出建议
- 💡 推荐最佳匹配版本

#### 2.2.3 配置管理

```bash
# 查看和修改配置
zzm config list          # 列出所有配置项
zzm config get <key>     # 获取配置值
zzm config set <key> <val> # 设置配置值
zzm config edit          # 打开编辑器编辑配置文件
```

**主要配置项**：

```toml
# ~/.zzm/config.toml

[general]
# 安装目录
install_dir = "~/.zzm/versions"
# 默认下载源
download_mirror = "official"  # official | github | mirror_url

[zig]
# 默认 Zig 版本
default = "0.13.0"
# 是否自动更新
auto_update = false

[zls]
# 默认 ZLS 版本（留空表示跟随 Zig）
default = ""
# 安装方式: prebuilt | source
install_mode = "prebuilt"
# 从源码编译时的优化选项
build_optimize = "ReleaseSafe"

[ide]
# 默认集成的编辑器
default_editor = "vscode"
# 是否自动生成配置
auto_config = true

[compatibility]
# 兼容性检查级别: strict | loose | warn
mode = "warn"
# 是否允许不兼容的组合
allow_mismatch = false
```

### 2.3 辅助功能

#### 2.3.1 信息查询

```bash
# 详细的环境信息展示
zzm info
# 输出示例：
# ════════════════════════════════════════
#  Zig/ZLS Version Manager v0.1.0
# ════════════════════════════════════════
#
#  Zig Compiler:
#    Version:  0.13.0
#    Path:     C:\Users\user\.zzm\versions\zig\0.13.0\zig.exe
#    Status:   ✓ Active
#
#  ZLS:
#    Version:  0.13.0
#    Path:     C:\Users\user\.zzm\versions\zls\0.13.0\zls.exe
#    Status:   ✓ Active
#
#  Compatibility: ✓ Matched (Recommended Pair)
#
#  Installed Versions:
#    Zig: 0.11.0, 0.12.0, 0.13.0, master
#    ZLS: 0.11.0, 0.12.0, 0.13.0, master
#
#  Project Config: None (using global default)
# ════════════════════════════════════════
```

#### 2.3.2 清理和维护

```bash
zzm clean                # 清理旧版本缓存
zzm prune                # 移除未标记保留的旧版本
zzm cache clear          # 清空下载缓存
zzm doctor               # 运行诊断程序
zzm update self          # 更新 zzm 本身
```

#### 2.3.3 自动补全

```bash
# 生成 shell 自动补全脚本
zzm completion bash      # Bash 补全
zzm completion zsh       # Zsh 补全
zzm completion powershell # PowerShell 补全
zzm completion fish      # Fish 补全
```

---

## 3. 非功能性需求

### 3.1 性能要求

| 指标 | 要求 | 说明 |
|-----|------|------|
| **启动速度** | < 100ms | CLI 冷启动时间 |
| **版本切换** | < 500ms | 符号链接/路径更新 |
| **安装速度** | 取决于网络 | 下载 + 解压，显示进度条 |
| **内存占用** | < 50MB | 运行时内存 |

### 3.2 兼容性要求

**操作系统支持**：
- ✅ Windows 10/11 (x86_64)
- ✅ macOS 10.15+ (Intel & Apple Silicon)
- ✅ Linux (x86_64, aarch64)

**终端支持**：
- PowerShell / CMD / Git Bash (Windows)
- Terminal.app / iTerm2 (macOS)
- 所有主流终端模拟器 (Linux)

### 3.3 可用性要求

- **错误信息友好**：清晰的错误提示和解决方案建议
- **交互式引导**：首次使用提供 setup wizard
- **色彩输出**：支持彩色日志输出（可通过 `--no-color` 关闭）
- **进度显示**：长时间操作显示进度条

### 3.4 安全性要求

- **完整性校验**：下载文件进行 SHA256 校验
- **权限控制**：避免不必要的管理员权限请求
- **安全下载**：仅从官方或可信镜像源下载

---

## 4. 用户故事 (User Stories)

### US-01: 快速开始新项目

**作为** 一个刚接触 Zig 的开发者
**我想要** 一键安装 Zig 和 ZLS 并配置好 IDE
**以便于** 能够立即开始编码而无需手动配置环境

**验收标准**：
- Given 我刚安装了 zzm
- When 执行 `zzm setup 0.13.0 --with-zls --ide vscode`
- Then 自动下载并安装 Zig 0.13.0 和对应的 ZLS
- And 生成 VSCode 配置文件
- And 提示我重启 VSCode 以生效

### US-02: 多项目并行开发

**作为** 一个同时维护多个 Zig 项目的开发者
**我想要** 在不同项目目录下自动使用不同的 Zig/ZLS 版本
**以便于** 避免版本冲突且无需手动切换

**验收标准**：
- Given 我有两个项目 A (需要 0.12.0) 和 B (需要 0.13.0)
- When 在项目 A 中执行 `zzm use 0.12.0 --project`
- And 在项目 B 中执行 `zzm use 0.13.0 --project`
- When 分别在这两个目录打开终端
- Then 项目 A 自动使用 Zig 0.12.0 + 匹配的 ZLS
- And 项目 B 自动使用 Zig 0.13.0 + 匹配的 ZLS

### US-03: 智能版本兼容

**作为** 一个开发者
**我想要** 系统提醒我当前 Zig 和 ZLS 版本是否兼容
**以便于** 避免因版本不匹配导致的奇怪错误

**验收标准**：
- Given 当前使用 Zig 0.13.0 和 ZLS 0.12.0
- When 执行 `zzm info` 或任何命令
- Then 显示警告："⚠️ ZLS version mismatch detected"
- And 推荐升级到 ZLS 0.13.0
- When 执行 `zzm sync`
- Then 自动将 ZLS 切换到推荐的 0.13.0 版本

### US-04: 团队环境统一

**作为** 团队技术负责人
**我想要** 通过配置文件锁定团队的 Zig/ZLS 版本
**以便于** 保证所有成员的开发环境一致

**验收标准**：
- Given 项目中有 `.zzmrc` 文件指定了版本
- When 新成员克隆项目并执行 `zzm restore`
- Then 自动安装 `.zzmrc` 中指定的所有版本
- And 配置好全局和项目级别的版本设置

### US-05: CI/CD 集成

**作为** CI/CD 工程师
**我想要** 在脚本中精确控制 Zig 和 ZLS 的版本
**以便于** 构建流水线可重复且可靠

**验收标准**：
- Given 一个 CI 环境
- When 执行 `zzm install 0.13.0 --with-zls --yes`
- Then 在非交互模式下静默安装指定版本
- And 可以通过环境变量覆盖默认配置
- And 安装失败时返回非零退出码

---

## 5. 约束条件

### 5.1 技术约束

- **实现语言**：Rust (Edition 2024)
- **包管理**：Cargo
- **CLI 框架**：待选型（clap, pico-args 等）
- **运行时**：无额外运行时依赖（静态链接）
- **单二进制分发**：所有平台提供单一可执行文件

### 5.2 业务约束

- **开源协议**：MIT 或 Apache-2.0
- **文档语言**：中文（主）+ 英文（次）
- **版本号规范**：遵循 SemVer (语义化版本)

### 5.3 用户体验约束

- **学习成本**：熟悉 nvm/rbm 的开发者应在 5 分钟内上手
- **命令风格**：遵循现代 CLI 最佳实践（类似 git, rustup）
- **输出格式**：默认人类可读，支持 `--json` 输出供机器解析

---

## 6. 术语表

| 术语 | 定义 |
|-----|------|
| **ZZM** | Zig-Zls Manager 的缩写，本工具的命令名称 |
| **Zig** | Zig 编程语言编译器 |
| **ZLS** | Zig Language Server，Zig 的语言服务器协议实现 |
| **Version** | 语义化版本号（如 0.13.0）或特殊标识符（master, stable） |
| **Channel** | 版本通道（stable, nightly/master, maintenance） |
| **Global** | 全局（用户级别）配置，影响所有 shell 会话 |
| **Local/Project** | 项目级别配置，仅在特定项目目录生效 |
| **Compatibility Matrix** | Zig 与 ZLS 版本兼容性映射表 |

---

## 7. 附录

### A. 参考资源

- [Zig 官方网站](https://ziglang.org/)
- [Zig 语言圣经](https://course.ziglang.cc/)
- [ZLS GitHub 仓库](https://github.com/zigtools/zls)
- [zigup - Zig 版本管理器](https://github.com/marler8997/zigup)
- [zvm - Zig Version Manager](https://github.com/hendriknielaender/zvm)

### B. 竞品对比

| 特性 | zigup | zvm (hendriknielaender) | **zig-zls-manager (本项目)** |
|-----|-------|------------------------|---------------------------|
| Zig 版本管理 | ✅ | ✅ | ✅ |
| ZLS 版本管理 | ❌ | ⚠️ 附属功能 | ✅ **独立完整支持** |
| 版本兼容性检查 | ❌ | ⚠️ 基础 | ✅ **智能矩阵** |
| 项目级配置 | ❌ | ✅ | ✅ |
| IDE 集成 | ❌ | ⚠️ 手动 | ✅ **自动生成** |
| 跨平台 | ✅ | ✅ | ✅ |
| Rust 实现 | ❌ (Zig) | ❌ (Go) | ✅ |
| 单二进制 | ✅ | ✅ | ✅ |

### C. 版本历史

| 版本 | 日期 | 作者 | 修改内容 |
|-----|------|------|---------|
| v1.0.0 | 2026-04-23 | 项目组 | 初始版本，完成需求梳理 |

---

*本文档将随着项目演进持续更新。*
