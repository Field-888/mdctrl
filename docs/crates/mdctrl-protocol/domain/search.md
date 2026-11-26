# mdctrl-protocol 搜索

> 文档状态：草案 · 最后复核：2026-11-25

> search 定义全文检索与文件名/路径查找，消费索引数据并标注 `IndexVersion`。

## 1 定位

search 是阅读与检索的查询域，负责全文检索、文件名与路径查找、结果排序、分页与截断。它使用 index 提供的索引数据与 `IndexVersion`；索引构建、存储与更新由 index 域负责。

| 服务原语 | access | execution | 作用 | policy |
| --- | --- | --- | --- | --- |
| `search.query` | `Read` | `Unary` | 全文检索 | `workspace.read` |
| `search.find` | `Read` | `Unary` | 文件名与路径查找 | `workspace.read` |

search 使用 base 的 `DocumentPath`、`IndexVersion`、`PageRequest`、`PageInfo` 与 `SourceSpan`。命中打开由 document 域的 `document.view` 完成。

## 2 全文检索

```text
SearchQueryRequest {
  query,
  path?,
  page: PageRequest
}

SearchQueryReply {
  hits: [SearchHit],
  page: PageInfo,
  index_version
}
```

- `query` 是纯文本，client 将其作为文本传递，词边界与查询表达式由 server 处理。
- `path?` 将查询限制在工作区内的某个目录或路径前缀。
- `page` 使用 base 的 `PageRequest` 与 `PageInfo`。
- `index_version` 标注结果基于哪一版索引。

## 3 路径查找

```text
SearchFindRequest {
  pattern,
  path?,
  page: PageRequest
}

SearchFindReply {
  hits: [FindHit],
  page: PageInfo,
  index_version
}
```

- `pattern` 是文件名或路径片段。
- `path?` 将查找限制在工作区内的某个目录或路径前缀。
- 结果按 server 定义的顺序返回，client 按返回顺序展示。
- 文件树与目录浏览使用 `document.list`；`search.find` 服务于快速路径查找。

## 4 结果模型

```text
SearchHit {
  path,
  title?,
  score,
  snippet: SearchSnippet
}

SearchSnippet {
  text,
  source_span,
  highlights: [SourceSpan]
}

FindHit {
  path,
  title?,
  score?
}
```

- `path` 是工作区相对文档路径。
- `title` 来自文档元数据或首个标题。
- `score` 是 server 排序得分；client 按返回顺序展示。
- `snippet.source_span` 指向片段起点，`highlights` 标出命中范围。
- 打开命中时使用 `document.view`，并按 `source_span` 定位。
- `FindHit` 只提供路径、标题与可选得分。

## 5 排序、分页与限制

- 排序由 server 定义，保证稳定顺序；同分结果按路径升序。
- `PageInfo.truncated` 标记结果截断；`total` 在计数成本高时省略。
- 查询长度、结果数量与分页参数受 base 限制；超出时返回 `protocol.limit_exceeded`。
- 结果数量超过上限时截断，截断状态由 `PageInfo` 表达。
- client 按返回顺序展示，排序规则属于查询契约。

## 6 版本、失效与错误

- `SearchQueryReply` 与 `SearchFindReply` 携带 `IndexVersion`。
- `index.updated` 到达后，client 重新查询；旧结果可按 `IndexVersion` 判断过期。
- 索引重建期间，旧索引继续服务查询；结果携带旧索引的 `IndexVersion`。
- 索引缺失或损坏时返回 `index.unavailable`。
- 路径越界或指向内部状态时返回 `path.rejected`。

| 错误码 | 场景 | retryable |
| --- | --- | --- |
| `index.unavailable` | 索引缺失或损坏 | false |
| `path.rejected` | 路径越界或指向内部状态 | false |
| `permission.denied` | principal 缺少能力 | false |
| `protocol.limit_exceeded` | 查询或结果超出限制 | false |
| `internal` | 未预期错误 | false |

## 7 与其他域协作

| 域 | 协作方式 |
| --- | --- |
| index | 提供全文、文件名与路径数据，以及 `IndexVersion` |
| document | 命中通过 `document.view` 打开；`source_span` 用于定位 |
| workspace | 提供工作区范围与路径校验 |
| agent | 通过 `search.query` 与 `search.find` 获取候选文档 |
| plugin / MCP | 通过只读服务原语使用搜索结果 |

跨域流程见 [domain.md](../domain.md)。
