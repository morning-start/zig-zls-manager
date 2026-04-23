# Zig/ZLS 版本管理器 - 开发任务清单

## 📋 文档信息

- **版本**: v1.0.0
- **创建日期**: 2026-04-24
- **当前阶段**: Phase 1 - MVP (v0.1.0)
- **关联文档**: [ROADMAP.md](./ROADMAP.md), [architecture.md](./architecture.md)

---

## 🎯 当前目标：Phase 1 MVP (v0.1.0)

**目标日期**: 2026年7月中旬（10周开发周期）

**核心交付物**:
- ✅ 可用的 Zig 版本管理（install/uninstall/list/use/current）
- ✅ 基础 ZLS 管理（--with-zls 参数）
- ✅ VS Code IDE 集成
- ✅ 完整的测试套件
- ✅ CI/CD 基础配置

---

## 📅 Sprint 规划

### Sprint 0: 项目初始化 (第 1 周)

**目标**: 搭建项目骨架，确保可编译运行

#### 任务列表

- [ ] **[P0] T-001: 初始化 Rust 项目**
  - 使用 `cargo init --name zzm` 创建项目
  - 配置 Cargo.toml（Edition 2024）
  - 创建目录结构：
    ```
    src/
    ├── main.rs
    ├── cli.rs
    ├── core/
    │   ├── mod.rs
    │   ├── zig_manager.rs
    │   ├── zls_manager.rs
    │   └── compatibility.rs
    ├── infra/
    │   ├── mod.rs
    │   ├── downloader.rs
    │   ├── filesystem.rs
    │   ├── path_manager.rs
    │   └── checksum.rs
    ├── platform/
    │   ├── mod.rs
    │   └── trait.rs
    ├── output/
    │   └── mod.rs
    └── utils/
        ├── error.rs
        └── version.rs
    tests/
    └── docs/
    ```

- [ ] **[P0] T-002: 配置核心依赖**
  在 Cargo.toml 中添加：
  ```toml
  [dependencies]
  clap = { version = "4", features = ["derive"] }
  tokio = { version = "1", features = ["full"] }
  reqwest = { version = "0.11", features = ["json", "stream"] }
  serde = { version = "1", features = ["derive"] }
  serde_json = "1"
  toml = "0.8"
  anyhow = "1"
  thiserror = "1"
  tracing = "0.1"
  console = "0.15"
  tabled = "0.15"
  indicatif = "0.17"
  dialoguer = "0.11"
  semver = "1"
  sha2 = "0.10"
  flate2 = "1"
  zip = "0.6"
  dirs = "5"

  [dev-dependencies]
  tempfile = "3"
  assert_cmd = "2"
  ```

- [ ] **[P0] T-003: 定义错误类型体系**
  文件: `src/utils/error.rs`
  ```rust
  use thiserror::Error;

  #[derive(Error, Debug)]
  pub enum ZzmError {
      #[error("IO 错误: {0}")]
      Io(#[from] std::io::Error),

      #[error("网络请求失败: {0}")]
      Network(#[from] reqwest::Error),

      #[error("下载失败: {url} ({reason})")]
      DownloadFailed { url: String, reason: String },

      #[error("无效的版本号: {version}")]
      InvalidVersion { version: String },

      #[error("版本 '{version}' 未找到")]
      VersionNotFound { version: String },

      #[error("版本 '{version}' 已安装")]
      AlreadyInstalled { version: String },

      #[error("校验和不匹配")]
      ChecksumMismatch,

      #[error("配置错误: {0}")]
      Config(String),

      #[error("不支持的平台")]
      UnsupportedPlatform,
  }

  pub type Result<T> = std::result::Result<T, ZzmError>;
  ```

- [ ] **[P0] T-004: 初始化日志系统**
  文件: `src/main.rs` (或 src/lib.rs)
  ```rust
  use tracing_subscriber;

  fn init_logging(verbose: bool) {
      let level = if verbose {
          tracing::Level::DEBUG
      } else {
          tracing::Level::INFO
      };

      tracing_subscriber::fmt()
          .with_max_level(level)
          .init();
  }
  ```

**验收标准**:
- ✅ `cargo build` 编译通过
- ✅ `cargo test` 运行无报错
- ✅ `cargo clippy` 无警告

---

### Sprint 1: CLI 框架 + 平台抽象 (第 2 周)

**目标**: 实现命令行解析和平台适配层

#### 任务列表

- [ ] **[P0] T-005: 实现 CLI 命令定义**
  文件: `src/cli.rs`
  - 定义 `Cli` 结构体（全局选项）
  - 定义 `Commands` 枚举（子命令）
  - 实现 `install`, `uninstall`, `list`, `use`, `current` 子命令
  - 实现 `zls` 子命令组
  - 实现 `ide`, `config`, `setup` 子命令组
  - 参考 architecture.md 第 2.1.1 节的完整命令树

- [ ] **[P0] T-006: 实现 Platform trait 和适配器**
  文件: `src/platform/trait.rs`, `windows.rs`, `macos.rs`, `linux.rs`
  - 定义 `PlatformTrait` trait
  - 实现 WindowsPlatform（shim 文件、注册表操作）
  - 实现 MacOSPlatform（符号链接、plist）
  - 实现 LinuxPlatform（符号链接、.bashrc/.zshrc）
  - 实现运行时平台检测函数

- [ ] **[P1] T-007: 实现输出格式化模块**
  文件: `src/output/mod.rs`
  - 成功消息（绿色 ✓）
  - 警告消息（黄色 ⚠）
  - 错误消息（红色 ✗）
  - 表格输出封装
  - JSON 输出模式开关

**验收标准**:
- ✅ `zzm --help` 显示完整的帮助信息
- ✅ `zzm --version` 显示版本号
- ✅ 所有子命令都有帮助文本
- ✅ 平台检测正确识别 OS

---

### Sprint 2: 外部 API 集成 (第 3-4 周)

**目标**: 实现对 Zig 和 ZLS API 的访问能力

#### 任务列表

- [ ] **[P0] T-008: 实现 Zig JSON API 客户端**
  文件: `src/infra/zig_api.rs` (新建)
  - 定义数据结构（参考 api-reference.md 第 2 章）
    ```rust
    struct ZigDownloadIndex { master: ZigVersionInfo }
    struct ZigVersionInfo { date, platforms: PlatformBinaries }
    struct PlatformBinaries { windows, macos, linux: Vec<BinaryAsset> }
    struct BinaryAsset { os, arch, filename, size, shasum, url }
    ```
  - 实现 `fetch_zig_versions()` 异步函数
  - 实现本地缓存（文件系统 + TTL）
  - 实现平台自动匹配逻辑
  - 单元测试：Mock 数据验证解析逻辑

- [ ] **[P0] T-009: 实现 ZLS GitHub API 客户端**
  文件: `src/infra/zls_api.rs` (新建)
  - 定义数据结构（参考 api-reference.md 第 3 章）
    ```rust
    struct GithubRelease { tag_name, name, prerelease, assets }
    struct GithubAsset { id, name, size, browser_download_url }
    ```
  - 实现 `fetch_zls_releases()` 异步函数
  - 实现分页处理（per_page=100）
  - 实现认证头支持（可选 GitHub Token）
  - 实现速率限制处理（检查 X-RateLimit-Remaining）
  - 实现稳定版本过滤（排除 prerelease/draft）
  - 单元测试：Mock 数据验证

- [ ] **[P0] T-010: 实现版本解析工具**
  文件: `src/utils/version.rs`
  - 解析完整版本号："0.13.0" → Version { major: 0, minor: 13, patch: 0 }
  - 解析简写版本："0.13" → 查询最新 0.13.x
  - 特殊标识符处理："master", "stable", "nightly"
  - 版本比较和排序
  - VersionRange 匹配（用于兼容性矩阵）

- [ ] **[P1] T-011: 实现下载管理器**
  文件: `src/infra/downloader.rs`
  - HTTP GET 请求（reqwest）
  - 流式下载（bytes_stream）
  - 进度回调接口
  - 超时配置（connect timeout, read timeout）
  - 重试机制（3次，指数退避：100ms, 200ms, 400ms）
  - User-Agent 设置："zzm/{version}"
  - 集成测试：实际下载小文件验证

**验收标准**:
- ✅ `zzm list --remote` 能返回 Zig 远程版本列表
- ✅ `zzm zls list --remote` 能返回 ZLS releases 列表
- ✅ API 响应被正确缓存（第二次请求更快）
- ✅ 网络超时有合理的错误提示

---

### Sprint 3: 核心安装流程 (第 5-6 周)

**目标**: 实现 Zig 安装、卸载、切换的完整流程

#### 任务列表

- [ ] **[P0] T-012: 实现路径管理器**
  文件: `src/infra/path_manager.rs`
  - 初始化 ~/.zzm/ 目录结构
    ```
    ~/.zzm/
    ├── bin/              # 符号链接/shim
    ├── versions/
    │   ├── zig/          # 各版本 Zig
    │   └── zls/          # 各版本 ZLS
    ├── cache/            # 下载缓存
    ├── config.toml       # 全局配置
    └── installed.json    # 已安装版本索引
    ```
  - 创建/删除符号链接（跨平台）
  - Windows shim 文件生成
  - PATH 更新提示生成
  - 目录清理工具方法

- [ ] **[P0] T-013: 实现文件系统操作模块**
  文件: `src/infra/filesystem.rs`
  - tar.gz 解压（使用 flate2 + tar crate）
  - zip 解压（使用 zip crate）
  - 自动检测压缩格式
  - 安全解压（防止路径遍历攻击）
  - 文件权限设置（Unix executable bit）

- [ ] **[P0] T-014: 实现校验和验证模块**
  文件: `src/infra/checksum.rs`
  - SHA256 计算（sha2 crate）
  - 与预期值比对
  - 错误时提供详细信息（期望值 vs 实际值）
  - 可选：minisign 签名验证

- [ ] **[P0] T-015: 实现 ZigManager 核心逻辑**
  文件: `src/core/zig_manager.rs`
  数据结构：
  ```rust
  struct ZigVersion { version, channel, download_url, checksum_sha256, size }
  struct ZigInstallation { version, install_path, installed_at, is_active }
  ```
  核心方法：
  - `install(version, options)` → 下载 → 校验 → 解压 → 注册
  - `uninstall(version)` → 删除目录 → 清理链接 → 更新元数据
  - `list_installed()` → 读取 installed.json → 返回列表
  - `list_remote()` → 调用 Zig API → 格式化返回
  - `use_version(version, scope)` → 更新符号链接 → 写入 .zzmrc（如果 project scope）
  - `current()` → 读取符号链接目标 → 返回版本信息
  - `resolve_version(input)` → 版本号解析与补全

- [ ] **[P0] T-016: 集成 install/uninstall/list/use/current 命令**
  文件: `src/main.rs` 或独立的 command handler 模块
  - 将 CLI 子命令路由到 ZigManager 方法
  - 处理参数验证
  - 格式化输出结果
  - 错误处理和用户友好提示

- [ ] **[P1] T-017: 实现缓存管理器**
  文件: `src/infra/cache.rs`
  - 缓存存储路径管理
  - TTL 过期检查
  - 缓存清理命令
  - 磁盘空间监控
  - LRU 淘汰策略（可选）

**验收标准**:
- ✅ `zzm install 0.13.0` 成功安装并可用
- ✅ `zig version` 显示正确的版本号
- ✅ `zzm list --installed` 显示已安装版本
- ✅ `zzm use 0.12.0` 成功切换
- ✅ `zzm uninstall 0.11.0` 成功卸载
- ✅ 下载过程中显示进度条
- ✅ 校验失败时有明确错误信息

---

### Sprint 4: ZLS 集成 + IDE 支持 (第 7-8 周)

**目标**: 实现 --with-zls 功能和 VS Code 集成

#### 任务列表

- [ ] **[P0] T-018: 实现 ZLSManager 基础逻辑**
  文件: `src/core/zls_manager.rs`
  数据结构：
  ```rust
  struct ZlsVersion { version, compatible_zig, download_url, source_repo }
  struct ZlsInstallation { version, install_path, zig_version, install_mode }
  ```
  核心方法：
  - `find_compatible_version(zig_version)` → 根据 Zig 版本查找匹配的 ZLS
  - `install_from_release(version)` → 从 GitHub Releases 下载预编译版
  - `list_installed()` / `current()`
  - `use_version(version)`

- [ ] **[P0] T-019: 实现 --with-zls 参数集成**
  - 修改 `install` 命令处理逻辑
  - 并行下载 Zig + ZLS（tokio::join!）
  - 自动匹配兼容版本
  - 统一进度显示（两个进度条？或合并？）
  - 原子性保证：任一失败则回滚两者

- [ ] **[P0] T-020: 实现 VS Code IDE 集成**
  文件: `src/core/ide/vscode.rs`
  - 生成 `.vscode/settings.json`
    ```json
    {
      "zig.path": "<path_to_zig>",
      "zig.zls.path": "<path_to_zls>",
      "[zig]": {
        "editor.defaultFormatter": "ziglang.vscode-zig",
        "editor.formatOnSave": true
      }
    }
  - 实现 `zzm ide config vscode` 命令
  - 检测是否已有 .vscode/settings.json（合并而非覆盖）

- [ ] **[P0] T-021: 实现 `zzm ide path` 命令**
  - 输出当前 zig 和 zls 的绝对路径
  - 支持 `--json` 格式
  - 用途：供其他脚本或工具引用

- [ ] **[P1] T-022: 实现基础兼容性检查**
  文件: `src/core/compatibility.rs`
  - 内置硬编码的兼容性规则（v0.11→zls 0.11, v0.13→zls 0.13 等）
  - `check(zig_ver, zls_ver)` → CompatibilityStatus enum
  - 在 `use` 和 `install --with-zls` 时自动调用
  - 不匹配时输出警告（不阻止操作）

**验收标准**:
- ✅ `zzm install 0.13.0 --with-zls` 同时安装两个工具
- ✅ `zzm zls current` 显示当前 ZLS 版本
- ✅ `zzm ide config vscode` 生成正确的 settings.json
- ✅ `zzm ide path` 输出正确的路径
- ✅ 版本不匹配时有警告提示

---

### Sprint 5: 配置管理 + 测试完善 (第 9 周)

**目标**: 完善配置系统和测试覆盖率

#### 任务列表

- [ ] **[P0] T-023: 实现配置管理器**
  文件: `src/core/config.rs`
  - TOML 配置文件读写
  - 配置结构体定义（参考 architecture.md 2.2.4 节）
  - 默认值处理
  - 多层配置合并（项目 > 用户 > 系统 > 内置）
  - 实现 `config list/get/set/edit` 命令

- [ ] **[P0] T-024: 补充单元测试**
  目标覆盖率 > 70%
  - 版本解析逻辑测试（边界情况）
  - API 客户端 Mock 测试
  - 文件路径计算测试
  - 配置合并逻辑测试
  - 兼容性规则匹配测试

- [ ] **[P0] T-025: 编写集成测试**
  文件: `tests/integration/`
  - `test_install_flow.rs`: 完整安装流程测试
  - `test_switch_flow.rs`: 版本切换流程测试
  - `test_ide_integration.rs`: IDE 配置生成测试
  - 使用 tempfile 创建临时环境

- [ ] **[P1] T-026: 实现基础诊断功能 (`zzm info`)**
  - 显示当前环境状态
  - 已安装版本列表
  - 当前活动版本
  - 配置文件位置
  - 兼容性状态

- [ ] **[P1] T-027: 实现缓存清理 (`zzm clean`)**
  - `--all`: 清理所有缓存
  - `--dry-run`: 仅显示将要清理的内容
  - 显示释放的磁盘空间

**验收标准**:
- ✅ `zzm config list` 显示所有配置项
- ✅ `cargo test` 通过，覆盖率 > 70%
- ✅ 集成测试覆盖主要用户场景
- ✅ `zzm info` 输出清晰的环境信息

---

### Sprint 6: 发布准备 (第 10 周)

**目标**: CI/CD 配置、文档完善、打包发布

#### 任务列表

- [ ] **[P0] T-028: 配置 GitHub Actions CI**
  文件: `.github/workflows/ci.yml`
  - Linux 构建（x86_64-unknown-linux-gnu）
  - Windows 构建（x86_64-pc-windows-msvc）
  - macOS 构建（x86_64-apple-darwin, aarch64-apple-darwin）
  - Clippy lint 检查
  - Rustfmt 格式化检查
  - 单元测试执行
  - 构建产物 Artifact 上传

- [ ] **[P0] T-029: 编写 CHANGELOG.md**
  - 遵循 Keep a Changelog 格式
  - 记录 v0.1.0 的所有变更
  - 分类：Added / Changed / Fixed

- [ ] **[P0] T-030: 完善 README.md**
  - 项目简介和特性列表
  - 快速开始指南（5 分钟上手）
  - 安装方式（二进制下载 / Cargo install）
  - 基础用法示例
  - 截图/GIF（可选但推荐）
  - 贡献指南链接
  - License 信息

- [ ] **[P0] T-031: 编写 usage.md 使用指南**
  - 详细的功能说明
  - 所有命令的完整示例
  - 常见问题 FAQ
  - 故障排除指南

- [ ] **[P1] T-032: 性能基准测试**
  - 启动时间测量
  - 内存占用测量
  - 大量版本场景下的性能
  - 记录基准数据作为后续优化参照

- [ ] **[P1] T-033: 发布 v0.1.0-alpha.1**
  - 打 Git Tag: `v0.1.0-alpha.1`
  - 创建 GitHub Release
  - 上传构建产物
  -发布公告（Discourse/Twitter/Mastodon）

**验收标准**:
- ✅ CI 绿灯（所有平台构建通过）
- ✅ 所有文档齐全且准确
- ✅ Release 页面有下载链接
- ✅ 至少在 1 个真实环境中完成端到端测试

---

## 📊 任务优先级说明

| 优先级 | 含义 | 处理方式 |
|-------|------|---------|
| **P0** | 必须完成，阻塞后续任务 | 立即开始，优先处理 |
| **P1** | 重要但不紧急 | P0 完成后尽快处理 |
| **P2** | 有空再做 | 时间充裕时考虑 |

---

## 🔗 任务依赖关系图

```
T-001 (项目初始化)
  ↓
T-002 (依赖配置) → T-003 (错误类型) → T-004 (日志)
  ↓
T-005 (CLI 框架) ← T-006 (平台抽象)
  ↓
T-008 (Zig API) ← T-010 (版本解析) → T-009 (ZLS API)
  ↓                              ↓
T-011 (下载器)              T-015 (ZigManager) ← T-012 (路径管理)
  ↓                              ↓           ↓
T-013 (文件系统)         T-016 (命令集成) ← T-014 (校验和)
  ↓
T-018 (ZLSManager) → T-019 (--with-zls)
  ↓
T-020 (VS Code) + T-021 (ide path) + T-022 (兼容性)
  ↓
T-023 (配置) → T-026 (info) + T-027 (clean)
  ↓
T-024 (单元测试) + T-025 (集成测试)
  ↓
T-028 (CI/CD) → T-029-T-031 (文档) → T-033 (发布)
```

---

## 📝 开发规范

### 代码风格

- 使用 `cargo fmt` 格式化代码
- 通过 `cargo clippy` 检查（零警告）
- 遵循 Rust 2018 edition 惯例
- 公共 API 必须有文档注释（`///`）
- 复杂逻辑必须有行内注释

### 提交规范

遵循 Conventional Commits:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Type 类型**:
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档变更
- `style`: 代码格式（不影响功能）
- `refactor`: 重构（不是新功能也不是修复）
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建/工具链/辅助工具变更

**示例**:
```
feat(core): implement zig install command

Add the ability to download and install specific Zig versions.
Includes progress bar display and checksum verification.

Closes #123
```

### 分支策略

```
main (生产分支)
  ↑
develop (开发分支)
  ↑
feature/install-zig (功能分支)
feature/api-integration
bugfix/progress-bar-crash
```

- 从 `develop` 创建 feature 分支
- 完成后 PR 到 `develop`
- 测试通过后 merge
- 准备发布时从 `develop` 创建 `release/x.y.z` 分支
- 最终 merge 到 `main`

---

## ⏰ 时间估算参考

| 任务类型 | 平均耗时 | 说明 |
|---------|---------|------|
| 新增文件 + 基础实现 | 2-4 小时 | 包含思考和初步编码 |
| 完整功能实现 | 4-8 小时 | 包含错误处理和边界情况 |
| 单元测试编写 | 2-3 小时 | 目标 80%+ 覆盖率 |
| 集成测试编写 | 3-5 小时 | Mock 环境 + 场景覆盖 |
| 文档编写 | 1-2 小时 | README/CHANGELOG/注释 |
| Code Review + 修改 | 1-2 小时 | 团队协作成本 |

**每周可用工时假设**: 20-25 小时（兼职）或 40 小时（全职）

---

## ✅ 完成标准 checklist

每个任务完成后确认：

- [ ] 代码通过 `cargo build`
- [ ] 代码通过 `cargo clippy`（无警告）
- [ ] 代码通过 `cargo fmt`（格式化一致）
- [ ] 新增代码有对应的单元测试
- [ ] 测试全部通过 `cargo test`
- [ ] 公共 API 有文档注释
- [ ] Git commit message 符合规范
- [ ] 如有必要，更新相关文档

---

## 🐛 已知问题 & 技术债务

*(随着开发进展持续更新)*

- [ ] #001: 需要确定是否支持 Zig 0.10 及更早版本（API 可能不同）
- [ ] #002: Windows 下长路径问题（MAX_PATH 限制）
- [ ] #003: 代理服务器支持（HTTP_PROXY 环境变量）
- [ ] #004: 离线模式支持（纯本地操作）
- [ ] #005: 国际化（i18n）框架选择

---

## 📞 问题反馈

遇到阻塞问题？

1. 先查看 [api-reference.md](./api-reference.md) 确认 API 使用方式
2. 查看 [architecture.md](./architecture.md) 确认架构设计决策
3. 在团队频道讨论（Slack/Discord）
4. 如果是 Bug，创建 Issue 并标记为 `blocker`

---

## 📜 变更日志

| 日期 | 版本 | 修改内容 |
|-----|------|---------|
| 2026-04-24 | v1.0.0 | 初始版本，建立 Phase 1 任务清单 |

---

*下次更新时间*: 每个 Sprint 结束时回顾并更新此文档*

**当前负责人**: 待定

**最后评审**: 2026-04-24
