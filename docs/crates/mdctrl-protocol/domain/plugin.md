# mdctrl-protocol 插件

> 文档状态：草案 · 最后复核：2026-11-27

> plugin 定义插件声明、能力、命令、状态、插件视图与写入协作。

## 1 定位

plugin 是写入与扩展的派生域，负责插件声明、能力授权、命令注册、状态管理、插件视图与写入提案协作。插件通过 external port 接入 server，以独立进程运行；宿主按 manifest 与授予能力校验服务原语调用。

插件贡献的命令注册为服务原语，与内置命令进入同一目录。插件写入通过 write 域提案，审批由 User 完成。进程隔离服务于故障隔离，接口权限约束插件通过宿主发出的请求；插件代码以当前用户权限运行，安装即信任。

plugin 使用 [base.md](../base.md) 的 `Value`、`Timestamp`、`PageInfo` 与错误模型。插件视图由 plugin 域定义，client 在插件面板显示；文档写入与撤销由 [write.md](write.md) 负责；`[plugins]` 声明与能力记录属于 workspace 配置。

插件进程管理、进程通信、进程监管与 manifest 安装细节见 server 与 external port 文档。

## 2 服务原语

管理原语：

| 原语 | access | execution | 作用 | policy |
| --- | --- | --- | --- | --- |
| `plugin.list` | `Read` | `Unary` | 返回插件摘要列表 | `workspace.read` |
| `plugin.install` | `Write` | `Unary` | 安装插件并授权能力 | User |
| `plugin.remove` | `Write` | `Unary` | 移除插件 | User |
| `plugin.doctor` | `Read` | `Unary` | 返回插件状态、错误与诊断 | User |
| `plugin.enable` | `Write` | `Unary` | 重新启用已禁用插件 | User |
| `plugin.disable` | `Write` | `Unary` | 停用插件 | User |

插件贡献的命令以服务原语进入目录：

```text
plugin.<plugin_id>.<command>
```

管理原语使用 `plugin.list`、`plugin.install` 等保留名；插件 id 与命令名避开这些保留字。插件进入 `Ready` 后，`runtime.catalog` 返回其注册命令；client 通过 `plugin.status` 获知状态变化并重新读取目录。

`plugin.management` 是可选 client 能力，存在时 client 可以调用安装、移除、启用与停用原语；缺失时 client 读取插件列表与运行状态。

## 3 Manifest 与能力

```text
PluginManifest {
  id,
  name?,
  description?,
  version,
  protocol,
  entry,
  capabilities: [PluginCapability],
  contributes: [PluginContribution]
}

PluginContribution {
  kind,              // Command | View | Exporter
  id,
  access?,           // Read | Write
  description?
}
```

```text
PluginCapability {
  workspace.read,
  workspace.propose,
  workspace.undo,
  command.invoke,
  network.access
}
```

- `workspace.read`：读取文档、搜索、反链、健康、历史与媒体资源等只读查询。
- `workspace.propose`：提交写入提案。
- `workspace.undo`：撤销自己发起的已执行提案。
- `command.invoke`：通过命令总线调用其他命令。
- `network.access`：声明插件需要访问网络。

授予能力 = manifest 申请 ∩ 用户批准。`plugin.install` 请求携带用户批准的能力集合；`plugin.list` 返回申请能力与授予能力。

## 4 生命周期与状态

```text
PluginState {
  Declared,
  Starting,
  Handshaking,
  Ready,
  Failed,
  Disabled,
  Stopping,
  Stopped
}
```

- `Declared`：已声明，尚未启动。
- `Starting` / `Handshaking`：进程启动与协议握手。
- `Ready`：可用，命令已注册。
- `Failed`：崩溃、超时、非法消息或协议不兼容。
- `Disabled`：重启次数超限或用户停用。
- `Stopping` / `Stopped`：正常关闭。

`plugin.status` 事件：

```text
plugin.status {
  plugin_id,
  state,
  version?,
  requested_capabilities?,
  granted_capabilities?,
  error?,
  updated_at
}
```

- scope 为 `plugin(plugin_id)`。
- 状态变化、能力授权变化、命令注册变化与失败都发布该事件。
- client 收到后重新读取 `plugin.list`、`plugin.doctor` 或 `runtime.catalog`。

## 5 插件命令与视图

插件命令是服务原语，输入输出使用插件声明的结构：

```text
PluginCommandRequest {
  input: Value
}

PluginCommandResult {
  data?: Value,
  view?: PluginView
}

PluginView {
  title?,
  content            // 纯文本
}
```

- `PluginCommandRequest` 与 `PluginCommandResult` 由 manifest 声明，wire 使用 `Value` 承载插件自定义载荷。
- `PluginView.content` 使用纯文本；宿主清理控制字符与终端转义序列。
- 宿主校验文本大小与格式；插件视图中的工作区路径按 workspace 规则校验。
- client 在插件面板显示 `PluginView`；文档 View 保持文档专属。
- 插件命令的 `access` 由 manifest 声明；`Write` 命令通过 `workspace.propose` 修改文档。

## 6 写入与溯源

- 插件通过 `workspace.propose` 提交提案，`principal` 为 `Plugin`。
- 提案保持 `Pending`，由 User 审批。
- 插件撤回 `Pending` 提案使用 `workspace.propose`；撤销已执行提案使用 `workspace.undo`。
- `Provenance` 记录 `plugin_id`、`plugin_version`、`plugin_command`、`proposal_id` 与 `undo_of`。
- 插件直接写文件时，变化由 workspace 作为外部变化观察并进入 history。

## 7 管理操作

```text
PluginListReply {
  plugins: [PluginSummary]
}

PluginSummary {
  id,
  name?,
  version,
  state,
  requested_capabilities,
  granted_capabilities,
  source?,
  last_error?
}

PluginInstallRequest {
  source,
  version?,
  granted: [PluginCapability]
}

PluginRemoveRequest {
  plugin_id
}

PluginDoctorRequest {
  plugin_id
}

PluginDoctorReport {
  plugin_id,
  state,
  manifest?,
  last_error?,
  restart_count,
  stderr_summary?
}
```

`plugin.install` 校验 manifest、版本、来源与校验和，更新 `mdctrl.toml` 的插件声明与锁；`plugin.remove` 停止进程并移除声明与锁。插件声明属于共享配置，由 User 显式操作。

## 8 错误

| 错误码 | 场景 | retryable |
| --- | --- | --- |
| `plugin.not_found` | 插件 id 不存在 | false |
| `plugin.invalid_manifest` | manifest 非法或注册项越界 | false |
| `plugin.unsupported_version` | 插件协议版本不兼容 | false |
| `plugin.disabled` | 插件处于禁用状态 | false |
| `plugin.failed` | 插件崩溃、启动失败或请求失败 | true |
| `plugin.timeout` | 插件请求超时 | true |
| `plugin.invalid_message` | 插件返回非法消息或非法视图 | false |
| `plugin.install_failed` | 安装失败 | false |
| `plugin.remove_failed` | 移除失败 | false |
| `permission.denied` | 能力不足或调用者无权 | false |
| `protocol.limit_exceeded` | 消息、视图或输出超出限制 | false |
| `internal` | 未预期错误 | false |

## 9 与其他域的协作

| 域 | 协作方式 |
| --- | --- |
| workspace | 提供 `[plugins]` 声明、路径校验、能力批准记录与配置变更 |
| write | 插件提案进入审批、写入与撤销；记录插件溯源 |
| document / search / index / history / health | 插件通过 `workspace.read` 使用只读服务原语 |
| runtime | 提供服务原语目录、事件、会话与插件管理能力 |
| view | 宿主清理 `PluginView` 文本；client 在插件面板显示，文档 View 保持文档专属 |
| TUI / CLI | 展示插件列表、能力、状态与命令；安装与移除由 User 操作 |
| external port | 管理插件进程、进程通信、manifest 与故障隔离 |

跨域流程见 [domain.md](../domain.md)。
