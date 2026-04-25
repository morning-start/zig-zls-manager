# Changelog

本文件记录 zig-zls-manager (zzm) 的所有重要变更。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
版本号遵循 [Semantic Versioning](https://semver.org/lang/zh-CN/)。

---

## [0.1.0-alpha.1] - 2026-04-25

首个 alpha 版本，包含核心版本管理功能。

### Added

- **核心命令**: `zzm install`、`zzm uninstall`、`zzm list`、`zzm use`、`zzm current`
- **ZLS 管理**: `zzm zls install`、`zzm zls uninstall`、`zzm zls list`、`zzm zls use`、`zzm zls current`
- **--with-zls 参数**: 安装 Zig 时自动匹配并安装兼容的 ZLS 版本
- **VS Code 集成**: `zzm ide config vscode` 自动生成 `.vscode/settings.json`
- **IDE 路径输出**: `zzm ide path` 输出 zig/zls 二进制路径
- **IDE 诊断**: `zzm ide check`、`zzm ide doctor` 检查 IDE 配置状态
- **配置管理**: `zzm config list/get/set/edit` 管理用户配置
- **兼容性检查**: Zig ↔ ZLS 版本兼容性检测与警告
- **环境诊断**: `zzm doctor` 诊断开发环境问题
- **环境信息**: `zzm info` 显示当前平台和安装状态
- **缓存清理**: `zzm clean`/`zzm clean --all`/`zzm clean --dry-run`
- **一键初始化**: `zzm setup` 引导新用户设置开发环境
- **Shell 补全**: `zzm completion` 生成 Bash/Zsh/PowerShell/Fish 补全脚本
- **全局选项**: `--json`（JSON 输出）、`--verbose`（详细日志）、`--no-color`（禁用颜色）
- **API 客户端**: Zig JSON API + ZLS GitHub Releases API，支持缓存和速率限制处理
- **下载管理器**: HTTP 流式下载 + 进度条 + 指数退避重试
- **校验和验证**: SHA256 校验和比对
- **多格式解压**: tar.gz / tar.xz / zip 自动检测与安全解压
- **平台适配**: Windows（shim 文件）、macOS（符号链接）、Linux（符号链接）
- **单元测试**: 166 个测试覆盖核心模块

### Changed

- 无

### Fixed

- 修正 `resolve_version` 边界 bug — `"0."` 输入现在正确返回 `InvalidVersion`
- 清理 22 个 dead code warnings — 预留 API 添加 `#[allow(dead_code)]` 标注
- 修正 `IdeConfig` Default derive 与 serde `default_true` 的语义差异

### Security

- SHA256 校验和验证防止下载篡改
- 安全解压防止路径遍历攻击

---

## [0.1.0-dev] - 2026-04-24

### Added

- 项目初始化（Rust 2024 edition）
- 分层架构搭建（CLI → Core → Infra → Platform）
- 完整文档体系（spec/architecture/api-reference/usage/comparison/ROADMAP/TODO）

[0.1.0-alpha.1]: https://github.com/user/zig-zls-manager/releases/tag/v0.1.0-alpha.1
[0.1.0-dev]: https://github.com/user/zig-zls-manager/tree/develop