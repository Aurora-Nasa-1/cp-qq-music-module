# AGENTS.md

本文件用于指导在 `netease-qq-music-api` 仓库内工作的智能体与协作者，确保改动可维护、可验证、且不破坏现有 API 行为。

## 1. 项目概览

- 项目类型：Rust library（`edition = "2024"`）
- 包名：`netease-qq-music-api`
- crate 路径名：`netease_qq_music_api`
- 目标：提供统一的音乐 API 调用接口，封装网易云（Netease）与 QQ 音乐（Tencent）相关能力（搜索、详情、播放、登录、歌单、发现等）

## 2. 代码结构与职责

- `src/lib.rs`
  - 对外导出：`client`、`error`、`models`
  - `platform` 为内部实现细节（未对外公开）
- `src/client.rs` 与 `src/client/*`
  - 对外入口 `MusicClient`
  - 使用类型态（kind 类型）组织链式请求构建器
  - 业务能力按模块拆分：`search`、`detail`、`playback`、`discover`、`playlist`、`login`
- `src/platform/netease/*`
  - 网易云平台具体实现与模型转换
- `src/platform/tencent/*`
  - QQ 音乐平台具体实现与模型转换（含 MQTT 登录流程）
- `src/models.rs`
  - 跨平台统一数据模型与 token 结构
- `src/error.rs`
  - 统一错误类型定义（如参数缺失、平台 token 不匹配等）

## 3. 开发原则

- 保持对外 API 稳定：
  - 非必要不要更改公开类型、公开函数签名与返回结构。
- 优先最小改动：
  - 改动仅覆盖当前需求，不做无关重构。
- 平台边界清晰：
  - 跨平台统一逻辑放在 `client/models/error`。
  - 平台专有逻辑放在各自 `platform/*` 子模块中。
- 错误语义明确：
  - 参数校验失败必须返回可区分的 `ClientError` 变体。
  - 保持现有错误文案与行为一致，避免破坏上层匹配逻辑。

## 4. 常用命令

```bash
# 格式化
cargo +nightly fmt

# 静态检查
cargo clippy --all-targets --all-features -- -D warnings

# 测试
cargo test

```

## 5. 测试策略

- 新增功能时至少补充一项：
  - 参数校验测试，或
  - 平台 token 约束测试，或
  - 结果映射正确性测试
- 若变更涉及登录、播放链路或模型转换，优先补回归测试。

## 6. 改动约束（面向智能体）

- 不要提交密钥、cookie、token、二维码原始会话信息。
- 不要把临时调试输出长期保留在库代码中。
- 避免引入不必要依赖；新增依赖需有明确收益。
- 修改公共模型字段前，必须同步检查：
  - 平台转换代码
  - 请求返回解析代码
  - 相关测试断言

## 7. 提交前检查清单

- 代码可编译：`cargo check`
- 已格式化：`cargo +nightly fmt`
- 本地测试通过：`cargo test`
- 无无关文件改动（特别是 `target/`、IDE 临时文件）

## 8. Git Commit 信息格式

- 提交信息使用如下结构：

```text
<type>(<scope>): <subject>

<body>

<footer>
```

- 说明：
  - `type`：变更类型（如 `feat`、`fix`、`docs`、`refactor`、`test`、`chore`）
  - `scope`：影响范围（如 `client`、`netease`、`tencent`、`models`、`error`）
  - `subject`：一句话概述改动，使用祈使句，尽量简洁明确
  - `body`：可选，说明改动动机、方案与影响
  - `footer`：可选，记录 breaking changes、issue/PR 关联等

- 示例：

```text
fix(release): need to depend on latest rxjs and zone.js

The version in our package.json gets copied to the one we publish,
and users need the latest of these.
```

## 9. 推荐工作流

1. 先定位改动应落在 `client` 统一层还是 `platform` 实现层。
2. 先写/补测试，再做实现（至少保证参数校验路径有覆盖）。
3. 小步提交，确保每次改动都可解释、可回滚。
