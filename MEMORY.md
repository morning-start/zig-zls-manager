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

## 当前状态 (2026-04-26)
- **阶段**: Phase 1 MVP 完成 + 架构优化重构完成 + 阶段 1(T-060+T-066) 完成 + 阶段 2(T-061) 完成
- **编译**: ✅ cargo clippy -D warnings 零警告通过
- **测试**: ✅ 194/194 全部通过
- **架构变更**: ZigManager/ZlsManager → ToolManager<T: VersionProvider> 泛型抽象
- **新增模块**: core::channel, core::tool_manager, core::callbacks, infra::api_cache
- **删除模块**: core::zig_manager, core::zls_manager
- **最近新增**: HTTP Range 断点续传 + InstallCallbacks 输出解耦 + OnceLock 懒加载 + ToolIndexEntry 统一数据结构（消除 15+ match 分支）
- **待办**: 推送到 GitHub → 创建 Tag → 创建 Release

## 架构优化要点
- 统一 Channel 枚举（替代 ZigChannel/ZlsChannel）
- 统一目标三元组解析（platform::parse_target_triple）
- ToolManager<T> 泛型抽象 + VersionProvider trait
- ApiCache<T> 泛型缓存层
- 流式 SHA256 校验（内存恒定）
- compatibility::VersionParts → utils::version::Version

## 关键教训
- **serde 模型必须基于实际 API 响应设计**: Zig API 返回扁平平台键（如 `x86_64-macos`）+ `{ tarball, shasum, size }`，而非嵌套的 `ZigPlatforms` 结构。使用 `#[serde(flatten)]` 捕获动态键名，`#[serde(rename)]` 解决 JSON camelCase 与 Rust snake_case 命名冲突。
- **`is_platform_key()` 过滤模式**: 当 `#[serde(flatten)]` 捕获了非平台键（如 `src`、`bootstrap`）时，需在业务逻辑中通过白名单/特征匹配过滤。

## 主要竞品
- `zigup`: 仅管理 Zig，已停止维护。
- `zvm (tristanisham)`: Go 实现，ZLS 为附属功能。
- `zvm (hendriknielaender)`: Zig 实现，有自动检测但 ZLS 管理有限。