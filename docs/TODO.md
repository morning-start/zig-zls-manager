# Zig/ZLS 版本管理器 - 待办事项

## 📋 文档信息

- **版本**: v6.0.0
- **更新日期**: 2026-04-28
- **适用版本**: zig-zls-manager v0.1.0+
- **关联文档**: [ROADMAP.md](./ROADMAP.md) | [architecture.md](./architecture.md) | [analysis.md](../workflow/analysis.md)
- **当前阶段**: 阶段 6 - 自我更新 + P3 体验优化

---

## ✅ 已完成

| 任务 | 描述 | 关键变更 |
|------|------|---------|
| T-044 | 统一 Channel 枚举 | 合并 `ZigChannel`/`ZlsChannel` → `core::channel::Channel` |
| T-045 | 合并目标三元组解析 | 统一到 `platform::parse_target_triple()` |
| T-046 | 合并 VersionParts 和 Version | 删除 `VersionParts`，改用 `Version::from_str` |
| T-047 | ToolManager 泛型抽象 | 新增 `ToolManager<T: VersionProvider>`，净删除 ~400 行重复代码 |
| T-048 | ApiCache 泛型缓存层 | 新增 `ApiCache<T>` 消除缓存逻辑重复 |
| T-049 | 流式 SHA256 校验 | `BufReader` 流式校验，内存恒定 |
| T-050 | ToolManager 单元测试 | +24 测试 (157→181) |
| T-052 | Zig API serde 模型修复 | 重写适配实际 API (+6 测试) |
| T-043 | 数字字面量可读性 | 添加下划线分隔符 |
| T-061 | 泛型彻底化 — ToolIndexEntry | `InstalledIndex` → `HashMap<ToolKind, Vec<ToolIndexEntry>>` |
| T-062 | 交互式 Setup Wizard | `cmd_setup_wizard()` 使用 dialoguer 交互式引导 |
| T-063 | `zzm restore` 命令 | 新增 `ProjectManager` + `restore` 子命令 |
| T-064 | Commands 层 OutputDispatcher | `OutputRow` trait + `output_list()` 统一调度 |
| T-065 | ProjectManager 完整实现 | `save`/`set_zig_version`/`set_zls_version`/`resolve_zls_version` |
| T-066 | AppContext OnceCell 懒加载 | `PathManager` 改为 `OnceLock` 单例复用 |
| T-067 | `zzm sync` 功能增强 | 兼容性矩阵推荐 + dry-run + LikelyCompatible 状态处理 |
| T-068 | `zzm pair` 命令 | 手动绑定 Zig↔ZLS 版本关系，写入 .zzmrc |
| T-070 | PostInstallHook Trait 抽象 | `VersionProvider::post_install_hook()` 默认实现 |
| T-071 | 索引读取合并优化 | `install()` 3→1 次，`use_version()` 2→1 次 |
| T-072 | 符号链接操作合并 | 4 个方法合并为 `update_version_symlinks()` + `remove_version_symlinks()` |
| T-074 | `zzm prune` 移除旧版本 | `PrunableVersion(OutputRow)` + `batch_uninstall` + 交互确认 |
| T-076 | `zzm doctor` 诊断增强 | 环境变量/符号链接有效性/磁盘空间/兼容性检查 |
| #006 | 并行下载 Zig+ZLS | `download_only()` + `install_from_cache()` + `tokio::join!` |
| #007 | install 原子性回滚 | ZLS 安装失败时回滚 Zig，保持一致性 |
| T-025 | 集成测试 | 16 个集成测试（索引/配置/兼容性/数据结构） |

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

```
阶段 1 ✅ ─→ 阶段 2 ✅ ──→ 阶段 3 ✅ ──→ 阶段 4 ✅ ──→ 阶段 5 ✅ ──→ 阶段 6 🟡
T-060 ✅      T-061 ✅       T-064 ✅       T-062 ✅       #007 ✅       T-075
T-066 ✅      T-064 ✅       T-065 ✅       T-063 ✅       T-025 ✅       P3 项
(输出         (泛型         (Commands      (Wizard       (原子性+       (自我更新+
 解耦+        彻底化+       数据抽象+      +restore)     集成测试)     体验优化)
 OnceCell)   Commands)    Project)
```

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
| 2026-04-28 | v6.0.0 | 根据 workflow 文档重新规划，阶段 6 启动 |
| 2026-04-26 | v5.2.0 | 完成 #007 install 原子性回滚 + T-025 集成测试，P2 全部完成 |
| 2026-04-26 | v5.1.0 | 精简 TODO：移除已完成项详细描述 |
| 2026-04-26 | v5.0.0 | 完成 #006 并行下载，重新规划优先级 |
| 2026-04-26 | v4.5.0 | 完成 T-070/T-071/T-072/T-074/T-076 |
| 2026-04-26 | v4.3.0 | 完成 T-062 Setup Wizard + T-063 restore 命令 |
| 2026-04-26 | v4.2.0 | 完成 T-061 泛型彻底化 |
| 2026-04-26 | v4.0.0 | 基于 architecture-optimization-v2.md 全面重写 |