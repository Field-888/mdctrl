# mdctrl-protocol 工作区

> 文档状态：草案 · 最后复核：2026-11-25

> workspace 定义工作区状态、配置契约、服务原语、事件与错误。

## 1 定位

workspace 是基础状态域，负责根目录、工作区状态、配置、身份、路径契约与扫描摘要。它对外提供 `workspace.init`、`workspace.status`、`workspace.config.get` 与 `workspace.config.set`，并发布 `workspace.config_changed` 与 `workspace.scan_updated`。

workspace 使用 [base.md](../base.md) 的标识、版本、时间、错误与通用值，使用 [runtime.md](../runtime.md) 的控制原语与会话语义。文档解析、索引、健康、写入、媒体、Agent 与插件由对应域负责。

本地状态布局、锁与端点、扫描监听、路径解析与配置解析的细节见 server 文档。

## 2 根目录与工作区状态

普通命令从起点目录向上查找最近的标记：

- `mdctrl.toml`：团队共享配置；
- `.mdctrl/`：本地状态与根标记。

第一个包含任一标记的目录即工作区根。起点可以来自 `--workspace` 参数，也可以来自当前目录或打开文件所在目录。

`workspace.init` 以执行命令时的 cwd 为根，父级工作区保持原状。嵌套工作区以最近的根为准。

| 状态 | 条件 | 行为 |
| --- | --- | --- |
| `Ready` | `mdctrl.toml` 与 `.mdctrl/` 都存在 | 正常启动 |
| `SharedOnly` | 只有 `mdctrl.toml` | 自动建立本地状态，完成后进入 `Ready` |
| `LocalOnly` | 只有 `.mdctrl/` | 使用内置默认配置启动，可执行 `workspace.init` 补齐共享配置 |
| `Migrating` | 本地 schema 较旧 | 备份后迁移 |
| `Error` | 状态损坏或 schema 较新 | 返回明确错误 |

工作区状态通过 `workspace.status` 暴露。`SharedOnly` 自动建立本地状态并进入 `Ready`；`LocalOnly` 使用默认配置运行；`Migrating` 与 `Error` 通过错误码与状态字段表达。

根目录可以是 Git 仓库根、仓库子目录，也可以位于 Git 仓库之外；Git 根由 history 探测。起点与根目录都做规范化，符号链接起点解析到真实目录。

## 3 工作区身份与路径

`WorkspaceId` 由 server 分配，按不透明字符串精确比较；同一工作区在本机的 server 重启与 embedded/daemon 切换中保持稳定，新克隆、新机器或重建本地状态生成新 ID。它用于会话与缓存的工作区身份校验；路径与权限由 workspace 校验承担。`WorkspaceSchemaVersion` 表达工作区状态结构版本；schema 较新时返回 `workspace.schema_unsupported`。

runtime 握手的 `WorkspaceInfo` 使用 `WorkspaceId` 与 `WorkspaceSchemaVersion`。`workspace.status` 同时返回这两个值。

协议使用工作区相对 `DocumentPath`；绝对路径在 server 内部解析。workspace 负责路径的最终校验，越界或指向内部状态时返回 `path.rejected`。`DocumentPath` 的合法形式由 base 定义。

## 4 配置契约

配置分共享层与本地层：

| 层 | 作用 |
| --- | --- |
| 共享 | 团队约定的工作区、索引、健康、插件与 Agent 设置 |
| 本地 | 本机环境、渲染器路径、provider 端点与凭据引用 |

优先级从低到高：内置默认 → 共享层 → 本地层 → 启动参数。

规则：

- 共享层拥有团队语义：`[workspace]` 的 include/exclude、`[health]` 规则、`[index]` 分词与权重、`[plugins]` 声明与锁。
- 本地层覆盖环境相关键：媒体渲染器路径、provider 端点、Git 可执行文件路径、日志级别与凭据引用。
- 凭据放在本地配置或系统凭据存储；共享层出现疑似凭据时给出警告。
- 配置解析或类型错误返回 `workspace.config_invalid`，携带文件、行号与键路径。
- 未知键记录警告并继续，保留向前兼容。
- 运行中配置变化先校验；有效则发布 `workspace.config_changed`，由各域决定是否重建；无效则保留旧配置并报告。

`workspace.config.get` 返回有效配置、来源层与每层指纹；凭据只返回引用名。

`workspace.config.set` 是 CLI 与 TUI 共用的配置修改入口：

- `layer` 显式指定 `Shared` 或 `Local`。
- `changes` 使用 `Set` 与 `Unset` 表达单个键的修改；数组与表按整体替换。
- `dry_run` 完成解析、校验与差异生成，返回结果并保留文件。
- `base_fingerprint` 用于乐观并发；省略时按当前文件写入，与当前文件指纹不同时返回 `workspace.config_conflict`。
- 本地层写入团队语义键返回 `workspace.config_forbidden`；共享层写入凭据返回 `workspace.config_forbidden`。
- 写入采用全量校验与原子替换；写后重新加载失败时恢复原内容并返回错误。
- 成功后发布 `workspace.config_changed`，client 重新读取配置。

## 5 服务原语

| 原语 | access | execution | 作用 | policy |
| --- | --- | --- | --- | --- |
| `workspace.init` | `Write` | `Unary` | 在 cwd 初始化或补齐工作区 | User |
| `workspace.status` | `Read` | `Unary` | 返回工作区状态、配置摘要、扫描摘要与锁信息 | `workspace.read` |
| `workspace.config.get` | `Read` | `Unary` | 返回有效配置、来源层与指纹 | User |
| `workspace.config.set` | `Write` | `Unary` | 修改共享层或本地层配置 | User |

`workspace.init` 保持幂等，保留已有的共享配置与本地状态，并在 Git 仓库中确保 `.mdctrl/` 被忽略。它返回创建项、跳过项与警告：

```text
WorkspaceInitResult {
  workspace_id,
  schema_version,
  root_label,
  created,
  skipped,
  warnings
}
```

`workspace.status` 返回：

```text
WorkspaceStatus {
  workspace_id,
  schema_version,
  state,
  display_name,
  config_summary: { sections, warnings },
  scan_summary,
  lock?
}
```

`ScanSummary` 描述扫描状态与覆盖完整性：

```text
ScanSummary {
  state,              // Scanning | Ready | Incomplete | Error
  complete,
  documents,
  failed_regions: [{ path, reason }],
  updated_at
}
```

`LockInfo` 描述当前工作区持有者：

```text
LockInfo {
  pid,
  mode,               // Embedded | Daemon
  started_at,
  protocol_version
}
```

`workspace.config.get` 返回：

```text
ConfigView {
  effective,
  layers: [{ layer, fingerprint, values }],
  warnings
}
```

`workspace.config.set` 请求与结果：

```text
ConfigSetRequest {
  layer: Shared | Local,
  changes: [ConfigChange],
  base_fingerprint?,
  dry_run?
}

ConfigChange {
  path,
  op: Set | Unset,
  value?
}

ConfigSetResult {
  layer,
  changed_paths,
  fingerprint,
  diff?,
  warnings
}
```

`Set` 携带 `value`；`Unset` 省略 `value`。`path` 使用点分隔的 TOML 键路径；同一请求内同一路径只出现一次。`workspace.status` 的配置摘要与锁信息用于诊断；`workspace.config.set` 的校验、冲突与差异语义在第 4 节定义。

## 6 事件与错误

事件：

```text
workspace.config_changed   { layer, sections, changed_paths, effective_at }
workspace.scan_updated     { complete, documents, failed_regions, updated_at }
```

`workspace.config_changed` 在配置写入成功或运行中配置变更生效后发布；`workspace.scan_updated` 在扫描完成或覆盖状态变化后发布。client 收到事件后按需重新读取状态或配置。

错误：

| 错误码 | 场景 | retryable |
| --- | --- | --- |
| `workspace.not_found` | 未找到根标记 | false |
| `workspace.locked` | 锁被其他 server 持有且端点缺失 | true |
| `workspace.config_invalid` | 配置解析或类型错误 | false |
| `workspace.config_conflict` | `base_fingerprint` 与当前文件不同 | true |
| `workspace.config_forbidden` | 本地层写入团队键，或共享层写入凭据 | false |
| `workspace.schema_unsupported` | 本地 schema 较新 | false |
| `workspace.schema_migration_failed` | 迁移失败 | false |
| `workspace.state_invalid` | 工作区状态损坏 | false |
| `workspace.not_writable` | 工作区状态或配置写入失败 | false |
| `path.rejected` | 路径越界或指向内部状态 | false |
| `document.not_found` | 文档缺失 | false |
| `permission.denied` | principal 缺少能力 | false |
| `internal` | 未预期错误 | false |

`workspace.locked` 的错误详情携带 `LockInfo`。

## 7 相关域

- workspace 向 document、history、index、health 提供文档清单、扫描摘要、变化流与路径校验。
- write 使用 workspace 的路径校验与原子写语义；提案、差异、审批与版本检查由 write 域定义。
- media 通过 workspace 校验并读取本地资源；渲染、缓存与资源句柄由 media 域定义。
- agent 与 plugin 通过服务原语访问文档与配置；路径由 workspace 校验。
- Git 根与元数据由 history 提供；workspace 扫描排除 `.git/`。
- 跨域流程见 [domain.md](../domain.md)。
