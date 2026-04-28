## zig-zls-manager 项目概况

## 项目背景与定位
- **项目名称**: zig-zls-manager (zzm)
- **核心目标**: 同时管理 Zig 编译器与 ZLS (Zig Language Server) 的版本，解决两者版本兼容性问题。
- **技术栈**: Rust 2024 edition, clap, tokio, reqwest, serde
- **差异化优势**:
  - ZLS 作为一等公民独立管理，而非附属功能。
  - 维护 Zig ↔ ZLS 版本兼容性矩阵，提供自动检测与警告。
  - 支持项目级配置 (.zzmrc) 锁定版本组合。
  - 自动化 IDE (VS Code, Neovim, Helix) 配置生成。

## 当前状态 (2026-04-28)
- **阶段**: Phase 1 MVP + Phase 2 增强版 + Phase 2.5 稳定版全部完成 → 阶段 6 P3 体验优化进行中
- **编译**: ✅ cargo clippy -D warnings 零警告通过
- **测试**: ✅ 231/231 全部通过（214 单元 + 16 集成 + 1 文档）
- **代码质量**: 泛型抽象完成，代码重复度降低 50%+
- **启动速度**: < 100ms（优化前 > 200ms）

## 已完成功能清单

### 核心命令
- ✅ `zzm install <version>` - 安装 Zig（支持 --with-zls 并行下载）
- ✅ `zzm uninstall <version>` - 卸载 Zig
- ✅ `zzm use <version>` - 切换版本（--global/--project）
- ✅ `zzm list` - 列出版本（--installed/--remote）
- ✅ `zzm current` - 显示当前版本
- ✅ `zzm zls install/use/list/uninstall` - 独立 ZLS 管理
- ✅ `zzm ide config/check/path` - IDE 配置管理
- ✅ `zzm config list/get/set/edit` - 配置管理
- ✅ `zzm doctor` - 诊断工具
- ✅ `zzm clean` - 缓存清理
- ✅ `zzm prune` - 批量移除旧版本
- ✅ `zzm sync` - 同步兼容性矩阵
- ✅ `zzm pair` - 手动绑定版本关系
- ✅ `zzm restore` - 还原项目版本配置
- ✅ `zzm setup --wizard` - 交互式安装向导

### 架构优化（已完成）
- ✅ 统一 Channel 枚举（替代 ZigChannel/ZlsChannel）
- ✅ 统一目标三元组解析（platform::parse_target_triple）
- ✅ ToolManager<T> 泛型抽象 + VersionProvider trait（删除 ~400 行重复代码）
- ✅ ApiCache<T> 泛型缓存层
- ✅ 流式 SHA256 校验（内存恒定）
- ✅ VersionParts → Version 统一
- ✅ OutputDispatcher 统一输出调度
- ✅ AppContext OnceCell 懒加载
- ✅ PostInstallHook Trait 抽象
- ✅ 索引读取合并优化（install 3→1 次，use_version 2→1 次）
- ✅ 符号链接操作合并

### 稳定性增强（已完成）
- ✅ #006 并行下载 Zig+ZLS（`tokio::join!`）
- ✅ #007 install 原子性回滚（ZLS 失败时回滚 Zig）
- ✅ T-025 集成测试（16 个，覆盖索引/配置/兼容性/数据结构）
- ✅ lib.rs 公共 API 暴露

## 关键架构变更记录
- **新增模块**: 
  - `core::channel` - 统一通道枚举
  - `core::tool_manager` - 泛型工具管理器
  - `core::callbacks` - 回调机制
  - `core::project` - 项目配置管理
  - `infra::api_cache` - 泛型缓存层
  - `output::dispatcher` - 输出调度
  - `commands::pair` - 版本绑定命令
  - `commands::prune` - 版本清理命令
- **删除模块**: 
  - `core::zig_manager` - 合并到 ToolManager
  - `core::zls_manager` - 合并到 ToolManager

## 关键教训
- **serde 模型必须基于实际 API 响应设计**: Zig API 返回扁平平台键（如 `x86_64-macos`）+ `{ tarball, shasum, size }`，而非嵌套结构。使用 `#[serde(flatten)]` 捕获动态键名，`#[serde(rename)]` 解决命名冲突。
- **`is_platform_key()` 过滤模式**: 当 `#[serde(flatten)]` 捕获了非平台键（如 `src`、`bootstrap`）时，需在业务逻辑中通过白名单/特征匹配过滤。
- **泛型抽象的价值**: ToolManager<T> 统一了 Zig 和 ZLS 的管理逻辑，代码重复度从约 65% 降至 < 15%。

## 当前待办（阶段 6）
- **T-075**: `zzm update self` 自我更新功能
- **T-073**: ConfigManager 自动字段映射优化
- **T-080**: Shell 自动补全生成
- **T-081**: 清理 dead_code 标注
- **T-082**: 兼容性矩阵远程更新
- **T-083**: IDE 配置自动检测
- **T-042**: IdeConfig 结构体重构

## 边缘场景问题
- **#002**: Windows 长路径问题（低优先级）
- **#003**: 代理服务器支持（中优先级）
- **#004**: 离线模式支持（低优先级）

## 主要竞品
- `zigup`: 仅管理 Zig，已停止维护。
- `zvm (tristanisham)`: Go 实现，ZLS 为附属功能。
- `zvm (hendriknielaender)`: Zig 实现，有自动检测但 ZLS 管理有限。

## 预计收益
- 代码重复度降低 50%+
- 启动时间减少 30%+
- 下载速度提升 40%+（并行下载）
- 用户入门时间从 10 分钟降低到 2 分钟