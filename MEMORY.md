## zig-zls-manager 项目概况

## 项目背景与定位
- **项目名称**: zig-zls-manager (zzm)
- **核心目标**: 同时管理 Zig 编译器与 ZLS (Zig Language Server) 的版本，解决两者版本兼容性问题。
- **技术栈**: Rust 2024 edition, clap, tokio, reqwest, serde
- **差异化优势**:
  - ZLS 作为一等公民独立管理，而非附属功能。
  - 维护 Zig ↔ ZLS 版本兼容性矩阵，提供自动检测与警告。
  - 支持项目级配置 (.zzmrc) 锁定版本组合。
  - 自动化 IDE (VS Code, Neovim) 配置生成。

## 当前状态 (2026-04-25)
- **阶段**: Phase 1 MVP — Sprint 0-6 全部完成，待推送创建 Release
- **编译**: ✅ cargo build + clippy 零警告通过
- **测试**: ✅ 166/166 全部通过
- **文档**: ✅ 全部同步更新（architecture/api-reference/ROADMAP/TODO/README/CHANGELOG/usage/benchmarks）
- **已修复**: 版本解析边界 bug、dead code warnings、clippy 零警告
- **性能**: Release 二进制 5.2 MB，启动 < 50ms
- **待办**: 推送到 GitHub → 创建 Tag v0.1.0-alpha.1 → 创建 Release

## 主要竞品
- `zigup`: 仅管理 Zig，已停止维护。
- `zvm (tristanisham)`: Go 实现，ZLS 为附属功能。
- `zvm (hendriknielaender)`: Zig 实现，有自动检测但 ZLS 管理有限。