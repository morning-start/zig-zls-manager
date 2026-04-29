# Zig/ZLS 版本管理器 - 待办事项

## 📋 文档信息

- **版本**: v7.0.0
- **更新日期**: 2026-04-29
- **适用版本**: zig-zls-manager v1.2.1
- **关联文档**: [ROADMAP.md](./ROADMAP.md) | [architecture.md](./architecture.md)
- **当前阶段**: 阶段 7 - 体验优化收尾

---

## 🔵 待实现功能

### 高优先级

| 任务 | 描述 | 涉及文件 |
|------|------|----------|
| **`zzm update self` 自我更新** | 从 GitHub Releases 检测并安装最新版本 | `src/commands/update.rs` |
| **ConfigManager 自动字段映射** | 使用 serde 自动反射替代手动 `match` 映射 | `src/core/config.rs` |

### 中优先级

| 任务 | 描述 | 涉及文件 |
|------|------|----------|
| **兼容性矩阵远程更新** | 从 GitHub 拉取最新兼容性规则并缓存 | `src/core/compatibility.rs` |
| **IDE 配置自动检测** | 检测当前 IDE 是否安装 + 配置是否生效 | `src/commands/ide.rs` |

### 低优先级

| 任务 | 描述 | 涉及文件 |
|------|------|----------|
| **清理 dead_code 标注** | 系统性审查：要么实现，要么删除 | `src/platform/trait_def.rs` 等 |
| **IdeConfig 结构体重构** | 拆分为嵌套结构体 `VsCodeConfig` | `src/core/config.rs` |

---

## 🟡 边缘场景优化

| 问题 | 描述 | 优先级 |
|------|------|--------|
| **Windows 长路径** | 使用 `\\?\` 前缀扩展路径限制 | 低 |
| **代理服务器** | `reqwest::Client` 启用系统代理 | 中 |
| **离线模式** | 检测网络可用性，仅使用本地缓存 | 低 |

---

## 📊 已完成功能总结

### Phase 1 - MVP (v0.1.0) ✅
- ✅ 核心 Zig 版本管理（install/uninstall/list/use/current）
- ✅ ZLS 基础管理（附属模式）
- ✅ VS Code 集成
- ✅ 配置管理
- ✅ 平台适配（Windows/macOS/Linux）

### Phase 2 - 增强版 (v0.2.0) ✅
- ✅ **独立 ZLS 管理** (`zzm zls install/use/list/uninstall`)
- ✅ **兼容性矩阵**（内置规则 + `zzm sync`）
- ✅ **项目级配置**（`.zzmrc` + `zzm pair` + `zzm restore`）
- ✅ **扩展 IDE 集成**（Neovim/Helix + `zzm ide check/doctor`）
- ✅ **交互式 Setup Wizard** (`zzm setup --wizard`)
- ✅ **Shell 自动补全** (`zzm completion`)
- ✅ **`zzm prune` 移除旧版本**
- ✅ **`zzm doctor` 诊断**
- ✅ **`zzm clean` 缓存清理**

### Phase 2.5 - 稳定版 (v0.2.5) ✅
- ✅ **泛型彻底化**（`ToolManager` + `ToolIndexEntry` 统一数据结构）
- ✅ **Core 层输出解耦**（`InstallCallbacks` 回调方案）
- ✅ **Commands 层数据抽象**（`OutputDispatcher`）
- ✅ **AppContext OnceCell 懒加载**
- ✅ **并行下载优化**（`tokio::join!`）
- ✅ **原子性安装与回滚**
- ✅ **性能优化**（启动 < 100ms）
- ✅ **集成测试**（231 个测试全部通过）

---

## 📝 变更日志

| 日期 | 版本 | 修改内容 |
|------|------|---------|
| 2026-04-29 | v7.0.0 | 清理已完成功能，仅保留真实待办，更新到 v1.2.1 版本状态 |
| 2026-04-28 | v6.3.0 | 补充架构优化高优先级项 |
| 2026-04-28 | v6.2.0 | 根据 workflow/analysis.md 更新 |
