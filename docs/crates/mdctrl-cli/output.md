# 输出

> 文档状态：草案 · 最后复核：2026-11-14

> 人类读 stdout，脚本读 JSON，诊断读 stderr：三者互不污染。

**相关文档**：[mdctrl-cli](README.md) · [命令组合](commands.md) · [数据命令](data-commands.md) · [开发模式](../../devMode.md#54-文档规则)

## 1 输出通道

| 通道 | 内容 | 消费者 |
| --- | --- | --- |
| stdout | 命令结果 | 用户、脚本、管道 |
| stderr | 进度、警告、诊断、调试日志 | 用户、CI 日志 |
| 退出码 | 结果分类 | 脚本、CI |

规则：

- stdout 只放结果；`--json-stream` 时结构化进度属于结果流，其余模式进度写入 stderr；
- `--json` 时 stdout 是单个信封，`--json-stream` 时是 NDJSON 流，均不混入其他文本；
- 默认人类可读模式 stdout 只放人类可读结果，不混入 JSON 或日志；
- stderr 不使用颜色以外的控制序列；
- 管道断开（SIGPIPE）时停止写入并正常退出；
- 输出内容不包含密钥、令牌、私有地址与个人路径。

## 2 输出模式

| 模式 | 触发 | 特点 |
| --- | --- | --- |
| 人类可读 | 默认且 stdout 是 TTY | 颜色、对齐、分组、摘要 |
| 纯文本 | stdout 不是 TTY | 无色、无进度、稳定分隔 |
| 结构化 | `--json` | 固定字段、机器可解析 |
| 静默 | `--quiet` | 仅输出结果与错误 |

模式选择在解析阶段确定，执行期间不切换。`--no-color` 只关闭颜色，不改变模式。

## 3 人类可读输出

- 使用终端宽度对齐，宽字符按显示宽度计算；
- 分组标题与缩进保持层级清晰；
- 路径默认显示相对工作区根；
- 长列表提供总数与截断提示；
- 警告与错误使用符号与文字，不只依赖颜色；
- 空结果给出明确的“无结果”与建议范围。

示例：

```text
docs/architecture.md:120  transport
docs/architecture.md:131  daemon
docs/devPlan.md:88        传输
3 results (limit 20)
```

## 4 结构化输出

### 4.1 信封

单次命令输出一个 JSON 对象：

```json
{
  "schema_version": 1,
  "command": "status",
  "ok": true,
  "data": {},
  "error": null,
  "warnings": []
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `schema_version` | integer | 结构版本，字段不兼容时递增 |
| `command` | string | 规范命令名 |
| `ok` | boolean | 命令是否成功执行，不表示检查结论 |
| `data` | object/array/null | 命令结果 |
| `error` | object/null | 失败详情 |
| `warnings` | array | 非阻断提示 |

`ok` 只表示执行是否完成：`check` 执行成功时 `ok=true`、`error=null`；达到 `--fail-on` 阈值返回 5，否则返回 0。检查结论放在 `data` 中；`ok=false` 时必有 `error`。

### 4.2 错误对象

```json
{
  "kind": "workspace_not_initialized",
  "message": "workspace is not initialized",
  "hint": "run `mdctrl init`"
}
```

- `kind` 是稳定标识，供脚本分支；
- `message` 面向人类，可本地化但不保证稳定；
- `hint` 可选，提供下一步操作；
- 错误对象不包含堆栈；调试堆栈通过 `--verbose` 写入 stderr。

### 4.3 流式输出

流式输出需要显式 `--json-stream`，默认 `--json` 永远输出单个信封：

- `--json-stream` 仅适用于 `index update` 与 `index rebuild`，其他命令返回用法错误（退出码 2）；
- stdout 是 NDJSON，每行一个对象，包含 `schema_version`、`command`、`type`、`data`、`error`；
- `type` 取值为 `progress`、`item`、`summary`、`error`；
- 最后一行必须是 `summary` 或 `error`，不得省略；
- 流式输出仍遵守退出码语义；
- 未使用 `--json-stream` 时不输出 NDJSON；进度写入 stderr，结果写入 stdout（`--json` 时为单个信封）。

## 5 字段稳定性

| 类别 | 稳定性 |
| --- | --- |
| `schema_version`、`command`、`ok`、`error.kind` | 稳定 |
| `data` 中的核心字段 | 稳定，新增字段向后兼容 |
| 人类可读文案 | 不承诺稳定 |
| 进度文本 | 不承诺稳定 |
| 调试字段 | 不稳定，仅 `--verbose` |

字段变更规则：

- 新增可选字段递增次版本，不递增 `schema_version`；
- 删除、改名或类型变化递增 `schema_version`；
- 文档与测试同步更新；
- 破坏性变更在 PR 中说明迁移方式。

## 6 结构化数据来源

- 字段由协议类型序列化而来，CLI 不重新命名；
- 时间使用 RFC 3339 UTC；
- 路径同时提供相对路径，必要时提供绝对路径；
- 行号与列号从 1 开始；
- 严重度使用固定枚举：`error`、`warning`、`info`；
- 计数使用整数，不使用本地化数字格式。

## 7 颜色与终端

- 颜色能力按 `TERM`、`COLORTERM` 与 TTY 检测；
- `NO_COLOR` 存在时禁用颜色；
- `--no-color` 优先于自动检测；
- 16 色终端使用基础色映射，保持错误与警告可区分；
- 不使用背景色表达关键信息，避免浅色终端不可读。

## 8 退出码细节

| 退出码 | 触发条件 | stdout |
| --- | --- | --- |
| 0 | 成功 | 结果 |
| 1 | 运行时失败 | 空或部分结果 |
| 2 | 用法错误 | 空 |
| 3 | 工作区错误 | 空 |
| 4 | 数据不可用 | 空或状态对象 |
| 5 | `check` 达到 `--fail-on` 阈值 | 完整体检结果 |
| 130 | 被中断 | 可能部分结果 |

规则：

- `--json` 失败时仍输出信封，`ok=false`；
- 用法错误在解析阶段发生，通常不输出 JSON；
- `ok=true` 对应退出码 0 或 5；`ok=false` 对应 1、2、3、4 或 130；
- `check` 的检查结论只放在 `data`，不通过 `ok` 或 `error` 表达；
- CI 依据退出码分支，不解析人类可读文本。

## 9 CI 与诊断

- 警告写入 stderr，不改变退出码；
- `--verbose` 输出连接、工作区解析与请求耗时；
- 诊断信息避免输出完整文档内容；
- 颜色在 CI 中默认关闭；
- 结构化输出可直接作为报告输入。

## 10 测试关注点

- TTY 与非 TTY 的输出差异；
- `--json` 信封字段与 `schema_version`；
- 错误 `kind` 与退出码映射；
- 宽字符对齐与截断；
- SIGPIPE 行为；
- 人类可读输出的快照稳定性边界。
