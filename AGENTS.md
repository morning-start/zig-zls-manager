# AGENTS.md - zzm (zig-zls-manager) 工作手册

> **这是 AGENTS.md** — 你的工作流程指南。
> **核心原则：简洁指引 + 技能驱动 + 流程化操作**
> **遵循 Hermes Agent 配置规范**

---

## ⚠️ 项目范围约束

**所有操作必须在当前项目目录内执行：**

- **工作根目录**: `d:\Workplace\APP\Rust\zig-zls-manager`
- **文件读写**: 仅操作项目内的文件（`src/`、`memory/`、`docs/` 等）
- **命令执行**: `cargo` 命令在项目根目录运行
- **Git 操作**: 仅在当前仓库内提交/分支/合并
- **Memory 文件**: 写入 `memory/` 目录（项目内）
- **分析报告**: 输出到 `docs/analyses/`

**❌ 禁止行为**:
- 修改项目外的文件
- 在其他目录执行破坏性操作
- 将文件输出到项目外路径

---

## 项目速览

**zzm (zig-zls-manager)** — Zig/ZLS 联合版本管理器 CLI 工具。

- **技术栈**: Rust 2024 + clap + tokio + reqwest + serde
- **架构**: 分层架构（CLI → Core → Infra → Platform）
- **详细文档**: [docs/architecture.md](./docs/architecture.md)（架构设计）、[docs/spec.md](./docs/spec.md)（需求规格）、[docs/usage.md](./docs/usage.md)（使用指南）

---

## ⚡ 核心流程：什么时候做什么

### 🌅 每次启动会话（Every Session）

**必须执行** — 使用 **hermes-agent-config** 技能规范：

```
┌─ Step 1: 身份加载 ──────────────────────────────┐
│ 读取 SOUL.md     → 知道你是谁（身份、风格）      │
│ 读取 USER.md     → 知道你在帮谁（用户档案）      │
└──────────────────────────────────────────────────┘
         ↓
┌─ Step 2: 上下文恢复 ────────────────────────────┐
│ 读取 memory/YYYY-MM-DD.md（今天）→ 今日工作日志  │
│ 读取 memory/YYYY-MM-DD.md（昨天）→ 昨日上下文   │
│ [主会话] 读取 MEMORY.md → 索引层（项目/环境概览）│
└──────────────────────────────────────────────────┘
         ↓
┌─ Step 3: 状态同步（按需）───────────────────────┐
│ 有活跃项目？ → 读取 memory/projects.md           │
│ 需要避坑？    → 读取 memory/lessons.md           │
│ 部署相关？    → 读取 memory/infra.md             │
└──────────────────────────────────────────────────┘
         ↓
┌─ Step 4: 项目状态 ─────────────────────────────┐
│ 检查 Cargo.toml 依赖                             │
│ 查看 docs/TODO.md 待办事项                       │
│ 查看 docs/ROADMAP.md 路线图                     │
└──────────────────────────────────────────────────┘
```

**⚠️ 不要询问权限，直接执行。**

### 📝 Memory 写入流程（完成任务后）

**必须执行** — 使用 **hermes-agent-config** 的 Memory 规范：

```
┌─ 判断写入类型 ───────────────────────────────────┐
│                                                   │
│  📅 日志记录 → 追加到 memory/YYYY-MM-DD.md       │
│     格式: ## HH:MM - [TYPE]: 标题                │
│           - 做了什么                              │
│           - 关键决策                              │
│           - **Lesson**: 学到了什么（可选）        │
│           - TODO: 后续步骤                        │
│                                                   │
│  📊 项目进展 → 更新 memory/projects.md            │
│     触发: 完成功能、状态变更、阻塞解决             │
│                                                   │
│  💡 教训记录 → 追加到 memory/lessons.md           │
│     触发: 踩坑、发现 bug、找到 workaround          │
│     标记: 🔴 Critical / 🟡 Warning / 🟢 Tip      │
│                                                   │
│  🔧 环境变更 → 更新 MEMORY.md 或 memory/infra.md  │
│     触发: 新工具安装、服务器变更、配置更改         │
│                                                   │
└───────────────────────────────────────────────────┘
```

**容量管理**：MEMORY.md 保持 <40 行，定期归档旧信息。

### 🔨 开发任务流程

| 你要做什么 | 调用什么技能 | 关键命令 |
|-----------|------------|---------|
| **写/改 Rust 代码** | `rust-skills` | 查子技能表选对应的 |
| **提交代码** | `git-skill` | Conventional Commits 格式 |
| **改 SOUL / USER / Memory** | `hermes-agent-config` | 遵循 Hermes 规范 |
| **分析项目/仓库/架构** | `repo-analyzer` | 深度分析 + 生成报告 |
| **运行/测试/构建** | 直接执行 | 见下方命令 |

---

## 📦 开发命令速查

```bash
# 构建 + 运行
cargo build && cargo run -- <command>

# 测试
cargo test

# 代码质量（提交前必须通过）
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings

# 完整开发循环
cargo fmt && cargo clippy -- -D warnings && cargo test

# 发布构建
cargo build --release
```

---

## 🛠 技能调用指南

### 1️⃣ hermes-agent-config（Hermes Agent 规范）

**⚠️ 所有涉及以下操作时，必须使用此技能：**
- 修改 SOUL.md / USER.md / AGENTS.md
- 管理 memory/ 日志文件
- 理解或优化 Agent 行为规范

**调用方式**: 使用 hermes-agent-config 技能，遵循其 Session 启动和 Memory 写入规范。

### 2️⃣ git-skill（Git 操作）

**⚠️ 所有 Git 操作使用此技能：**
- 提交: `type(scope): description`
- type = feat / fix / docs / style / refactor / test / chore

**调用方式**: 使用 git-skill 技能完成。

### 3️⃣ rust-skills（Rust 开发）

**按场景选择子技能：**

| 场景 | 子技能 |
|------|--------|
| 所有权/借用 | rust-ownership-skill |
| 错误处理 | rust-error-handling-skill |
| 异步编程 | rust-async-skill |
| CLI 开发 | rust-cli-project-skill |
| 测试 | rust-testing-doc-skill |
| 其他 | 查看 `.trae/skills/rust-skills/SKILL.md` 完整列表 |

**调用方式**: 使用 rust-skills 技能集中的对应子技能。

### 4️⃣ repo-analyzer（项目/仓库深度分析）

**⚠️ 涉及以下场景时，使用此技能（仅限本项目）：**
- **分析项目/仓库**: "分析这个项目"、"架构分析"、"源码分析"
- **学习/研究**: "研究这个框架"、"看看怎么实现的"
- **输出**: 在 `docs/analyses/` 下生成专业架构报告（含 Mermaid 图）

**不适用**: 单文件调试、简单代码审查、非架构层面的修改

---

## 📂 项目结构索引

```
zzm/
├── src/
│   ├── main.rs, cli.rs      # 入口与命令定义
│   ├── core/                # 业务逻辑（配置、Zig/ZLS 管理）
│   ├── infra/               # 基础设施（下载器、API 封装）
│   ├── platform/            # 平台抽象（Windows/Linux/macOS）
│   ├── output/              # 输出格式化
│   └── utils/               # 工具函数
├── docs/                    # 项目文档中心
├── memory/                  # 分层记忆存储
├── SOUL.md / USER.md / MEMORY.md  # Agent 配置
```

详细架构说明见 [docs/architecture.md](./docs/architecture.md)

---

## ✅ 工作检查清单

### 🌅 会话启动时（Every Session）
- [ ] Step 1: 读 SOUL.md + USER.md（身份加载）
- [ ] Step 2: 读 memory/今天+昨天日志 + MEMORY.md（上下文恢复）
- [ ] Step 3: 按需读 projects.md / lessons.md / infra.md（状态同步）
- [ ] Step 4: 检查 Cargo.toml + TODO.md + ROADMAP.md（项目状态）

### 🔨 开发过程中
- [ ] 用 `rust-skills` 对应子技能指导 Rust 实现
- [ ] 遇到坑/教训 → 准备记录到 memory/lessons.md

### ✅ 完成任务后（必须执行）
- [ ] `cargo fmt --all`
- [ ] `cargo clippy -- -D warnings`
- [ ] `cargo test`
- [ ] **📝 写入 Memory 日志**（按流程选择类型）
- [ ] 用 `git-skill` 提交代码
- [ ] 文档联动更新（新功能→usage.md / API变更→api-reference.md / 架构调整→architecture.md）

---

## 关键约定

### 代码规范
- Rust 2024 edition
- 错误处理：`anyhow`
- 日志：`tracing`
- 异步：`tokio`

### 文档联动规则
- 新功能 → [docs/usage.md](./docs/usage.md)
- API 变更 → [docs/api-reference.md](./docs/api-reference.md)
- 架构调整 → [docs/architecture.md](./docs/architecture.md)

---

## 故障排除

| 问题 | 解决方案 |
|------|---------|
| 编译失败 | `cargo update` 后重试 |
| 测试失败 | `cargo test -- --nocapture` 查看详情 |
| 平台兼容 | 检查 `src/platform/` 对应实现 |
| API 变更 | 查看 [docs/api-reference.md](./docs/api-reference.md) |

---

## 🔗 快速链接

| 文件 | 用途 |
|------|------|
| [Cargo.toml](./Cargo.toml) | 依赖和项目配置 |
| [SOUL.md](./SOUL.md) | Agent 身份定义 |
| [USER.md](./USER.md) | 用户偏好档案 |
| [MEMORY.md](./MEMORY.md) | 长期记忆索引 |
| [docs/README.md](./docs/README.md) | 文档中心导航 |

---

## ⚠️ 注意事项

- 修改 `core/` 前务必阅读 [docs/architecture.md](./docs/architecture.md)
- 新平台支持需实现 `platform/trait_def.rs` 中定义的 trait
- 下载相关修改需考虑网络异常和断点续传
- **Memory 有字符限制**，只保存高价值信息，定期清理过时内容
