# Changelog

本文件记录 zig-zls-manager (zzm) 的所有重要变更。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
版本号遵循 [Semantic Versioning](https://semver.org/lang/zh-CN/).

---

## [1.2.1] - 2026-04-29

### 新增功能

- **`zzm pair`**: 手动绑定 Zig↔ZLS 版本关系，写入项目 `.zzmrc` 配置
- **`zzm restore`**: 读取项目 `.zzmrc` 配置，安装缺失版本并切换
- **`zzm prune`**: 批量移除旧版本，释放磁盘空间
- **`zzm setup --wizard`**: 交互式安装向导，引导新用户配置
- **`zzm completion`**: Shell 自动补全脚本生成（支持 bash/zsh/fish/powershell）

### 架构优化

- **泛型彻底化**: `ToolIndexEntry` 统一数据结构，消除 15+ `match` 分支
- **Core 层输出解耦**: `InstallCallbacks` 回调方案，支持纯净 `--json` 输出
- **Commands 层数据抽象**: `OutputDispatcher` 统一输出调度，消除重复逻辑
- **AppContext OnceCell 懒加载**: 单例复用，代码更清晰
- **ApiCache 泛型缓存层**: 统一缓存 Zig/ZLS API 响应
- **PostInstallHook Trait 抽象**: 消除硬编码 ZLS 特例
- **索引读取合并优化**: 减少不必要的文件 I/O

### 性能提升

- **启动速度**: < 100ms（优化前 > 200ms）
- **并行下载**: Zig + ZLS 使用 `tokio::join!` 并行下载，速度提升 ~40%
- **流式 SHA256 校验**: 内存占用恒定，不随文件大小增长
- **代码重复度**: 降低 50%+

### 稳定性增强

- **原子性安装与回滚**: ZLS 安装失败时自动回滚 Zig 变更
- **集成测试**: 16 个集成测试覆盖核心功能
- **单元测试**: 214 个单元测试，总计 231 个测试全部通过
- **代码质量**: `cargo clippy -D warnings` 零警告

### 文档完善

- 更新 [ROADMAP.md](./docs/ROADMAP.md) 到 v1.2.1 状态
- 更新 [TODO.md](./docs/TODO.md)，清理已完成功能
- 完善 [usage.md](./docs/usage.md) 使用指南

---

## [1.2.0] - 2026-04-28

### 架构优化

- **统一 Channel 枚举**: 替代 ZigChannel/ZlsChannel
- **统一目标三元组解析**: `platform::parse_target_triple`
- **ToolManager 泛型抽象**: 开始消除重复代码

### 功能完善

- **兼容性矩阵**: 内置 Zig↔ZLS 版本映射
- **`zzm sync`**: 同步到推荐组合

---

## [1.1.0] - 2026-04-27

### 新增功能

- **`zzm ide doctor`**: 诊断 IDE 配置问题
- **`zzm info`**: 详细环境信息展示

### 稳定性提升

- 边界情况处理（网络中断、磁盘空间不足）

---

## [0.1.0] - 2026-04-26

### 新增功能

- **独立 ZLS 管理**: `zzm zls install/use/list/uninstall`
- **扩展 IDE 集成**: Neovim/Helix 支持
- **项目级配置**: `.zzmrc` 基础支持

### 修复

- 修正 `resolve_version` 边界 bug
- 清理 22 个 dead_code warnings
- 修正 `IdeConfig` Default derive 与 serde `default_true` 的语义差异

---

## [0.1.0-alpha.1] - 2026-04-25

首个 alpha 版本，包含核心版本管理功能。

### 新增功能

- **核心命令**: `zzm install`/`uninstall`/`list`/`use`/`current`
- **ZLS 管理**: `zzm zls install`/`uninstall`/`list`/`use`/`current`
- **`--with-zls` 参数**: 安装 Zig 时自动匹配并安装兼容 ZLS
- **VS Code 集成**: `zzm ide config vscode` 自动生成配置
- **IDE 路径输出**: `zzm ide path`
- **IDE 诊断**: `zzm ide check`
- **配置管理**: `zzm config list/get/set/edit`
- **兼容性检查**: Zig↔ZLS 版本兼容性检测与警告
- **环境诊断**: `zzm doctor`
- **环境信息**: `zzm info`
- **缓存清理**: `zzm clean`/`--all`/`--dry-run`
- **Shell 补全**: `zzm completion` 框架
- **全局选项**: `--json`/`--verbose`/`--no-color`
- **API 客户端**: Zig JSON API + ZLS GitHub Releases API，支持缓存
- **下载管理器**: HTTP 流式下载 + 进度条 + 指数退避重试
- **校验和验证**: SHA256 校验和比对
- **多格式解压**: tar.gz/tar.xz/zip 自动检测与安全解压
- **平台适配**: Windows（shim 文件）、macOS/Linux（符号链接）
- **单元测试**: 166 个测试覆盖核心模块

### 修复

- 修正 `resolve_version` 边界 bug - `"0."` 输入现在正确返回 `InvalidVersion`
- 清理 22 个 dead_code warnings - 预留 API 添加 `#[allow(dead_code)]` 标注
- 修正 `IdeConfig` Default derive 与 serde `default_true` 的语义差异

### 安全

- SHA256 校验和验证防止下载篡改
- 安全解压防止路径遍历攻击

---

## [0.1.0-dev] - 2026-04-24

### 新增

- 项目初始化（Rust 2024 edition）
- 分层架构搭建（CLI → Core → Infra → Platform）
- 完整文档体系（spec/architecture/api-reference/usage/comparison/ROADMAP/TODO）

---

[1.2.1]: https://github.com/user/zig-zls-manager/releases/tag/v1.2.1
[1.2.0]: https://github.com/user/zig-zls-manager/releases/tag/v1.2.0
[1.1.0]: https://github.com/user/zig-zls-manager/releases/tag/v1.1.0
[0.1.0]: https://github.com/user/zig-zls-manager/releases/tag/v0.1.0
[0.1.0-alpha.1]: https://github.com/user/zig-zls-manager/releases/tag/v0.1.0-alpha.1
[0.1.0-dev]: https://github.com/user/zig-zls-manager/tree/develop
