## 关联 Issue

Closes #

## 变更说明

<!-- 做了什么、为什么这样做 -->

## 目标分支

- [ ] 成员分支 → MVP 分支
- [ ] MVP 分支 → main（由仓库拥有者合并）

## 影响范围

- [ ] protocol
- [ ] core
- [ ] render
- [ ] server
- [ ] tui
- [ ] cli
- [ ] docs
- [ ] ci

## 接口与协议变化

<!-- 无变化则写“无” -->

## 测试

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
```

结果：

## 文档

- [ ] 已更新对应 `docs/MVPn/` 或 crate 文档
- [ ] 本次变更无需文档更新

## Agent session

<!-- docs/archive/sessions/...；无 Agent 参与则写“无” -->

## 自查

- [ ] 变更聚焦单一主题
- [ ] 已自查 diff，不含无关改动
- [ ] 新增依赖已说明用途、许可证与体积影响
- [ ] 已移除密钥、令牌与个人路径
- [ ] 快照或测试夹具变更已说明原因

## 评审

- [ ] 至少一名非作者评审
- [ ] CI 通过
