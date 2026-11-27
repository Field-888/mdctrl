# mdctrl-protocol 媒体

> 文档状态：草案 · 最后复核：2026-11-27

> media 定义公式、Mermaid 与图片资源的读取、状态、事件与降级。

## 1 定位

media 是写入与扩展的派生域，负责公式、Mermaid 与图片资源的后台生成、缓存、读取、就绪通知与降级。View 只携带媒体占位与资源描述；资源就绪后 client 通过 `media.read` 获取内容。

media 的职责：

- 接收 document 与 view 解析出的媒体节点；
- 调度后台生成，管理外部渲染器；
- 按内容与渲染参数缓存资源；
- 维护资源状态、依赖关系与失效；
- 提供分块资源读取；
- 在终端能力不足或生成失败时提供降级内容。

Markdown 媒体节点的识别与占位尺寸由 render 与 view 定义；终端图片协议编码与 Cell 栅格化由 client 负责。media 负责资源生成、缓存、读取与降级。

media 使用 [base.md](../base.md) 的 `ResourceId`、`DocumentVersion`、`ByteSize`、`Timestamp` 与错误模型。`MediaDescriptor`、`MediaExtent` 与 `MediaFallback` 由 view 域定义；本地图片路径由 workspace 校验；文档写入进入 write 域。

媒体生成器、缓存存储、任务队列与外部进程管理的实现见 server 文档。

## 2 服务原语

| 原语 | access | execution | 作用 | policy |
| --- | --- | --- | --- | --- |
| `media.read` | `Read` | `Unary` | 分块读取已就绪的媒体资源 | `workspace.read` |

媒体生成由 View 装配触发：View 中出现 `Pending` 描述符时，server 调度后台生成；缓存命中时直接返回 `Ready`。`media.read` 只读取已就绪资源。

`media.read` 使用分块读取，避免大资源进入单条消息：

```text
MediaReadRequest {
  resource_id,
  offset?,
  max_bytes?
}

MediaReadReply {
  resource_id,
  mime,
  size,
  offset,
  data,       // Binary
  eof
}
```

- `offset` 省略时为 0。
- `max_bytes` 省略时由 server 选择块大小；显式提供时受 server 与 `MAX_MESSAGE_BYTES` 约束，server 可以返回更小的块。
- `size` 是资源总字节数。
- `data` 使用 base 的 `Binary`；wire 使用 base64 字符串，embedded typed channel 使用原始字节。
- `eof = true` 表示已读到末尾。
- `offset` 越界或 `max_bytes` 非法返回 `media.invalid_range`。

`media.read` 只读取 `Ready` 资源。`Pending` 返回 `media.not_ready`；`Failed` 返回 `media.failed`；资源不存在或已被淘汰返回 `media.not_found`。

## 3 资源模型

### 3.1 类型与状态

`MediaDescriptor` 由 view 域定义：

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
```

媒体类型：

| kind | 来源 | 默认降级 |
| --- | --- | --- |
| `Formula` | 行内或行间数学公式 | 文本近似或公式源码 |
| `Mermaid` | Mermaid 代码块 | 源码 |
| `Image` | 工作区本地图片 | `alt` 与路径 |

状态：

```text
MediaState {
  Pending,
  Ready,
  Failed
}
```

- `Pending`：资源已识别，尚未生成。
- `Ready`：资源可读取，View 可携带 `actual`。
- `Failed`：资源生成失败，client 使用 `fallback`。

状态从 `Pending` 进入 `Ready` 或 `Failed`。缓存命中时，View 可以一开始就返回 `Ready`。缓存淘汰不改变 View 中的描述符；client 读取失败后重新请求 View。

### 3.2 `ResourceId`

`ResourceId` 标识一个媒体产物，而不是文档中的固定位置。以下输入变化时产生新的 `ResourceId`：

- 媒体类型或源内容；
- 公式或 Mermaid 的源码；
- 本地图片的内容指纹；
- 主题模式与前景色；
- 目标尺寸或终端能力对应的渲染参数；
- 渲染器或位图化参数。

同一产物可以被多个文档引用；`media.ready` 按引用文档分别通知。`ResourceId` 是不透明值，缓存淘汰或 server 重启后可能失效。

View 使用 `reserved` 表达预留尺寸，使用 `actual` 表达资源实际尺寸。默认策略是固定占位：client 将实际资源缩放到 `reserved`，不自行重排。需要改变布局时，由 server 生成新的 View。

### 3.3 降级内容

`MediaFallback` 由 view 域定义：

```text
MediaFallback {
  Text(String) | Source(String)
}
```

- `Formula` 失败时优先显示文本近似，其次显示源码。
- `Mermaid` 失败时显示源码。
- `Image` 失败时显示 `alt` 与路径。
- 终端不支持图片协议时，View 直接使用文本或源码的普通布局行。

## 4 生成、缓存与失效

View 装配时查询媒体资源：

```text
缓存命中 → Ready，无事件
缓存未命中 → Pending → 后台生成
                         ├─ 成功 → Ready + media.ready
                         └─ 失败 → Failed + media.failed
```

缓存由 server 维护。缓存命中时 View 直接返回 `Ready`；未命中时进入 `Pending` 并在后台生成。缓存键与淘汰策略属于 server 实现，不进入协议；`ResourceId` 在源内容、渲染参数、主题或目标尺寸变化时改变。

生成与失效规则：

- 公式与 Mermaid 的源码变化由 `document.changed` 驱动新的 View。
- 本地图片内容变化时文档版本不变；media 维护图片路径到引用文档的依赖，内容指纹变化后生成新的 `ResourceId`，并通知引用文档重新请求 View。
- 图片删除后对应位置显示缺失提示与 `fallback`；图片恢复后按新指纹命中或生成新资源。
- 公式或 Mermaid 语法错误、渲染超时、外部工具缺失或资源过大时进入 `Failed`，使用 `fallback`。
- 生成任务不可用时进入 `Failed`，client 使用 `fallback`，阅读不被阻塞；后续 View 可以重新触发生成。
- 缓存淘汰后 `media.read` 返回 `media.not_found`；client 重新请求 View 获取当前 `ResourceId`。

## 5 事件与失效

### 5.1 `media.ready`

```text
media.ready {
  resource_id,
  document,
  document_version,
  kind,
  mime,
  size,
  extent?,
  updated_at
}
```

- scope 为 `document(path)`。
- `document_version` 用于丢弃过期事件。
- `extent` 是实际媒体尺寸，用于 client 缩放或更新占位。
- client 收到后：
  - `document_version` 已过期：丢弃；
  - `resource_id` 在当前 View 中：调用 `media.read` 并替换对应区域；
  - `resource_id` 不在当前 View 中：重新请求受影响的 View。
- 缓存命中时 View 直接返回 `Ready`，不重复发布 `media.ready`。

### 5.2 `media.failed`

```text
media.failed {
  resource_id,
  document,
  document_version,
  kind,
  error,
  updated_at
}
```

- `error` 使用 base 的 `ErrorInfo`。
- client 使用 View 中的 `fallback` 显示源码、文本近似或缺失提示。
- 失败资源只影响 View 降级，文档内容保持不变。

### 5.3 失效顺序

- 文档版本变化后，旧版本的 `media.ready` 与 `media.failed` 被 client 丢弃。
- 本地图片变化不改变文档版本；事件仍携带当前 `document_version`，client 在资源 ID 未知时重新请求 View。
- 同一文档的媒体事件按 scope 顺序到达；client 以当前 View 与 `document_version` 判断是否应用。

## 6 终端能力与降级

media 根据 client 的终端能力选择生成与降级路径：

- 没有图片协议时，server 在 View 中直接提供文本近似或源码，不生成位图。
- 支持半块字符时，server 生成低分辨率位图，client 以半块字符显示。
- 支持 Kitty、Sixel 或 iTerm2 时，server 生成对应目标的位图，client 负责终端编码。
- `theme_mode` 影响公式与 Mermaid 的前景色，进入资源身份与缓存键。
- client 没有 `media.image` 能力时使用 `fallback`；server 可以跳过不必要的生成。

当前设计采用 server 侧位图化：公式、Mermaid 与 SVG 由 server 输出位图，client 只负责终端图片协议编码与显示。server 侧位图化需要单元格像素尺寸；`TerminalProfile` 提供 `cell_width_px` 与 `cell_height_px`，缺失时 server 使用默认尺寸，client 按 Cell 缩放。

本地图片以工作区相对路径引用，经 workspace 校验后读取；远程图片默认不下载，View 显示 `alt` 与链接。图片类型、字节大小与像素尺寸受 `MAX_RESOURCE_BYTES` 及 server 限制约束，超出时使用 `fallback`。

## 7 错误

| 错误码 | 场景 | retryable |
| --- | --- | --- |
| `media.not_found` | `ResourceId` 不存在、已淘汰或已失效 | false |
| `media.not_ready` | 资源仍在生成 | true |
| `media.failed` | 资源生成失败 | false |
| `media.unavailable` | 媒体服务、渲染器或缓存暂时不可用 | true |
| `media.invalid_range` | `offset` 或 `max_bytes` 非法 | false |
| `path.rejected` | 图片路径越界或指向内部状态 | false |
| `permission.denied` | principal 缺少 `workspace.read` | false |
| `protocol.limit_exceeded` | 读取块或消息超出上限 | false |
| `internal` | 未预期错误 | false |

## 8 与其他域的协作

| 域 | 协作方式 |
| --- | --- |
| document | 提供媒体节点的源位置、`DocumentVersion` 与 `document.changed` |
| view | 定义 `MediaDescriptor`、`MediaExtent`、`MediaFallback` 与占位 |
| workspace | 校验本地图片路径，提供资源变化监听与大小限制 |
| write | 写入只处理文本引用；媒体二进制留在 media 资源与缓存 |
| index | 索引消费文档文本与元数据；媒体二进制留在 media 缓存 |
| health | 健康规则保持现有四类；媒体失败通过 ViewDiagnostic 表达 |
| plugin / MCP | 按接入策略使用只读媒体读取；媒体生成由 server 完成 |
| TUI / CLI | TUI 负责终端图片协议与 Cell 栅格化；CLI 使用文本或源码降级 |

跨域流程见 [domain.md](../domain.md)。
