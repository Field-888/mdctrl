# mdctrl-protocol 健康检查

> 文档状态：草案 · 最后复核：2026-11-26

> health 读取链接图、历史、Git 快照与文档元数据，生成结构化健康报告。

## 1 定位

health 是阅读与检索的派生域，负责断链、孤儿、过期与缺失元数据检查，并按路径范围、规则集合与严重度生成报告。健康结果按需重新计算，供 TUI 健康面板、CLI `check` 与 agent 使用。

| 原语 | access | execution | 作用 | policy |
| --- | --- | --- | --- | --- |
| `health.check` | `Read` | `Unary` | 执行健康检查并返回报告 | `workspace.read` |

health 使用 base 的 `DocumentPath`、`IndexVersion`、`SourceSpan`、`PageRequest` 与 `PageInfo`。健康报告中的 `IndexCoverage` 由 index 域提供。

## 2 规则模型

| rule | 默认严重度 | 判定 |
| --- | --- | --- |
| `broken-link` | `error` | 内部链接指向缺失文档、缺失锚点，或越出工作区 / 指向内部状态 |
| `orphan` | `info` | 入链来源为空，且入口文档列表未覆盖该路径 |
| `stale` | `warning` | 最近复核、最近变更或最近提交超过配置周期 |
| `missing-metadata` | `warning` | 缺少配置要求的 frontmatter 字段 |

- 规则 id 与默认严重度稳定；规则参数（过期周期、必需字段、入口文档等）来自 `[health]` 配置。
- `broken-link` 检查正文链接；代码块与行内代码按文本处理。外部 URL 由外部工具负责。
- failed region 内的链接目标按未知状态处理，等待下次扫描确认。
- `orphan` 的入链统计只计其他文档的链接。
- `stale` 检查覆盖未归档文档；时间来源按 frontmatter 复核时间、history 最近变更、Git 最近提交的优先级选择。
- `missing-metadata` 按 glob 分组要求字段；字段存在但为空视为缺失。

## 3 请求与报告

```text
HealthCheckRequest {
  paths?,
  rules?,
  severity?,
  page: PageRequest
}

HealthReport {
  scope: {
    paths?,
    rules?,
    severity?
  },
  summary: HealthSummary,
  findings: [HealthFinding],
  page: PageInfo,
  index_version,
  index_coverage?,
  git_available,
  checked_at
}

HealthSummary {
  documents,
  findings,
  truncated,
  by_severity: { error, warning, info },
  by_rule
}
```

- `paths` 限制检查范围；省略时覆盖整个工作区。
- `rules` 限制规则集合；未知规则返回 `health.rule_not_found`。
- `severity` 取 `error`、`warning`、`info`，过滤 `findings` 的展示；`summary` 始终基于所选路径与规则集合的完整结果。
- `page` 控制 `findings` 分页；`summary.truncated` 标记结果截断。
- `index_version` 标注链接图与索引数据基于哪一版索引。
- `index_coverage` 在索引覆盖存在缺口时携带失败区域。
- `git_available` 表示 `stale` 是否使用 Git 提交时间。
- `checked_at` 是本次检查时间。

## 4 结果模型

```text
HealthFinding {
  id,
  rule,
  severity,          // error | warning | info
  path,
  source_span?,
  message,
  target?,
  related?: [DocumentPath]
}
```

- `id` 在相同输入下稳定，用于选中、忽略与前后结果比较。
- `path` 是发现所在文档；`source_span` 指向具体位置。
- `target` 用于链接类规则；`related` 列出相关文档。
- `message` 面向人类；稳定判断依赖 `rule` 与 `severity`。
- 结果按严重度降序、路径升序、行号升序、`id` 升序排列。

## 5 排序、过滤与分页

- `findings` 按严重度降序、路径升序、行号升序、`id` 升序排列。
- `severity` 过滤只影响 `findings`；`summary` 保留完整计数。
- `page` 使用 `PageRequest` 与 `PageInfo`；结果数量受 `MAX_RESULTS` 约束。
- 结果截断时 `summary.truncated` 标记，`summary` 计数保持完整。
- TUI 与 CLI 共享同一报告；CLI 将报告映射为 `--fail-on` 与退出码，退出码语义由 CLI 定义。

## 6 版本、覆盖与失效

- `HealthReport.index_version` 与 `index_coverage` 表达数据来源与覆盖质量。
- `git_available = false` 时，`stale` 使用 frontmatter 与 history 时间。
- 索引缺失或损坏时返回 `index.unavailable`。
- 健康结果作为派生状态按需重新计算。

`health.updated` 在索引更新或相关文档变化后发布，scope 为 `workspace`：

```text
health.updated {
  index_version,
  paths?,
  updated_at
}
```

client 收到后重新调用 `health.check`。

## 7 错误与协作

错误：

| 错误码 | 场景 | retryable |
| --- | --- | --- |
| `index.unavailable` | 索引缺失或损坏 | false |
| `health.rule_not_found` | 请求包含未知规则 | false |
| `path.rejected` | 路径越界或指向内部状态 | false |
| `permission.denied` | principal 缺少能力 | false |
| `protocol.limit_exceeded` | 查询或结果超出限制 | false |
| `internal` | 未预期错误 | false |

协作：

| 域 | 协作方式 |
| --- | --- |
| index | 提供链接图、锚点、`IndexVersion` 与覆盖信息 |
| history | 提供最近变更、最近观察与 Git 快照，用于 `stale` |
| document | 提供元数据、frontmatter 与生命周期 |
| workspace | 提供文档清单、扫描覆盖与路径校验 |
| agent | 读取健康报告，形成审阅上下文 |
| TUI / CLI | TUI 展示面板；CLI 将报告映射为 `--fail-on` 与退出码 |

跨域流程见 [domain.md](../domain.md)。
