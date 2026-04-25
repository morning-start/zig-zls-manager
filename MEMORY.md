## zig-zls-manager 项目概况
## 项目背景与定位
- **项目名称**: zig-zls-manager (zzm)
- **核心目标**: 同时管理 Zig 编译器与 ZLS (Zig Language Server) 的版本，解决两者版本兼容性问题。
- **技术栈**: Rust
- **差异化优势**:
  - ZLS 作为一等公民独立管理，而非附属功能。
  - 维护 Zig ↔ ZLS 版本兼容性矩阵，提供自动检测与警告。
  - 支持项目级配置 (.zzmrc) 锁定版本组合。
  - 自动化 IDE (VS Code, Neovim) 配置生成。
- **主要竞品**:
  - `zigup`: 仅管理 Zig，已停止维护。
  - `zvm (tristanisham)`: Go 实现，ZLS 为附属功能。
  - `zvm (hendriknielaender)`: Zig 实现，有自动检测但 ZLS 管理有限。
