# mdctrl-protocol 文档

> 文档状态：草案 · 最后复核：2026-11-26

> document 定义文档身份、状态、摘要、列表、查看、大纲、链接与反链。

## 1 定位

document 是基础状态域，负责文档身份、元数据、生命周期、摘要、列表、查看请求、大纲、出链与反链。它向 CLI、TUI、plugin 与 MCP 提供只读服务原语；文档写入进入 write 域。

document 使用 [base.md](../base.md) 的标识、版本、时间、范围、错误与通用值，使用 [runtime.md](../runtime.md) 的控制原语与会话语义。View 的布局行、样式 token、媒体占位与诊断由 view 域定义；历史记录、链接图与写入分别由 history、index 与 write 域负责。

文档读取、解析、布局、元数据提取、链接解析、历史存储与索引实现的细节见 server 文档。

## 2 身份与版本

文档身份使用 base 的 `DocumentPath`：工作区相对、UTF-8、`/` 分隔。绝对路径由 workspace 解析，协议使用相对路径。

`DocumentVersion` 由 `DocumentFingerprint` 与 `DocumentRevision` 组成：

- `DocumentFingerprint` 表达内容身份，用于 `exact` 匹配与写入基准；
- `DocumentRevision` 是 server 维护的单调序号，用于排序与展示。

`document.list`、`document.metadata`、`document.outline` 与 `document.view` 返回 `DocumentVersion`。`document.links` 与 `document.backlinks` 返回 `IndexVersion`，标注链接图基于哪一版索引。

`document.view` 的请求可以携带可选 `version`：

- 省略 `version`：按 latest 返回当前版本；
- 携带 `version`：按 exact 匹配当前指纹，匹配失败返回 `document.version_mismatch`，错误详情携带当前 `DocumentVersion`。

`generation` 由 client 维护，放在请求元数据中，server 在响应中回传。

## 3 状态、摘要与元数据

文档状态：

```text
DocumentState {
  Present,
  Unreadable,
  Unknown,
  Deleted
}
```

- `Present`：文档可读取；
- `Unreadable`：权限、编码或大小限制导致读取失败；
- `Unknown`：扫描覆盖存在缺口，当前状态未知；
- `Deleted`：文档已确认缺失，通常出现在变更事件中。

`Unknown` 表示当前状态或历史 head 的覆盖缺口；历史节点记录 `Present`、`Unreadable` 与 `Deleted` 三种状态。

生命周期：

```text
DocumentLifecycle {
  state,        // Draft | Review | Confirmed | Archived | Unknown
  reviewed_at?,
  owner?
}
```

文档摘要：

```text
DocumentSummary {
  path,
  title?,
  state,
  version?,          // 最后一次可读版本
  size,
  modified_at,
  lifecycle,
  tags
}
```

文档元数据在摘要之上携带原始 frontmatter：

```text
DocumentMetadata {
  summary,
  frontmatter: Map<String, Value>
}
```

`title` 优先取 frontmatter，其次取首个标题。`tags` 与 `lifecycle` 从 frontmatter 解析；`frontmatter` 保留原始键值，供扩展字段与健康检查使用。`Unreadable` 与 `Unknown` 通过摘要与错误码共同表达；`Deleted` 文档通过 `document.not_found` 表达。

## 4 文档列表

`document.list` 用于文件树与目录浏览，返回分页摘要：

```text
DocumentListRequest {
  prefix?,
  page: PageRequest
}

DocumentListReply {
  documents: [DocumentSummary],
  page: PageInfo,
  scan_complete,
  scan_updated_at
}
```

- `prefix` 表示目录或路径前缀；省略时从工作区根开始。
- `page` 使用 base 的 `PageRequest` 与 `PageInfo`。
- `scan_complete` 与 `scan_updated_at` 来自 workspace 的扫描摘要；扫描覆盖存在缺口时，列表仍返回已知文档，client 据此提示结果覆盖有限。
- 列表包含 `Present`、`Unreadable` 与 `Unknown` 状态的文档；`Deleted` 文档通过变更事件与历史记录表达。

## 5 打开与查看

`document.view` 是打开与阅读的主原语：

```text
DocumentViewRequest {
  path,
  viewport,
  version?,
  theme_mode?
}
```

- `viewport` 使用 base 的 `Viewport`，包含布局行起点、高度与终端宽度；宽度以 Cell 为单位。
- `version` 省略时按 latest 返回；携带时按 exact 匹配。
- `theme_mode` 影响媒体资源与主题相关布局。

响应返回 view 域定义的 `View`，其中包含文档路径、`DocumentVersion`、布局行、样式、媒体占位与诊断。`document.view` 成功后记录最近观察，由 history 保存观察记录；`document.metadata`、`document.outline`、`document.links` 与 `document.backlinks` 保持只读。

`document.view` 的错误：

- 路径越界：`path.rejected`；
- 文档缺失：`document.not_found`；
- 文档读取失败：`document.unreadable`；
- 扫描状态未知：`document.state_unknown`；
- exact 版本匹配失败：`document.version_mismatch`，错误详情携带当前 `DocumentVersion`。

## 6 大纲、链接与反链

大纲：

```text
DocumentOutline {
  version,
  nodes: [OutlineNode]
}

OutlineNode {
  level,
  title,
  anchor,
  source_span
}
```

大纲按文档顺序排列，client 按 `level` 组装树。`anchor` 与链接解析使用同一套锚点规则。

出链：

```text
DocumentLinks {
  links: [DocumentLink],
  page: PageInfo,
  index_version
}

DocumentLink {
  target,
  kind,          // Document | Resource | External | Unresolved
  anchor?,
  source_span,
  resolved_path?
}
```

反链：

```text
DocumentBacklinks {
  backlinks: [Backlink],
  page: PageInfo,
  index_version
}

Backlink {
  source,
  source_span,
  anchor?
}
```

链接解析相对文档所在目录进行；`Unresolved` 表示目标缺失或锚点未找到，供 health 域生成断链结果。`IndexVersion` 标注链接图版本，client 可以据此判断结果是否伴随索引更新而失效。

## 7 事件与错误

文档变更事件：

```text
document.changed {
  path,
  version?,
  state,
  reason,        // Created | Modified | Deleted | Unreadable | Restored | Merged
  changed_at
}
```

事件 scope 为 `document(path)`，在 history 提交变更后发布。client 收到后重新读取 `document.list`、`document.metadata`、`document.view`、大纲或链接结果；View 与链接结果的刷新由各自版本字段约束。

错误：

| 错误码 | 场景 | retryable |
| --- | --- | --- |
| `document.not_found` | 文档缺失 | false |
| `document.unreadable` | 权限、编码或大小限制导致读取失败 | false |
| `document.state_unknown` | 扫描覆盖存在缺口，状态未知 | true |
| `document.version_mismatch` | exact 请求的版本未命中当前指纹 | false |
| `path.rejected` | 路径越界或指向内部状态 | false |
| `permission.denied` | principal 缺少能力 | false |
| `internal` | 未预期错误 | false |

`document.links` 与 `document.backlinks` 依赖 index；索引缺失或暂时失败时返回 index 域的 `index.unavailable`。

## 8 与其他域的协作

| 域 | 协作方式 |
| --- | --- |
| workspace | 提供文档清单、快照、扫描摘要与路径校验；document 生成摘要、列表与 View 请求 |
| history | 维护文档版本、观察记录与变更溯源；document 暴露版本与 `document.changed` |
| index | 提供链接图、反链与 `IndexVersion`；document 提供出链与反链结果 |
| view | 定义 `View` 载荷、布局行、样式、媒体占位与诊断 |
| write | 使用 `DocumentVersion` 与指纹作为写入基准；提案、差异、审批与写入由 write 域定义 |
| health | 读取文档元数据与链接结果，生成断链、过期与元数据检查 |
| search | 搜索命中通过 `document.view` 打开；文件名查找由 search 域负责 |
| media | 提供媒体资源、`media.read`、就绪/失败事件与降级；View 承载占位、实际尺寸与降级 |
| agent / plugin | 通过 document 原语读取文档、元数据、大纲与链接；写入进入 write 域 |

跨域流程见 [domain.md](../domain.md)。
