# mdctrl-protocol 文档历史

> 文档状态：草案 · 最后复核：2026-11-26

> history 记录文档观察、内容与状态变化、身份关联、写入与撤销溯源。

## 1 定位

history 是基础状态域，负责记录 server 对文档的观察、内容与状态变化、身份关联、写入与撤销溯源。每个 `DocumentPath` 维护一颗 `DocHistoryTree`；节点组成有向无环图，默认视图呈现 Active 节点，Audit 视图保留完整观察记录。

history 使用 [base.md](../base.md) 的 `DocumentPath`、`DocumentVersion`、`Timestamp`、`PageRequest` 与 `PageInfo`。文档查看、写入与健康分别由 document、write 与 health 提供。

历史库 schema、迁移、监听、对账与 Git 合并实现见 server 文档。

## 2 服务原语

| 原语 | access | execution | 作用 | policy |
| --- | --- | --- | --- | --- |
| `history.summary` | `Read` | `Unary` | 返回文档摘要、当前头与分支头 | `workspace.read` |
| `history.tree` | `Read` | `Unary` | 返回文档 DAG 窗口与 refs | `workspace.read` |
| `history.node` | `Read` | `Unary` | 返回节点详情与 changes | `workspace.read` |
| `history.changes` | `Read` | `Unary` | 返回文档或工作区的 change 列表 | `workspace.read` |
| `history.provenance` | `Read` | `Unary` | 返回单次写入或撤销的溯源详情 | `workspace.read` |

服务原语保持只读，按 `workspace.read` 能力授权。查询形状在第 6 节定义，溯源形状在第 7 节定义。

## 3 身份与 DAG

文档历史以 `DocumentPath` 为身份键。路径 A 消失、路径 B 出现时，历史分别保留 A 与 B 的树，通过 `IdentityLink` 表达重命名关联；节点保留原路径。

```text
DocHistoryTree {
  path,
  nodes: [DocHistoryNode],
  current: HistoryHead,
  heads: [HistoryHead],
  refs: [GitRef]
}
```

- `parent_ids` 表达文件历史父节点。
- 无 Git 时节点单父，图呈链表。
- 有 Git 时分支产生多个子节点；merge 节点使用多个 `parent_ids`。
- `current` 表达当前工作区或检出位置；`heads` 表达分支与引用位置；`refs` 表达 branch、tag 与 detached HEAD。

```text
HistoryHead {
  ref?,
  node_id?,
  state,             // Present | Unreadable | Deleted | Unknown
  version?,
  git?
}

GitRef {
  name,
  kind,              // Branch | Tag | Detached
  commit,
  node_id?
}
```

`Unknown` 是覆盖缺口状态，只出现在 `HistoryHead`；节点状态取 `Present`、`Unreadable` 与 `Deleted`。

## 4 节点、变更与状态

### 4.1 节点

```text
DocHistoryNode {
  id,                // HistoryNodeId
  path,
  kind,              // Created | Modified | Deleted | Unreadable | Restored | Merged
  state,             // Present | Unreadable | Deleted
  version?,          // 节点内最后一次可读版本
  parent_ids: [HistoryNodeId],
  started_at,
  ended_at?,
  change_count,
  status,            // Active | Absorbed | Abandoned | Archived
  status_reason?,    // Committed | Superseded | BranchSwitch | Discarded | Rewritten
  absorbed_by?,      // Absorbed 节点指向承接的 commit 节点
  absorbed_node_ids?,// commit 节点引用被吸收的工作区链
  git?,              // GitAnchor
  changes?           // 详情查询时返回
}
```

```text
HistoryNodeStatus {
  Active,
  Absorbed,
  Abandoned,
  Archived
}

HistoryNodeStatusReason {
  Committed,
  Superseded,
  BranchSwitch,
  Discarded,
  Rewritten
}
```

### 4.2 变更

```text
HistoryChange {
  id,                // ChangeId
  node_id,
  path,
  kind,              // 与节点 kind 对齐
  observed_at,
  version?,
  size?,
  source,            // InitialScan | Reconcile | Watch | ServerWrite | Git
  provenance_id?
}
```

非 `Modified` 节点包含一条 change；`Modified` 节点是 series，包含窗口内的多条 change。审批写入与撤销产生的 change 都使用 `source = ServerWrite`。撤销 `Create` 提案时 `kind = Deleted`，撤销 `Update` 提案时 `kind = Modified`；`provenance_id` 分别指向原写入或撤销产生的 `Provenance`。

### 4.3 状态机

| 父状态 | 观察结果 | 节点 kind | 新状态 |
| --- | --- | --- | --- |
| 无父节点 | 可读 | `Created` | `Present` |
| `Present` | 指纹变化 | `Modified` | `Present` |
| `Present` | 读取失败 | `Unreadable` | `Unreadable` |
| `Present` | 确认缺失 | `Deleted` | `Deleted` |
| `Unreadable` | 可读 | `Restored` | `Present` |
| `Unreadable` | 确认缺失 | `Deleted` | `Deleted` |
| `Deleted` | 可读 | `Restored` | `Present` |
| `Deleted` | 读取失败 | `Unreadable` | `Unreadable` |
| 多父合并 | 内容合入 | `Merged` | `Present` |
| 任意 | 扫描覆盖缺口 | 无新节点 | `Unknown` 头 |

- 纯观察更新 `HistorySummary.last_observed_at`，节点列表保持原样。
- 扫描覆盖缺口把 head 置为 `Unknown`，change 列表保持原样。
- 每次内容或状态变化建立一个节点；`kind` 表达节点相对父节点的变化类型。

### 4.4 Modified series

`Modified` 节点使用时间窗口压缩连续编辑：

- 第一次 `Modified` 开启 series；
- 窗口内后续 `Modified` 向同一节点追加 change；
- 窗口参数进入 `[history]` 配置，默认值由配置维护；
- series 在空闲超限、跨度超限、change 数达到上限、出现非 `Modified` 变化、观察到 commit 或 branch/HEAD 变化时结束；
- 结束后写入 `ended_at`，下一次 `Modified` 开启新 series；
- 节点 `version` 取 series 内最后一次可读版本，`change_count` 记录 change 数量。

## 5 整合与视图

工作区未提交变更进入 DAG：

- 第一次未提交修改开启 `Modified` series；
- 节点 `GitAnchor.commit` 指向当前 HEAD，`worktree_state = Dirty`；
- `parent_ids` 指向当前分支的文件头节点；
- refs 与 `current` 指向该工作区节点。

活跃工作区链指当前分支自上一个 commit 以来、尚未被 commit 吸收的连续节点链。

### 5.1 commit 吸收

commit 改变该文件时：

- 建立 Active commit 节点；
- `parent_ids` 指向活跃工作区链的起点父节点；merge 时指向 Git 父提交对应的文件节点；
- `absorbed_node_ids` 引用活跃工作区链；
- `GitAnchor` 记录 commit、父提交、branch 与 `worktree_state = Clean`；
- 工作区链节点标记 `Absorbed`，原因 `Committed`，`absorbed_by` 指向 commit 节点；
- refs 与 `current` 指向 commit 节点；
- commit 节点的 `change_count` 汇总自身与被吸收链的 change 数量，详情查询按时间顺序合并 changes。

commit 未改变该文件时，节点列表保持原样，工作区链保持 Active。commit 内容与链分歧时，commit 节点按 Git 父提交建立；参与结果的链标记 `Absorbed`，未参与结果的链标记 `Abandoned`，原因 `Superseded`。

### 5.2 分支切换与丢弃

- 分支切换、detached HEAD 变化：当前工作区链标记 `Abandoned`，原因 `BranchSwitch`；`current` 移到目标分支头。
- 丢弃未提交修改：工作区链标记 `Abandoned`，原因 `Discarded`；`current` 回到分支头。

### 5.3 rebase / amend / force-push

- 新 commit 节点保持 Active；
- ref 更新后，失去引用的旧提交节点标记 `Archived`，原因 `Rewritten`；
- 仍被其他 branch 或 tag 引用的节点保持 Active；
- refs 更新到新节点。

### 5.4 Active 与 Audit

- `Active` 视图只返回 `Active` 节点；
- `Audit` 视图返回全部节点与状态；
- Active 节点的 `parent_ids` 指向 Active 节点；
- 分支结构由 refs 指向 commit 节点表达；
- dirty 叶子、放弃链与旧提交节点保留在 Audit 视图；
- 隐藏节点按 `[history]` 保留策略清理。

## 6 查询类型与结果

### 6.1 摘要

```text
HistorySummaryRequest {
  path
}

HistorySummary {
  path,
  current: HistoryHead,
  heads: [HistoryHead],
  first_seen_at,
  last_observed_at,
  last_changed_at,
  last_node_id?,
  identity_links: [IdentityLink]
}
```

`current` 表达当前工作区或检出状态；`heads` 表达各分支头；`last_node_id` 指向最近一次观察产生的 Active 节点。已删除文档保留摘要；路径在历史中缺失时返回 `history.not_found`。

### 6.2 树

```text
HistoryTreeRequest {
  path,
  view,              // Active | Audit
  ref?,
  depth?,
  limit?,
  cursor?
}

HistoryTreeReply {
  nodes: [DocHistoryNode],
  refs: [GitRef],
  current,
  heads,
  truncated,
  cursor?
}
```

`view = Active` 只返回 Active 节点；`view = Audit` 返回全部节点与状态。`ref` 从指定分支或标签开始；`depth`、`limit` 与 `cursor` 控制图窗口与继续加载；`truncated` 表示当前窗口之外仍有节点。

### 6.3 节点

```text
HistoryNodeRequest {
  node_id
}

HistoryNodeDetail {
  node,
  changes: [HistoryChange],
  absorbed_nodes?
}
```

`history.node` 按节点 id 返回详情；commit 节点按 `absorbed_node_ids` 合并工作区链的 changes，`absorbed_nodes` 提供审计引用。隐藏节点同样可以通过节点 id 查询。

### 6.4 变更

```text
HistoryChangesRequest {
  path?,
  since?,
  view,              // Active | Audit
  page: PageRequest
}

HistoryChangesReply {
  changes: [HistoryChange],
  page: PageInfo
}
```

省略 `path` 时返回工作区范围的 change 列表；`since` 是包含下界的时间过滤；默认只返回 Active 节点的 changes，`view = Audit` 包含隐藏节点；结果按 `observed_at` 降序、`ChangeId` 降序排列。

### 6.5 溯源

```text
HistoryProvenanceRequest {
  provenance_id
}
```

`history.provenance` 按 `ProvenanceId` 返回单次写入或撤销的溯源详情。

## 7 溯源、身份关联与 Git 锚点

### 7.1 溯源

```text
Provenance {
  id,                // ProvenanceId
  operation,         // Write | Undo
  node_id?,
  principal,         // User | Agent | Plugin | Mcp
  provider?,
  model?,
  plugin_id?,
  plugin_version?,
  plugin_command?,
  proposal_id?,      // Write 时指向提案
  undo_of?,          // Undo 时指向原 ProposalId
  run_id?,
  context: [ProvenanceContext],
  approved_by?,
  created_at,
  base_version?,
  new_version?,
  status             // Applied | Abandoned | NeedsReview
}

ProvenanceContext {
  path,
  version
}
```

- 溯源用于 server 写入与撤销；扫描、监听与 Git 触发的 change 没有 provenance；
- `operation` 区分普通写入与撤销；普通写入为 `Write`，撤销为 `Undo`；
- 普通写入的 `proposal_id` 指向提案；撤销的 `proposal_id` 为空，`undo_of` 指向原提案；
- 撤销产生新的 `Provenance`，不复用原写入的 `provenance_id`；
- `principal` 表达操作来源；撤销时是执行撤销的主体；
- `approved_by` 表达审批者；撤销不填 `approved_by`；
- `status` 表达该次操作结果，不表达当前文档状态；原写入与撤销记录都为 `Applied`；
- agent 写入与撤销保留 provider、model、run id 与上下文路径版本；
- plugin 写入保留 `plugin_id`、`plugin_version`、`plugin_command` 与 `proposal_id`；撤销保留同等插件信息与 `undo_of`；
- `context` 保存路径与版本；撤销的 `context` 记录被撤销版本；
- 未知溯源返回 `history.provenance_not_found`。

### 7.2 身份关联

```text
IdentityLink {
  from,
  to,
  kind,              // Candidate | Confirmed
  evidence: [IdentityEvidence],
  created_at
}

IdentityEvidence {
  SameFingerprint,
  ServerRename,
  StableId,
  UserConfirmed,
  FsRename
}
```

- 相同指纹作为候选证据；多个同内容文件同时出现时保留多个候选；
- `ServerRename`、稳定标识与用户确认可以形成确认关联；
- 历史保留原路径的树，通过关联串起两段历史；
- `HistorySummary` 返回以当前路径为 `from` 或 `to` 的关联。

### 7.3 Git 锚点

```text
GitAnchor {
  commit?,
  parent_commits?: [commit],
  author_name?,
  committed_at?,
  branch?,
  worktree_state?   // Clean | Dirty | Unknown
}
```

`parent_ids` 表达文件历史父节点；`GitAnchor.parent_commits` 表达 Git 提交父节点。Git 持有提交图、branch、tag、diff 与提交信息；`DocHistoryTree` 通过 `GitAnchor` 与 `GitRef` 关联提交与分支。作者信息默认保留姓名。

Git 缺失时，`DocHistoryTree` 以链表维护，Git 字段省略；Git 出现后新增节点带锚点，已有节点保持原样。

## 8 事件、错误与协作

### 8.1 事件

```text
history.changed {
  reason,              // Node | Change | Integrate | Ref | Provenance | Identity
  path?,
  node_id?,
  absorbed_node_ids?,
  archived_node_ids?,
  provenance_id?,
  identity_from?,
  identity_to?,
  observed_at
}
```

- `Node`：新建 Active 节点；
- `Change`：向开放 series 追加 change；
- `Integrate`：commit 吸收、工作区链放弃、旧提交归档导致 Active 视图变化；
- `Ref`：branch、tag 或 HEAD 变化；
- `Provenance`：溯源创建或状态变化，包括撤销产生的溯源；
- `Identity`：身份关联创建或确认；
- scope 为 `workspace`；client 收到后重新读取摘要、树或变更；
- 纯观察与 Git 元数据刷新通过摘要查询体现；
- `document.changed` 使用 document scope，表达文档状态变化。

### 8.2 错误

| 错误码 | 场景 | retryable |
| --- | --- | --- |
| `history.not_found` | 路径在历史中缺失 | false |
| `history.node_not_found` | 节点 id 缺失 | false |
| `history.provenance_not_found` | 溯源 id 缺失 | false |
| `history.unavailable` | 历史存储暂时失败 | true |
| `path.rejected` | 路径越界或指向内部状态 | false |
| `permission.denied` | principal 缺少能力 | false |
| `internal` | 未预期错误 | false |

### 8.3 协作

| 域 | 协作方式 |
| --- | --- |
| workspace | 提供扫描结果与去抖变化流；history 记录观察与变更 |
| document | `document.view` 触发观察；`document.changed` 与 `history.changed` 共同刷新界面 |
| write | 审批写入与撤销完成后登记节点与溯源；history 提供 `Provenance` 查询 |
| health | 读取 `last_changed_at`、`GitAnchor` 与生命周期字段，判断过期状态 |
| agent | 读取历史摘要、树、变更与溯源，形成上下文；写入进入 write 域 |
| plugin | 插件提案的溯源通过 write 域记录，history 提供查询 |

跨域流程见 [domain.md](../domain.md)。
