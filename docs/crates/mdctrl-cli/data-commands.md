# 数据命令

> 文档状态：草案 · 最后复核：2026-11-24

> `check` 与 `index` 是 CI 的主要接口：规则与索引由 server 拥有，CLI 只定义调用与输出契约。

**相关文档**：[mdctrl-cli](README.md) · [输出](output.md) · [命令组合](commands.md) · [mdctrl-tui 搜索、健康与跳转](../mdctrl-tui/search-health.md)

## 1 职责归属

| 能力 | 所有者 | 说明 |
| --- | --- | --- |
| 健康规则与判定 | server 的 health service | `ylgd0915` 提供规则与结果模型 |
| 索引构建、分词与排序 | server 的 index service | `ylgd0915` 提供索引语义 |
| 命令解析与参数校验 | `mdctrl-cli` | 本文定义 |
| 结果序列化与退出码 | `mdctrl-cli` | 本文定义 |
| 结果展示 | CLI 与 TUI | 共享字段语义 |

CLI 不复制规则，也不在客户端重算严重度。字段名与枚举由协议定义，本文给出当前约定。

## 2 `check`

### 2.1 用法

```text
mdctrl check [paths...] [选项]

选项
  --rule <id>          限制规则，可重复
  --severity <level>   最低展示级别：error | warning | info
  --fail-on <level>    达到该级别时退出码为 5：error | warning | info | never
  --allow-partial      允许不完整检查按成功结束；默认不完整返回 4
  --json               结构化输出
  --no-color           禁用颜色
  --quiet              人类可读模式仅输出摘要；结构化模式保留完整字段
```

默认：

- 范围为整个工作区；
- 展示所有级别；
- `--severity` 只影响 `findings` 展示，`summary` 与 `--fail-on` 使用完整结果；
- `--fail-on warning`；
- 检查覆盖不完整时默认返回 4，保留已检查部分的结果；`--allow-partial` 允许按成功结束，并在 `coverage.complete=false` 中标记；
- 人类可读输出，非 TTY 时无色。

### 2.2 规则集合

| 规则 id | 严重度 | 含义 |
| --- | --- | --- |
| `broken-link` | error | 指向不存在文档或锚点的链接 |
| `missing-metadata` | warning | 缺少约定的 frontmatter 字段 |
| `orphan` | info | 没有入链且不在索引入口的文档 |
| `stale` | warning | 超过配置的观察或复核周期 |

规则 id 稳定，新增规则不改变既有 id。规则参数（例如过期天数）来自 `mdctrl.toml`，不在命令行重复定义。规则 id 与结果字段是候选契约，需 `ylgd0915` 确认后固化；当前 `mdctrl-protocol` 尚未实现。

### 2.3 结果模型

```json
{
  "schema_version": 1,
  "command": "check",
  "ok": true,
  "data": {
    "scope": { "paths": ["docs"], "rules": null },
    "coverage": {
      "complete": true,
      "checked_documents": 42,
      "unchecked": []
    },
    "filtered": false,
    "summary": {
      "documents": 42,
      "findings": 3,
      "truncated": true,
      "by_severity": { "error": 1, "warning": 2, "info": 0 },
      "by_rule": { "broken-link": 1, "missing-metadata": 2 }
    },
    "findings": [
      {
        "rule": "broken-link",
        "severity": "error",
        "path": "docs/architecture.md",
        "line": 128,
        "column": 5,
        "message": "link target not found: docs/missing.md",
        "target": "docs/missing.md",
        "related": []
      }
    ]
  },
  "error": null,
  "warnings": []
}
```

字段规则：

- `ok` 表示检查成功执行，不表示是否存在问题；执行成功且覆盖完整时 `ok=true`、`error=null`；达到 `--fail-on` 阈值返回 5，否则返回 0；
- 检查覆盖不完整时默认 `ok=false`、`error.kind=check_incomplete`、退出码 4，`data` 仍保留已检查部分的结果；使用 `--allow-partial` 时 `ok=true`，退出码按发现问题的严重度取 0 或 5，`coverage.complete=false`；
- `coverage.complete` 表示本次检查是否覆盖了请求范围；目录无法枚举、文档无法读取或规则无法判定时为 `false`；
- `coverage.unchecked` 列出未检查的路径、规则与原因，例如 `{ "path": "docs/private", "rule": null, "reason": "permission_denied" }`；未知目录中的文档数保持未知，不计入 `summary.documents`；
- `findings` 按严重度降序、路径与行号升序排列；
- `findings` 受 `--severity` 过滤，`filtered` 标记是否发生过滤；
- `path` 为工作区相对路径；
- `line`、`column` 从 1 开始，未知时为 `null`；
- `target` 仅链接类规则使用；
- `related` 保存相关文档或锚点，供跳转与详情使用；
- `summary` 只统计已检查文档，始终包含未过滤的完整计数；展示过滤不会改变计数；`summary.truncated` 只表示问题列表被截断，不表示检查覆盖不完整。

### 2.4 退出码

| 条件 | 退出码 |
| --- | --- |
| 无达到阈值的问题 | 0 |
| 达到 `--fail-on` 阈值 | 5 |
| 执行失败 | 1 |
| 工作区或配置错误 | 3 |
| 检查覆盖不完整（默认） | 4 |
| 索引或数据不可用 | 4 |

`--fail-on never` 只关闭发现问题的严重度阈值；检查覆盖不完整时仍默认返回 4，除非使用 `--allow-partial`。

### 2.5 人类可读输出

```text
error   docs/architecture.md:128  broken-link
        link target not found: docs/missing.md
warning docs/devPlan.md:42        stale
        last reviewed 120 days ago

3 findings (1 error, 2 warnings)
```

规则：

- 每个问题两行：位置摘要与消息；
- 末尾输出汇总，并说明已检查文档数和未检查范围；
- `--quiet` 在人类可读模式下只输出汇总；结构化模式保留完整字段；
- 只有完整检查且无发现时输出“未发现问题”；
- 过滤后无结果时输出“当前筛选无结果”，同时保留完整汇总；
- 覆盖不完整时输出“检查未完成”与未检查范围，不能显示“未发现问题”。

## 3 `index`

### 3.1 子命令

```text
mdctrl index status [--json]
mdctrl index update [paths...] [--json | --json-stream]
mdctrl index rebuild [--json | --json-stream] [--yes]
```

| 子命令 | 行为 | 是否修改索引 |
| --- | --- | --- |
| `status` | 报告索引元数据 | 否 |
| `update` | 增量更新，未变化文档跳过 | 是 |
| `rebuild` | 保留旧索引，临时构建后原子替换 | 是 |

### 3.2 状态模型

```json
{
  "schema_version": 1,
  "command": "index.status",
  "ok": true,
  "data": {
    "state": "ready",
    "task": "none",
    "documents": 42,
    "index_version": 3,
    "updated_at": "2026-11-24T08:30:00Z",
    "last_error": null
  },
  "error": null,
  "warnings": []
}
```

`state` 表示当前可读索引的状态，`task` 表示进行中的任务：

| 字段 | 取值 | 含义 |
| --- | --- | --- |
| `state` | `missing` | 没有可用索引 |
| `state` | `ready` | 有可用索引 |
| `state` | `stale` | 可用索引落后于工作区 |
| `state` | `error` | 没有可用索引，且最近一次构建失败 |
| `task` | `none` | 没有进行中的索引任务 |
| `task` | `building` | 首次构建 |
| `task` | `rebuilding` | 保留旧索引的全量重建 |
| `task` | `updating` | 增量更新 |

状态组合示例：

| 场景 | `state` | `task` | 查询行为 |
| --- | --- | --- | --- |
| 首次构建 | `missing` | `building` | 返回 `index_unavailable`；文件名查找仍可用 |
| 带旧库重建 | `ready` | `rebuilding` | 查询读取旧版本并标记索引滞后 |
| 重建失败且旧库可用 | `ready` | `none` | 继续使用旧索引，`last_error` 记录原因 |
| 重建失败且无旧库 | `error` | `none` | 全文搜索不可用；文件名查找仍可用 |

### 3.3 更新与重建

- `update` 只处理自上次索引后变化的文档；
- `rebuild` 保留现有索引，在独立临时位置全量构建并校验，成功后原子切换；
- 构建失败时清理临时产物，继续使用旧索引，`last_error` 记录原因；
- 旧索引在切换成功前始终可读，构建期间读取方不会看到半成品；
- 并发更新由 server 内部索引任务状态串行化：默认非阻塞，第二个请求在任务进行中立即返回“已有构建”（退出码 4，`kind=index_busy`），不排队等待；
- `.mdctrl/lock` 只保证每个工作区一个 server，daemon 运行期间持续持有，不用于索引任务互斥；embedded 与 daemon 均由 server 内部任务状态判断冲突。

### 3.4 进度与流式输出

默认：人类可读模式在 stdout 输出可读结果，`--json` 在 stdout 输出单个信封；进度写入 stderr，可被 `--quiet` 抑制。

`--json-stream` 时 stdout 使用 NDJSON：

```json
{"schema_version":1,"command":"index.update","type":"progress","phase":"scan","done":12,"total":42,"data":null,"error":null}
{"schema_version":1,"command":"index.update","type":"progress","phase":"index","done":30,"total":42,"data":null,"error":null}
{"schema_version":1,"command":"index.update","type":"summary","data":{"updated":30,"skipped":12,"errors":0},"error":null}
```

规则：

- `--json-stream` 仅适用于 `index update` 与 `index rebuild`；
- 每行包含 `schema_version`、`command`、`type`、`data`、`error`；
- 最后一行必须是 `summary` 或 `error`；
- 未使用 `--json-stream` 时不输出 NDJSON，`--json` 始终是单个信封；
- 进度不改变退出码。

### 3.5 退出码

| 条件 | 退出码 |
| --- | --- |
| 成功 | 0 |
| 构建或写入失败 | 1 |
| 参数错误 | 2 |
| 工作区错误 | 3 |
| 索引不可用或已有构建 | 4 |

## 4 机器契约

- `schema_version` 与命令名固定；
- 字段集合与类型由协议类型决定；
- 新增字段不改变既有字段语义；
- 枚举值新增视为兼容，移除视为不兼容；
- 排序规则属于契约，脚本可以依赖；
- 时间统一 RFC 3339 UTC；
- 计数与行号使用整数；
- 文本消息不保证稳定，脚本应使用 `rule`、`severity` 与 `kind`。

## 5 CI 用法

```bash
# 建立或更新索引
mdctrl index update --json

# 健康检查，warning 及以上阻断
mdctrl check --json --fail-on warning

# 只生成报告，不阻断
mdctrl check --json --fail-on never > report.json

# 检查指定目录
mdctrl check docs/crates --severity error
```

CI 建议：

- 先确认 `index status`，避免索引缺失导致误判；
- 将 JSON 结果归档为构建产物；
- 依据退出码分支，不解析人类可读文本；
- 大型仓库使用路径范围限制检查时间。

## 6 性能与边界

- `check` 按文档并行读取，结果在内存中聚合后排序；
- 超大结果集只输出前 N 条，`summary` 保留完整计数，并标记 `truncated`；
- 索引构建使用流式写入，避免一次性持有全部文档；
- 路径范围限制在扩展名与忽略规则内；
- 外部渲染与网络不在数据命令中发生。

## 7 与 TUI 的一致性

- TUI 健康面板与 `check` 使用同一规则与严重度；
- TUI 搜索结果与 `search` 使用同一索引与排序；
- CLI 字段是 TUI 展示字段的子集，避免两套语义；
- 两边对空结果、错误分类与截断策略保持相同约定。

## 8 测试关注点

- 四条规则的判定与排序；
- `--fail-on` 各档位的退出码；
- `--json` 信封与 `schema_version`；
- 索引状态的五个取值与迁移；
- 构建失败保留旧索引；
- NDJSON 流的最后一行；
- 中文查询与索引更新。
