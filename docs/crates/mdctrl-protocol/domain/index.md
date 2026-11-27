# mdctrl-protocol 索引

> 文档状态：草案 · 最后复核：2026-11-25

> index 为搜索、链接与健康检查提供派生索引、链接图、锚点与索引状态。

## 1 定位

index 是阅读与检索的基础派生域，为 search、document.links/backlinks 与 health 提供全文索引、链接图、锚点与索引状态。工作区文档是事实来源；索引数据可以随时重建。

search 域定义 `search.query` 与 `search.find`；index 提供它们消费的数据与 `IndexVersion`。document 域定义 `document.links` / `document.backlinks` 的结果形状；index 提供链接图与锚点数据。索引使用 base 的 `IndexVersion`、`DocumentPath` 与 `PageInfo`。

## 2 服务原语

| 原语 | access | execution | 作用 | policy |
| --- | --- | --- | --- | --- |
| `index.status` | `Read` | `Unary` | 返回索引状态、任务、版本与覆盖 | `workspace.read` |
| `index.update` | `Write` | `Job` | 增量更新索引 | User |
| `index.rebuild` | `Write` | `Job` | 全量重建索引 | User |

```text
IndexStatusRequest {}

IndexUpdateRequest {
  paths?
}

IndexRebuildRequest {}
```

`index.status` 返回 `IndexStatus`。`index.update` 与 `index.rebuild` 返回 `Accepted { job_id }`，进度通过 `runtime.job.*` 事件与 `index.status` 观察。索引任务进行中再次请求返回 `index.busy`。`paths` 是可选的工作区相对文档路径列表；省略时覆盖整个工作区。

## 3 状态、任务与版本

```text
IndexState {
  Missing,
  Ready,
  Stale,
  Error
}

IndexTaskKind {
  Building,
  Updating,
  Rebuilding
}

IndexStatus {
  state: IndexState,
  task?,             // IndexTask
  index_version,
  documents,
  updated_at,
  coverage: IndexCoverage,
  last_error?
}

IndexTask {
  kind: IndexTaskKind,
  job_id,
  progress: { done, total?, phase? }
}

IndexCoverage {
  complete,
  failed_regions: [{ path, reason }],
  updated_at
}
```

- `index_version` 是索引生命周期内的单调版本，随提交批次或重建递增；索引缺失时为 0。
- `state = Stale` 表示工作区变化尚未进入索引；覆盖存在缺口由 `coverage.complete` 表达。
- `state = Error` 表示索引损坏或最近一次构建失败，原因记录在 `last_error`。
- 旧索引可读时，`state` 保持 `Ready` 或 `Stale`，构建、更新与重建通过 `task` 并行展示。
- 索引缺失时 `state = Missing`，`task = Building`。
- `IndexVersion` 只标注索引派生结果；文档版本与 client `generation` 由 document 与 runtime 负责。

## 4 索引内容与增量更新

索引覆盖：

- 路径、标题、各级标题、正文与元数据文本；
- 文件名与路径查找所需的数据；
- 出链、解析结果、锚点与未解析链接；
- 命中片段到源码位置的映射。

增量更新：

- 指纹与已索引指纹相同的文档跳过；
- 变化的文档重新读取、解析并更新全文与链接图；
- 删除确认后移除索引项；
- failed region 内的已知文档保留旧索引并标记过期；
- 队列非空时状态为 `Stale`；
- 更新期间旧索引继续服务查询，结果携带当前 `IndexVersion`。

`index.update` 在索引缺失时执行全量构建；已有索引时执行增量更新。

## 5 重建与切换

`index.rebuild` 使用当前 schema 全量构建：

- 构建期间旧索引继续服务查询；
- 覆盖存在缺口时，failed regions 的文档从旧索引复制并标记过期；
- 成功后切换到新索引，递增 `IndexVersion`，发布 `index.updated`；
- 失败时继续使用旧索引，记录 `last_error`，发布 `index.failed`；
- 已有任务时返回 `index.busy`。

## 6 事件与错误

事件：

```text
index.updated {
  index_version,
  state,
  coverage,
  updated_at
}

index.failed {
  job_id?,
  error
}
```

- `index.updated` 在批次提交或重建完成后发布；
- `index.failed` 在构建或更新失败后发布；
- 两个事件的 scope 为 `workspace`；
- 任务进度通过 `runtime.job.*` 事件提供。

错误：

| 错误码 | 场景 | retryable |
| --- | --- | --- |
| `index.unavailable` | 索引缺失或损坏 | false |
| `index.busy` | 已有构建或更新任务 | true |
| `index.failed` | 构建或更新失败 | false |
| `path.rejected` | 路径越界或指向内部状态 | false |
| `permission.denied` | principal 缺少能力 | false |
| `internal` | 未预期错误 | false |

## 7 与其他域协作

| 域 | 协作方式 |
| --- | --- |
| workspace | 提供扫描结果与变化流；index 消费并更新 |
| document | 提供链接与反链查询；index 提供链接图与 `IndexVersion` |
| search | 使用全文、文件名与链接数据；结果携带 `IndexVersion` |
| health | 读取链接图与索引覆盖，生成断链与健康结果 |
| write | 文档写入通过变化流触发增量更新 |
| agent | 通过 search 与 document 使用索引结果 |

跨域流程见 [domain.md](../domain.md)。
