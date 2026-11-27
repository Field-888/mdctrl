# mdctrl-protocol 文档写入

> 文档状态：草案 · 最后复核：2026-11-27

> write 定义编辑提案、差异、审批、写入、撤销与溯源关联。

## 1 定位

write 是写入与扩展的派生域，负责文档写入。User、Agent、Plugin 与 Mcp 对文档的修改统一进入提案流程，经过差异生成、审批、版本检查与原子写入后落盘。

write 有两条写入路径：

- 审批通过后应用提案；
- owner 在时间窗口内撤销已经执行的写入。

两条路径共用同一套 apply 流程、版本检查与历史记录。人工编辑、agent 与 plugin 只是提案来源不同，落盘语义一致。

write 使用 [base.md](../base.md) 的 `ProposalId`、`DocumentVersion`、`DocumentFingerprint`、`SourceSpan`、`Timestamp`、`PageRequest`、`PageInfo` 与错误模型。document 提供文档状态与写入基准；history 记录变更与 `Provenance`；workspace 提供路径校验与原子写入语义。

提案存储、撤销缓存、内部写门与原子替换的实现见 server 文档。

## 2 服务原语

| 原语 | access | execution | 作用 | policy |
| --- | --- | --- | --- | --- |
| `write.propose` | `Write` | `Unary` | 创建编辑提案 | `workspace.propose` |
| `write.get` | `Read` | `Unary` | 按 id 返回提案详情 | `workspace.read` |
| `write.list` | `Read` | `Unary` | 返回提案摘要列表 | `workspace.read` |
| `write.approve` | `Write` | `Unary` | 审批并应用全部或部分差异 | User |
| `write.reject` | `Write` | `Unary` | 拒绝待审批提案 | User |
| `write.withdraw` | `Write` | `Unary` | 撤回 `Pending` 提案 | owner |
| `write.undo` | `Write` | `Unary` | 撤销时间窗口内已执行的写入 | owner + `workspace.undo` |

`write.propose` 接收完整的新内容，由 server 生成差异。`write.approve` 是 User 的落盘决策；TUI 将用户保存映射为 `write.propose` 与 `write.approve` 的自审批。Agent、Plugin 与 Mcp 的提案保持 `Pending`，由 User 审批。

`write.get` 与 `write.list` 返回按调用者计算的 `undo_available`。同一提案对不同主体可以返回不同的撤销可用状态。

## 3 提案、差异与归属

### 3.1 创建提案

```text
WriteProposeRequest {
  path,
  base_version?,
  content
}
```

- `path` 使用 base 的 `DocumentPath`。
- `content` 是完整的新内容，差异由 server 生成。
- `base_version` 省略时，server 读取当前版本并记录为基准。
- `base_version` 与当前版本不匹配时返回 `write.stale_base`，不创建提案。
- 路径不存在时生成 `Create` 提案；路径存在时生成 `Update` 提案。
- `Create` 提案的 `base_version` 为空；携带 `base_version` 时返回 `write.stale_base`。
- 提案默认进入 `Pending`，由 server 设置 `expires_at?`。

`principal` 由会话绑定决定，client 不自行声明来源。

### 3.2 提案形状

```text
Proposal {
  id,
  path,
  kind,              // Create | Update
  status,
  principal,         // User | Agent | Plugin | Mcp
  base_version?,
  new_version?,
  diff: [DiffHunk],
  applied_hunks?,
  conflict?: { current_version },
  undo_available,
  undo_deadline?,
  undone_at?,
  undone_version?,
  created_at,
  updated_at,
  expires_at?,
  provenance_id?
}
```

```text
ProposalStatus {
  Pending,
  Applied,
  PartiallyApplied,
  Conflicted,
  Failed,
  NeedsReview,
  Rejected,
  Withdrawn,
  Expired,
  Undone
}
```

状态含义：

| 状态 | 含义 |
| --- | --- |
| `Pending` | 等待审批 |
| `Applied` | 全部差异已写入 |
| `PartiallyApplied` | 选中差异已写入 |
| `Conflicted` | 审批时版本不匹配 |
| `Failed` | 写入失败，文档未改变 |
| `NeedsReview` | 写入结果不确定，需要人工核对 |
| `Rejected` | User 拒绝提案 |
| `Withdrawn` | owner 撤回提案 |
| `Expired` | 提案超过有效期 |
| `Undone` | 已执行写入在撤销窗口内被撤销 |

`Approved` 与 `Applying` 是内部过渡状态，不进入协议。`Conflicted` 是终态，重新写入需要创建新提案。

`undo_available` 表达调用者当前是否可以撤销。owner 由 server 内部保存为稳定主体身份，不以会话为粒度。

### 3.3 差异

```text
DiffHunk {
  id,                // HunkId，提案内稳定
  old_span?,
  new_span?,
  lines: [DiffLine]
}

DiffLine {
  kind,              // Context | Add | Remove
  text
}
```

- `old_span` 与 `new_span` 使用 base 的 `SourceSpan`。
- 差异按行生成，`Context` 行用于展示上下文。
- `HunkId` 在单个提案内稳定，用于局部采纳。
- `write.approve` 省略 `hunks` 表示应用全部差异；显式提供时必须非空。

### 3.4 归属与权限

`Proposal` 的 owner 是提案创建时绑定的稳定主体身份。`principal` 表达主体类型；owner 由 server 保存并用于权限判定。

| 操作 | User 提案 | Agent / Plugin / Mcp 提案 |
| --- | --- | --- |
| `write.withdraw` | owner（User） | owner（策略主体） |
| `write.reject` | User | User |
| `write.approve` | User（自审批） | User |
| `write.undo` | owner（User） | owner（策略主体）+ `workspace.undo` |

- 审批与拒绝是 User 的决策。
- 撤回与撤销是 owner 的权利。
- 策略主体只能撤回或撤销自己的提案；其他 agent、plugin、MCP 与 User 都不能代撤销。
- `write.undo` 先校验归属、能力与提案状态，再检查撤销缓存与时间窗口。

## 4 审批与写入

### 4.1 审批

```text
WriteApproveRequest {
  proposal_id,
  hunks?
}
```

- 只接受 `Pending` 提案。
- 省略 `hunks`：应用全部差异，结果为 `Applied`。
- 提供 `hunks`：只应用选中的差异块；覆盖全部差异时为 `Applied`，否则为 `PartiallyApplied`，并记录 `applied_hunks`。
- 未选中的差异随提案结束，不进入文档。
- 审批前版本检查：
  - 当前指纹与 `base_version` 一致：继续；
  - 不一致：状态变为 `Conflicted`，`conflict.current_version` 携带当前版本，文档保持原内容。
- `Create` 提案在审批时目标路径已存在：状态变为 `Conflicted`。
- `Update` 提案在审批时目标路径缺失：状态变为 `Conflicted`。
- 已过期返回 `write.proposal_expired`。
- 包含未知差异块或空选择返回 `write.invalid_hunk`。
- 非 User 调用返回 `permission.denied`。

审批以 `DocumentFingerprint` 比较版本；`DocumentRevision` 用于展示与排序。`write.diff_hunks` 表示 client 可以展示并提交局部差异选择；能力缺失时 `write.approve` 省略 `hunks`，应用全部差异。

### 4.2 写入

审批通过后进入统一 apply 流程：

1. 再次确认当前版本；
2. 在 server 内部按路径获取短暂 `WriteGate`；
3. 使用临时文件与原子替换；
4. `Create` 使用不覆盖已有文件的创建方式；
5. 更新提案状态与 `new_version`。

写入由 server 串行执行。write 不引入协议可见的长期文档锁；外部编辑不参与内部写门，版本检查以指纹为最终依据。

写入结果：

- 成功：状态为 `Applied` 或 `PartiallyApplied`。
- 写入失败：状态为 `Failed`，文档保持原内容。
- 结果不确定：状态为 `NeedsReview`，保留证据供人工核对。

写入成功后：

- history 记录变更与 `Provenance`；
- document 发布 `document.changed`；
- write 发布 `proposal.changed`；
- 所有 client 重新读取文档与提案状态。

`document.changed` 的 `reason` 由变更类型决定：创建为 `Created`，修改为 `Modified`，删除为 `Deleted`。

### 4.3 拒绝与撤回

```text
WriteRejectRequest {
  proposal_id
}

WriteWithdrawRequest {
  proposal_id
}
```

- `write.reject`：User 拒绝 `Pending` 提案，状态变为 `Rejected`。
- `write.withdraw`：owner 撤回 `Pending` 提案，状态变为 `Withdrawn`。
- 拒绝与撤回都不修改文档。
- 对已过期 `Pending` 提案执行审批、拒绝或撤回返回 `write.proposal_expired`。
- 对已结束提案再次审批、拒绝或撤回返回 `write.invalid_state`。

## 5 撤销

### 5.1 撤销请求

```text
WriteUndoRequest {
  proposal_id
}
```

- 只接受 `Applied` 与 `PartiallyApplied` 提案。
- 权限为 owner，策略主体还需要 `workspace.undo`。
- 自撤销不再走人工审批，但仍使用统一 apply 流程。
- 每个提案最多撤销一次，撤销后不支持 redo。
- 撤销成功后状态为 `Undone`，记录 `undone_at` 与 `undone_version`。
- 撤销失败时提案保持原状态，缓存条目保留，可在窗口内重试。

### 5.2 版本保护与内容恢复

撤销前比较当前文档指纹与提案的 `new_version`：

- 一致：允许撤销；
- 不一致：返回 `write.undo_conflict`，保留当前内容；
- 目标文档缺失：返回 `write.undo_conflict`。

内容恢复：

- `Create` 撤销：版本检查通过后删除文件，不再二次确认。
- `Update` 撤销：恢复写入前完整内容，产生新的文档版本。
- `PartiallyApplied` 撤销：恢复写入前完整内容，未采纳的差异不参与撤销。
- 删除或恢复失败返回 `write.undo_failed`，提案保持原状态。

`Create` 撤销删除文档后 `undone_version` 为空；`Update` 撤销的 `undone_version` 是恢复后产生的新版本。

### 5.3 撤销缓存

```text
UndoEntry {
  proposal_id,
  path,
  kind,
  previous_content?,
  previous_version?,
  applied_version,
  applied_hunks?,
  created_at,
  expires_at,
  principal
}
```

- `applied_version` 记录提案的 `new_version`。
- `principal` 记录提案发起主体；执行撤销的主体写入新的 `Provenance`。
- 只保存在 server 内存，不写入 `.mdctrl/`、日志、事件或协议响应。
- 以 `proposal_id` 索引，每个提案一条。
- 上限只按总字节数计算，由 `[write]` 配置提供 `undo_cache_bytes`。
- 达到上限时按 FIFO 淘汰最早加入的条目，直到能容纳新条目。
- 单条记录超过总上限时不进入缓存。
- 撤销成功后删除对应条目。
- 到期清理；server 关闭、daemon 重启或 embedded 退出时整体丢弃。
- 被淘汰或丢失后 `undo_available = false`，`write.undo` 返回 `write.undo_unavailable`。
- 超过撤销窗口返回 `write.undo_expired`。

撤销窗口由 server 在写入成功后设置，截止时间记录在 `undo_deadline`。撤销窗口不延长；撤销成功后不再产生新的撤销窗口。

### 5.4 撤销溯源

`Provenance` 增加：

```text
operation,       // Write | Undo
undo_of?         // Undo 时指向原 ProposalId
```

```text
普通写入：
  operation = Write
  proposal_id = P
  undo_of = null

撤销写入：
  operation = Undo
  proposal_id = null
  undo_of = P
```

- 撤销记录的 `principal` 是执行撤销的主体。
- 撤销不重新填 `approved_by`；原审批链保留在原写入记录中。
- 撤销产生新的 `Provenance`，不复用原写入的 `provenance_id`。
- `base_version` 为被撤销版本，`new_version` 为撤销后版本。
- `Create` 撤销在 history 中记录 `Deleted`；`Update` 撤销记录 `Modified`。
- 原提案与撤销记录都保留，审计可以串联完整链路。
- 撤销缓存中的正文不进入 history、日志、事件或协议响应。
- 长期记录由 history 承担，撤销缓存只负责短期恢复。

## 6 查询

```text
WriteGetRequest {
  proposal_id
}
```

`write.get` 返回完整 `Proposal` 与差异。

```text
WriteListRequest {
  path?,
  status?,
  page: PageRequest
}

WriteListReply {
  proposals: [ProposalSummary],
  page: PageInfo
}

ProposalSummary {
  id,
  path,
  kind,
  status,
  principal,
  base_version?,
  created_at,
  updated_at,
  expires_at?,
  undo_available,
  undo_deadline?
}
```

- `path` 为精确匹配；省略时返回工作区范围的提案。
- `status` 过滤单个提案状态。
- 列表按 `updated_at` 降序排列，同时间按 `ProposalId` 降序。
- `undo_available` 与 `undo_deadline` 按调用者计算。
- 列表不返回文档正文。

## 7 事件与错误

### 7.1 事件

```text
proposal.changed {
  proposal_id,
  path,
  status,
  updated_at
}
```

- scope 为 `workspace`。
- 提案创建、审批、拒绝、撤回、过期、写入失败、需要核对与撤销都发布该事件。
- client 收到后重新读取 `write.get` 或 `write.list`。
- 文档内容变化由 document 发布 `document.changed`，write 不重复定义。

### 7.2 错误

| 错误码 | 场景 | retryable |
| --- | --- | --- |
| `write.proposal_not_found` | 提案 id 缺失 | false |
| `write.stale_base` | `base_version` 与当前版本不匹配 | false |
| `write.invalid_hunk` | 审批包含未知差异块或空选择 | false |
| `write.invalid_state` | 对已结束提案执行审批、拒绝、撤回或撤销 | false |
| `write.proposal_expired` | `Pending` 提案已过期 | false |
| `write.undo_unavailable` | 撤销缓存丢失或被 FIFO 淘汰 | false |
| `write.undo_expired` | 超过撤销窗口 | false |
| `write.undo_conflict` | 当前文档版本已变化或文档缺失 | false |
| `write.undo_failed` | 删除或恢复写入失败 | false |
| `path.rejected` | 路径越界或指向内部状态 | false |
| `permission.denied` | principal 缺少能力或不是 owner | false |
| `protocol.limit_exceeded` | 内容或差异超出限制 | false |
| `internal` | 未预期错误 | false |

## 8 与其他域的协作

| 域 | 协作方式 |
| --- | --- |
| workspace | 提供路径校验、配置、文档快照与原子写入语义 |
| document | 提供 `DocumentVersion` 与文档状态；写入成功后发布 `document.changed` |
| history | 记录变更与 `Provenance`；撤销记录使用 `operation = Undo` 与 `undo_of` |
| index | 文档变化后由 `document.changed` 触发索引失效与更新 |
| health | 文档变化后重新评估断链、孤儿、过期与元数据结果 |
| view | 写入成功后 client 重新读取 `document.view` 并重新栅格化 |
| agent | 读取上下文并提交提案；审批由 User 完成；可撤销自己的提案 |
| plugin | 通过 `workspace.propose` 提交提案；审批由 User 完成；可撤销自己的提案 |
| media | 写入只处理文本引用；媒体资源生成、读取与降级由 media 域负责 |
| Mcp | 写入按接入策略开放；开放后使用同一提案、审批与归属规则 |
| TUI / CLI | 展示差异、执行审批、拒绝、撤回与撤销；协议不定义退出码 |

跨域流程见 [domain.md](../domain.md)。
