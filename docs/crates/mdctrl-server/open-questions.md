# 待确认事项

> 文档状态：草案 · 最后复核：2026-11-22

> 本文只做索引：每个问题的完整描述、背景与建议写在对应主题文档的“待确认事项”一节，这里按确认人汇总，便于协作者集中处理。

**相关文档**：[server 总览](README.md) · [开发模式 7.2 决策规则](../../devMode.md#72-决策规则)

**目录**

- [1 使用方式](#1-使用方式)
- [2 需要三人共识](#2-需要三人共识)
- [3 需要 field 确认](#3-需要-field-确认)
- [4 需要 silkage15 确认](#4-需要-silkage15-确认)
- [5 ylgd0915 自行决定](#5-ylgd0915-自行决定)
- [6 与基线文档的冲突与缺口](#6-与基线文档的冲突与缺口)
- [7 对接点](#7-对接点)

## 1 使用方式

- 决策规则沿用[开发模式 7.2](../../devMode.md#72-决策规则)：跨模块接口由提供方与使用方共同确认；阶段范围、接口或重大决策需要三人共识；
- 问题编号前缀对应文档：SRV（README）、WS（workspace）、HIS（history）、IDX（index）、HLT（health）、GIT（git）、WR（write-path）、AGT（agent-mcp）、MED（media）、PLG（plugin）、TRN（transport）；
- 表中记录尚待确认的设计和对接进度。双方确认后，把结论写回主题文档，并在此标注日期。
- 确认后如涉及协议或公共 API 变更，按 [AGENTS.md](../../../AGENTS.md) 获得人工确认后再进入实现。

## 2 需要三人共识

| 编号 | 问题 | 起草者建议 |
| --- | --- | --- |
| [WS-1](workspace.md#13-待确认事项) | 根目录识别是否同时接受 `mdctrl.toml`；新克隆时是否自动完成本地初始化 | 接受；自动完成本地部分 |
| [WS-2](workspace.md#13-待确认事项) | 无标记目录中运行的行为 | 报错并提示 `init` |
| [WS-3](workspace.md#13-待确认事项) | embedded 持锁时其他进程的行为 | MVP1 报错提示；daemon 设计时评估 embedded 兼听 IPC |
| [WS-5](workspace.md#13-待确认事项) | `.mdctrl/` 新增 `server.json`、`schema`、`cache/tmp/`、`logs/` | 新增 |
| [SRV-2](README.md#12-待确认事项) | server 公共 API 的入口范围 | 三个入口，MVP1 只实现 embedded |
| [SRV-3](README.md#12-待确认事项) | 异步运行时的引入时机与范围 | MVP2 随 agent/MCP 引入 |
| [HIS-2](history.md#12-待确认事项) | 内容指纹哈希算法与依赖 | 待提出具体依赖后评审 |
| [HIS-6](history.md#12-待确认事项) | 历史保留策略默认值 | 默认不清理 |
| [HLT-2](health.md#10-待确认事项) | `stale` 是否支持正文头部的复核日期 | 支持可配置模式 |
| [GIT-3](git.md#9-待确认事项) | 作者邮箱的展示与进入 agent 上下文 | 默认只用姓名 |
| [WR-1](write-path.md#12-待确认事项) | 人工保存是否“自批准” | 是 |
| [WR-5](write-path.md#12-待确认事项) | 删除与重命名是否纳入提案 | MVP3 不纳入 |
| [WR-6](write-path.md#12-待确认事项) | 差异算法自研或引入依赖 | 待评估 |
| [WR-7](write-path.md#12-待确认事项) | 三方合并是否进入后续规划 | 后续规划 |
| [WR-9](write-path.md#12-待确认事项) | 可选的正文溯源标注 | 默认关闭 |
| [AGT-1](agent-mcp.md#15-待确认事项) | provider 端点能否写入共享配置 | 待讨论 |
| [AGT-2](agent-mcp.md#15-待确认事项) | 远程 provider 数据外发确认方式 | 每工作区首次确认 |
| [MED-4](media.md#14-待确认事项) | 远程图片下载开关 | 默认关闭 |
| [PLG-3](plugin.md#14-待确认事项) | 插件能力批准记录的位置 | 待讨论 |
| [PLG-4](plugin.md#14-待确认事项) | 操作系统级沙箱是否进入后续规划 | 进入后续研究，当前不承诺 |
| [TRN-1](transport.md#15-待确认事项) | daemon 与 `serve` 进入哪个 MVP | 三人决定后同步 devPlan 与 PR #5 |

## 3 需要 field 确认

`field` 负责 protocol、core、render 与 media，同时是本任务的测试执行人。

| 编号 | 问题 |
| --- | --- |
| [SRV-1](README.md#12-待确认事项) | server 错误类别是否作为 protocol 错误类型的基础 |
| [WS-8](workspace.md#13-待确认事项) | protocol 中文档路径的表示与非 UTF-8 路径 |
| [HIS-1](history.md#12-待确认事项) | core 文档身份是否只用路径；身份延续（重命名）在 core 模型中如何表达 |
| [IDX-1](index.md#12-待确认事项) | render 提供给索引的抽取接口与锚点规则 |
| [IDX-2](index.md#12-待确认事项) | 索引版本的暴露与命名，与请求代次 `generation`、文档版本区分（与 `silkage15`） |
| [IDX-5](index.md#12-待确认事项) | 中文两字查询验收阈值与验证执行 |
| [HLT-5](health.md#10-待确认事项) | 健康结果中的数据来源说明（与 `silkage15`） |
| [HLT-8](health.md#10-待确认事项) | 不完整检查的报告字段与 CLI 退出行为（与 `silkage15`） |
| [WR-8](write-path.md#12-待确认事项) | 换行风格与 BOM 的保留（与 `silkage15`） |
| [MED-1](media.md#14-待确认事项) | SVG 转位图在 server 还是 client |
| [MED-2](media.md#14-待确认事项) | 占位尺寸策略（与 `silkage15`） |
| [MED-3](media.md#14-待确认事项) | MathJax 进程模型 |
| [MED-5](media.md#14-待确认事项) | 媒体模块是否感知终端能力 |
| [PLG-7](plugin.md#14-待确认事项) | 结构化视图节点集合（与 `silkage15`） |
| [TRN-3](transport.md#15-待确认事项) | protocol 是否定义取消消息（与 `silkage15`） |
| [TRN-4](transport.md#15-待确认事项) | 溢出通知、按运行 id 领取最终报告与接收确认（与 `silkage15`） |
| [TRN-6](transport.md#15-待确认事项) | `exact` 版本不符时的响应形式（与 `silkage15`） |
| — | 审阅 [media.md](media.md) 全文并接管维护（含本地图片缓存键与依赖失效，§7、§9） |

## 4 需要 silkage15 确认

`silkage15` 负责 TUI、CLI、plugin host 与 transport。

| 编号 | 问题 |
| --- | --- |
| [WS-7](workspace.md#13-待确认事项) | 被忽略的 Markdown 是否允许配置为仍然索引 |
| [WS-10](workspace.md#13-待确认事项) | 重复初始化保留已有共享配置的建议已发至 [PR #5](https://github.com/Field-888/mdctrl/pull/5#issuecomment-7077126178)，待客户端确认 |
| [IDX-3](index.md#12-待确认事项) | 索引状态拆为 `state` + `task`（修改 PR #5 草案） |
| [IDX-4](index.md#12-待确认事项) | 索引缺失时查询的行为 |
| [IDX-7](index.md#12-待确认事项) | 索引状态是否标注覆盖不完整与失败区域 |
| [HLT-1](health.md#10-待确认事项) | 采用 PR #5 的四个规则 id 与默认严重度 |
| [HLT-4](health.md#10-待确认事项) | 健康结果是否缓存 |
| [HLT-6](health.md#10-待确认事项) | 严重度过滤位置 |
| [HLT-7](health.md#10-待确认事项) | frontmatter 解析失败的诊断形式 |
| [HLT-8](health.md#10-待确认事项) | CLI 与 TUI 怎样展示不完整检查，以及机器可读的退出状态 |
| [MED-1](media.md#14-待确认事项) | 若采用 server 转位图，客户端补充像素尺寸协商与缺省降级 |
| [TRN-4](transport.md#15-待确认事项) | 重新同步后，发起会话查询运行结果及确认接收 |
| [WR-2](write-path.md#12-待确认事项) | TUI 审批界面所需的差异数据形状 |
| [WR-3](write-path.md#12-待确认事项) | 待审批提案的持久化与会话断开处理 |
| [AGT-5](agent-mcp.md#15-待确认事项) | MCP 入口的 CLI 子命令形态 |
| [PLG-1](plugin.md#14-待确认事项) | manifest 格式与字段 |
| [PLG-2](plugin.md#14-待确认事项) | 插件安装位置 |
| [PLG-5](plugin.md#14-待确认事项) | 插件 stdio 分帧 |
| [PLG-6](plugin.md#14-待确认事项) | 插件 I/O 线程模型 |
| [TRN-2](transport.md#15-待确认事项) | `.mdctrl/sessions/` 持久化内容 |
| [TRN-5](transport.md#15-待确认事项) | daemon 本地 IPC 实现与依赖 |
| [TRN-6](transport.md#15-待确认事项) | `exact` 版本不符时的响应形式（与 `field`） |
| [TRN-7](transport.md#15-待确认事项) | daemon 排空超时后强制终止的交互 |
| — | 审阅 [plugin.md](plugin.md)、[transport.md](transport.md) 全文并接管维护（含请求代次、文档版本、索引版本的区分，transport §6） |

## 5 ylgd0915 自行决定

模块内部事项，由 owner 决定后写回对应文档：SRV-4、WS-4、WS-6、HIS-3、HIS-4、HIS-5、IDX-6、HLT-3、GIT-1、GIT-2、GIT-4、GIT-5、WR-4、AGT-3、AGT-4、AGT-6。

需要先做技术验证、再由 owner 决定的事项：WS-9（内部目录别名：Windows 短文件名、硬链接）、WR-10（替换时保留或交换被替换文件的平台机制，新建文件不覆盖的落位方式）。
另见 [WR-11](write-path.md#12-待确认事项)：写入全过程使用同一解析目标与父目录的跨平台验证。

## 6 与基线文档的冲突与缺口

起草过程中发现的、需要在基线文档层面处理的问题。本任务不修改基线文档，只在此记录：

| 问题 | 涉及 | 详见 |
| --- | --- | --- |
| `.mdctrl` 作为唯一根标记，但被 Git 忽略，新克隆无法识别工作区 | [架构 6.2](../../architecture.md#62-mdctrl) | WS-1 |
| embedded 形态持锁后，其他进程既不能获得锁也不能连接 | [架构 14](../../architecture.md#14-部署形态) | WS-3 |
| “写入经过审批”对人工保存的适用方式未说明 | [架构 7.1](../../architecture.md#71-写入路径)、[架构 15](../../architecture.md#15-基本约束与安全) | WR-1 |
| daemon 未出现在 devPlan 的任何 MVP 范围中 | [devPlan 4](../../devPlan.md#4-五个-mvp) | TRN-1 |
| `.mdctrl/` 目录列表未包含日志目录，而技术选型要求日志写入本地状态目录 | [架构 6.2](../../architecture.md#62-mdctrl)、[技术选型 3.2](../../techStack.md#32-并发与错误) | WS-5 |
| “写入带版本检查”在外部进程并发修改时无法做到原子的比较并写入，存在残余窗口 | [架构 16](../../architecture.md#16-架构决策) | [write-path 6](write-path.md#6-版本检查与冲突)、WR-10 |
| Issue #3 把交付物限定在 `docs/crates/mdctrl-server/`，而贡献指南要求 session 记录放在 `docs/archive/sessions/` | [CONTRIBUTING 10](../../../CONTRIBUTING.md#10-agent-coding-留痕) | 需在 PR 中说明处理方式 |

## 7 对接点

本文与其他 Issue 的接口关系。对方文档尚未合并时，只记录本方的需求与假设。

| 对方 | 对接内容 | 本方文档 | 状态 |
| --- | --- | --- | --- |
| Issue #2 foundation（core） | 工作区模型、文档身份、历史模型、生命周期字段、配置模型、总线与注册 | [workspace](workspace.md)、[history](history.md)、[README 3.3](README.md#33-注册与分发) | foundation 文档尚无 PR，本方按基线假设 |
| Issue #2 foundation（protocol） | 错误类别、路径表示、索引状态、健康结果、提案与差异、取消、重新同步事件 | 多篇，见第 3 节 | 待协商 |
| Issue #2 foundation（render） | 索引抽取接口、锚点规则、媒体节点、占位尺寸、行内公式文本近似 | [index](index.md)、[media](media.md) | 待协商 |
| Issue #4 / PR #5 clients（TUI） | 搜索、反链、跳转、健康面板、View、媒体占位与审批 | [index](index.md)、[health](health.md)、[media](media.md)、[write-path](write-path.md) | 已发起评审；健康覆盖、索引状态、媒体尺寸与 View 响应待双方确认 |
| Issue #4 / PR #5 clients（CLI） | 重复初始化、检查覆盖、索引状态与退出码 | [workspace](workspace.md)、[health](health.md)、[index](index.md)、[错误模型](README.md#9-错误模型) | 见 WS-10、HLT-8、IDX-3 与 SRV-1；PR #5 的修改由客户端负责人确认 |
