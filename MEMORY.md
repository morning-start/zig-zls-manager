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
- **阶段**: Phase 1 MVP — Sprint 0-5 核心功能完成，测试与发布准备中
- **编译**: ✅ cargo build 零警告通过（22 个 dead code 已全部标注 `#[allow(dead_code)]`）
- **测试**: ✅ 166/166 全部通过（T-024 单元测试补充完成：51→166，增幅 225%）
- **已修复**: 版本解析边界 bug、dead code warnings、clippy 零警告
- **待办**: 集成测试(T-025) → CI/CD → 发布

## 主要竞品
- `zigup`: 仅管理 Zig，已停止维护。
- `zvm (tristanisham)`: Go 实现，ZLS 为附属功能。
- `zvm (hendriknielaender)`: Zig 实现，有自动检测但 ZLS 管理有限。