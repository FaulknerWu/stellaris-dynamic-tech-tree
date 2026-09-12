# Repository Guidelines

## 项目结构与模块组织

本仓库是纯 Rust workspace。依赖方向固定为 `dtt-cli → dtt-application → { dtt-core, dtt-stellaris }` 以及 `dtt-stellaris → dtt-core`。

- `crates/dtt-core`：纯领域模型与算法（帝国快照、条件 AST 与求值、科技资格、依赖图、ASCII 渲染）。不包含文件、路径、ZIP、SQLite 或 Jomini。
- `crates/dtt-stellaris`：Stellaris 适配层（Clausewitz、存档、加载顺序、游戏数据、本地化、输出与路径发现）。
- `crates/dtt-application`：共享应用入口（生成编排、进度、取消、环境检测与存档检查）。
- `apps/dtt-cli`：命令行入口 `dtt`，只调用应用层公共 API。

核心库应保持与具体交互层解耦。文档放在 `docs`，构建产物 `target` 不应手动编辑。

## 构建、测试与开发命令

- `cargo build --workspace`：构建所有 Rust crate。
- `cargo check --workspace`：快速检查所有 Rust crate 的类型与依赖。
- `cargo test --workspace`：运行 Rust workspace 测试。
- `cargo run -p dtt-cli -- <args>`：运行 CLI，本地调试命令行流程。

## 编码风格与命名约定

Rust 使用 2021 edition、`rust-version = "1.88"`，保持 `rustfmt` 默认格式；模块、函数与变量用 `snake_case`，类型与枚举用 `PascalCase`。所有 crate 均应禁用 unsafe，新增代码不得引入 unsafe。注释应解释设计原因或业务约束，不重复代码表面行为。

## 测试规范

当前测试入口是 `cargo test --workspace`。新增核心逻辑优先在相关 Rust 模块中添加单元测试，测试名称使用可读的 `snake_case`，描述预期行为。

## Commit 与 Pull Request 规范

历史提交采用 Conventional Commits 风格，例如 `feat(cli): add CLI binary, path discovery, and generate module`。提交信息使用英文，格式为 `type(scope): summary`，summary 使用简短祈使句或动作短语。PR 应说明变更目的、主要实现、验证命令与影响范围，关联问题时写明 issue 编号。

## 配置与安全提示

不要提交本地游戏路径、用户存档、临时生成 MOD、启动器数据库或平台私有配置。新增日志与诊断信息时不得泄露用户目录中的敏感数据。
