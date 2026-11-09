# 文档索引

> 文档状态：已确认 · 最后复核：2026-11-09

> mdctrl 文档的导航入口：先读什么、文档在哪里、如何维护。

**相关入口**：[贡献指南](../CONTRIBUTING.md) · [许可证](../LICENSE)

**目录**

- [1 文档地图](#1-文档地图)
- [2 阅读路径](#2-阅读路径)
- [3 目录结构](#3-目录结构)
- [4 阶段文档](#4-阶段文档)
- [5 crate 文档](#5-crate-文档)
- [6 Agent 归档](#6-agent-归档)
- [7 索引维护](#7-索引维护)

## 1 文档地图

五份基线文档构成项目共识，其他文档在此基础上展开。

| 文档 | 内容 | 何时阅读 |
| --- | --- | --- |
| [愿景](vision.md) | 产品定位、问题、用户、能力与设计原则 | 了解项目方向 |
| [架构](architecture.md) | 系统结构、crate 边界、状态所有权、部署 | 参与设计与实现前 |
| [技术选型](techStack.md) | 技术选型、候选对比与验证状态 | 引入依赖或做技术决策前 |
| [开发模式](devMode.md) | 阶段模型、文档结构、角色与完成定义 | 开始协作前 |
| [开发计划](devPlan.md) | 五个 MVP、依赖、分工、版本号与发布条件 | 认领任务前 |

## 2 阅读路径

| 目标 | 顺序 |
| --- | --- |
| 了解项目 | [愿景](vision.md) → [架构](architecture.md) → [开发计划](devPlan.md) |
| 开始贡献 | [架构](architecture.md) → [开发模式](devMode.md) → [开发计划](devPlan.md) → 对应 `docs/MVPn/` |
| 做技术决策 | [技术选型](techStack.md) → [架构决策](architecture.md#16-架构决策) → 对应 crate 文档 |

## 3 目录结构

```text
docs/
  vision.md
  architecture.md
  techStack.md
  devMode.md
  devPlan.md
  README.md
  crates/<crate>/
  MVPn/
  archive/sessions/
```

| 路径 | 内容 |
| --- | --- |
| `docs/*.md` | 基线文档与本索引 |
| `docs/crates/<crate>/` | 长期 crate 技术文档 |
| `docs/MVPn/` | 各阶段的目标、设计、接口与测试 |
| `docs/archive/sessions/` | Agent coding session 归档 |

文档分层、目录结构与文档规则见[开发模式](devMode.md#5-文档与接口)。

## 4 阶段文档

每个 MVP 对应一个 `docs/MVPn/` 目录：

- `README.md`：阶段说明与阶段总结；
- `design/`：阶段设计，入口为 `design/README.md`；
- `interfaces.md`：本阶段的开发接口；
- `testing.md`：测试计划、执行人与证据。

目录结构、协商方式与维护规则见[开发模式](devMode.md#52-mvp-文档目录)。

## 5 crate 文档

长期技术文档位于 `docs/crates/<crate>/`，入口为 `README.md`。内部结构由 crate owner 决定。

模块边界见[架构](architecture.md#12-模块与-crate)，技术选择见[技术选型](techStack.md#14-选型结论)，主责分工见[开发计划](devPlan.md#61-crate-主责)。

## 6 Agent 归档

Agent coding 的 session 记录位于 `docs/archive/sessions/`。命名、元数据与脱敏规则见[贡献指南](../CONTRIBUTING.md#10-agent-coding-留痕)，目录说明见[session 归档](archive/sessions/README.md)。

## 7 索引维护

新增、移动或删除文档时同步更新本索引。文档与代码同 PR、信息单一位置等规则见[开发模式](devMode.md#54-文档规则)。
