# 系统架构

## 项目用途与模块划分

本项目读取 Stellaris 游戏数据、启动器当前启用的 MOD 和存档中的帝国属性，筛选可能获得的科技，生成后续解锁关系树，并将结果写入游戏本地化文件。玩家可在科技悬浮提示中查看解锁路线及额外前置要求。

分析以生成时的帝国身份为基准，保留联邦、外交、事件和研究进度等条件在未来满足的可能。结果适合用作长期路线参考；实际可获得的科技仍由游戏运行状态决定。使用较晚的存档时，分析基准也随该存档更新。

| 路径 | 职责 |
| --- | --- |
| `crates/dtt-core` | 帝国快照、条件模型与求值、科技资格、变体选择、依赖图和 ASCII 排版。 |
| `crates/dtt-stellaris` | Clausewitz 解析、存档与游戏数据读取、加载顺序、触发器策略、本地化、输出和路径发现。 |
| `crates/dtt-application` | 环境检测、存档扫描与检查、生成编排、进度、取消和报告。 |
| `crates/dtt-i18n` | 产品语言注册、语言协商、Fluent 资源和类型化消息接口。 |
| `apps/dtt-cli` | 命令行参数、应用层调用和终端展示；可执行文件名为 `dtt`。 |
| `apps/dtt-desktop` | React 界面与 `src-tauri` 中的 IPC 适配；可执行文件名为 `dtt-gui`。 |

业务依赖方向为 `{ dtt-cli, dtt-desktop } → dtt-application → { dtt-core, dtt-stellaris }`，以及 `dtt-stellaris → dtt-core`。CLI、应用层和 Stellaris 输出适配器使用 `dtt-i18n`。核心领域层通过通用数据和回调与适配层协作；文件系统、ZIP、SQLite、Jomini 和翻译运行时集中在外围模块。

Rust workspace 使用 2024 edition，工具链由 `rust-toolchain.toml` 固定为 `1.98.0`。主要依赖包括 `jomini 0.35`、`zip 8`、`rusqlite 0.40`、`clap 4.5`、`thiserror 2` 和 `anyhow 1`；具体约束见根目录及各 crate 的 `Cargo.toml`。桌面端说明见 [前端开发说明](frontend.md)。

## 生成流程

`dtt-application/src/generation/pipeline.rs` 实现 `run_generation`。入口先校验设置和路径，再依次执行下列阶段。每次阶段切换先检查取消令牌，再调用可选的进度回调；百分比用于展示阶段进度。

| 阶段 ID | 进度 | 任务 |
| --- | --- | --- |
| `SAVE_PARSE` | 5% | 从存档提取帝国快照，或复制调用方提供的快照。 |
| `LOAD_ORDER` | 20% | 读取启动器 playset 和 MOD 加载顺序。 |
| `INGEST_TECH` | 30% | 读取变量、内联脚本、脚本触发器、舰船文化和科技定义。 |
| `RELATIONS` | 35% | 评估科技资格、选择显示变体并构建依赖图。 |
| `INGEST_L10N` | 45% | 按目标语言读取科技描述，处理变体描述回退。 |
| `RENDER` | 50% | 为每项保留科技渲染后续解锁树。 |
| `CYCLES` | 60% | 汇总资格、循环、变体和数据读取诊断。 |
| `WRITE_OUTPUT` | 80% | 写入本地化文件和诊断报告，清理旧语言产物。 |
| `DONE` | 100% | 返回统计、报告和文件操作结果。 |

`RunGenerationResult` 包含数据源数量、科技数量、结构化报告及 `RunOutcome`。文件操作结果分为 `written`、`removed` 和 `failed`；存在写入、格式化或清理失败时，输出状态为 `incomplete`，其余为 `success`。取消请求在阶段边界生效，已完成的文件操作会保留。

## 存档与帝国快照

### 容器读取与格式检测

`dtt-stellaris/src/save/container.rs` 将 `.sav` 作为 ZIP 打开，读取 `meta` 和 `gamestate`。格式检测检查 `gamestate` 的前 128 字节：出现 NUL 字节，或以 `bin`、`sav` 开头时标记为二进制，其余按文本处理。应用层接受文本内容，遇到二进制内容返回 `UnsupportedBinarySave`。是否为铁人模式属于存档元数据，生成入口依据实际内容格式判断可用性。

存档库扫描只读取元数据和 `gamestate` 前缀，用于区分 `Text`、`Binary`、`Corrupt`。应用层支持本地存档与 Steam 云存档的本地副本扫描，按账号、战役组织结果，再按游戏日期、修改时间和文件名排序。

### 国家选择

解析器遍历顶层 `player` 列表中的国家引用并去重。生成入口遇到唯一候选时自动选择，遇到零个或多个候选时分别返回 `PlayerCountryMissing`、`PlayerCountryRequired`。

`inspect_save` 返回 `player_countries` 和可选 `snapshot`。多个候选且调用方尚未指定 `country_id` 时，返回候选列表供界面选择。桌面端可在选择国家后再次检查存档，并将国家 ID 传给生成入口；CLI 使用自动选择流程。

### 快照内容

`dtt-core/src/empire.rs` 定义 `Snapshot`。身份字段使用 `Option` 区分缺失信息和已知值，列表中的 `Some([])` 表示已知空集合。

| 字段 | 来源或用途 |
| --- | --- |
| `ethics` | 国家 `ethos` 块中的思潮。 |
| `government.civics` | `government.civics` 中的国民理念。 |
| `government.authority`、`origin`、`government_type` | 政体、起源和政府类型；政府类型取自 `government.type`。 |
| `founder_species` | 通过 `founder_species_ref` 查询 `species_db`，提取创始物种及特性。 |
| `founder_species.archetype` | 根据特性推断：机械特性对应 `MACHINE`，石质特性对应 `LITHOID`，其余已知特性集合对应 `BIOLOGICAL`。 |
| `graphical_culture` | 舰船文化，结合游戏数据中的舰船类别求值。 |
| `country_type` | 国家 `type`，其中 `colony` 归一为 `default`。 |
| `is_nomadic`、`is_ai` | 国家标记；玩家表确认的国家将 `is_ai` 设为 `false`。 |
| `ascension_perks`、`traditions` | 飞升天赋和传统列表，用于界面展示；相关条件采用过程性策略。 |

Rust 调用方可使用 `RunGenerationRequest::from_snapshot` 直接提供快照，适用于外部数据接入或假设推演。两种输入均通过同一套生成流程和求值引擎。

## 游戏数据与本地化读取

### 加载顺序

`load_order` 读取 `launcher-v2.sqlite` 中当前激活 playset 的已启用 MOD，按 `position`、`displayName` 升序排列。游戏本体位于索引 0，MOD 按加载顺序追加。缺少激活 playset 时，数据源仅包含本体。

相对路径相同的文件仅保留加载顺序中最后一个数据源的版本，包括空文件覆盖。剩余文件按数据源加载顺序读取，源内按相对路径排序，不同文件中的同名定义采用后读取的值。`replace_path` 会移除更早数据源中对应路径前缀的文件；`localisation/replace` 在普通本地化合并完成后单独覆盖。MOD 目录存在而 `descriptor.mod` 缺失时，系统记录诊断并继续读取该目录。

### 科技定义

`game_data` 依次读取共享变量、内联脚本、脚本触发器、舰船文化和科技。科技文件范围为 `common/technology` 直属的 `.txt` 文件；`category/`、`tier/` 等子目录属于科技元数据，本流程仅提取直属文件中的科技定义。

科技文件中的顶层 `@KEY` 声明按出现顺序更新文件变量表，读取定义时使用当时的变量值。`inline_script` 先展开到定义中，随后提取字段并编译条件。缺失、循环或格式错误的内联调用保留诊断；条件中的失败调用按未知条件处理，科技或变体字段层级的失败调用则导致该定义被跳过。

| 字段 | 处理方式 |
| --- | --- |
| `area`、`tier` | 保存领域和阶层；有效值缺失时记录诊断并跳过该定义。 |
| `prerequisites` | 保存必需前置 `all_of` 与 `OR` 可选组 `any_of_groups`。 |
| `potential` | 编译为 `CompiledCondition`，用于科技资格判断。 |
| `is_dangerous`、`is_rare` | 保存显示标记。 |
| `technology_swap` | 按声明顺序保存变体名称、触发器和可选领域。 |
| `inline_script` | 在字段提取前展开。 |
| 已识别的其他字段 | 跳过研发数值、AI 策略、界面修饰和机制标记。 |
| 未识别字段 | 保留字段名与来源，列入诊断。 |

当前跳过的已识别字段为 `start_tech`、`category`、`cost`、`cost_per_level`、`weight`、`weight_modifier`、`weight_groups`、`mod_weight_if_group_picked`、`ai_weight`、`ai_update_type`、`modifier`、`prereqfor_desc`、`gateway`、`is_reverse_engineerable`、`feature_flags`、`levels`、`starting_potential`、`is_insight`、`icon` 和 `is_repeatable`。

初始科技同样保留显式前置关系。必需项和各个 OR 组分别保存；变体 ID 具有唯一基础科技归属时，前置引用会归一到基础 ID。重复的已消费单值字段取第一处并记录诊断；仅大小写有差异的字段单独报告。

变体缺少 `trigger` 时使用空 `AND`，即恒真。变体领域缺省时沿用基础领域，非法领域值会产生诊断并跳过该变体；其余覆盖字段由当前提取器跳过。

### 科技描述

本地化读取器匹配 `l_<语言>.yml` 或 `*_l_<语言>.yml`，跳过文件名以 `zztechtree` 开头的生成文件。解析器支持 `KEY:0 "值"` 和 `KEY: "值"` 两种形式，并对读取的描述进行空白规范化。

保留的词条须以 `_desc` 结尾，且对应已知科技或变体 ID。生成时优先采用选中变体的描述，缺失时回退到基础科技描述。

## 条件分析与科技筛选

### 模块职责

- `dtt-stellaris/clausewitz/script` 保存布尔结构、分支、作用域、集合和脚本参数。
- `dtt-stellaris/game_data/lower` 绑定变量、展开脚本并检测循环，输出 `CompiledCondition`。
- `dtt-stellaris/analysis` 提供 `World` 和 `AnalysisContext`；`scope` 解析对象关系，`policy` 解释触发器与分析策略。
- `dtt-core/condition` 定义通用条件与 `EvaluationContext` 接口，计算逻辑边界。
- `dtt-core/technology/eligibility` 筛选科技并计算前置闭包；`technology/swap` 按顺序确定显示变体。

### 条件结构与作用域

`Condition` 包含 `All`、`Any`、`Not`、`If`、`Scope`、`AnyObject`、`Scripted`、`Predicate` 和 `Unknown`。谓词保留名称、运算符与参数；脚本调用节点保留调用链，便于定位诊断。

相邻 `if` 分别求值，`else_if` 和 `else` 延续当前分支链。空 `AND` 为真，空 `OR` 为假，`always = no` 为确定禁用。求值只访问可达分支，并在诊断中保留作用域和调用路径。

上下文区分选定国家、创始物种、未来可能对象和未知对象。`this` 指向当前对象，`root` 指向科技条件入口国家，`prev`、`prevprev` 按作用域栈回溯；脚本触发器调用沿用现有作用域栈。入口缺少事件来源，因此 `from`、`fromfrom` 返回带原因的 Unknown。

从选定国家访问 `species`、`owner_species`、`founder_species` 时使用创始物种。联邦、首都、领袖及已识别集合采用未来可能对象，同时保留已知所属关系，例如玩家 `any_owned_planet` 元素的 `owner` 仍指向玩家国家。

`any_country`、`any_relation`、`any_neighbor_country`、`any_subject`、`any_owned_species`、`any_owned_planet`、`any_owned_leader`、`any_situation` 在适用上下文中具有明确对象类型。未知集合保留块结构并返回未知信息。集合按未来可能存在的元素分析，块内 `always = no` 和确定的身份限制仍有效。可选作用域后缀 `?` 在对象存在性未知时综合存在、缺席两种路径。

### 逻辑边界与分析策略

每个条件产生 `TruthBounds { guaranteed, possible }`，分别表示确定性和可能性，取值均为 `True`、`False` 或 `Unknown`。

| 条件状态 | `guaranteed` | `possible` |
| --- | --- | --- |
| 身份条件成立 | True | True |
| 身份条件排除 | False | False |
| 过程性门槛 | False | True |
| 语义未知或资料缺失 | Unknown | Unknown |

`AND`、`OR` 分别组合两个边界；`NOT` 交换边界再取反。分支求值保留同一 `limit` 的互斥关系。例如，机械物种与事件条件的 OR 对生物帝国仍有可能成立；二者的 AND 则由物种条件确定排除。事件条件取反后也保留未来满足的可能。

身份策略评估选定国家及其物种的思潮、理念、政体、起源、政府类型、国家类型、AI、游牧、特性、原型和舰船类别。目标类型与数据完整性在求值前校验，缺失身份资料记为 Unknown。未来对象的属性按可能性处理，玩家身份事实仅用于对应对象。

过程性策略涵盖时间、通信、危机、DNA、联邦、标记、遗珍、议案、资源、DLC、科技、飞升天赋、传统、附属关系、对象存在、局势类型和政策。DLC 按全路线展示策略保留；未绑定变量和畸形参数记为 Unknown。过程性诊断归入 `deferred_triggers`，语义未知的条件归入 `unknown_triggers`。

该算法计算局部逻辑边界。跨节点的事件互斥、谓词相关性和完整事件可达性属于分析范围之外，因此结果可能包含实际难以同时满足的组合。身份基准固定为生成时快照，飞升和物种转化后的身份需通过新的快照重新分析。

### 科技资格

`potential` 从选定国家入口求值，随后按显式前置关系收敛科技集合。

- `possible = False`：排除科技并记录身份限制。
- `possible = True`：保留科技；确定性尚未成立时记录相应标记。
- `possible = Unknown`：默认 `include_flagged` 保留并标记；`exclude_strict` 排除；`error` 中断生成。

未知策略针对语义或数据的未知状态，过程性门槛仍按未来可能性处理。依赖图的边仅来自 `prerequisites`，条件内的 `has_technology` 参与求值策略。

### 显示变体

变体与资格共用求值引擎，选择时遵循声明顺序：

1. `possible = False` 时继续检查下一项。
2. `guaranteed = True` 时选中当前项，使用其显示 ID 和可选领域，记录 `Matched`。
3. 当前项可能匹配、确定性尚未成立时，默认 `keep_base` 停止选择并保留基础显示，记录 `Uncertain`；`error` 策略中断生成。
4. 所有项均被排除时记录 `NoMatch`。

`Uncertain` 保留基础 ID、领域和依赖。显示 ID 冲突会作为错误处理。报告和桌面界面分别统计三种变体结果。

## 图谱与渲染

图谱节点为资格筛选后的科技，边来自归一化的前置关系，支持物理、社会、工程之间的跨领域依赖。图模块检测自引用和循环；渲染器为每项科技展开后续解锁树，并将额外前置条件附在节点末尾。同一棵树中的重复节点只展开一次。

`dtt-core/render` 负责连接线、缩进、遍历和截断，节点文本由回调提供。`dtt-stellaris/output/render.rs` 补充游戏链接、图标、颜色和翻译提示。节点格式为：

```text
({tier})['technology:{base_id}', {area_icon}{color}${display_id}$§!]
```

链接使用基础 ID，名称使用显示 ID。领域图标分别为 `£physics£`、`£society£`、`£engineering£`；危险、稀有和常规科技分别使用 `§R`、`§M`、`§W`，危险标记优先。额外前置和 OR 关系按输出语言格式化。

ASCII 连接线使用 `|   `、四个空格和 `|-`，截断标识为 `...`。`RenderLimits` 默认深度为 16、同级数量为 64、总节点预算为 4096。根节点直接后继超过 128 时返回 `OverlongRoot`，适配层将其转换为省略提示。CLI 和桌面端采用默认值，Rust 调用方可通过生成请求调整。

树正文经适配层转换后，值内换行保存为字面转义序列 `\n`。文件中的各条本地化记录仍以 LF 分隔。空树显示终点提示。

## 文件输出

应用层以当前可执行文件所在目录为输出根目录。每种选定语言生成两份文件，诊断报告写在根目录：

| 相对路径 | 内容 |
| --- | --- |
| `localisation/{lang}/zztechtreemain_l_{lang}.yml` | 树标题、终点提示和各科技的解锁树。 |
| `localisation/{lang}/replace/zztechtreereplaced_l_{lang}.yml` | 科技描述、阶层提示和树文本引用。 |
| `dtt-save-report.txt` | 资格、未知条件、过程性条件、循环、变体及读取诊断。 |

`.yml` 使用带 BOM 的 UTF-8 编码和 LF 分隔。写入通过 `AtomicWriteFile` 逐文件提交。输出器还会清理本次未选语言目录下、符合上述两个固定文件名的旧产物，并记录清理结果。

游戏输出支持 `english` 和 `simp_chinese`，默认 `english`。重新生成时，选定语言文件会被当前结果覆盖。

## 命令行与桌面入口

CLI 提供两个子命令：

- `dtt detect-paths`：展示发现的游戏目录、文档目录、启动器数据库和 Steam 库。
- `dtt generate <SAVE>`：从文本存档生成本地化文件。

生成命令通过 `--stellaris-root`、`--documents-dir`、`--launcher-db` 覆盖自动发现结果。文档目录用于定位启动器数据库，输出根目录由可执行文件位置决定。

可重复指定 `--language <LANG>`。`--unknown-strategy` 接受 `include-flagged`、`exclude-strict`、`error`，默认第一项；`--swap-unknown-strategy` 接受 `keep-base`、`error`，默认第一项。全局 `--locale` 选择产品文案及报告语言，缺省时使用系统语言协商结果。

桌面端按设置、存档与帝国、生成、结果四步组织流程。Tauri 提供 `get_bootstrap_data`、`resolve_environment`、`scan_save_library`、`inspect_save`、`run_generation`、`cancel_generation`、`open_output_directory` 七个命令。生成任务在阻塞线程执行，通过 `Channel` 发送阶段进度；同一时刻仅允许一个生成任务。

## 国际化与维护入口

`dtt-i18n` 提供 `AppLocale`、`Translator` 和类型化消息接口，产品语言为 `en`、`zh-Hans`。游戏输出语言使用独立的 `GameLanguage`。报告语言由 `RunGenerationRequest.presentation.report_locale` 在任务创建时确定，`generation/report.rs` 组织结构化数据，`generation/report_text.rs` 负责文本渲染。界面切换语言后，磁盘报告保留生成时的语言。

Rust 翻译资源位于 `crates/dtt-i18n/locales`，前端资源位于 `apps/dtt-desktop/src/i18n/locales`。游戏文案使用 Fluent 的 game 域，并关闭双向文本隔离符。Tauri DTO 传递错误码、枚举和参数，前端根据当前语言展示诊断。

常用维护命令在仓库根目录执行：

| 命令 | 用途 |
| --- | --- |
| `cargo check --workspace` | 检查 Rust 类型和依赖。 |
| `cargo test --workspace` | 运行 Rust 单元与集成测试。 |
| `pnpm bindings` | 从 Rust DTO 导出 TypeScript 契约。 |
| `pnpm i18n:generate` | 更新前端翻译资源类型。 |
| `pnpm i18n:check` | 检查翻译契约、资源类型和静态文案。 |
| `pnpm desktop:test` | 运行前端测试。 |
| `pnpm desktop:build` | 执行 TypeScript 检查并构建前端资源。 |
