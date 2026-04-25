# Zig/ZLS 版本管理器 - 项目路线图

## 📋 文档信息

- **版本**: v1.0.0
- **创建日期**: 2026-04-24
- **状态**: 活跃
- **关联文档**: [spec.md](./spec.md), [architecture.md](./architecture.md), [TODO.md](./TODO.md)

---

## 🎯 项目愿景

构建一个专业级的 **Zig + ZLS 联合版本管理 CLI 工具**，成为 Zig 生态系统中最优秀的版本管理解决方案。

### 核心价值主张

1. **ZLS 作为一等公民**: 独立的 ZLS 管理能力，而非附属功能
2. **智能兼容性**: 自动检测和警告版本不匹配问题
3. **开发者体验**: 一键初始化、IDE 集成、项目级配置
4. **跨平台一致性**: Windows/macOS/Linux 统一的用户体验
5. **高性能与安全**: Rust 实现 + SHA256 校验 + 安全下载

---

## 📅 总体时间线

```
2026 Q2 ┃──────────────────────────────────────┐
        │  Phase 1: MVP (v0.1.0)               │
        │  核心功能 + 基础 IDE 集成              │
        ├──────────────────────────────────────┤
2026 Q3 ┃                                      │
        │  Phase 2: 增强版 (v0.2.0)            │
        │  独立 ZLS 管理 + 项目配置 + 多 IDE    │
        ├──────────────────────────────────────┤
2026 Q4 ┃                                      │
        │  Phase 2.5: 稳定版 (v0.2.5)          │
        │  性能优化 + 错误处理完善               │
        ├──────────────────────────────────────┤
2027 Q1 ┃                                      │
        │  Phase 3: 专业版 (v0.3.0+)           │
        │  GUI/TUI + 插件系统 + 团队协作        │
        └──────────────────────────────────────┘
```

---

## 🚀 Phase 1: MVP (v0.1.0)

**目标**: 提供可用的核心版本管理功能，满足个人开发者基本需求

**预计周期**: 8-10 周（2026年5月-7月）

**状态**: 🟡 进行中（Sprint 0-5 核心功能完成，测试与发布准备中）

### 1.1 核心功能范围

#### ✅ 必须完成 (Must Have)

##### A. 项目基础架构
- [x] Rust 项目初始化（Cargo, Edition 2024）
- [x] 目录结构搭建（src/, tests/, docs/）
- [x] 依赖库集成（clap, tokio, reqwest, serde 等）
- [x] 错误类型定义（thiserror）
- [x] 日志系统配置（tracing）

##### B. CLI 框架
- [x] 命令行参数解析（clap derive）
- [x] 全局选项支持（--json, --verbose, --no-color）
- [x] 子命令路由系统
- [x] 帮助信息生成
- [x] 版本信息显示

##### C. 外部 API 集成
- [x] Zig JSON API 客户端
  - GET https://ziglang.org/download/index.json
  - 数据结构定义（serde Deserialize）
  - 平台自动检测与匹配
  - 本地缓存机制（TTL: 1小时）
- [x] ZLS GitHub Releases API 客户端
  - GET https://api.github.com/repos/zigtools/zls/releases
  - Release/Asset 数据结构定义
  - 分页处理
  - 认证支持（可选 GitHub Token）
  - 速率限制处理

##### D. Zig 版本管理（核心）
- [x] `zzm install <version>` - 安装指定版本
  - 版本号解析（完整格式、简写如 "0.13" → "0.13.0"）
  - 远端版本列表查询
  - 文件下载（带进度条）
  - SHA256 校验和验证
  - 解压到 ~/.zzm/versions/zig/<version>/
  - 创建符号链接/shim 到 ~/.zzm/bin/zig
- [x] `zzm uninstall <version>` - 卸载版本
  - 删除版本目录
  - 清理符号链接（如果是当前活动版本）
  - 更新 installed.json 元数据
- [x] `zzm list` - 列出版本
  - `--installed`: 已安装版本
  - `--remote`: 远端可用版本
  - 表格输出（使用 tabled）
  - `--json`: JSON 格式输出
- [x] `zzm use <version>` - 切换版本
  - 更新 ~/.zzm/bin/zig 符号链接
  - `--global` (默认): 用户全局切换
  - `--project`: 项目级切换（写入 .zzmrc）— Phase 2
  - `--default`: 设为默认版本
- [x] `zzm current` - 显示当前版本
  - 当前激活的 Zig 版本
  - 安装路径
  - 构建日期（如果可用）

##### E. ZLS 基础管理（附属模式）
- [x] `zzm install <version> --with-zls`
  - 自动查找匹配的 ZLS 版本
  - 并发下载 Zig + ZLS
  - 自动配置兼容性
- [x] `zzm zls current` - 显示当前 ZLS 版本
- [x] `zzm zls list --installed` - 已安装的 ZLS

##### F. 基础 IDE 集成
- [x] VS Code 支持
  - 生成 `.vscode/settings.json`
  - 配置 zig.path 和 zig.zls.path
- [x] `zzm ide path` - 输出当前工具路径
  - 格式: JSON 或纯文本
  - 供其他工具或脚本引用

##### G. 基础设施层
- [x] 下载管理器
  - HTTP/HTTPS 请求（reqwest）
  - 进度回调（indicatif ProgressBar）
  - 断点续传（Range 请求）— 未实现
  - 超时处理
  - 重试机制（指数退避）
- [x] 文件系统操作
  - tar.gz / tar.xz / zip 解压（flate2, xz2, zip crate）
  - 符号链接创建（跨平台）
  - Windows shim 可执行文件生成
  - 目录创建与清理
- [x] 路径管理器
  - ~/.zzm/ 目录结构初始化
  - bin/, versions/, cache/ 管理
  - PATH 环境变量更新提示
- [x] 校验和验证
  - SHA256 计算（sha2 crate）
  - minisign 签名验证（可选）— 未实现
- [x] 缓存管理
  - 下载缓存存储
  - TTL 过期清理
  - 磁盘空间监控

##### H. 配置管理
- [x] TOML 配置文件读写
  - ~/.zzm/config.toml
  - 配置结构体定义
  - 默认值处理
  - 配置合并策略（用户 > 系统 > 内置）
- [x] `zzm config list/get/set/edit`

##### I. 输出系统
- [x] 彩色终端输出（console crate）
- [x] 表格展示（tabled crate）
- [x] 进度条显示（indicatif crate）
- [x] JSON 输出模式（serde_json）
- [x] 成功/错误/警告消息样式

##### J. 平台抽象层
- [x] Platform trait 定义
- [x] Windows 适配器实现
- [x] macOS 适配器实现
- [x] Linux 适配器实现
- [x] 运行时平台检测

##### K. 测试框架
- [ ] 单元测试（核心模块覆盖率 > 80%）— 当前 51 个测试（50 通过/1 失败），覆盖率不足
- [ ] 集成测试（关键流程）— 目录为空，未编写
- [ ] Mock 数据生成
- [ ] CI/CD 基础配置（GitHub Actions）— 未配置
  - Linux 构建
  - Windows 构建
  - macOS 构建
  - Lint 检查（clippy）
  - 格式化检查（rustfmt）

#### ⚠️ 可以延后 (Should Have - MVP+)

- [x] 交互式 setup wizard (`zzm setup --wizard`) — 框架已实现，向导内容待完善
- [x] Shell 自动补全脚本生成 (`zzm completion`)
- [x] `zzm doctor` 诊断程序
- [x] `zzm clean` 缓存清理
- [ ] Neovim/Helix IDE 集成
- [x] 兼容性警告系统（Phase 2 完善）

### 1.2 技术里程碑

| 里程碑 | 目标 | 验收标准 | 预计时间 |
|-------|------|---------|---------|
| **M1: 项目骨架** | 可运行的空壳程序 | `zzm --help` 正常工作 | 第 1 周 |
| **M2: API 集成** | 能查询远程版本 | `zzm list --remote` 返回版本列表 | 第 2-3 周 |
| **M3: 下载系统** | 能下载并解压 | 手动触发下载成功 | 第 4-5 周 |
| **M4: 核心安装** | install/use/uninstall 完整流程 | 三大命令正常工作 | 第 6-7 周 |
| **M5: ZLS 集成** | --with-zls 参数工作 | 同时安装 Zig + ZLS | 第 8 周 |
| **M6: VS Code** | IDE 配置生成 | `zzm ide config vscode` 生效 | 第 9 周 |
| **M7: 发布准备** | CI/CD + 文档 + 测试 | 所有测试通过，可发布 v0.1.0 | 第 10 周 |

### 1.3 交付物清单

- ✅ zzm 可执行文件（Windows/macOS/Linux x86_64）
- ✅ 完整源代码（Rust）
- ✅ 单元测试 + 集成测试
- ✅ README.md（快速开始指南）
- ✅ CHANGELOG.md
- ✅ GitHub Actions CI 配置
- ✅ 基础文档（usage.md 使用指南）

---

## 🔧 Phase 2: 增强版 (v0.2.0)

**目标**: 完善独立 ZLS 管理能力，增加项目级配置和多 IDE 支持

**预计周期**: 10-12 周（2026年8月-10月）

**状态**: ⚪ 规划中

### 2.1 新增功能

#### A. 独立 ZLS 版本管理
- [ ] `zzm zls install <version>` - 独立安装 ZLS
  - 从 GitHub Releases 下载预编译二进制
  - 或从源码编译（`--from-source`）
  - 指定使用的 Zig 版本进行编译
- [ ] `zzm zls uninstall <version>`
- [ ] `zzm zls use <version>` - 独立切换 ZLS 版本
- [ ] `zzm zls list --remote` - 查看 ZLS 可用版本
- [ ] ZLS 版本元数据管理
  - compatible_zig 字段
  - build_requirements 字段
  - install_mode (Prebuilt/Source)

#### B. 兼容性矩阵系统
- [ ] 内置兼容性规则数据库
  - VersionRange 匹配
  - recommended_zls 推荐
  - known_issues 已知问题
- [ ] `zzm sync` - 同步到推荐组合
  - 自动检测最佳匹配
  - `--dry-run` 预览模式
- [ ] 智能警告系统
  - ⚠️ 版本不匹配警告
  - ❌ 严重不兼容阻止
  - 💡 升级建议
- [ ] 兼容性矩阵在线更新（可选）
  - 从远程 URL 加载最新规则
  - 本地缓存 + TTL

#### C. 项目级配置 (.zzmrc)
- [ ] `zzm init` - 初始化项目配置
  - 交互式选择版本
  - 生成 .zzmrc 文件（JSON/TOML）
- [ ] `zzm use --project` - 项目级版本设置
  - 写入 .zzmrc
  - 向上递归查找最近的 .zzmrc
- [ ] `zzm restore` - 还原项目依赖版本
  - 读取 .zzmrc
  - 自动安装缺失版本
  - 切换到正确版本组合
- [ ] .zzmrc 格式支持
  ```json
  {
    "zig": "0.13.0",
    "zls": "0.13.0",
    "compatibility": "strict"
  }
  ```

#### D. 扩展 IDE 集成
- [ ] Neovim 支持
  - lspconfig 配置片段
  - 自动检测 zzm 路径
- [ ] Helix 支持
  - languages.toml 生成
  - LSP server 配置
- [ ] `zzm ide check` - 检测 IDE 配置状态
- [ ] `zzm ide doctor` - 诊断常见问题
  - 路径有效性
  - 版本匹配性
  - 配置完整性

#### E. 用户体验提升
- [ ] 交互式 Setup Wizard
  - 引导新用户
  - 选择默认版本
  - 选择编辑器
  - 一键初始化
- [ ] Shell 补全脚本
  - Bash
  - Zsh
  - PowerShell
  - Fish
- [ ] `zzm info` 详细信息展示
  - 当前环境完整状态
  - 已安装版本列表
  - 兼容性状态
  - 项目配置信息
- [ ] `zzm clean` 缓存管理
  - `--all`: 清理所有缓存
  - `--dry-run`: 预览
  - 智能建议（基于磁盘空间）
- [ ] `zzm doctor` 诊断程序
  - PATH 检查
  - 权限检查
  - 网络连接测试
  - 配置文件校验

### 2.2 性能与稳定性优化

- [ ] 启动速度优化（目标 < 100ms）
  - 延迟加载非必需模块
  - 配置文件懒读取
- [ ] 并发下载优化
  - Zig + ZLS 并行下载
  - 连接复用（HTTP/2）
- [ ] 内存占用优化（目标 < 50MB 运行时）
- [ ] 错误处理完善
  - 用户友好的中文错误消息
  - 建议修复措施
  - 错误码体系

---

## 🎨 Phase 2.5: 稳定版 (v0.2.5)

**目标**: 优化性能、完善错误处理、提升用户体验

**预计周期**: 4-6 周（2026年11月-12月）

**状态**: ⚪ 规划中

### 2.5.1 重点改进

#### A. 性能优化
- [ ] 冷启动性能剖析和优化
- [ ] 大量版本场景下的性能测试
- [ ] 缓存命中率优化
- [ ] 并发安全测试和修复

#### B. 稳定性增强
- [ ] 边界情况处理
  - 网络中断恢复
  - 磁盘空间不足
  - 权限不足降级
  - 并发访问冲突
- [ ] 日志系统完善
  - 结构化日志输出
  - 日志级别控制
  - 日志轮转
- [ ] 健壮性测试
  - 混沌工程模拟
  - 压力测试

#### C. 用户体验打磨
- [ ] 错误消息国际化（中英双语）
- [ ] 进度条美化
- [ ] 输出格式优化
- [ ] 文档完善
  - 使用视频教程
  - FAQ 整理
  - 故障排除指南

---

## 🚀 Phase 3: 专业版 (v0.3.0+)

**目标**: 高级功能、插件系统、团队协作支持

**预计周期**: 2027 年 Q1-Q3

**状态**: ⚪ 远期规划

### 3.1 高级功能

#### A. TUI/GUI 界面（可选）
- [ ] Terminal UI（使用 ratatui 库）
  - 交互式版本浏览器
  - 可视化安装向导
  - 图形化配置编辑器
- [ ] Web Dashboard（可选）
  - 本地 HTTP 服务
  - 浏览器界面
  - REST API

#### B. 插件系统
- [ ] 插件 API 设计
  - Hook 点定义
  - 事件系统
  - 沙箱执行
- [ ] 示例插件
  - 自定义下载源插件
  - 通知插件（Discord/Slack）
  - 统计上报插件
- [ ] 插件市场（远期）

#### C. 团队协作功能
- [ ] 锁文件支持（zzm.lock）
  - 固定精确版本
  - 团队共享
  - Git 友好
- [ ] 团队配置模板
  - 组织级标准配置
  - 继承和覆盖机制
- [ ] CI/CD Docker 镜像
  - 官方镜像维护
  - 多版本支持
  - 轻量化变体

#### D. 高级诊断与分析
- [ ] 性能监控（可选启用）
  - 操作耗时统计
  - 匿名遥测
- [ ] 使用统计面板
  - 版本分布
  - 操作频率
  - 平台分布
- [ ] 智能建议引擎
  - 基于历史行为的推荐
  - 异常检测

### 3.2 生态系统扩展

- [ ] 包管理器集成
  - Homebrew (macOS/Linux)
  - Scoop (Windows)
  - Winget (Windows)
  - Cargo (crates.io)
- [ ] 编辑器插件
  - VS Code 扩展（高级功能）
  - JetBrains 插件
- [ ] 第三方集成
  - GitHub Actions Action
  - GitLab CI 模板
  - Dockerfile 模板

---

## 📊 关键指标与目标

### 技术指标

| 指标 | Phase 1 目标 | Phase 2 目标 | Phase 3 目标 |
|-----|-------------|-------------|-------------|
| **启动速度** | < 200ms | < 100ms | < 50ms |
| **内存占用** | < 80MB | < 50MB | < 30MB |
| **测试覆盖率** | > 70% | > 85% | > 90% |
| **二进制大小** | < 15MB | < 12MB | < 10MB |
| **API 响应缓存命中率** | > 80% | > 95% | > 98% |

### 用户体验指标

| 指标 | Phase 1 | Phase 2 | Phase 3 |
|-----|---------|---------|---------|
| **首次安装成功率** | > 90% | > 95% | > 99% |
| **命令学习曲线** | 5 分钟上手 | 3 分钟上手 | 即学即用 |
| **错误自解决率** | > 60% | > 80% | > 90% |
| **NPS（净推荐值）** | > 40 | > 60 | > 75 |

### 生态指标

| 指标 | Phase 1 | Phase 2 | Phase 3 |
|-----|---------|---------|---------|
| **GitHub Stars** | > 100 | > 500 | > 2000 |
| **活跃贡献者** | 1-3 人 | 5-10 人 | > 20 人 |
| **包管理器分发** | 仅源码 | 2-3 个 | 5+ 个 |
| **IDE 插件** | VS Code 基础 | 3 个 IDE | 5+ 个 |

---

## ⚠️ 风险与缓解措施

### 技术风险

| 风险 | 可能性 | 影响 | 缓解措施 |
|-----|-------|------|---------|
| Zig API 格式变更 | 中 | 高 | 版本锁定 + 适配层 + 监控 |
| GitHub API 限制 | 中 | 中 | 缓存优化 + Token 认证 |
| 跨平台兼容性问题 | 高 | 高 | 充分测试 + CI 多平台 |
| 性能瓶颈 | 低 | 中 | Profiling + 早期基准测试 |

### 业务风险

| 风险 | 可能性 | 影响 | 缓解措施 |
|-----|-------|------|---------|
| 竞品抢先实现类似功能 | 中 | 中 | 快速迭代 + 差异化优势 |
| Zig 1.0 发布改变生态 | 低 | 高 | 灵活架构 + 快速适应 |
| 维护者精力不足 | 中 | 高 | 社区建设 + 贡献者培养 |

---

## 🔄 版本发布计划

### 发布节奏

- **Phase 1**: 每 2 周一个 alpha/beta 版本
- **Phase 2**: 每月一个小版本（patch），每季度一个大版本（minor）
- **Phase 3**: 按需发布，保持稳定

### 版本命名规范

遵循 **SemVer 2.0.0**:
- **MAJOR**: 不兼容的 API 变更
- **MINOR**: 向后兼容的功能新增
- **PATCH**: 向后兼容的问题修正

**预发布标识符**:
- `alpha`: 内部测试
- `beta`: 公开测试
- `rc`: 候选版本
- 无标识符: 正式发布

**示例**:
- `v0.1.0-alpha.1` → `v0.1.0-beta.1` → `v0.1.0-rc.1` → `v0.1.0`
- `v0.1.0` → `v0.1.1` → `v0.1.2` (patch)
- `v0.1.0` → `v0.2.0` (minor, 新功能)

---

## 📝 决策记录

### ADR-004: Phase 优先级决策

**背景**: 资源有限，需要确定功能实现的先后顺序

**决定**:
1. **Phase 1 先做 Zig 核心 + ZLS 附属模式**
   - 理由: Zig 是主要工具，ZLS 作为附加功能降低复杂度
   - 风险: 可能无法满足需要独立管理 ZLS 的用户
2. **Phase 2 再做独立 ZLS 管理**
   - 理由: 积累经验后再处理更复杂的场景
   - 收益: 可以做得更好，避免过度设计

**替代方案**:
- A: 一开始就做完整的独立管理（风险高，周期长）❌
- B: 只做 Zig 管理，不做 ZLS（不符合产品定位）❌
- C: **当前方案**（渐进式演进）✅

### ADR-005: IDE 支持优先级

**背景**: 需要决定先支持哪些编辑器

**决定**:
1. **Phase 1: VS Code only**
   - 理由: 市场占有率最高（~70%），配置最简单
2. **Phase 2: Neovim + Helix**
   - 理由: 开发者社区常用，配置标准化程度高
3. **Phase 3: JetBrains + Emacs + Sublime**
   - 理由: 小众但有忠实用户群体

---

## 🙏 贡献指南

我们欢迎所有形式的贡献！详见 [CONTRIBUTING.md](../CONTRIBUTING.md)（待创建）。

### 如何参与

1. **报告 Bug**: 创建 Issue，提供复现步骤
2. **功能建议**: 讨论 Feature Request
3. **代码贡献**: Fork → 开发 → PR → Review → Merge
4. **文档改进**: 修正错误或补充内容
5. **测试反馈**: 在你的环境中测试并提供反馈

### 开发规范

- 遵循 Rust 2018 edition 编码规范
- 使用 `rustfmt` 格式化代码
- 通过 `clippy` 检查
- 为公共 API 编写文档注释
- 为新功能编写测试

---

## 📞 联系方式

- **Issues**: [GitHub Issues](https://github.com/user/zig-zls-manager/issues)
- **Discussions**: [GitHub Discussions](https://github.com/user/zig-zls-manager/discussions)
- **Email**: 待定

---

## 📜 变更日志

| 版本 | 日期 | 修改内容 |
|-----|------|---------|
| v1.0.0 | 2026-04-24 | 初始版本，建立完整路线图框架 |
| v1.1.0 | 2026-04-25 | Phase 1 状态审查：更新已完成项标记（B-J 大部分已完成），标注测试/CI 待办 |

---

*本文档将随着项目进展持续更新，反映最新的规划决策。*

**下次评审日期**: 2026-05-24（Phase 1 启动后 1 个月）
