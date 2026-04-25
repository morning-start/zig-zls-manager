# zzm — Zig/ZLS Version Manager

专业级的 **Zig + ZLS 联合版本管理** CLI 工具。

[![CI](https://github.com/user/zig-zls-manager/actions/workflows/ci.yml/badge.svg)](https://github.com/user/zig-zls-manager/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

---

## 为什么选择 zzm？

| 特性 | zzm | zigup | zvm (Go) | zvm (Zig) |
|------|-----|-------|----------|-----------|
| Zig 版本管理 | ✅ | ✅ | ✅ | ✅ |
| **ZLS 独立管理** | ✅ | ❌ | 附属 | 有限 |
| **兼容性矩阵** | ✅ | ❌ | ❌ | 自动检测 |
| **IDE 集成** | ✅ | ❌ | ❌ | ❌ |
| **项目级配置** | ✅ (Phase 2) | ❌ | ❌ | ❌ |
| 跨平台 | ✅ | ✅ | ✅ | ✅ |
| 活跃维护 | ✅ | ❌ | ✅ | ✅ |

**核心差异化**：ZLS 作为一等公民独立管理，维护 Zig ↔ ZLS 版本兼容性矩阵。

---

## 快速开始

### 安装

```bash
# Cargo 安装（推荐）
cargo install --git https://github.com/user/zig-zls-manager

# 或从 Release 下载二进制
# https://github.com/user/zig-zls-manager/releases
```

### 基础用法

```bash
# 安装 Zig 0.13.0
zzm install 0.13.0

# 同时安装匹配的 ZLS
zzm install 0.13.0 --with-zls

# 切换版本
zzm use 0.13.0

# 查看当前版本
zzm current

# 查看远程可用版本
zzm list --remote
```

### ZLS 管理

```bash
# 独立安装 ZLS
zzm zls install 0.13.0

# 切换 ZLS 版本
zzm zls use 0.13.0

# 查看当前 ZLS 版本
zzm zls current
```

### IDE 集成

```bash
# 配置 VS Code
zzm ide config vscode

# 查看 zig/zls 路径
zzm ide path

# 检查 IDE 配置
zzm ide check
```

### 环境管理

```bash
# 环境信息
zzm info

# 诊断环境问题
zzm doctor

# 清理缓存
zzm clean
zzm clean --all
zzm clean --dry-run
```

---

## 所有命令

| 命令 | 说明 |
|------|------|
| `zzm install <version>` | 安装 Zig 版本 |
| `zzm uninstall <version>` | 卸载 Zig 版本 |
| `zzm list` | 列出版本 |
| `zzm use <version>` | 切换版本 |
| `zzm current` | 显示当前版本 |
| `zzm zls <subcmd>` | ZLS 子命令组 |
| `zzm setup` | 一键初始化开发环境 |
| `zzm sync` | 同步到推荐 Zig+ZLS 组合 |
| `zzm info` | 显示环境信息 |
| `zzm config <subcmd>` | 配置管理 |
| `zzm ide <subcmd>` | IDE 集成管理 |
| `zzm clean` | 清理缓存 |
| `zzm doctor` | 运行诊断 |
| `zzm completion <shell>` | 生成 Shell 补全 |

### 全局选项

| 选项 | 说明 |
|------|------|
| `--json` | JSON 格式输出 |
| `--verbose` | 详细日志 |
| `--no-color` | 禁用颜色 |

---

## 从源码构建

```bash
git clone https://github.com/user/zig-zls-manager.git
cd zig-zls-manager
cargo build --release
# 二进制位于 target/release/zzm
```

---

## 文档

- [使用指南](docs/usage.md) — 详细的功能说明和示例
- [技术架构](docs/architecture.md) — 分层架构设计
- [API 参考](docs/api-reference.md) — 公共 API 文档
- [路线图](docs/ROADMAP.md) — 开发计划
- [竞品分析](docs/comparison.md) — 与其他工具的对比

---

## 贡献

欢迎贡献！请确保：

1. `cargo fmt --all -- --check` 通过
2. `cargo clippy --all-targets -- -D warnings` 零警告
3. `cargo test` 全部通过
4. 提交信息遵循 Conventional Commits

---

## License

MIT OR Apache-2.0，任选其一。