# Zig/ZLS 版本管理器 - 待办事项

## 📋 文档信息

- **版本**: v3.0.0
- **更新日期**: 2026-04-25
- **当前阶段**: Phase 1 MVP + 架构优化重构完成
- **编译状态**: ✅ cargo clippy -D warnings 零警告
- **测试状态**: ✅ 157/157 全部通过

---

## 🔴 已完成（架构优化重构）

### T-044: 统一 Channel 枚举 ✅

- **问题**: `ZigChannel` 和 `ZlsChannel` 重复定义，`InstalledZigVersion.channel` 使用 String 而非枚举
- **完成**: 合并为 `core::channel::Channel`，`channel` 字段改为枚举类型
- **涉及文件**: `src/core/channel.rs`(新增), `src/infra/zig_api.rs`, `src/infra/zls_api.rs`, `src/infra/path_manager.rs`, `src/core/zig_manager.rs`, `src/core/zls_manager.rs`, `src/commands/list.rs`, `src/commands/zls.rs`

### T-045: 合并目标三元组解析 ✅

- **问题**: `zig_api::parse_target_triple` 和 `zls_api::parse_zls_target_triple` 完全相同
- **完成**: 统一到 `platform::parse_target_triple()`
- **涉及文件**: `src/platform/trait_def.rs`, `src/platform/mod.rs`, `src/infra/zig_api.rs`, `src/infra/zls_api.rs`

### T-046: 合并 VersionParts 和 Version ✅

- **问题**: `compatibility::VersionParts` 与 `utils::version::Version` 功能重复
- **完成**: `CompatibilityChecker` 改用 `Version::from_str` 解析，删除 `VersionParts` 和 `parse_version_parts()`
- **涉及文件**: `src/core/compatibility.rs`

### T-047: 引入 ToolManager 泛型抽象 ✅

- **问题**: ZigManager 和 ZlsManager 90% 代码重复（~860 行）
- **完成**: 新增 `ToolManager<T: VersionProvider>` 泛型结构体 + `VersionProvider` trait + `ToolKind` 枚举
- **删除**: `src/core/zig_manager.rs`, `src/core/zls_manager.rs`
- **新增**: `src/core/tool_manager.rs`（~450 行，净删除约 400 行重复代码）
- **涉及文件**: `src/commands/mod.rs`, `src/commands/list.rs`, `src/commands/zls.rs`, `src/commands/install.rs`, `src/commands/setup.rs`, `src/commands/info.rs`

### T-048: 提取 ApiCache 泛型缓存层 ✅

- **问题**: `ZigApiClient` 和 `ZlsApiClient` 缓存逻辑重复
- **完成**: 新增 `ApiCache<T: Serialize + DeserializeOwned>` 泛型缓存
- **新增**: `src/infra/api_cache.rs`
- **涉及文件**: `src/infra/zig_api.rs`, `src/infra/zls_api.rs`

### T-049: 流式 SHA256 校验 ✅

- **问题**: 校验时将整个文件读入内存（~100MB），内存占用过高
- **完成**: 新增 `checksum::verify_checksum_streaming()` 使用 BufReader 流式校验
- **涉及文件**: `src/infra/checksum.rs`, `src/core/tool_manager.rs`

---

## 🟡 中优先级

### T-025: 编写集成测试

- **问题**: `tests/integration/` 目录为空，无任何集成测试
- **目标**:
  - `test_install_flow.rs`: 完整安装流程测试
  - `test_switch_flow.rs`: 版本切换流程测试
  - `test_ide_integration.rs`: IDE 配置生成测试
  - 使用 tempfile 创建临时环境

### T-050: 补充 ToolManager 单元测试

- **问题**: 架构重构后删除了旧 Manager 的测试，`tool_manager.rs` 本身缺少测试
- **目标**:
  - ToolManager 创建和初始化测试
  - InstalledVersion 枚举方法测试
  - 索引操作辅助方法测试
  - 流式校验测试

### T-051: Core 层输出解耦

- **问题**: ToolManager 直接调用 `console_output::print_*`，混合了业务逻辑和展示逻辑
- **方案**: 回调函数（`InstallCallbacks`），Commands 层注入具体输出实现
- **触发**: Phase 2 支持 `--json` 全局输出或 TUI 模式时实施

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
| 2026-04-25 | v3.0.0 | 完成 T-044~T-049 架构优化重构：ToolManager 泛型抽象、统一 Channel/目标三元组/版本解析、ApiCache 泛型缓存、流式校验 |
| 2026-04-25 | v2.1.0 | 完成 T-034~T-041 全部优化项，移除冗余 async/clone，修复 clippy 警告，重构 cmd_doctor |
| 2026-04-25 | v2.0.0 | 基于 clippy pedantic/nursery 检测重构，仅保留未完成项，新增 T-034~T-043 |