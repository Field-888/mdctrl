# mdctrl-protocol Agent

> 文档状态：草案 · 最后复核：2026-11-26

> agent 定义内置 agent 的工作流、运行、上下文、报告与提案协作。

## 1 定位

agent 是写入与扩展的派生域，提供内置 agent 的分析、审阅与写作工作流。agent 通过只读服务原语获取上下文，通过 write 域提交提案；提案审批由 User 完成。

agent 使用 [base.md](../base.md) 的 `RunId`、`DocumentPath`、`DocumentVersion`、`ErrorInfo` 与错误模型，使用 [runtime.md](../runtime.md) 的 Job、事件与取消语义。文档、搜索、健康、历史与 Git 信息由对应域提供；文档写入与撤销由 [write.md](write.md) 负责；provider 与 MCP 属于 external port。

Agent 的模型调用、上下文预算、工具循环、报告保留与 provider 适配实现见 server 文档。

## 2 服务原语

| 原语 | access | execution | 作用 | policy |
| --- | --- | --- | --- | --- |
| `agent.run` | `Write` | `Job` | 启动一个 agent 工作流 | User |
| `agent.get` | `Read` | `Unary` | 按 `run_id` 返回运行状态与报告 | `workspace.read` + run owner |

`agent.run` 使用 runtime 的 Job：返回 `Accepted { job_id }`，进度通过 `agent.progress` 与 `runtime.job.*` 推送，取消使用 `runtime.job.cancel`。`agent.progress` 携带 `run_id`；`runtime.job.status` 的结果也包含 `AgentRun`。`agent.get` 在运行期间与结束后都可读取。

`agent.stream` 是可选能力，存在时 `agent.progress` 可以携带流式文本片段；缺失时只提供阶段与进度。

## 3 运行模型

```text
AgentRunRequest {
  workflow,          // Analyze | Review | Write
  targets?,          // [DocumentPath]
  instruction?
}
```

```text
AgentGetRequest {
  run_id
}
```

- `Analyze` 与 `Review` 至少提供一个 `targets` 文档。
- `Write` 使用 `targets` 表示相关文档，使用 `instruction` 描述目标与约束。
- `provider` 与 `model` 由本地配置决定，不在请求中传递凭据。
- `agent.get` 返回 `AgentRun`。

工作流：

```text
AgentWorkflow {
  Analyze,
  Review,
  Write
}
```

| workflow | 输出 | 写入 |
| --- | --- | --- |
| `Analyze` | 分析报告 | 无 |
| `Review` | 审阅报告 | 无 |
| `Write` | 报告与提案 | 通过 `write.propose` 创建提案 |

运行：

```text
AgentRun {
  run_id,
  job_id,
  workflow,
  state,             // Queued | Running | Cancelling | Succeeded | Failed | Cancelled
  targets,
  provider?,
  model?,
  context: [AgentReference],
  progress: AgentProgress,
  report?,
  proposal_ids?,
  incomplete?,
  error?,
  started_at,
  updated_at,
  ended_at?
}

AgentProgress {
  phase,             // Context | Provider | Tool | Proposal
  done,
  total?,
  message?
}

AgentReference {
  path,
  version?,
  source_span?
}

AgentReport {
  content,           // 纯文本
  references: [AgentReference]
}
```

- `state` 与 runtime Job 状态对齐。
- `provider` 与 `model` 以 provider 实际返回为准，用于展示与溯源。
- `context` 是上下文清单，记录路径与版本；提示词与输出全文留在运行侧。
- `report` 是只读文本报告，使用纯文本内容与引用路径。
- `proposal_ids` 关联 write 域的提案。
- `incomplete` 标记取消或失败时保留的部分结果。
- `error` 使用 base 的 `ErrorInfo`。

运行结果在发起会话内保留，`agent.get` 可以重复读取；会话关闭后释放。server 可以按容量淘汰终态结果，读取已淘汰的运行返回 `agent.run_not_found`。

## 4 上下文与工具

Agent 的上下文来源：

- `targets` 指定的文档；
- 当前文档与相关文档；
- 出链与反链；
- 全文搜索与文件名查找；
- 健康报告；
- 历史摘要、变更与溯源；
- Git 元数据与差异。

上下文装配规则：

- 按 provider 的上下文上限分配预算。
- 优先级为用户选择、当前文档、健康结果、反链摘录、历史与 Git 信息。
- 超出预算时截断低优先级来源，并在报告中说明。
- 上下文清单记录路径与版本，进入 `AgentRun.context` 与写入溯源。
- 文档内容作为数据参与上下文；工具调用与写入路径由 server 的白名单和 write 域约束。

Agent 的工具与 client 查询一一对应，权限与 client 查询相同。路径统一经过 workspace 校验；工具输出受大小上限约束，超出时截断并标注。

## 5 写入与溯源

Agent 的文档修改通过 write 域提交：

- Agent 提案的 `principal` 为 `Agent`；
- 提案保持 `Pending`，由 User 审批；
- Agent 可以撤销自己发起的提案，需要 `workspace.undo` 能力；
- 冲突时由 User 决定是否重新运行 agent 生成新提案。

Agent 写入的 `Provenance` 记录：

```text
provider?,
model?,
run_id?,
context: [{ path, version }]
```

溯源保留 provider、模型、运行 id 与上下文路径版本；提示词与输出全文留在运行侧。提案与运行通过 `proposal_id` 与 `run_id` 关联。

## 6 Provider 与 MCP

Provider 是 external port：

- provider 端点、模型名与凭据引用来自配置，凭据只保存在本地；
- 未配置 provider 时，`agent.run` 返回 `provider.unavailable`；
- 远程 provider 的首次使用由 client 在运行前确认端点；
- provider 的用量信息只用于展示；
- provider 错误在 `AgentRun.error` 与 `Reply.error` 中返回稳定错误码。

MCP 由 external port 提供：

- MCP host 将文档、搜索、健康与反链映射为只读查询；
- 外部 agent 使用 `Principal = Mcp`；
- MCP 写入暂缓，开放后使用 write 域的提案、审批与归属规则；
- MCP 的 tools、resources、prompts 与 stdio 细节属于 external port。

## 7 事件与错误

`agent.progress`：

```text
agent.progress {
  run_id,
  job_id,
  progress: AgentProgress,
  delta?,
  updated_at
}
```

- scope 为 `agent(run_id)`。
- `delta` 仅在 `agent.stream` 能力可用时发送。
- Job 生命周期继续使用 `runtime.job.*`，scope 为 `job(job_id)`。
- 慢 client 的进度可以合并或丢弃；最终报告通过 `agent.get` 或 Job 结果读取。

错误：

| 错误码 | 场景 | retryable |
| --- | --- | --- |
| `agent.run_not_found` | `run_id` 不存在或已过期 | false |
| `agent.busy` | 当前会话运行数达到上限 | true |
| `provider.unavailable` | provider 未配置或不可达 | true |
| `provider.auth_failed` | provider 认证失败 | false |
| `provider.rate_limited` | provider 限流 | true |
| `provider.context_overflow` | 上下文超出模型上限 | false |
| `provider.invalid_response` | provider 响应格式非法 | false |
| `path.rejected` | 路径越界或指向内部状态 | false |
| `permission.denied` | principal 缺少能力或不是 run owner | false |
| `protocol.limit_exceeded` | 请求或结果超出限制 | false |
| `internal` | 未预期错误 | false |

## 8 与其他域的协作

| 域 | 协作方式 |
| --- | --- |
| document | 提供文档、元数据、版本与大纲；agent 读取上下文 |
| search / index | 提供搜索、出链与反链结果 |
| history | 提供历史摘要、变更与溯源；记录 agent 写入 |
| health | 提供健康报告，供审阅工作流使用 |
| write | Agent 提交提案；User 审批；Agent 可撤销自己的提案 |
| workspace | 校验路径，提供 provider 配置与工作区上下文 |
| runtime | 提供 Job、进度、取消、事件与会话生命周期 |
| provider | 外部端口，提供模型调用、流式输出与工具调用 |
| MCP | 外部端口，映射只读文档、搜索、健康与反链查询 |
| TUI / CLI | 启动运行、展示进度与报告、查看关联提案 |

跨域流程见 [domain.md](../domain.md)。
