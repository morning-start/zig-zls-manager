# Zig/ZLS 版本管理器文档中心

欢迎使用 **zig-zls-manager (zzm)** 项目文档！本目录包含了项目的完整技术文档和使用指南。

---

## 📚 文档导航

### 1. [需求规格说明书 (spec.md)](./spec.md)

**适用人群**: 产品经理、项目负责人、开发者

**内容概要**:
- 项目背景与目标
- 详细的功能需求（Zig 管理、ZLS 管理、联合管理、IDE 集成）
- 非功能性需求（性能、兼容性、可用性、安全性）
- 用户故事与验收标准
- 竞品对比分析
- 术语表

**何时阅读**:
- 了解项目整体需求和功能范围
- 评估功能优先级和实现难度
- 编写测试用例和验收标准

---

### 2. [技术架构设计 (architecture.md)](./architecture.md)

**适用人群**: 系统架构师、后端开发者、技术负责人

**内容概要**:
- 系统分层架构（CLI 层、业务逻辑层、基础设施层、平台抽象层）
- 模块详细设计（命令行解析、Zig/ZLS 管理器、兼容性检查器、配置管理器、IDE 集成）
- 数据流设计
- 错误处理策略
- 扩展性设计
- 代码结构示例

**何时阅读**:
- 开始编码前的系统设计理解
- 新增功能时的架构参考
- 代码审查和质量保证

---

### 3. [用户使用指南 (usage.md)](./usage.md)

**适用人群**: 最终用户、开发者、DevOps 工程师

**内容概要**:
- 快速开始（安装、首次运行向导）
- 核心功能使用（Zig 版本管理、ZLS 版本管理、联合管理）
- 项目级版本管理
- IDE 集成（VS Code、Neovim、Helix）
- 常见使用场景（新项目初始化、多项目并行、团队协作、CI/CD）
- 高级功能（配置管理、缓存管理、自动补全、Shell 钩子）
- 故障排除

**何时阅读**:
- 首次使用 zzm 工具
- 查找特定命令的使用方法
- 解决使用过程中遇到的问题

---

### 4. [竞品分析 (comparison.md)](./comparison.md)

**适用人群**: 产品决策者、市场分析师、技术选型负责人

**内容概要**:
- 三个主要竞品的详细分析（zigup、zvm-tristanisham、zvm-hendriknielaender）
- 功能对比矩阵
- 技术栈对比
- 维护状态评估
- 本项目的差异化优势
- 市场定位建议

**何时阅读**:
- 进行技术选型决策
- 了解市场竞争格局
- 制定产品差异化策略

---

### 5. [外部 API 参考文档 (api-reference.md)](./api-reference.md)

**适用人群**: 后端开发者、API 集成工程师、DevOps 工程师

**内容概要**:
- Zig 官方下载 API（https://ziglang.org/download/index.json）
  - JSON 数据结构详解
  - 字段说明和类型定义
  - 多平台二进制文件信息
  - Rust 代码示例
- ZLS GitHub Releases API（https://api.github.com/repos/zigtools/zls/releases）
  - 完整 JSON 结构示例
  - Asset 对象字段说明
  - 文件命名规范
  - 平台映射关系
- API 集成最佳实践
  - 缓存策略实现
  - 错误处理与重试机制
  - 并发请求优化
  - User-Agent 设置规范
- 统一数据模型设计
  - 版本信息结构体定义
  - 数据转换函数
  - 跨平台抽象层
- 测试与验证方法
  - 单元测试示例
  - Mock 数据生成
  - 快速测试命令
- 注意事项与限制
  - 速率限制说明
  - 认证方式配置
  - URL 稳定性提示

**何时阅读**:
- 开发版本查询和下载功能
- 集成外部 API 到 zzm 工具
- 理解数据结构和字段含义
- 编写 API 相关的单元测试

---

## 🎯 快速入门路径

### 对于新用户

1. **5 分钟上手**: 阅读 [usage.md - 快速开始](./usage.md#1-快速开始)
2. **核心功能**: 浏览 [usage.md - 核心功能使用指南](./usage.md#2-核心功能使用指南)
3. **常见问题**: 查阅 [usage.md - 故障排除](./usage.md#5-故障排除)

### 对于开发者

1. **了解需求**: 阅读 [spec.md](./spec.md) 的第 1-3 章
2. **理解架构**: 精读 [architecture.md](./architecture.md) 的第 1-2 章
3. **开始编码**: 参考 [architecture.md](./architecture.md) 的模块详细设计
4. **测试验证**: 对照 [spec.md](./spec.md) 的用户故事和验收标准

### 对于技术决策者

1. **项目价值**: 阅读 [spec.md - 项目概述](./spec.md#1-项目概述)
2. **竞品分析**: 精读 [comparison.md](./comparison.md)
3. **技术方案**: 浏览 [architecture.md - 架构总览](./architecture.md#1-架构总览)
4. **实施计划**: 查看 [spec.md - 约束条件](./spec.md#5-约束条件)

---

## 📊 文档统计

| 文档 | 行数 | 字数（估算） | 最后更新 |
|-----|------|------------|---------|
| spec.md | ~450 | ~8,000 | 2026-04-23 |
| architecture.md | ~1,540 | ~25,000 | 2026-04-23 |
| usage.md | ~1,290 | ~20,000 | 2026-04-23 |
| comparison.md | ~360 | ~6,000 | 2026-04-23 |
| **总计** | **~3,640** | **~59,000** | - |

---

## 🔗 外部资源

- **Zig 官方网站**: https://ziglang.org/
- **Zig 语言圣经**: https://course.ziglang.cc/
- **ZLS GitHub 仓库**: https://github.com/zigtools/zls
- **zigup**: https://github.com/marler8997/zigup
- **zvm (tristanisham)**: https://github.com/tristanisham/zvm
- **zvm (hendriknielaender)**: https://github.com/hendriknielaender/zvm

---

## 📝 文档维护

### 更新流程

1. **需求变更**: 先更新 `spec.md`，然后同步到 `architecture.md` 和 `usage.md`
2. **架构调整**: 更新 `architecture.md`，并在 `spec.md` 中记录变更原因
3. **功能新增**: 同时更新 `spec.md`（需求）、`architecture.md`（设计）、`usage.md`（使用说明）
4. **版本发布**: 更新所有文档的版本号和日期

### 文档规范

- 使用 Markdown 格式
- 代码块指定语言类型（```rust、```bash、```json 等）
- 表格对齐，保持可读性
- 使用 emoji 增强视觉效果（适度）
- 中文为主，技术术语保留英文

### 贡献指南

欢迎提交文档改进建议：
- 发现错别字或表述不清 → 提交 Issue
- 补充缺失内容 → 提交 Pull Request
- 翻译其他语言 → 创建对应语言的文档副本

---

## 💡 反馈与建议

如果您在使用文档过程中遇到任何问题或有改进建议，请通过以下方式反馈：

- 🐛 **报告问题**: 创建 GitHub Issue
- 💬 **讨论交流**: 加入项目 Discord/Slack 频道
- 📧 **邮件联系**: 发送邮件至项目维护者

---

*本文档将持续更新，以反映项目的最新进展。*

**最后更新时间**: 2026-04-23
