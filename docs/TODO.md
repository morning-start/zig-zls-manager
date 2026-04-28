# Zig/ZLS 版本管理器 - 待办事项

## 📋 文档信息

- **版本**: v6.1.0
- **更新日期**: 2026-04-28
- **适用版本**: zig-zls-manager v0.1.0+
- **关联文档**: [ROADMAP.md](./ROADMAP.md) | [architecture.md](./architecture.md) | [analysis.md](../workflow/analysis.md)
- **当前阶段**: 阶段 6 - 自我更新 + P3 体验优化

---

## 🔵 P3 - 体验优化 + 边缘场景

### T-075: `zzm update self` 自我更新

- **问题**: `spec.md` §2.3.2 定义但未实现，用户需手动下载新版本
- **方案**: 从 GitHub Releases 检测新版本 → 下载替换当前二进制
- **实现策略**:
  - `zzm update self`: 检查 GitHub Releases 最新版本
  - 下载新二进制到临时文件 → 原子替换（rename）
  - Windows 需特殊处理（运行中二进制不可替换 → 用 shim/重命名策略）
  - `--check` 仅检查不更新，`--force` 跳过版本比较
- **涉及文件**: `src/commands/update.rs`（新增）
- **工作量**: 2 天 | **风险**: 中

### T-073: ConfigManager 自动字段映射

- **问题**: `get()`/`set()` 使用 `match` 逐字段映射，新增配置项需改 3 处
- **方案**: serde 自动反射或宏驱动映射
- **涉及文件**: `src/core/config.rs`
- **工作量**: 2 天 | **风险**: 中

### T-080: Shell 自动补全生成

- **问题**: `spec.md` §2.3.3 定义但未实现
- **方案**: 新增 `zzm completion <shell>` 子命令，使用 `clap_complete::generate()` 动态生成
- **工作量**: 1 天 | **风险**: 低

### T-081: 清理 dead_code 标注

- **问题**: 多处 `#[allow(dead_code)]` 遮盖了未实现的功能
- **方案**: 系统性审查——实现、删除或用 `cfg(feature)` 控制
- **涉及文件**: `src/platform/trait_def.rs`, `src/core/tool_manager.rs`, `src/infra/downloader.rs` 等
- **工作量**: 1 天

### T-082: 兼容性矩阵远程更新

- **问题**: 兼容性规则硬编码在本地，Zig/ZLS 新版本发布后需手动更新代码
- **方案**: 从 GitHub 拉取最新兼容性规则并缓存（TTL: 24h），远程不可用时回退到内置规则
- **工作量**: 2 天 | **风险**: 低

### T-083: IDE 配置自动检测

- **问题**: 仅支持生成配置，未检测当前 IDE 是否安装 + 配置是否生效
- **方案**: 扫描 IDE 安装路径 + 验证配置文件内容
- **工作量**: 1.5 天

### T-042: IdeConfig 结构体重构

- **问题**: `IdeConfig` 所有字段都有 `vscode` 前缀，clippy 建议拆分
- **方案**: 拆分为嵌套结构体 `VsCodeConfig`
- **涉及文件**: `src/core/config.rs`

---

## ⚪ 边缘场景 & 遗留问题

### #002: Windows 长路径问题

- **问题**: Windows MAX_PATH (260) 限制，深层目录可能超限
- **方案**: 使用 `\\?\` 前缀扩展路径限制
- **优先级**: 低

### #003: 代理服务器支持

- **问题**: `reqwest::Client` 未读取 HTTP_PROXY/HTTPS_PROXY 环境变量
- **方案**: 使用 `reqwest::ClientBuilder::default_proxy(true)` 启用系统代理
- **优先级**: 中 | **工作量**: 0.5 天

### #004: 离线模式支持

- **问题**: 无网络时命令仍尝试网络请求
- **方案**: 检测网络可用性，离线时跳过远程请求，仅使用本地缓存
- **优先级**: 低

---

## 📐 实施路线图

| 阶段 | 交付内容 | 状态 |
|------|---------|------|
| **阶段 1** | T-060 Core 层输出解耦 + T-066 OnceCell | ✅ 完成 |
| **阶段 2** | T-061 泛型彻底化 + T-064 Commands 数据抽象 | ✅ 完成 |
| **阶段 3** | T-065 ProjectManager 完整实现 | ✅ 完成 |
| **阶段 4** | T-062 Interactive Wizard + T-063 restore 命令 | ✅ 完成 |
| **阶段 5** | #007 install 原子性 + T-025 集成测试 | ✅ 完成 |
| **阶段 6** | T-075 自我更新 + P3 体验优化项 | 🟡 进行中 |

---

## 📝 变更日志

| 日期 | 版本 | 修改内容 |
|-----|------|---------|
| 2026-04-28 | v6.1.0 | 移除已完成内容，已完成项迁移至 ROADMAP.md 和 MEMORY.md |
| 2026-04-28 | v6.0.0 | 根据 workflow 文档重新规划，阶段 6 启动 |