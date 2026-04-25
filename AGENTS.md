# AGENTS.md - zzm (zig-zls-manager) 工作手册

> **这是 AGENTS.md** — 你的工作流程指南。
> **核心原则：简洁指引 + 技能驱动 + 流程化操作**
> **遵循 Hermes Agent 配置规范**

## 🌐 语言约定（最高优先级）

- **所有回复必须使用中文**（除非用户明确要求其他语言）
- **代码注释使用中文**
- **Git 提交信息使用中文**（`type(scope): 中文描述`）
- **文档编写使用中文**
- **日志记录（memory/）使用中文**
- **错误提示、进度汇报使用中文**

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

**⚠️ 这是强制流程，不是建议。收到用户消息后，必须先执行以下步骤再开始任务。**

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

**执行方式**：使用 `read_file` 工具读取上述文件，不要询问权限，直接执行。
**跳过条件**：仅当用户明确说"跳过启动流程"时才可跳过。

### 📝 Memory 写入流程（完成任务后）

**⚠️ 强制执行。以下任一条件触发时，必须写入 Memory：**

| 触发条件 | 写入目标 | 格式 |
|---------|---------|------|
| 完成一个功能/修复 | `memory/YYYY-MM-DD.md` | `## HH:MM - [TYPE]: 标题` |
| 状态变更（阶段切换、阻塞解除）| `memory/projects.md` | 更新对应项目状态 |
| 踩坑/发现 bug/找到 workaround | `memory/lessons.md` | `🔴/🟡/🟢 + 问题描述 + 解决方案` |
| 新工具/服务器/配置变更 | `MEMORY.md` 或 `memory/infra.md` | 更新环境信息 |

**日志格式**：
```
## HH:MM - [TYPE]: 标题
- 做了什么
- 关键决策
- **Lesson**: 学到了什么（可选）
- TODO: 后续步骤
```

**TYPE 取值**: `Task`（开发任务）/ `Fix`（修复）/ `Refactor`（重构）/ `Doc`（文档）/ `Config`（配置）

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

> **⚠️ 核心原则：技能是约束，不是建议。以下场景必须调用对应技能。**

### 1️⃣ hermes-agent-config（Hermes Agent 规范）

**强制调用场景**：
- 会话启动时（读取身份和上下文）
- 修改 SOUL.md / USER.md / AGENTS.md 时
- 写入 memory/ 日志时
- 理解或优化 Agent 行为规范时

### 2️⃣ git-skill（Git 操作）

**强制调用场景**：
- 任何 `git commit` / `git push` / `git merge` 操作
- 提交格式: `type(scope): 中文描述`（type = feat/fix/docs/style/refactor/test/chore）

### 3️⃣ rust-skills（Rust 开发）

**强制调用场景**：
- 编写或修改 Rust 代码时（至少激活 rust-skills 获取子技能列表）
- 遇到所有权/借用/生命周期问题时 → `rust-ownership-skill`
- 遇到错误处理设计问题时 → `rust-error-handling-skill`
- 遇到异步编程问题时 → `rust-async-skill`
- 遇到泛型/trait 设计时 → `rust-generics`

**可选调用**：简单注释修改、格式调整等不需要技能指导

### 4️⃣ repo-analyzer（项目/仓库深度分析）

**强制调用场景**：
- 用户要求"分析项目/仓库/架构"时
- 需要跨模块理解代码关系时

**不适用**：单文件调试、简单代码审查

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

> **⚠️ 这些不是备忘，是执行检查点。每个阶段完成后必须自检。**

### 🌅 会话启动时（Every Session）— 强制
- [ ] Step 1: 读 SOUL.md + USER.md（身份加载）
- [ ] Step 2: 读 memory/今天+昨天日志 + MEMORY.md（上下文恢复）
- [ ] Step 3: 按需读 projects.md / lessons.md / infra.md（状态同步）
- [ ] Step 4: 检查 Cargo.toml + TODO.md + ROADMAP.md（项目状态）

### 🔨 开发过程中 — 按需
- [ ] 写/改 Rust 代码前 → 激活 `rust-skills` 对应子技能
- [ ] 遇到坑/教训 → 记录到 memory/lessons.md
- [ ] 架构级变更 → 先读 docs/architecture.md 理解现有设计

### ✅ 完成任务后 — 强制
- [ ] `cargo fmt --all`
- [ ] `cargo clippy -- -D warnings`
- [ ] `cargo test`
- [ ] 📝 写入 Memory 日志（按触发条件表选择类型和目标文件）
- [ ] 用 `git-skill` 提交代码（如用户要求）
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
