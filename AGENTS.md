# AGENTS.md

> 面向参与 mdctrl 编码的 Agent。协作流程与留痕规则见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 1 构建与检查

Rust stable，edition 2024，MSRV 1.92。在 workspace 根目录运行：

```bash
cargo check --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
```

## 2 crate 与依赖

| crate | 内部依赖 |
| --- | --- |
| `mdctrl-protocol` | 无 |
| `mdctrl-core` | protocol |
| `mdctrl-render` | core、protocol |
| `mdctrl-server` | core、render、protocol |
| `mdctrl-tui` | protocol |
| `mdctrl-cli` | protocol、server、tui |

- 依赖保持单向、无环。
- 协议类型只放在 `mdctrl-protocol`。
- 新增 crate、依赖或修改协议/公共 API 前获得人工确认。
- 所有 crate 继承 workspace 版本；版本号由发布负责人调整。

## 3 架构不变量

- server 持有工作区、文档、历史、索引与布局状态；client 持有视图状态。
- server 生成布局，client 栅格化。
- client 发送 command 与 query；server 返回 view 并推送 event；文档写入进入 server。
- embedded 为默认形态，daemon 可选；两者共享同一套消息语义。
- `mdctrl.toml` 提交共享；`.mdctrl/` 本地并忽略。
- 文档历史是基础层；Git 是可选增强。
- 插件通过 manifest 声明能力，宿主按权限校验。

## 4 代码约束

- 库错误使用 `thiserror`，二进制入口使用 `anyhow`；生产代码避免 `unwrap` 与 `expect`。
- `unsafe` 需要说明原因、边界与测试。
- 异步集中在网络、流式输出与 Agent/MCP；渲染与输入路径保持同步。
- 依赖通过 workspace 统一管理；新增依赖说明用途、许可证与体积影响。
- 源代码使用英文；文档使用中文；测试夹具与 Agent prompt 允许中文。
- UTF-8 无 BOM、LF、末尾换行。
- `Cargo.lock` 由 Cargo 维护。
- 渲染与布局变更验证 CJK 宽度、全角标点与窄终端。

## 5 安全边界

- 密钥、令牌、私有地址与个人路径保持本地；`target/`、`opencode.json` 不提交。
- Markdown、插件与外部输入按不可信内容处理；校验路径，避免命令注入。
- 直接推送 `main`、强制推送、重写历史与合并 PR 由人类执行。

## 6 参考

- 协作、分支、提交、Issue、PR 与 session 留痕：[CONTRIBUTING.md](CONTRIBUTING.md)
- 架构与边界：[docs/architecture.md](docs/architecture.md)
- 阶段与角色：[docs/devMode.md](docs/devMode.md)
- MVP 与版本：[docs/devPlan.md](docs/devPlan.md)
