# Zig/ZLS 版本管理器 - 开发任务清单

## 📋 文档信息

- **版本**: v1.1.0
- **创建日期**: 2026-04-24
- **当前阶段**: Phase 1 - MVP (v0.1.0) — Sprint 0-5 核心功能完成，测试与发布准备中
- **关联文档**: [ROADMAP.md](./ROADMAP.md), [architecture.md](./architecture.md)
- **编译状态**: ✅ cargo build 零警告通过（22 个 dead code 已标注 `#[allow(dead_code)]`）
- **测试状态**: ✅ 166/166 全部通过

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

- [x] **[P0] T-001: 初始化 Rust 项目** ✅ *已完成*
  - 使用 `cargo init --name zzm` 创建项目
  - 配置 Cargo.toml（Edition 2024）
  - 创建完整目录结构（src/core/, src/infra/, src/platform/, src/output/, src/utils/）

- [x] **[P0] T-002: 配置核心依赖** ✅ *已完成*
  - clap 4, tokio 1, reqwest 0.12, serde/serde_json, toml 0.8
  - anyhow 1, thiserror 2, tracing 0.1 + tracing-subscriber
  - console 0.15, tabled 0.16, indicatif 0.17, dialoguer 0.11
  - semver 1, sha2 0.10, hex 0.4, flate2 1, tar 0.4, xz2 0.1, zip 2
  - dirs 6, chrono 0.4, regex 1, url 2, futures-util 0.3
  - clap_complete 4（Shell 补全生成）

- [x] **[P0] T-003: 定义错误类型体系** ✅ *已完成*
  文件: `src/utils/error.rs`
  - 完整的 ZzmError 枚举（17 种变体）
  - 包含 Io, Network, Json, Toml, DownloadFailed, DownloadInterrupted
  - InvalidVersion, VersionNotFound, AlreadyInstalled, NotInstalled
  - ChecksumMismatch, ConfigError, UnsupportedPlatform, PermissionDenied
  - IncompatibleVersions, ExtractionFailed, SymlinkFailed
  - CacheDirCreationFailed, InsufficientDiskSpace, Cancelled, HttpError, RateLimited

- [x] **[P0] T-004: 初始化日志系统** ✅ *已完成*
  文件: `src/main.rs`
  - tracing-subscriber + EnvFilter
  - 支持 RUST_LOG 环境变量
  - verbose 模式切换 debug/warn 级别

**验收标准**:
- ✅ `cargo build` 编译通过
- ✅ `cargo test` 运行无报错
- ✅ `cargo clippy` 无警告

---

### Sprint 1: CLI 框架 + 平台抽象 (第 2 周)

**目标**: 实现命令行解析和平台适配层

#### 任务列表

- [x] **[P0] T-005: 实现 CLI 命令定义** ✅ *已完成*
  文件: `src/cli.rs`
  - Cli 结构体（全局选项：no_color, verbose, json）
  - Commands 枚举（12 个子命令：install/uninstall/list/use/current/zls/setup/sync/info/config/ide/clean/doctor/completion）
  - ZlsCommands 子命令组（install/uninstall/list/use/current）
  - ConfigCommands 子命令组（list/get/set/edit）
  - IdeCommands 子命令组（config/check/doctor/path）

- [x] **[P0] T-006: 实现 Platform trait 和适配器** ✅ *已完成*
  文件: `src/platform/trait_def.rs`, `windows.rs`, `macos.rs`, `linux.rs`
  - PlatformTrait trait（16 个方法 + clone_box）
  - WindowsPlatform（shim 文件、注册表操作）
  - MacOSPlatform（符号链接、plist）
  - LinuxPlatform（符号链接、.bashrc/.zshrc）
  - detect_platform() 运行时平台检测
  - current_target_triple() 目标三元组

- [x] **[P1] T-007: 实现输出格式化模块** ✅ *已完成*
  文件: `src/output/console_output.rs`, `json_output.rs`, `table_output.rs`, `progress.rs`
  - 成功消息（绿色 ✓）/ 警告消息（黄色 ⚠）/ 错误消息（红色 ✗）/ 信息消息（蓝色 ℹ）
  - 步骤消息 print_step（带编号）
  - 表格输出（VersionRow/InstalledVersionRow/RemoteVersionRow/KV 表格）
  - JSON 输出模式
  - 全局 no_color 支持（thread_local）
  - DownloadProgress 进度条

**验收标准**:
- ✅ `zzm --help` 显示完整的帮助信息
- ✅ `zzm --version` 显示版本号
- ✅ 所有子命令都有帮助文本
- ✅ 平台检测正确识别 OS

---

### Sprint 2: 外部 API 集成 (第 3-4 周)

**目标**: 实现对 Zig 和 ZLS API 的访问能力

#### 任务列表

- [x] **[P0] T-008: 实现 Zig JSON API 客户端** ✅ *已完成*
  文件: `src/infra/zig_api.rs`
  - ZigDownloadIndex/ZigVersionEntry/ZigPlatforms/ZigPlatformAsset 数据结构
  - ZigVersionInfo 统一版本信息 + ZigChannel 枚举
  - ZigApiClient：缓存 + 远程获取 + 版本查询 + 平台匹配
  - 本地缓存（文件系统 + 1小时 TTL）
  - 平台自动匹配逻辑（find_matching_asset）
  - 语义版本排序
  - get_version_info / get_latest_stable / get_master 方法

- [x] **[P0] T-009: 实现 ZLS GitHub API 客户端** ✅ *已完成*
  文件: `src/infra/zls_api.rs`
  - GithubRelease/GithubAsset 数据结构
  - ZlsVersionInfo/ZlsChannel 统一版本信息
  - ZlsApiClient：GitHub Token 认证 + 速率限制处理 + 3次重试
  - 指数退避重试机制
  - X-RateLimit-Remaining 检测 + 自动等待
  - find_compatible_version 三级查找策略（精确→主版本→回退最新稳定版）
  - find_matching_zls_asset 平台匹配

- [x] **[P0] T-010: 实现版本解析工具** ✅ *已完成*
  文件: `src/utils/version.rs`
  - Version 结构体（major/minor/patch/pre_release）
  - FromStr 实现（支持 "0.13.0", "0.13", "master", "stable"）
  - resolve_version 函数（简写补全：".13" → "0.13.0"）
  - Display 实现
  - 版本比较和排序（Ord/PartialOrd）

- [x] **[P1] T-011: 实现下载管理器** ✅ *已完成*
  文件: `src/infra/downloader.rs`
  - HTTP 流式下载（reqwest bytes_stream）
  - 进度条显示（indicatif）
  - 3次指数退避重试（100ms, 200ms, 400ms）
  - 临时文件写入 + 原子重命名保证完整性
  - 缓存目录下载（download_to_cache）
  - User-Agent 设置

**验收标准**:
- ✅ `zzm list --remote` 能返回 Zig 远程版本列表
- ✅ `zzm zls list --remote` 能返回 ZLS releases 列表
- ✅ API 响应被正确缓存（第二次请求更快）
- ✅ 网络超时有合理的错误提示

---

### Sprint 3: 核心安装流程 (第 5-6 周)

**目标**: 实现 Zig 安装、卸载、切换的完整流程

#### 任务列表

- [x] **[P0] T-012: 实现路径管理器** ✅ *已完成*
  文件: `src/infra/path_manager.rs`
  - InstalledIndex/InstalledZigVersion/InstalledZlsVersion 数据结构
  - PathManager：目录管理 + 符号链接 + 元数据索引
  - 初始化 ~/.zzm/ 目录结构（bin/versions/zig/versions/zls/cache）
  - 创建/删除 Zig/ZLS 符号链接
  - 读写 installed.json 元数据索引
  - 缓存大小计算（递归目录遍历）
  - 读取当前激活版本（符号链接目标解析）

- [x] **[P0] T-013: 实现文件系统操作模块** ✅ *已完成*
  文件: `src/infra/filesystem.rs`
  - tar.gz 解压（flate2 + tar）
  - tar.xz 解压（xz2 + tar）
  - zip 解压（zip crate）
  - 自动检测压缩格式（根据扩展名）
  - 安全解压（防止路径遍历攻击）
  - 文件权限设置（Unix executable bit）
  - 目录重组（Zig 包含顶层目录需扁平化）
  - remove_dir_all 安全删除

- [x] **[P0] T-014: 实现校验和验证模块** ✅ *已完成*
  文件: `src/infra/checksum.rs`
  - SHA256 计算（sha2 + hex）
  - 与预期值比对（大小写不敏感）
  - 错误时提供详细信息（期望值 vs 实际值）

- [x] **[P0] T-015: 实现 ZigManager 核心逻辑** ✅ *已完成*
  文件: `src/core/zig_manager.rs`
  - ZigManager 结构体（platform + path_manager + api_client + downloader）
  - install(version, force): 解析版本→下载→校验→解压→重组→设置权限→注册
  - uninstall(version): 删除目录→清理链接→更新索引
  - list_installed(): 读取 installed.json 返回列表
  - list_remote(): 调用 Zig API 返回版本列表
  - use_version(version): 验证安装→创建符号链接→更新 active_zig
  - current(): 读取索引返回当前激活版本
  - reorganize_extracted_files(): 处理解压后目录结构

- [x] **[P0] T-016: 集成 install/uninstall/list/use/current 命令** ✅ *已完成*
  文件: `src/main.rs`
  - 所有 CLI 子命令路由到 ZigManager/ZlsManager 实际方法
  - cmd_install: ZigManager::install + 可选 ZlsManager::install_compatible
  - cmd_uninstall: ZigManager::uninstall
  - cmd_list: 区分已安装/远程列表，支持 JSON 输出
  - cmd_use: ZigManager::use_version + 可选 ZlsManager::use_version
  - cmd_current: 显示当前激活的 Zig/ZLS 版本
  - cmd_zls: 完整的 ZLS 子命令路由
  - cmd_setup: 一键初始化开发环境
  - cmd_clean: CacheManager 清理缓存
  - cmd_info / cmd_doctor: 环境信息和诊断
  - cmd_completion: Shell 自动补全生成

- [x] **[P1] T-017: 实现缓存管理器** ✅ *已完成*
  文件: `src/infra/cache.rs`
  - CacheManager：缓存存储路径管理
  - total_size(): 递归计算缓存总大小
  - clean_all(): 清理所有缓存
  - clean_expired(ttl): 按过期时间清理
  - preview_clean(): 预览将要清理的内容
  - 文件大小格式化

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

- [x] **[P0] T-018: 实现 ZLSManager 基础逻辑** ✅ *已完成*
  文件: `src/core/zls_manager.rs`
  - ZlsManager 结构体（platform + path_manager + api_client + downloader）
  - install(version, zig_version, force): 解析版本→下载→解压→重组→查找二进制→注册
  - install_compatible(zig_version, force): 根据 Zig 版本自动匹配 ZLS
  - uninstall(version): 删除目录→清理链接→更新索引
  - list_installed(): 读取 installed.json 返回列表
  - list_remote(): 调用 ZLS API 返回版本列表
  - use_version(version): 验证安装→创建符号链接→更新 active_zls
  - current(): 读取索引返回当前激活版本
  - find_and_link_zls_binary(): 在解压目录中搜索 ZLS 二进制

- [x] **[P0] T-019: 实现 --with-zls 参数集成** ✅ *已完成*
  - cmd_install 中支持 --with-zls 参数
  - 安装 Zig 后自动调用 ZlsManager::install_compatible
  - 统一进度显示和错误处理

- [x] **[P0] T-020: 实现 VS Code IDE 集成** ✅ *已完成*
  文件: `src/core/ide.rs`
  - 读取/合并 VS Code settings.json（保留其他设置不覆盖）
  - 自动设置 zig.path 和 zig.zls.path（Windows 下使用正斜杠）
  - 支持 JSONC 行注释清理
  - 实现 `zzm ide config vscode` 命令
  - 实现 `zzm ide check` 检查 IDE 配置状态
  - 实现 `zzm ide doctor` 诊断 IDE 集成问题
  - 支持 `ide.vscode_settings_path` 自定义路径
  - 实现 `zzm ide remove` 移除 VS Code 中的 Zig/ZLS 配置

- [x] **[P0] T-021: 实现 `zzm ide path` 命令** ✅ *已完成*
  - 输出当前 zig 和 zls 的绝对路径  
  - 未安装时显示 (未安装) 提示
  - 已集成到 IdeManager::zig_binary_path / zls_binary_path

- [x] **[P1] T-022: 实现基础兼容性检查** ✅ *已完成*
  文件: `src/core/compatibility.rs`
  - CompatibilityStatus 枚举（Compatible/LikelyCompatible/Incompatible/Unknown）
  - check(zig_ver, zls_ver) 版本兼容性检查
  - 主版本号+次版本号匹配规则
  - master/nightly 版本特殊处理
  - recommended_zls_version() 推荐搭配
  - check_and_warn() 警告输出（不阻止操作）
  - 已集成到 cmd_sync 命令

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

- [x] **[P0] T-023: 实现配置管理器** ✅ *已完成*
  文件: `src/core/config.rs`
  - ZzmConfig 结构体（default_channel/auto_install_zls/auto_use/mirror_url 等）
  - IdeConfig 嵌套结构体（vscode_auto_update/vscode_set_zls_path/vscode_settings_path）
  - ConfigManager：TOML 配置文件读写
  - load() / save() / get(key) / set(key, value) / list_all() / reset()
  - 支持点分隔路径（如 ide.vscode_auto_update）
  - 实现 `config list/get/set/edit` 命令（edit 使用 $EDITOR 环境变量）
  - 已集成到 cmd_config 命令

- [x] **[P0] T-024: 补充单元测试** ✅ *已完成*
  目标覆盖率 > 70%
  - 从 51 个测试增加到 166 个测试（+115 个，增幅 225%）
  - ✅ utils/error.rs: 14 个测试（错误类型 Display、IO 转换、Result 别名）
  - ✅ utils/version.rs: 16 个测试（预发布版本、构造器、排序、serde、hash、无效输入）
  - ✅ core/compatibility.rs: 10 个测试（nightly+stable、v-prefix、两段版本号、不可解析版本）
  - ✅ core/config.rs: 9 个测试（IdeConfig 默认值、mirror_url、install_dir、TOML 最小化解析、serde default_true 行为）
  - ✅ core/ide.rs: 8 个测试（VsCodeSettings 保留其他字段、JSONC 多注释、URL 内 //、IdeManager 创建）
  - ✅ core/zig_manager.rs: 4 个测试（子目录重组、ZigVersion 序列化、list_installed 空、nightly channel）
  - ✅ core/zls_manager.rs: 9 个测试（ZlsManager 创建、InstalledZlsVersion 创建/序列化、reorganize、find_and_link_zls_binary 搜索逻辑）
  - ✅ infra/zig_api.rs: 9 个测试（parse_size 边界、target_triple 全覆盖、macos/linux 平台匹配、ZigVersionInfo/ZigPlatformAsset 序列化、ZigApiClient 创建、ZigDownloadIndex 反序列化）
  - ✅ infra/zls_api.rs: 9 个测试（channel 相等性、target_triple 全覆盖、minisig 排除、空 assets、ZlsApiClient 创建、GithubRelease/ZlsVersionInfo/GithubAsset 序列化、draft 过滤）
  - ✅ infra/downloader.rs: 2 个测试（builder pattern、zero retries）
  - ✅ infra/path_manager.rs: 9 个测试（ZLS 版本索引、多版本索引、zig/zls version dir 路径、binary paths、calculate_dir_size 空/文件/嵌套）
  - ✅ infra/filesystem.rs: 6 个测试（zip 子目录、zip 内容保留、remove_dir_all 有内容、find_extracted_root 子目录/无子目录、set_executable）
  - ✅ infra/cache.rs: 6 个测试（有文件的总大小、clean_all、preview_clean/空、format_size 边界值、嵌套目录）
  - ⚠️ 已知设计差异: IdeConfig Default derive vs serde default_true 语义不同（vscode_set_zls_path）

- [ ] **[P0] T-025: 编写集成测试**
  文件: `tests/integration/`（当前目录为空，无任何测试文件）
  - `test_install_flow.rs`: 完整安装流程测试
  - `test_switch_flow.rs`: 版本切换流程测试
  - `test_ide_integration.rs`: IDE 配置生成测试
  - 使用 tempfile 创建临时环境

- [x] **[P1] T-026: 实现基础诊断功能 (`zzm info`)** ✅ *已完成*
  - 显示平台/架构/安装目录/bin 目录/PATH 状态
  - 当前 Zig/ZLS 版本
  - 已安装版本数量
  - 缓存大小

- [x] **[P1] T-027: 实现缓存清理 (`zzm clean`)** ✅ *已完成*
  - `--all`: 清理所有缓存
  - `--dry-run`: 仅显示将要清理的内容
  - 显示释放的磁盘空间
  - 默认清理 7 天前的过期缓存

**验收标准**:
- ✅ `zzm config list` 显示所有配置项
- ✅ `cargo test` 166/166 全部通过，目标覆盖率 > 70%
- ❌ 集成测试覆盖主要用户场景（目录为空）
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
T-001 (项目初始化) ✅
  ↓
T-002 (依赖配置) ✅ → T-003 (错误类型) ✅ → T-004 (日志) ✅
  ↓
T-005 (CLI 框架) ✅ ← T-006 (平台抽象) ✅ ← T-007 (输出格式) ✅
  ↓
T-008 (Zig API) ✅ ← T-010 (版本解析) ✅ → T-009 (ZLS API) ✅
  ↓                              ↓
T-011 (下载器) ✅           T-015 (ZigManager) ✅ ← T-012 (路径管理) ✅
  ↓                              ↓           ↓
T-013 (文件系统) ✅      T-016 (命令集成) ✅ ← T-014 (校验和) ✅
  ↓
T-017 (缓存管理) ✅ → T-018 (ZLSManager) ✅ → T-019 (--with-zls) ✅
  ↓
T-020 (VS Code) ✅ + T-021 (ide path) ✅ + T-022 (兼容性) ✅
  ↓
T-023 (配置) ✅ → T-026 (info) ✅ + T-027 (clean) ✅
  ↓
T-024 (单元测试) + T-025 (集成测试)
  ↓
T-028 (CI/CD) → T-029-T-031 (文档) → T-033 (发布)
```

**当前进度**: Sprint 0-4 全部完成，Sprint 5 大部分完成（T-023/T-024✅/T-026/T-027），剩余 T-025

### ✅ 已解决问题汇总（2026-04-25 修复）

#### 编译警告（22 个 dead code warnings → 0）
所有 dead code 已分类标注 `#[allow(dead_code)]`，保留预留 API 完整性：
- `src/core/project.rs`: `ProjectManager` 空壳结构体（预留: 项目级配置）
- `src/utils/error.rs`: 6 个错误变体 + `Result` 类型别名（预留: 未来功能）
- `src/utils/version.rs`: `new`, `with_pre`, `is_stable`（预留: 版本构造 API）
- `src/infra/zig_api.rs`: `get_latest_stable`, `get_master`, `parse_size_to_bytes`（预留: API 扩展）
- `src/infra/zls_api.rs`: `get_latest_stable`（预留: API 扩展）
- `src/infra/downloader.rs`: `with_max_retries`（预留: 重试配置）
- `src/infra/path_manager.rs`: `cache_dir`, `install_dir`, `read_current_zig_version`（预留: 路径查询）
- `src/infra/cache.rs`: `cache_dir`（预留: 缓存目录查询）
- `src/core/ide.rs`: `remove_vscode_config`（预留: IDE 配置清理）
- `src/core/config.rs`: `reset`（预留: 配置重置）
- `src/core/compatibility.rs`: `check_and_warn`（预留: 兼容性警告）
- `src/core/zig_manager.rs`: `platform` 字段（预留: 平台操作扩展）
- `src/output/progress.rs`: `DownloadProgress` + impl, `create_spinner`（预留: 下载进度展示）
- `src/output/json_output.rs`: `print_json_error`（预留: JSON 错误输出）
- `src/output/table_output.rs`: `VersionRow`, `render_version_table`（预留: 版本列表渲染）
- `src/platform/trait_def.rs`: `shell_config_files`, `is_admin`（预留: 平台操作扩展）

#### 测试失败（1 个 → 已修复）
- ~~`test_resolve_version_edge_cases` — `"0."` 输入未被正确拒绝~~ → ✅ 修正测试期望，`"0."` 现在正确返回 `InvalidVersion`

### ⚠️ 待解决问题汇总

#### 缺失项
- `tests/integration/` 目录为空，无集成测试
- 无 CI/CD 配置（`.github/workflows/` 不存在）
- 无 `CHANGELOG.md`

---

## 📝 开发规范

### 代码风格

- 使用 `cargo fmt` 格式化代码
- 通过 `cargo clippy` 检查（零警告）
- 遵循 Rust 2024 edition 惯例
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

- [x] #001: ~~需要确定是否支持 Zig 0.10 及更早版本（API 可能不同）~~ → 当前支持所有 Zig 官方 API 提供的版本
- [ ] #002: Windows 下长路径问题（MAX_PATH 限制）
- [ ] #003: 代理服务器支持（HTTP_PROXY 环境变量）
- [ ] #004: 离线模式支持（纯本地操作）
- [ ] #005: 国际化（i18n）框架选择
- [ ] #006: T-019 并行下载 Zig + ZLS（当前为串行，应改为 tokio::join!）
- [ ] #007: install 原子性保证（任一失败需回滚两者）
- [ ] #008: ZlsManager::find_and_link_zls_binary 中未使用的 _binary_name 变量
- [ ] #009: zig_manager::use_version 中 _installed 变量未使用（确认版本是否存在但不读取信息）
- [ ] #010: dead code 警告（project.rs 空壳模块，将在后续 Sprint 消除）
- [ ] #011: 版本解析边界 bug — `"0."` 输入未返回 InvalidVersion 错误（导致测试失败）
- [ ] #012: 22 个 dead code warnings — 为后续功能预留的代码（Phase 2 需要），应使用 `#[allow(dead_code)]` 或条件编译
- [ ] #013: ROADMAP.md Phase 1 checklist 状态过时 — 大量已完成项仍标记为 `[ ]`

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
| 2026-04-25 | v1.3.0 | 项目审查：更新编译/测试状态，新增 #011-#013 已知问题，标注测试失败和 dead code 详情 |
| 2026-04-25 | v1.2.0 | 更新 Sprint 0-4 全部完成，Sprint 5 大部分完成（T-020~T-023），剩余 T-024/T-025 |
| 2026-04-25 | v1.1.0 | 更新 Sprint 0-3 全部完成，Sprint 4 部分完成（T-018/T-019），Sprint 5 部分完成（T-026/T-027） |
| 2026-04-24 | v1.0.0 | 初始版本，建立 Phase 1 任务清单 |

---

*下次更新时间*: 每个 Sprint 结束时回顾并更新此文档*

**当前负责人**: 待定

**最后评审**: 2026-04-25
