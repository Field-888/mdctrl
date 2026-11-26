# mdctrl-protocol 视图

> 文档状态：草案 · 最后复核：2026-11-26

> view 定义 document.view 的 View 载荷、行模型、样式 token、媒体占位与诊断。

## 1 定位

view 的入口是 document 域的 `document.view`；view 域定义 View 载荷与失效语义。server 生成布局行与显示宽度，client 按 View 行栅格化为终端 Cell。

View 使用 [base.md](../base.md) 的 `DocumentPath`、`DocumentVersion`、`Viewport`、`SourceSpan`、`ResourceId` 与 `PageInfo`。文档查看请求、exact/latest 与观察语义由 [document.md](document.md) 定义；样式 token 到终端颜色的映射由 client 主题负责。

`Viewport` 使用 base 的 `{ start_line, height, width }`；`start_line` 为 0-based 布局行，`width` 以终端 Cell 为单位。`document.view` 请求以 `viewport` 表达视口。

## 2 View 结构

```text
View {
  document,          // DocumentPath
  version,           // DocumentVersion
  viewport,          // Viewport
  total_lines,
  lines: [ViewLine],
  media: [MediaDescriptor],
  diagnostics: [ViewDiagnostic]
}
```

- `total_lines` 是当前文档版本的布局行总数，供滚动条与滚动范围计算。
- `lines` 只包含当前视口覆盖的布局行。
- `media` 收集视口内媒体资源的描述符；`ViewLine.media_id` 引用对应条目。
- `diagnostics` 是视口内的行内诊断；workspace 级 findings 由 health 域提供。

## 3 行与片段

```text
ViewLine {
  block_id,          // BlockId，文档版本内稳定
  kind,              // Text | Code | TableRow | Rule | Media
  display_width,
  source_span?,
  media_id?,
  spans: [Span]
}

Span {
  text,
  style_token,       // StyleToken
  display_width,
  source_span?,
  anchor?,
  link?: ViewLink
}
```

`BlockId` 是文档版本内稳定的不透明字符串。`block_id` 把同一块折行产生的多行归组，支持选择、复制与后续编辑定位。
- `source_span` 把布局行与片段映射回源码位置。
- `display_width` 由 server 按 Unicode 显示宽度计算，client 以它为准布置 Cell。
- `LineKind` 表达布局差异；标题、引用、列表等语义由 `style_token` 表达。
- `Span.anchor` 用于标题跳转；正文链接使用 `ViewLink`。

```text
ViewLink {
  target,
  kind,              // Document | Resource | External | Unresolved
  anchor?,
  resolved_path?
}
```

`ViewLink` 是 document 域链接结果的轻量投影；完整链接列表与反链通过 `document.links` 与 `document.backlinks` 查询。

## 4 样式 token 与主题

View 使用语义 token：

| token | 用途 |
| --- | --- |
| `text` | 正文 |
| `muted` | 次要信息 |
| `accent` | 强调与可交互元素 |
| `link` | 链接 |
| `code` | 行内代码与代码块 |
| `heading1`–`heading6` | 标题层级 |
| `quote` | 引用 |
| `table_header` | 表头 |
| `marker` | 列表标记 |
| `selection` | 选中区域 |
| `border` / `border_focus` | 窗格边框与焦点 |
| `success` / `warning` / `danger` | 状态 |
| `diagnostic_error` / `diagnostic_warning` / `diagnostic_info` | 诊断标记 |

- client 把 token 映射到终端颜色与修饰。
- 未知 token 回退到 `text`，并记录诊断。
- 主题选择与终端配色由 client 管理；server 提供语义 token。
- `theme_mode` 影响媒体资源的明暗渲染；普通文本布局由 server 维持。

## 5 媒体占位

```text
MediaDescriptor {
  id,                // ResourceId
  kind,              // Formula | Mermaid | Image
  state,             // Pending | Ready | Failed
  alt?,
  reserved: MediaExtent,
  actual?: MediaExtent,
  fallback?: MediaFallback
}

MediaExtent {
  columns,
  rows
}

MediaFallback {
  Text(String) | Source(String)
}
```

- `MediaDescriptor.id` 是 media 域的 `ResourceId`，标识媒体产物；源内容、主题或目标尺寸变化时产生新的 ID。
- 资源就绪前使用 `reserved` 占位；就绪后 `actual` 用于缩放或替换，媒体区域保持原布局。
- 默认使用固定占位，媒体就绪时在预留区域替换。
- 资源失败时显示 `fallback` 的文本或源码。
- `media.ready` 事件到达后，client 按 `resource_id` 匹配当前 View；命中时替换对应区域，未命中时重新请求 View。`media.failed` 到达后使用 `fallback`；资源 ID 未知时重新请求 View。
- `media.ready` 与 `media.failed` 携带 `document_version`；client 丢弃与当前 View 不一致的事件。
- 需要重新布局时由 server 发送新的 View。
- 大资源通过 `ResourceId` 与 `media.read` 获取，View 载荷保持轻量。

## 6 诊断

```text
ViewDiagnostic {
  severity,          // error | warning | info
  code,
  message,
  source_span?,
  related?: [SourceSpan]
}
```

- `code` 是稳定标识；来自健康规则的诊断使用对应 rule id。`message` 面向人类。
- 诊断可以来自链接解析、媒体失败、编码问题或健康规则。
- 行内诊断通过 `source_span` 定位；client 在对应行显示标记。
- workspace 级 findings 由 `health.check` 提供，View 承载当前视口的行内诊断。

## 7 失效与版本

View 的有效性由三个条件决定：

- `DocumentVersion` 与当前文档版本一致；
- `viewport` 与当前请求一致；
- `generation` 与请求代次一致。

失效来源：

- `document.changed`：文档内容或状态变化，重新请求 View；
- `media.ready` / `media.failed`：按 `resource_id` 更新媒体区域，必要时重新请求；
- `index.updated` / `health.updated`：行内诊断或链接结果变化，重新请求；
- 终端宽度或主题模式变化：client 重新请求；
- client 丢弃 `generation` 或 `viewport` 与请求不同的过期响应。

View 失效由上游事件触发，client 按需重新查询。

## 8 与其他域协作

| 域 | 协作方式 |
| --- | --- |
| document | `document.view` 是入口；提供文档版本、链接与观察语义 |
| render | 生成布局行、断行、缩进与显示宽度 |
| media | 提供媒体资源、`media.read`、就绪/失败事件与降级内容 |
| plugin | 插件命令返回 `PluginView`；client 在插件面板显示，文档 View 保持文档专属 |
| index | 提供链接解析结果与行内诊断输入 |
| health | 提供 workspace findings；View 承载当前视口的行内诊断 |
| client | 消费 View、映射样式 token、栅格化 Cell、缓存视口 |

跨域流程见 [domain.md](../domain.md)。
