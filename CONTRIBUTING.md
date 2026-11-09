# 贡献指南

> 欢迎参与 mdctrl。本文说明开发环境、命名、代码风格、分支、提交、Issue 与 Agent 协作留痕。

**相关文档**：[产品愿景](docs/vision.md) · [架构](docs/architecture.md) · [技术选型](docs/techStack.md) · [开发模式](docs/devMode.md) · [开发计划](docs/devPlan.md)

**目录**

- [1 欢迎与适用范围](#1-欢迎与适用范围)
- [2 开发环境与快速开始](#2-开发环境与快速开始)
  - [2.1 前置条件](#21-前置条件)
  - [2.2 快速开始](#22-快速开始)
- [3 项目结构与命名](#3-项目结构与命名)
  - [3.1 目录结构](#31-目录结构)
  - [3.2 命名规范](#32-命名规范)
- [4 代码风格与语言规则](#4-代码风格与语言规则)
  - [4.1 Rust 代码风格](#41-rust-代码风格)
  - [4.2 编码与换行](#42-编码与换行)
  - [4.3 语言规则](#43-语言规则)
- [5 本地检查](#5-本地检查)
- [6 分支模型](#6-分支模型)
  - [6.1 main](#61-main)
  - [6.2 MVP 分支](#62-mvp-分支)
  - [6.3 成员分支](#63-成员分支)
  - [6.4 合并方式](#64-合并方式)
- [7 Commit message](#7-commit-message)
- [8 Pull Request](#8-pull-request)
- [9 GitHub Issue](#9-github-issue)
  - [9.1 Issue 与成员的对应](#91-issue-与成员的对应)
  - [9.2 模板](#92-模板)
  - [9.3 Issue 内容](#93-issue-内容)
  - [9.4 标签与里程碑](#94-标签与里程碑)
  - [9.5 流程](#95-流程)
- [10 Agent coding 留痕](#10-agent-coding-留痕)
  - [10.1 目录与命名](#101-目录与命名)
  - [10.2 文件结构](#102-文件结构)
  - [10.3 记录规则](#103-记录规则)
- [11 文档、许可证与行为准则](#11-文档许可证与行为准则)

## 1 欢迎与适用范围

mdctrl 是一个仓库原生的 Markdown 文档控制台。项目采用 Rust workspace，当前处于 scaffold 阶段。

本文面向所有贡献者，覆盖：

- 本地开发与检查；
- 文件、代码与分支命名；
- Rust 代码风格与语言规则；
- 分支模型、Commit message 与 Pull Request；
- GitHub Issue 的使用；
- Agent coding session 的留痕。

阶段协作与角色规则见 [开发模式](docs/devMode.md)，模块结构见 [架构](docs/architecture.md)，技术选择见 [技术选型](docs/techStack.md)。

## 2 开发环境与快速开始

### 2.1 前置条件

- Rust stable 工具链；
- Git；
- 一个支持 UTF-8 的终端与编辑器；
- 媒体渲染相关的可选工具在对应技术文档中说明。

### 2.2 快速开始

```bash
git clone <repository>
cd mdctrl
cargo check --workspace
cargo run -p mdctrl-cli -- --version
```

常用检查：

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
```

## 3 项目结构与命名

### 3.1 目录结构

```text
crates/
  mdctrl-protocol/
  mdctrl-core/
  mdctrl-render/
  mdctrl-server/
  mdctrl-tui/
  mdctrl-cli/

docs/
  vision.md
  architecture.md
  techStack.md
  devMode.md
  devPlan.md
  crates/<crate>/
  MVPn/
  archive/sessions/
```

### 3.2 命名规范

| 对象 | 规范 | 示例 |
| --- | --- | --- |
| crate / package | kebab-case | `mdctrl-render` |
| Rust 模块 / 文件 | snake_case | `document_model.rs` |
| 类型 / trait / enum | PascalCase | `DocumentView` |
| 函数 / 方法 / 变量 | snake_case | `open_document` |
| 常量 / 静态 | SCREAMING_SNAKE_CASE | `MAX_WIDTH` |
| feature | kebab-case | `media-render` |
| 测试函数 | 描述性 snake_case | `renders_nested_lists` |
| crate 文档目录 | `docs/crates/<crate>/` | `docs/crates/mdctrl-core/` |
| MVP 文档目录 | `docs/MVPn/` | `docs/MVP1/` |
| MVP 设计文件 | 领域或任务名，小写 kebab-case | `design/renderer.md` |
| Agent session | `YYYY-MM-DD-<handle>-<topic>.md` | `2026-11-10-ylgd0915-libsimple-spike.md` |
| 分支 | `<type>/<n>-<topic>` | `feat/1-tui-tree` |

变量名表达意图；布尔值使用 `is_`、`has_`、`should_` 前缀；错误类型以 `Error` 结尾；缩写保持通行写法。

文件名与目录名使用英文，避免个人姓名；owner 记录在文档索引或 Issue 中。

## 4 代码风格与语言规则

### 4.1 Rust 代码风格

- 使用 `cargo fmt`，以 rustfmt 默认格式为准；
- CI 运行 `cargo clippy --workspace --all-targets -- -D warnings`；
- 库错误使用 `thiserror`，二进制入口使用 `anyhow` 汇总；
- 错误传递使用 `?`；`unwrap` 与 `expect` 用于测试代码；
- 默认使用安全 Rust；引入 `unsafe` 时在 PR 中说明原因、边界与测试；
- 公共 API 提供文档注释；模块顶部写模块级说明；
- 一个模块围绕一个职责；跨模块依赖遵循 [架构](docs/architecture.md)；
- 异步代码集中在网络、流式输出与 Agent/MCP；渲染与输入路径保持同步；
- 测试命名描述行为；快照测试变更必须评审；
- 一个 PR 聚焦一个变更主题；
- 依赖通过 workspace 统一管理，新增依赖说明用途、许可证与体积影响。

### 4.2 编码与换行

- 所有文本文件使用 UTF-8 无 BOM；
- 换行统一为 LF；
- 文件末尾保留一个换行；
- 提交前清理行尾空白；
- 编码与换行由 `.editorconfig` 与 `.gitattributes` 约束。

### 4.3 语言规则

| 对象 | 语言 |
| --- | --- |
| 源代码文件 | 英文；标识符、注释、文档注释、日志与错误信息使用英文 |
| 文档 | 中文 |
| Commit message | 中文允许 |
| Issue / PR | 中文允许 |
| 分支名 | 英文小写 kebab-case |
| 标签 | 英文 |
| 测试夹具 | 中文允许，作为 CJK 测试数据 |
| Agent prompt / session | 中文允许，作为源代码规则的例外 |

面向用户的文案放在资源文件或测试夹具中，按产品语言维护。

## 5 本地检查

提交 PR 前运行：

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
cargo run -p mdctrl-cli -- --version
```

CI 使用 stable 工具链，MSRV 为 1.92。

## 6 分支模型

### 6.1 main

`main` 是持续稳定分支。`main` 通过 Pull Request 更新，仓库拥有者完成合并。

### 6.2 MVP 分支

每个 MVP 建立独立工作分支：

```text
mvp/<n>-<slug>
```

示例：

```text
mvp/1-reading-search
mvp/2-deep-reading-review
```

MVP 分支承载该阶段全部集成工作，功能稳定、测试完善后向 `main` 发起 PR。

### 6.3 成员分支

成员在 MVP 分支下维护自己的分支，也可以按需要建立共享分支：

```text
feat/<n>-<topic>
test/<n>-<topic>
shared/<n>-<topic>
```

示例：

```text
feat/1-tui-tree
test/1-search-corpus
shared/1-render-protocol
```

成员分支持续集成到对应 MVP 分支。

### 6.4 合并方式

- 成员分支 → MVP 分支：squash merge；
- MVP 分支 → main：merge commit，保留阶段历史；
- 合并后删除成员分支；
- MVP 分支并入 main 后删除。

## 7 Commit message

格式：

```text
<type>(<scope>): <summary>
```

`scope` 可选：

```text
feat: 添加 --version 输出
feat(cli): 添加 --version 输出
fix(render): 修复 CJK 表格宽度
docs(contributing): 补充 Agent session 规则
```

type 使用：

- `feat`：新功能；
- `fix`：缺陷修复；
- `docs`：文档；
- `style`：格式调整；
- `refactor`：重构；
- `perf`：性能；
- `test`：测试；
- `build`：构建与依赖；
- `ci`：CI；
- `chore`：杂项；
- `revert`：回滚。

scope 使用 crate 或模块名，例如 `protocol`、`core`、`render`、`server`、`tui`、`cli`、`docs`、`ci`。

约定：

- type 与 scope 使用英文；
- summary 使用中文或英文；
- 一个 commit 表达一个逻辑变更；
- body 说明原因与取舍；
- footer 关联 Issue：

```text
Closes #12
Refs #18
```

Agent 参与的提交在 PR 中关联 session 记录，commit message 保持常规格式。

## 8 Pull Request

PR 描述包含：

- 关联 Issue；
- 变更内容与原因；
- 影响范围与接口变化；
- 测试命令与结果；
- 文档更新；
- Agent session 记录链接；
- 自查清单：fmt、clippy、test、docs；
- 评审人确认。

规则：

- 至少一名非作者评审；
- CI 通过后合入；
- 跨模块接口由使用方参与评审；
- 大改动先开 Issue 或设计讨论，再进入实现；
- 成员分支向 MVP 分支发起 PR；
- MVP 分支向 main 发起 PR 由仓库拥有者完成合并。

## 9 GitHub Issue

### 9.1 Issue 与成员的对应

每个成员为自己的工作维护显式 Issue：

- 一个任务对应一个 Issue；
- 一位成员承担多个任务时，维护多个 Issue；
- Issue 作为工作记录、验收入口与 PR 关联点。

### 9.2 模板

项目使用三个模板：

- **Task**：一个可实现的贡献任务；
- **Bug**：缺陷复现与修复；
- **Design**：需要讨论的设计或接口问题。

### 9.3 Issue 内容

- 标题描述目标，例如 `[MVP1] 添加文件树导航`；
- 背景与目标；
- 范围与非目标；
- 验收标准；
- 依赖与风险；
- owner 与测试执行人；
- 关联 `docs/MVPn/`；
- 关联 crate 或模块；
- Agent session 链接。

### 9.4 标签与里程碑

最小标签集：

- `area/protocol`、`area/core`、`area/render`、`area/server`、`area/tui`、`area/cli`、`area/docs`；
- `type/task`、`type/bug`、`type/design`；
- `status/blocked`；
- `good first issue`。

Issue 归属对应 `MVPn` Milestone；项目看板列为 Todo、In Progress、Review、Done。

### 9.5 流程

- 非平凡工作先建 Issue，再开分支与 PR；
- 小型文档修订可以直接提交 PR；
- PR 使用 `Closes #<issue>` 关闭对应 Issue；
- 设计讨论结论写回 `docs/MVPn/design/`；
- 安全漏洞通过仓库的私有漏洞报告渠道提交。

## 10 Agent coding 留痕

### 10.1 目录与命名

Session 记录存放在：

```text
docs/archive/sessions/
```

文件命名：

```text
YYYY-MM-DD-<handle>-<topic>.md
```

示例：

```text
docs/archive/sessions/2026-11-10-ylgd0915-libsimple-spike.md
docs/archive/sessions/2026-11-11-silkage15-tui-layout.md
```

### 10.2 文件结构

```markdown
---
date: 2026-11-10
model: <model>
harness: <harness>
issue: "#12"
contributor: <handle>
branch: mvp/1-reading-search
---

<!-- 以下为 harness 导出的 session，已剔除敏感信息 -->
```

### 10.3 记录规则

- 元数据保留日期、模型、Harness、关联 Issue、贡献者与分支；
- 正文直接使用 harness 导出的 session；
- 开发者维护元数据与脱敏，撰写详细总结属于可选工作；
- session 文件与对应 PR 一起提交到 MVP 分支；
- 阶段 manager 在阶段结束检查记录完整性；
- 提交前完成脱敏，覆盖密钥、令牌、个人数据、私有地址与第三方受限内容；
- session 记录纳入项目双许可。

## 11 文档、许可证与行为准则

- 文档位于 `docs/`，与代码同 PR 更新；
- 文档结构、MVP 文档目录与 crate 文档规则见 [开发模式](docs/devMode.md)；
- 项目采用 MIT OR Apache-2.0 双许可，详见 [LICENSE](LICENSE)；
- 参与讨论与提交即表示同意遵守项目行为准则与安全政策。
