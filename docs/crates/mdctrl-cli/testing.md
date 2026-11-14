# 测试

> 文档状态：草案 · 最后复核：2026-11-14

> CLI 的契约是参数、输出与退出码：三者都必须被测试固定下来。

**相关文档**：[mdctrl-cli](README.md) · [命令组合](commands.md) · [输出](output.md) · [贡献指南](../../../CONTRIBUTING.md#5-本地检查)

## 1 测试层次

| 层次 | 对象 | 工具 |
| --- | --- | --- |
| 单元 | 参数解析、错误映射、格式化 | `cargo test` |
| 快照 | 帮助文本、人类可读输出、JSON 信封 | `insta` |
| 属性 | 退出码映射、路径规范化 | `proptest` |
| 集成 | 真实进程 + embedded server | `assert_cmd` |
| 契约 | 数据命令字段与排序 | 固定夹具 |
| 示例 | 文档中的命令示例 | 脚本校验 |

## 2 进程级测试

- 使用 `assert_cmd` 启动真实二进制；
- 显式设置工作目录与 `--workspace`；
- 清理环境变量，尤其是颜色与工作区相关变量；
- 不依赖用户主目录与网络；
- 临时目录使用 `tempfile`，测试结束自动清理；
- 断言 stdout、stderr 与退出码三者。

```rust
Command::cargo_bin("mdctrl")
    .arg("--version")
    .assert()
    .success()
    .stdout(predicate::str::starts_with("mdctrl "));
```

## 3 快照测试

- 帮助文本按命令生成快照，防止文档漂移；
- 人类可读输出在固定宽度下快照；
- JSON 输出使用结构化快照，避免字段顺序噪声；
- 快照变更必须在 PR 中说明原因；
- 时间、路径与版本等易变字段在快照前归一化。

## 4 契约测试

- `check` 与 `index` 的 JSON 信封字段与类型；
- `schema_version` 与命令名；
- 退出码与 `ok` 字段一致；
- 排序规则：严重度降序、路径与行号升序；
- `findings` 截断时 `summary` 计数完整；
- 错误对象的 `kind` 与退出码对应。

夹具以 JSON 文件保存，覆盖空结果、单条、多条、截断与错误。

## 5 工作区夹具

| 夹具 | 用途 |
| --- | --- |
| 最小工作区 | `mdctrl.toml` + 两篇互链文档 |
| 健康问题工作区 | 断链、孤儿、过期、缺失元数据各一例 |
| 中文工作区 | 中英混排、全角标点、长文件名 |
| 未初始化目录 | 工作区错误与提示 |
| 损坏配置 | 配置解析失败与退出码 |
| Git 与无 Git | 元数据降级 |

夹具初始化后不依赖 Git；需要 Git 的用例在测试内创建仓库。

## 6 不变量测试

- 同一输入在 TTY 与非 TTY 下退出码一致；
- `--json` 时 stdout 可被 JSON 解析器完整解析；
- `--quiet` 不改变退出码与结果字段；
- `--no-color` 不改变文本内容；
- 未知命令始终返回 2；
- 路径越界始终返回 3 或 4，且不读取越界文件；
- 中断返回 130，且不留下锁或临时文件。

## 7 集成测试

- 启动 embedded server 执行 `status`、`search`、`find`；
- `index update` 后 `search` 能命中新增文档；
- `check` 在健康问题夹具上返回 5 与预期清单；
- `serve` 启动后 `status` 可连接，关闭后释放锁；
- 并发两个 `index rebuild`，其中一个返回 4；
- 断线场景返回 4 并输出可操作提示。

## 8 示例校验

- 文档中的命令示例在 CI 中执行最小子集；
- 示例使用的路径存在于夹具；
- 输出示例与快照保持同步；
- 不执行破坏性或需要网络的示例。

## 9 CI 命令

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
```

CLI 集成测试在无 TTY 环境运行；`open` 的 TUI 行为由 `mdctrl-tui` 的测试覆盖。

## 10 验收对应

| Issue 验收项 | 对应测试 |
| --- | --- |
| CLI 命令面 | 解析测试、帮助快照 |
| 输出与退出码 | 进程级测试、退出码表测试 |
| 命令组合 | 集成测试、冲突参数测试 |
| 结构化输出 | JSON 契约测试、字段稳定性快照 |
| check/index 数据命令 | 契约测试、健康夹具、索引集成测试 |
