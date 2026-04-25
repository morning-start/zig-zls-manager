# Zig/ZLS 版本管理器 - 待办事项

## 📋 文档信息

- **版本**: v2.0.0
- **更新日期**: 2026-04-25
- **当前阶段**: Phase 1 MVP 已完成，进入优化迭代阶段
- **编译状态**: ✅ cargo clippy -D warnings 零警告
- **测试状态**: ✅ 168/168 全部通过

---

## 🔴 高优先级

### T-034: 移除不必要的 async 标记

- **问题**: 10 个函数标记了 `async` 但内部无 `await`，造成不必要的运行时开销
- **涉及文件**: `src/core/zig_manager.rs`, `src/core/zls_manager.rs`（`uninstall`, `current`, `list_installed` 等方法）
- **修复**: 移除 `async`，将调用处从 `.await` 改为直接调用

### T-035: 修复 unused self 问题

- **问题**: 2 个方法未使用 `self`，应为关联函数
- **涉及文件**: `src/core/compatibility.rs`（`CompatibilityChecker` 的某些方法）
- **修复**: 改为 `fn xxx()` 关联函数

### T-036: 处理类型转换精度丢失

- **问题**: 8 处 `u64 as f64` 和 `f64 as u64` 转换存在精度/截断风险
- **涉及文件**: `src/utils/format.rs:17-22`, `src/infra/zig_api.rs` 测试代码
- **修复**: 添加 `#[allow(clippy::cast_precision_loss)]` 或使用 `u128` 中间计算

### T-025: 编写集成测试

- **问题**: `tests/integration/` 目录为空，无任何集成测试
- **目标**:
  - `test_install_flow.rs`: 完整安装流程测试
  - `test_switch_flow.rs`: 版本切换流程测试
  - `test_ide_integration.rs`: IDE 配置生成测试
  - 使用 tempfile 创建临时环境

---

## 🟡 中优先级

### T-037: 批量修复 uninlined_format_args

- **问题**: ~70 处 `format!("{}", var)` 应改为 `format!("{var}")`
- **涉及文件**: `commands/`, `core/`, `infra/` 大量文件
- **修复**: 可用 `cargo clippy --fix --allow-dirty` 自动修复

### T-038: 修复 doc_markdown 警告

- **问题**: ~10 处文档中标识符缺少反引号
- **涉及文件**: `cli.rs`, `commands/mod.rs`, `config.rs`, `platform/`
- **修复**: 将 `ZigManager` 等加反引号 `` `ZigManager` ``

### T-039: 优化 Option 操作惯用法

- **问题**: 多处可简化的 Option 操作模式
- **涉及文件**:
  - `src/commands/info.rs:25-32` — `map().unwrap_or_else()` → `map_or_else()`
  - `src/commands/info.rs:126-134` — `match` → `Option::map_or_else()`
  - `src/commands/setup.rs:19` — 可用 `let...else` 简化
- **修复**: 使用更惯用的 Rust 写法

### T-040: 拆分 cmd_doctor 函数

- **问题**: `cmd_doctor` 函数 107 行，超过 100 行限制
- **涉及文件**: `src/commands/info.rs:84`
- **修复**: 提取子函数（如 `check_zig_status`, `check_zls_status`, `check_ide_status`）

### T-041: 标记 const fn 和移除冗余 clone

- **问题**: 5 处简单函数可标记为 `const`，5 处冗余 `.clone()`
- **涉及文件**: `src/core/config.rs`（`default_parallel`, `default_true`）, `src/infra/zig_api.rs`, `src/utils/version.rs`
- **修复**: 加 `const` 关键字，移除多余 clone

---

## 🟢 低优先级

### T-042: IdeConfig 结构体重构

- **问题**: `IdeConfig` 所有字段都有 `vscode` 前缀，clippy 建议拆分
- **涉及文件**: `src/core/config.rs:70`
- **修复**: 拆分为嵌套结构体 `VsCodeConfig`

### T-043: 数字字面量可读性优化

- **问题**: 9 处大数字缺少分隔符（如 `102400` → `102_400`）
- **涉及文件**: 测试代码
- **修复**: 添加下划线分隔符

### #002: Windows 长路径问题

- MAX_PATH 限制，可能影响深层目录操作

### #003: 代理服务器支持

- HTTP_PROXY 环境变量未处理

### #004: 离线模式支持

- 纯本地操作模式未实现

### #005: 国际化框架

- i18n 框架选择待定

### #006: 并行下载 Zig + ZLS

- 当前串行，应改为 `tokio::join!`

### #007: install 原子性

- 任一失败需回滚两者

---

## 📝 变更日志

| 日期 | 版本 | 修改内容 |
|-----|------|---------|
| 2026-04-25 | v2.0.0 | 基于 clippy pedantic/nursery 检测重构，仅保留未完成项，新增 T-034~T-043 |