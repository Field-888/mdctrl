# mdctrl-protocol 基础值模型

> 文档状态：草案 · 最后复核：2026-11-27

> base 定义 protocol 中跨原语、跨消息、跨外部端口共同使用的值、编码与兼容规则。

## 1 定位

base 是 protocol 的共同值模型。它规定标识、路径、版本、时间、范围、错误与通用值如何表示，以及这些值如何在 JSON 中编码、如何随协议演进而保持兼容。控制原语、服务原语、请求、响应与事件都使用 base 的值。

概念模型由 [concepts.md](concepts.md) 定义。控制原语的目录与生命周期由 [runtime.md](runtime.md) 定义，服务原语的目录与业务语义由 [domain.md](domain.md) 与 `domain/*.md` 定义，外部端口映射由 [external.md](external.md) 定义，Rust 类型、注册表与测试由 [implementation.md](implementation.md) 定义。base 提供这些文档共同使用的值与编码规则。

`Request`、`Response`、`Event` 的消息结构与请求生命周期在 runtime.md 展开；base 定义其中的标识、版本、错误与通用值。能力协商、终端能力与请求超时同样属于 runtime.md。

## 2 标识与身份值

### 2.1 通用标识

| 标识 | 所有者 | 用途 |
| --- | --- | --- |
| `PrimitiveId` | protocol | 原语身份 |
| `RequestId` | client | 会话内唯一的请求标识，用于关联响应 |
| `SessionId` | server | 会话句柄 |
| `WorkspaceId` | server | 工作区句柄 |
| `JobId` | server | 长任务句柄 |
| `SubscriptionId` | server | 事件订阅句柄 |
| `ProposalId` | server | 写入提案句柄 |
| `HunkId` | server | 写入差异块句柄 |
| `RunId` | server | agent 运行句柄 |
| `ResourceId` | server | 媒体资源句柄 |
| `PluginId` | server | 插件句柄 |

ID 使用不透明 ASCII 字符串，长度 1..=128 字节，区分大小写，序列化为 JSON 字符串。`RequestId` 由 client 生成，在会话内唯一；其余 ID 由 server 生成。server 按会话隔离 `RequestId` 命名空间，并以 `(SessionId, RequestId)` 关联请求与响应；内部请求句柄留在实现层。client 与 plugin 按不透明值处理 ID，ID 的内部结构由所有者定义。`WorkspaceId` 是与文件系统路径解耦的句柄。

每个 ID 在 Rust 中使用独立 newtype，具体类型名、serde 映射与构造校验见 implementation.md。

### 2.2 原语分类值

| 类型 | 取值 | 用途 |
| --- | --- | --- |
| `PrimitiveKind` | `Control`、`Service` | 区分控制原语与服务原语 |
| `Access` | `Read`、`Write` | 服务原语的读写属性 |
| `Execution` | `Unary`、`Job`、`Subscription` | 原语的执行方式 |

原语 ID 使用命名空间：`runtime.*` 归控制原语，`<domain>.*` 归服务原语。控制原语围绕 `Unary` 与 `Subscription` 展开，服务原语可以使用 `Unary`、`Job` 与 `Subscription`。原语描述符的完整字段由 runtime 与 domain 定义；base 提供分类值与标识。

### 2.3 身份值

| 类型 | 取值 | 用途 |
| --- | --- | --- |
| `ClientKind` | `Tui`、`Cli`、`Mcp`、`Test`、`Other` | 客户端类型 |
| `Principal` | `User`、`Agent`、`Plugin`、`Mcp` | 动作代表的身份 |

base 定义枚举与序列化形式。主体、会话、权限与审批语义由 concepts、runtime 与 domain 定义。

## 3 路径与资源

| 类型 | 形式 | 用途 |
| --- | --- | --- |
| `DocumentPath` | 工作区相对 UTF-8 路径 | 文档身份 |
| `ResourceId` | 不透明字符串 | 媒体资源 |
| `AnchorId` | 字符串 | 文档锚点 |

`DocumentPath` 使用 `/` 分隔与 UTF-8 编码。路径拒绝前导 `/`、空段、`.`、`..`、NUL、Windows 盘符与 UNC 前缀。任一路径段为 `.mdctrl` 或 `.git` 时，按大小写无关方式比较并拒绝。路径长度上限为 4096 字节，按字节比较；server 结合目标文件系统完成规范化与实际路径解析。

协议消息携带工作区相对路径，绝对路径留在 server 内部。protocol 使用 UTF-8 路径；server 对非 UTF-8 文件名生成诊断。`ResourceId` 由 server 分配，client 通过资源相关服务原语读取内容，媒体缓存路径留在 server。`AnchorId` 由 server 解析为文档位置，client 按字符串处理。

base 定义路径的合法形式；server 的工作区文档访问接口执行最终校验。

## 4 版本

| 版本 | 所有者 | 用途 |
| --- | --- | --- |
| `ProtocolVersion` | protocol | 握手与语义兼容 |
| `WireSchemaVersion` | protocol | 消息信封结构 |
| `WorkspaceSchemaVersion` | server | 本地状态迁移 |
| `DocumentVersion` | server | 文档版本 |
| `DocumentFingerprint` | server | 内容身份、写入基准、`exact` 匹配 |
| `DocumentRevision` | server | 排序与展示 |
| `IndexVersion` | server | 搜索、健康、反链结果标注 |
| `ViewGeneration` | client | 丢弃过期视图响应 |

`ProtocolVersion` 由 major 与 minor 组成，均为无符号整数。major 处理语义上的破坏性变化，minor 增加兼容能力。`WireSchemaVersion` 使用无符号整数，在消息信封结构发生破坏性变化时递增。`WorkspaceSchemaVersion` 用于 `.mdctrl` 本地状态迁移。

`DocumentVersion` 由 `DocumentFingerprint` 与 `DocumentRevision` 组成。`DocumentFingerprint` 是不透明字符串，表达文档内容身份；写入基准与 `exact` 匹配使用指纹。`DocumentRevision` 是 `u64` 单调序号，在 server 生命周期内用于排序与展示，server 重启后重新开始。

`IndexVersion` 是 `u64` 单调序号，随索引提交或重建递增，用于标注搜索、健康与反链结果基于哪一版索引。视图有效性由 `DocumentVersion` 与 `ViewGeneration` 负责，索引版本服务于索引派生结果。

`ViewGeneration` 由 client 维护，wire 字段为 `generation`。server 在响应中回传该值，并按不透明值处理。client 用它判断视图响应是否过期。

各版本服务不同目的，按用途选择。指纹与 schema 版本按相等比较；修订与索引版本在各自生命周期内按序比较；视图代次按不透明值处理。

## 5 时间、范围与大小

`Timestamp` 使用 RFC 3339 UTC 字符串，统一 `Z` 后缀，精确到毫秒。`DurationMs` 使用 `u64` 毫秒。显示时由 client 按本地时区呈现。

时间语义区分观察时间与发生时间。`observed_at` 表示 server 观察到变化的时间，由 server 生成；`occurred_at` 表示外部事件发生的时间，由来源提供。系统时钟回拨时保持单条记录内部一致，展示层按 UTC 处理。

| 类型 | 形式 | 用途 |
| --- | --- | --- |
| `SourcePosition` | 1-based 行号与列号 | 源码位置 |
| `SourceSpan` | `start` 与 `end` | 源码区间 |
| `Viewport` | 布局行起点、高度、宽度 | 视图请求 |
| `PageRequest` | `offset`、`limit` | 分页请求 |
| `PageInfo` | `truncated`、可选 `total` | 分页结果 |

`SourceSpan` 的结束端指向区间后的第一个位置。`Viewport` 的 `start_line` 指 server 生成的布局行，源码位置由 `SourceSpan` 表达，宽度以终端 Cell 为单位。`PageRequest` 使用从零开始的偏移；`PageInfo.total` 在计数成本高时省略，`truncated` 表示结果被截断。

`ByteSize` 使用 `u64` 字节数。限制类别覆盖单条消息、路径、查询串、结果数量、元数据与媒体资源：

| 限制 | 含义 |
| --- | --- |
| `MAX_MESSAGE_BYTES` | 单条消息大小 |
| `MAX_PATH_BYTES` | 文档路径长度 |
| `MAX_QUERY_CHARS` | 查询串长度 |
| `MAX_RESULTS` | 单次结果数量 |
| `MAX_METADATA_BYTES` | frontmatter 元数据大小 |
| `MAX_RESOURCE_BYTES` | 媒体资源大小 |

base 定义限制类别与语义；server 在握手中公布生效上限，client 按公布值裁剪请求。

## 6 通用值

`Value` 表示开放式 JSON 值：

```text
Null | Bool | Integer | Float | String | Array | Object
```

对象键为字符串，数组元素可以为任意 `Value`。浮点为有限值，拒绝 NaN 与 Infinity。整数保持在 JSON 安全范围内；精度敏感值使用字符串。

`Value` 用于 frontmatter 元数据、错误详情、plugin manifest 扩展与外部 DTO。结构化载荷使用具名类型；`Value` 服务于开放式与扩展字段。frontmatter 使用 `Map<String, Value>` 表达。

`Binary` 表示 base64 编码的字节串，JSON 中序列化为字符串；embedded typed channel 使用原始字节。

server 对 `Value` 的深度、字符串长度与集合大小设置上限，超出时返回 `protocol.limit_exceeded`。

## 7 结果、错误与警告

```text
Reply<T> { ok, data, error, warnings }
ErrorInfo { code, message, retryable, details?, hint? }
Warning { code, message, details? }
```

`Reply` 统一表达原语结果。`ok` 表示执行成功，业务结论放在 `data`。`ok=true` 时 `error` 为空；`ok=false` 时 `error` 存在。`data` 可以为空；`warnings` 与成功、失败正交。

`ErrorCode` 使用 `domain.condition` 命名，`internal` 表示内部错误；错误码保持稳定并避免复用。错误码族覆盖协议、运行时、工作区、路径、文档、索引、权限、插件、模型提供方、媒体与内部错误：

| 族 | 典型码 | 来源 |
| --- | --- | --- |
| `protocol.*` | `version_mismatch`、`unsupported_primitive`、`invalid_message`、`limit_exceeded` | 协议与消息 |
| `runtime.*` | `session_closed`、`cancelled`、`timeout`、`job_not_found` | 会话与运行时 |
| `workspace.*` | `not_found`、`locked`、`config_invalid`、`schema_unsupported` | 工作区 |
| `path.*` | `rejected` | 路径校验 |
| `document.*` | `not_found`、`version_mismatch` | 文档 |
| `index.*` | `unavailable`、`busy` | 索引 |
| `permission.*` | `denied` | 权限 |
| `plugin.*`、`provider.*`、`media.*` | `failed`、`unavailable` | 外部能力 |
| `internal` | `internal` | 未预期错误 |

`retryable` 标记瞬时错误，例如超时、忙与服务暂时失败。`message` 面向人类，使用英文；稳定判断依赖 `code`。`hint` 提供可执行的下一步；`details` 使用 `Value`。诊断堆栈进入日志，`ErrorInfo` 保持面向调用方的结构化信息。

`Warning` 使用同类 `code` 与 `message` 结构，`details` 承载可选上下文。`Reply` 同时用于 wire 响应与 CLI 输出；业务结论放在 `data`，警告独立于成功与失败。

## 8 序列化与兼容

### 8.1 序列化

wire 使用 JSON；embedded 使用 typed 值，保持相同逻辑形状。文本采用 UTF-8，无 BOM；字段名使用 `snake_case`。枚举序列化为字符串，未知枚举值按 unknown 处理。

可选字段省略表示 None；读取时 `null` 视为 None。读取端忽略未知字段。请求中的未知原语返回 `protocol.unsupported_primitive`；未知事件 topic 忽略并按需重新同步。

时间使用第 5 节规定的 UTC 格式；浮点为有限值。对象键顺序与比较无关；消息载荷拒绝注释与尾随数据。单条消息大小受上限约束，分帧由传输层负责。

protocol 类型通过 serde 暴露序列化；JSON 编解码位置与依赖见 implementation.md。

### 8.2 兼容

兼容变化包括新增可选字段、错误码、原语、事件 topic 与枚举值。读取端按 unknown 处理新值，旧客户端可以继续工作。

破坏性变化包括删除或重命名字段、改变字段类型或语义、删除枚举值、改变 ID 或错误码含义。破坏性变化使用新原语 ID 或提升 `ProtocolVersion` major。

`WireSchemaVersion` 随信封结构的破坏性变化递增；`WorkspaceSchemaVersion` 随本地状态结构变化递增。原语 ID 与错误码保持稳定；弃用项提供替代项，并保留至少一个 minor 周期。

每个值类型与信封维护契约夹具。兼容测试覆盖旧版本夹具、未知字段与跨传输行为，具体测试见 implementation.md。

## 9 命名约定

类型使用 PascalCase；字段与变量使用 snake_case；常量使用 SCREAMING_SNAKE_CASE。

类型后缀使用 `Id`、`Version`、`Descriptor`、`Info`、`Request`、`Response`、`Event` 与 `Reply`。原语 ID 使用 `domain.action`；错误码使用 `domain.condition`；事件 topic 使用 `domain.event`；能力标识使用 `area.feature`。

wire 字段名保持稳定；Rust 类型名遵循项目命名规则。每个值使用独立类型，减少裸字符串与裸整数在接口中混用。
