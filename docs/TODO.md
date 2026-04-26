# Zig/ZLS 版本管理器 - 待办事项

## 📋 文档信息

- **版本**: v4.1.0
- **更新日期**: 2026-04-26
- **适用版本**: zig-zls-manager v0.1.0+
- **关联文档**: [ROADMAP.md](./ROADMAP.md) | [architecture.md](./architecture.md) | [architecture-optimization-v2.md](./analyses/architecture-optimization-v2.md)
- **当前阶段**: Phase 1 MVP + 架构优化重构完成 → Phase 2 架构彻底化 + 功能实现
- **编译状态**: ✅ cargo clippy -D warnings 零警告
- **测试状态**: ✅ 190/190 全部通过

---

## ✅ 已完成（Phase 1 架构优化重构 + Phase 2 阶段 1）

| 任务 | 描述 | 关键变更 |
|------|------|---------|
| T-044 | 统一 Channel 枚举 | 合并 `ZigChannel`/`ZlsChannel` → `core::channel::Channel` |
| T-045 | 合并目标三元组解析 | 统一到 `platform::parse_target_triple()` |
| T-046 | 合并 VersionParts 和 Version | 删除 `VersionParts`，改用 `Version::from_str` |
| T-047 | ToolManager 泛型抽象 | 新增 `ToolManager<T: VersionProvider>`，净删除 ~400 行重复代码 |
| T-048 | ApiCache 泛型缓存层 | 新增 `ApiCache<T>` 消除缓存逻辑重复 |
| T-049 | 流式 SHA256 校验 | `BufReader` 流式校验，内存恒定 |
| T-050 | ToolManager 单元测试 | +24 测试 (157→181) |
| T-052 | Zig API serde 模型修复 | 重写 `ZigVersionEntry`/`ZigPlatformAsset` 适配实际 API (+6 测试) |
| T-043 | 数字字面量可读性 | 添加下划线分隔符 |
| T-060 | Core 层输出解耦 | 新增 `InstallCallbacks` 结构体，`ToolManager`/`CompatibilityChecker` 不再直接调用 `console_output`，Commands 层注入回调 |
| T-066 | AppContext OnceCell 懒加载 | `PathManager` 改为 `OnceLock` 单例复用，`path_manager()` 返回引用 |

---

## 🔴 P0 - 必须完成（架构彻底化 + 核心功能）

### T-061: 泛型彻底化 — 引入 ToolIndexEntry 统一数据结构

- **问题**: `InstalledIndex` 内部双份数据结构（`zig_versions` + `zls_versions`），导致 15+ 个 `match self.kind` 分支
- **方案**: 引入 `ToolIndexEntry` + `ToolExtraData` 统一数据层
- **实现**:
  ```rust
  pub struct ToolIndexEntry {
      pub version: String,
      pub install_path: PathBuf,
      pub installed_at: String,
      pub extra: ToolExtraData,
  }
  pub enum ToolExtraData {
      Zig { channel: Channel },
      Zls { zig_version: Option<String> },
  }
  // InstalledIndex 改为:
  pub struct InstalledIndex {
      pub tools: HashMap<ToolKind, Vec<ToolIndexEntry>>,
      pub active: HashMap<ToolKind, String>,
  }
  ```
- **收益**: 12+ 索引方法统一实现，消除所有 `match` 分支；新增工具只需添加 `ToolExtraData` 变体
- **风险**: `installed.json` 序列化格式变化，需提供旧格式兼容读取迁移策略
- **涉及文件**: `src/core/tool_manager.rs`, `src/infra/path_manager.rs`
- **工作量**: 5 天 | **风险**: 中
- **验证**: 190+ 测试全通过 + clippy 零警告 + 旧索引文件兼容读取

### T-062: 交互式 Setup Wizard

- **问题**: `spec.md` US-01 定义但未实现
- **方案**: 交互式引导用户选择 Zig 版本 → 推荐 ZLS 版本 → 确认安装 → 配置 IDE
- **依赖**: T-060（输出解耦）、T-065（ProjectManager）
- **涉及文件**: `src/commands/setup.rs`（重写）
- **工作量**: 3 天 | **风险**: 低

### T-063: `zzm restore` 命令

- **问题**: `spec.md` US-04 定义但未实现
- **方案**: 读取项目 `.zzmrc` 配置 → 安装缺失的 Zig/ZLS 版本 → 切换到项目指定版本
- **依赖**: T-065（ProjectManager）
- **涉及文件**: `src/commands/restore.rs`（新增）
- **工作量**: 2 天 | **风险**: 低

---

## 🟡 P1 - 应该完成（架构完善 + 重要功能）

### T-064: Commands 层数据转换抽象（OutputDispatcher）

- **问题**: 4 个命令重复"数据转换 + 输出调度"逻辑（`if json { json_output } else { table_output }`）
- **方案**: 提取 `OutputRow` trait + `output_list()` 统一调度函数
- **实现**:
  ```rust
  pub trait OutputRow {
      fn to_json(&self) -> serde_json::Value;
      fn to_table_row(&self) -> Vec<String>;
      fn table_headers() -> Vec<&'static str>;
  }
  pub fn output_list<T: OutputRow>(data: &[T], json: bool) { ... }
  ```
- **涉及文件**: `src/output/`（新增 dispatcher）, `src/commands/list.rs`, `src/commands/zls.rs`, `src/commands/install.rs`, `src/commands/info.rs`
- **工作量**: 2 天 | **风险**: 低

### T-065: ProjectManager 完整实现

- **问题**: `.zzmrc` 项目级配置是核心差异化功能，当前为空壳
- **方案**: 实现完整的 `ProjectManager`
- **实现**:
  ```rust
  pub struct ProjectConfig {
      pub zig: String,
      pub zls: Option<String>,
      pub compatibility: CompatibilityMode,
      pub ide: Option<String>,
      pub notes: Option<String>,
  }
  impl ProjectManager {
      pub fn find_project_config(&self, from_dir: &Path) -> Option<(PathBuf, ProjectConfig)>;
      pub fn init(&self, dir: &Path, config: &ProjectConfig) -> Result<()>;
      pub async fn restore(&self, dir: &Path) -> Result<RestoreResult>;
  }
  ```
- **查找策略**: 从当前目录向上递归查找 `.zzmrc`（JSON/TOML），也支持 `.zzm/config.toml` 目录格式
- **涉及文件**: `src/core/project.rs`（新增/重写）
- **工作量**: 3 天 | **风险**: 低

### T-067: `zzm sync` 功能增强

- **问题**: 当前 sync 功能过于简单，未基于兼容性矩阵推荐最优组合
- **方案**: 基于兼容性矩阵推荐 Zig+ZLS 最优版本组合
- **依赖**: T-065（ProjectManager）
- **涉及文件**: `src/commands/setup.rs`
- **工作量**: 2 天 | **风险**: 低

### T-068: `zzm pair` 手动绑定版本关系

- **问题**: `spec.md` §2.1.3 定义但未实现
- **方案**: 允许用户手动指定 Zig↔ZLS 版本对应关系，写入兼容性矩阵
- **涉及文件**: `src/commands/pair.rs`（新增）
- **工作量**: 1.5 天 | **风险**: 低

---

## 🟢 P2 - 可以完成（代码质量 + 辅助功能）

### T-070: PostInstallHook Trait 抽象

- **问题**: `post_install()` 用 `if self.kind == ToolKind::Zls` 硬编码特例
- **方案**: `VersionProvider` trait 新增可选 `post_install_hook()` 方法，ZlsApiClient 实现，ZigApiClient 返回 `Ok(())`
- **涉及文件**: `src/core/tool_manager.rs`, `src/infra/zig_api.rs`, `src/infra/zls_api.rs`
- **工作量**: 1 天 | **风险**: 低

### T-071: 索引读取合并优化

- **问题**: `install()` 中 `installed.json` 读取 3 次，`use_version()` 中读取 2 次
- **方案**: 合并读取逻辑，单次读取后传递引用
- **涉及文件**: `src/core/tool_manager.rs`
- **工作量**: 1 天 | **风险**: 低

### T-072: 符号链接操作合并

- **问题**: `use_version()` 连续调用 4 个符号链接方法
- **方案**: 合并为 `update_version_symlinks()` 和 `remove_version_symlinks()`
- **涉及文件**: `src/core/tool_manager.rs`
- **工作量**: 0.5 天 | **风险**: 低

### T-073: ConfigManager 自动字段映射

- **问题**: `get()`/`set()` 使用 `match` 逐字段映射，新增配置项需改 3 处
- **方案**: serde 自动反射或宏驱动映射
- **涉及文件**: `src/core/config.rs`
- **工作量**: 2 天 | **风险**: 中

### T-074: `zzm prune` 移除旧版本

- **问题**: `spec.md` §2.3.2 定义但未实现
- **方案**: 列出未使用的旧版本 → 确认 → 批量卸载
- **涉及文件**: `src/commands/prune.rs`（新增）
- **工作量**: 1.5 天 | **风险**: 低

### T-075: `zzm update self` 自我更新

- **问题**: `spec.md` §2.3.2 定义但未实现
- **方案**: 从 GitHub Releases 检测新版本 → 下载替换当前二进制
- **涉及文件**: `src/commands/update.rs`（新增）
- **工作量**: 2 天 | **风险**: 中

### T-076: `zzm doctor` 诊断增强

- **问题**: 当前检查项不全
- **方案**: 补充环境变量、符号链接有效性、磁盘空间、兼容性等检查项
- **涉及文件**: `src/commands/doctor.rs`
- **工作量**: 1 天 | **风险**: 低

### T-025: 编写集成测试

- **问题**: `tests/integration/` 目录为空
- **目标**: `test_install_flow.rs`, `test_switch_flow.rs`, `test_ide_integration.rs`，使用 tempfile 创建临时环境
- **工作量**: 3 天 | **风险**: 低

---

## 🔵 P3 - 未来考虑（体验优化 + 边缘场景）

### T-080: Shell 自动补全生成

- **问题**: `spec.md` §2.3.3 定义但未实现
- **方案**: clap 自带 `clap_complete` 生成 Bash/Zsh/Fish/PowerShell 补全脚本
- **工作量**: 1 天

### T-081: 清理 dead_code 标注

- **问题**: `PlatformTrait` 有 3 个 `#[allow(dead_code)]` 方法未实现
- **方案**: 系统性审查——要么实现，要么用 `cfg(feature)` 控制
- **工作量**: 1 天

### T-082: 兼容性矩阵远程更新

- **问题**: 兼容性规则硬编码在本地
- **方案**: 从 GitHub 拉取最新兼容性规则并缓存
- **工作量**: 2 天

### T-083: IDE 配置自动检测

- **问题**: 仅支持生成配置，未检测当前 IDE 是否安装 + 配置是否生效
- **方案**: 扫描 IDE 安装路径 + 验证配置文件内容
- **工作量**: 1.5 天

### T-042: IdeConfig 结构体重构

- **问题**: `IdeConfig` 所有字段都有 `vscode` 前缀，clippy 建议拆分
- **方案**: 拆分为嵌套结构体 `VsCodeConfig`
- **涉及文件**: `src/core/config.rs:70`

### #002: Windows 长路径问题

- MAX_PATH 限制，可能影响深层目录操作

### #003: 代理服务器支持

- HTTP_PROXY 环境变量未处理

### #004: 离线模式支持

- 纯本地操作模式未实现

### #006: 并行下载 Zig + ZLS

- 当前串行，应改为 `tokio::join!`

### #007: install 原子性

- 任一失败需回滚两者

---

## 📐 实施路线图

```
阶段 1 ✅ ─→ 阶段 2 ──→ 阶段 3 ───→ 阶段 4 ────→ 阶段 5
T-060 ✅      T-061       T-064        T-062         T-067
T-066 ✅      T-064       T-065        T-063         T-068
(输出         (泛型       (Commands    (Wizard       (sync增强
 解耦+        彻底化+     数据抽象+    +restore)     +pair)
 OnceCell)   Commands)  Project)
```

| 阶段 | 交付内容 | 验证方式 |
|------|---------|---------|
| **阶段 1** | T-060 Core 层输出解耦 + T-066 OnceCell | ✅ `cargo test` 190/190 + clippy 零警告 |
| **阶段 2** | T-061 泛型彻底化 + T-064 Commands 数据抽象 | 190+ 测试全通过 + clippy 零警告 |
| **阶段 3** | T-065 ProjectManager 完整实现 | 创建测试项目，验证 `.zzmrc` 读取/写入 |
| **阶段 4** | T-062 Interactive Wizard + T-063 restore 命令 | 手动测试新用户体验流程 |
| **阶段 5** | T-067 sync 增强 + T-068 pair 命令 + P2/P3 项 | 手动测试 + 集成测试 |

---

## 📝 变更日志

| 日期 | 版本 | 修改内容 |
|-----|------|---------|
| 2026-04-26 | v4.1.0 | 完成 T-060 Core 层输出解耦(InstallCallbacks) + T-066 AppContext OnceLock 懒加载，阶段 1 完成 |
| 2026-04-26 | v4.0.0 | 基于 architecture-optimization-v2.md 全面重写：按优先级矩阵(P0-P3)重新组织，新增 T-060~T-083，引入实施路线图(5阶段) |
| 2026-04-25 | v3.2.0 | 修复 T-052 Zig API serde 模型不匹配，重写适配实际 API 结构(+6测试) |
| 2026-04-25 | v3.1.0 | 完成 T-050 ToolManager 单元测试(+24)、T-043 数字字面量可读性优化 |
| 2026-04-25 | v3.0.0 | 完成 T-044~T-049 架构优化重构：ToolManager 泛型抽象、统一 Channel/目标三元组/版本解析、ApiCache 泛型缓存、流式校验 |