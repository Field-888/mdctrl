# 命令组合

> 文档状态：草案 · 最后复核：2026-11-14

> 每条命令都是“解析 → 校验 → 连接 → 请求 → 输出”的固定管线，组合规则保持可预测。

**相关文档**：[mdctrl-cli](README.md) · [输出](output.md) · [数据命令](data-commands.md) · [架构](../../architecture.md#8-协议边界)

## 1 语法

```text
mdctrl [全局选项] <命令> [命令选项] [位置参数...]

命令      := init | open | search | find | status | serve
           | check | index | mcp | plugin | help
全局选项  := --workspace <dir> | --json | --no-color | --quiet
           | --verbose | --version | --help
```

规则：

- 全局选项可出现在命令之前或之后，行为一致；
- `--` 之后的内容按位置参数处理；
- 选项值缺省或类型错误返回用法错误（退出码 2）；
- 未知命令不回退到模糊匹配，直接报错并提示 `help`；
- 帮助文本由同一份命令定义生成，避免文档与实现漂移。

## 2 命令定义

### 2.1 `init`

```text
mdctrl init [--force]
```

- 解析或创建工作区根目录；
- 创建 `.mdctrl` 及其子目录；
- 确保 `.mdctrl/` 被忽略；
- 写入 `mdctrl.toml` 与本地 schema 版本；
- 幂等：已初始化时不覆盖共享配置，除非显式 `--force`；
- 输出创建项与跳过项。

### 2.2 `open`

```text
mdctrl open <path> [--pane <pane>]
```

- 解析路径并检查工作区边界；
- 连接或启动 server；
- 请求初始 View；
- 把终端交给 `mdctrl-tui`；
- TUI 退出后恢复终端并返回其退出结果。

`open` 不接受 `--json`；需要结构化读取时使用 `search`、`check` 或 `status`。

### 2.3 `search`

```text
mdctrl search <query> [--limit <n>] [--path <glob>] [--json]
```

- 发送全文查询；
- 默认输出分组的人类可读结果；
- `--json` 输出结构化命中列表；
- 结果排序由 server 决定，CLI 不重排；
- 中文查询按 server 分词处理。

### 2.4 `find`

```text
mdctrl find <pattern> [--limit <n>] [--json]
```

- 文件名与路径查找；
- 默认输出相对工作区根的路径；
- 结果去重并保持稳定顺序；
- 与 TUI 文件名查找共享同一数据源。

### 2.5 `status`

```text
mdctrl status [--json]
```

- 报告工作区根、初始化状态、server 连接、索引状态与版本；
- 只读，不触发索引重建；
- 人类可读输出使用键值对，结构化输出使用稳定字段。

### 2.6 `serve`

```text
mdctrl serve [--foreground]
```

- 前台运行 daemon，独占 `.mdctrl/lock`；
- 监听本地 IPC；
- 收到 SIGINT/SIGTERM 后优雅关闭并释放锁；
- 端口或套接字路径写入本地状态，供 client 发现；
- 启动失败时区分“已有实例”（退出码 4）与配置错误（退出码 3）。

### 2.7 `check`

```text
mdctrl check [paths...] [--rule <id>]... [--severity <level>]
             [--fail-on <level>] [--json]
```

- 范围默认是整个工作区，可传入若干路径；
- `--rule` 可重复，限制规则集合；
- `--severity` 只过滤 `findings` 的展示，不影响 `summary` 与 `--fail-on` 判定；
- `--fail-on` 基于未过滤的完整结果决定退出码阈值；
- 执行成功时 `ok=true`、`error=null`；达到 `--fail-on` 阈值返回 5，否则返回 0；
- 详细数据契约见[数据命令](data-commands.md)。

### 2.8 `index`

```text
mdctrl index status [--json]
mdctrl index update [paths...] [--json | --json-stream]
mdctrl index rebuild [--json | --json-stream] [--yes]
```

- `status` 报告索引版本、文档数、最近更新时间；
- `update` 增量更新，未变化文档跳过；
- `rebuild` 保留现有索引，在临时位置全量构建并校验，成功后原子切换；失败时清理临时产物并继续使用旧索引，要求确认或 `--yes`；
- 默认进度写入 stderr；`--json-stream` 时进度与结果以 NDJSON 写入 stdout；
- 数据契约见[数据命令](data-commands.md)。

### 2.9 `mcp`

```text
mdctrl mcp [--read-only]
```

- 以 stdio 方式运行只读 MCP 服务；
- 暴露文档、搜索、健康与反链查询；
- 写入能力默认关闭；
- stdout 只用于协议帧，日志写入 stderr。

### 2.10 `plugin`

```text
mdctrl plugin list [--json]
mdctrl plugin install <source>
mdctrl plugin remove <name>
mdctrl plugin doctor <name> [--json]
```

- MVP4 提供；
- 安装与移除要求确认；
- `doctor` 输出 manifest、能力、权限与最近错误；
- 插件命令与内置命令共享同一命令总线。

## 3 命令到协议映射

| 命令 | 协议类型 | 主要字段 |
| --- | --- | --- |
| `init` | command | workspace root、force |
| `open` | command + query | path、viewport、capabilities |
| `search` | query | query、limit、path filter |
| `find` | query | pattern、limit |
| `status` | query | workspace、index、server |
| `serve` | 无 | 由 server 自行启动 |
| `check` | query | paths、rules、severity |
| `index status` | query | index metadata |
| `index update` | command | paths、incremental |
| `index rebuild` | command | confirm |
| `mcp` | 无 | 由 MCP host 接管 |
| `plugin *` | command | action、name、source |

映射保持一一对应；CLI 不合并多个语义不同的请求，避免部分失败难以归因。

## 4 解析与校验

- 先解析全局选项，再解析命令选项；
- 路径参数统一解析为工作区相对路径，保留用户输入用于错误信息；
- 通配符显式展开，展开结果去重；
- 选项冲突在解析阶段报错，例如 `--json` 与 `open`、`--json` 与 `--json-stream`；
- `--json-stream` 用于不支持流式的命令时返回用法错误（退出码 2）；
- 破坏性操作要求确认：交互式询问，非 TTY 且无 `--yes` 时报用法错误；
- client 在工作区解析阶段做预校验：规范化路径、拒绝越界路径与逃逸符号链接；预校验只为快速失败，不替代 server 的最终校验；
- server 在执行时重新校验 canonical 路径、符号链接、权限与文件存在性；
- 实际读写时 server 再次确认目标仍位于工作区边界内；校验后符号链接被替换也不得逃逸工作区；
- 文档版本检查负责内容冲突（TOCTOU 的内容一致性），不替代路径边界校验；具体文件访问机制由 server 文档承接；
- 不存在路径仅在创建类操作中允许，读取、搜索与检查默认拒绝。

## 5 信号与生命周期

| 信号 | 行为 |
| --- | --- |
| SIGINT | 取消当前请求，输出中断提示，退出码 130 |
| SIGTERM | 优雅关闭，释放锁，退出码 130 |
| SIGPIPE | 停止输出，正常退出 |
| SIGHUP | daemon 忽略，TUI 恢复终端后退出 |

`open` 与 `serve` 接管信号；其余命令在请求期间响应取消。

## 6 帮助与文档

- `mdctrl help <command>` 输出该命令的完整选项；
- `--help` 输出概要并指向 `help`；
- 帮助文本包含命令阶段标记（MVP1/MVP2/MVP4）；
- 帮助文本与命令定义同源，测试断言关键段落存在；
- 文档中的示例必须可执行，纳入示例校验。

## 7 组合示例

```bash
# 初始化并查看状态
mdctrl init
mdctrl status --json

# 阅读前先确认索引
mdctrl index status
mdctrl index update
mdctrl open docs/architecture.md

# CI 健康检查，达到阈值返回 5
mdctrl check --json --fail-on warning

# 脚本消费搜索结果
mdctrl search "transport" --json --limit 20

# 后台服务与多客户端
mdctrl serve &
mdctrl open README.md
```

## 8 兼容性

- 命令名、选项名与退出码为稳定契约；
- 新增选项保持向后兼容，移除或改名需要弃用周期；
- 位置参数顺序变化视为不兼容；
- 组合规则变更在文档与测试中同步；
- 未文档化行为不承诺兼容。
