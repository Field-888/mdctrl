# mdctrl

[![CI](https://img.shields.io/github/actions/workflow/status/Field-888/mdctrl/ci.yml?branch=main&style=flat-square&label=CI)](.github/workflows/ci.yml)
[![Rust stable](https://img.shields.io/badge/rust-stable-orange?style=flat-square&logo=rust)](rust-toolchain.toml)
[![Docs](https://img.shields.io/badge/docs-zh-blue?style=flat-square)](docs/README.md)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)](LICENSE)

> 仓库原生的 Markdown 文档控制台。
> A repository-native Markdown document console for the terminal.

mdctrl 在终端中阅读、检索、检查与编辑仓库内的 Markdown 文档，并把文档网络、历史与 Agent 协作放在同一个工作区里。

mdctrl reads, searches, checks, and edits Markdown documents in a repository from the terminal, bringing the document network, history, and agent collaboration into one workspace.

## 当前状态 / Current Status

项目处于脚手架阶段。Rust workspace、六个 crate、CI、文档基线与协作规范已经建立；功能实现从 MVP1 开始。

The project is in the scaffold stage. The Rust workspace, six crates, CI, documentation baseline, and collaboration rules are in place. Feature work starts with MVP1.

当前 CLI 提供 `--version` 与 `--help`。

The CLI currently provides `--version` and `--help`.

## 目标能力 / Target Capabilities

| 方向 Direction | 内容 Scope |
| --- | --- |
| 阅读与呈现 / Reading and presentation | 终端排版、主题、数学公式、Mermaid、能力降级 / Terminal layout, themes, math, Mermaid, graceful degradation |
| 文档网络与治理 / Document network and governance | 检索与导航、生命周期、断链与过期检测、CI 输出 / Search and navigation, lifecycle, broken-link and staleness checks, CI output |
| 交互与编辑 / Interaction and editing | TUI 三栏浏览、CLI 命令、渲染态热编辑 / Three-pane TUI, CLI commands, editing in the rendered view |
| 扩展机制 / Extensions | 命令、数据源、导出器与视图插件；manifest、权限与故障隔离 / Plugins for commands, data sources, exporters, and views; manifest, permissions, and fault isolation |
| Agent 协作 / Agent collaboration | 内置 agent、只读 MCP、写作、分析与审阅工作流、写入审批与溯源 / Built-in agent, read-only MCP, writing, analysis, and review workflows, write approval and provenance |

## 架构概览 / Architecture

- 模块化单体 + 协议 C/S / Modular monolith with a protocol-based client/server split.
- 默认嵌入式运行，daemon 可选 / Embedded mode by default; daemon optional.
- server 持有文档状态并生成布局，client 负责终端栅格化 / The server owns document state and produces layout; the client rasterizes it for the terminal.
- 六个 crate / Six crates: `mdctrl-protocol`、`mdctrl-core`、`mdctrl-render`、`mdctrl-server`、`mdctrl-tui`、`mdctrl-cli`.

详见[架构文档](docs/architecture.md)。 / See the [architecture document](docs/architecture.md).

## 快速开始 / Quick Start

需要 Rust stable，MSRV 1.92。

Rust stable is required; MSRV is 1.92.

```bash
cargo check --workspace
cargo test --workspace
cargo run -p mdctrl-cli -- --version
```

阅读、检索与编辑能力从 MVP1 开始，当前从源码构建。

Reading, search, and editing start with MVP1. Build from source for now.

## 仓库结构 / Repository Layout

```text
crates/       Rust crates
docs/         Baseline, MVP, and crate docs; session archive
.github/      CI, issue templates, and PR template
```

六个 crate 的职责与依赖方向见[架构文档](docs/architecture.md)。

See the [architecture document](docs/architecture.md) for crate responsibilities and dependency direction.

## 文档 / Documentation

| 文档 Document | 内容 Content |
| --- | --- |
| [文档索引](docs/README.md) Document index | 全部文档入口 / Entry point for all docs |
| [产品愿景](docs/vision.md) Vision | 定位、用户与能力 / Positioning, users, and capabilities |
| [架构](docs/architecture.md) Architecture | 系统结构与 crate 边界 / System structure and crate boundaries |
| [技术选型](docs/techStack.md) Tech stack | 技术选择与验证 / Technology choices and verification |
| [开发模式](docs/devMode.md) Development model | 阶段、角色与完成定义 / Stages, roles, and completion criteria |
| [开发计划](docs/devPlan.md) Development plan | MVP、分工与版本 / MVPs, ownership, and versions |

## 贡献 / Contributing

欢迎通过 Issue 与 PR 参与。开始前阅读[贡献指南](CONTRIBUTING.md)；使用编码 Agent 时遵循 [AGENTS.md](AGENTS.md)；安全漏洞按[安全政策](SECURITY.md)私下报告。

Issues and pull requests are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) before starting; coding agents follow [AGENTS.md](AGENTS.md); report security vulnerabilities privately according to [SECURITY.md](SECURITY.md).

## 许可证 / License

MIT OR Apache-2.0，详见 [LICENSE](LICENSE)。

Licensed under MIT OR Apache-2.0. See [LICENSE](LICENSE).
