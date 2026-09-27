# 系统架构设计文档

## 1. 项目概述

本项目是一款专为 Stellaris 设计的科技树静态分析引擎，它通过静态分析游戏本体数据、当前激活的 Playset 模组列表以及玩家帝国的实际存档状态，以生成时帝国身份为基准，保守纳入未来可能获得的科技并生成拓扑依赖子图。最终，系统将格式化的 ASCII 层级依赖树无缝注入到游戏本地化文本中，以纯本地化 MOD 的形式使玩家能够在游戏内悬浮提示（Tooltip）中即时查阅任意科技的前置依赖与后续解锁路线。

## 2. 技术栈与核心依赖

| 分层 / 领域 | 依赖组件 | 版本与特性 | 核心职责 |
|---|---|---|---|
| 文本解析 | `jomini` | `0.35` | Clausewitz 语法解析 |
| 存档解压 | `zip` | `8.x` | 解压 `.sav` 容器读取 `meta` 与 `gamestate` |
| 数据存储 | `rusqlite` | `0.40` | 读取启动器数据库 `launcher-v2.sqlite` |
| 命令行 | `clap` | `4.5` | 解析 `generate` 与 `detect-paths` 参数 |
| 库错误定义 | `thiserror` | `2.x` | 内部库模块强类型结构化错误 |
| CLI 错误处理 | `anyhow` | `1.x` | 命令行上层上下文包装与友好展示 |

---

## 3. 处理流水线

一次完整的生成过程由多个阶段组成。进入每个阶段时，应用层会调用可选进度回调并检查取消令牌。各个阶段的划分、默认进度百分比以及对应的输入输出如下：

| 序号 | 阶段 ID | 默认进度 | 主要任务 | 核心产出 | 涉及模块 |
|:---:|---|:---:|---|---|---|
| 1 | `SAVE_PARSE` | 5% | 解析 `.sav` 存档文件 | `empire::Snapshot` | `dtt-stellaris/save` |
| 2 | `LOAD_ORDER` | 20% | 读取 playset 模组加载顺序 | `load_order::Manifest` | `dtt-stellaris/load_order` |
| 3 | `INGEST_TECH` | 30% | 摄取解码变量与科技原始定义 | 科技原始词条集合 | `dtt-stellaris/game_data` |
| 4 | `RELATIONS` | 35% | 编译条件、资格判定与构图 | 科技依赖图与资格报告 | `dtt-core`, `dtt-stellaris` |
| 5 | `INGEST_L10N` | 45% | 读取目标语言本地化文本 | 本地化字典映射 | `dtt-stellaris/localisation` |
| 6 | `RENDER` | 50% | 渲染 ASCII 子树并标注节点样式 | 格式化科技树文本 | `dtt-core/render` |
| 7 | `CYCLES` | 60% | 汇总循环、变体与摄取诊断 | 结构化诊断报告 | `dtt-application/report` |
| 8 | `WRITE_OUTPUT` | 80% | 写出本地化 `.yml` 与诊断报告 | 磁盘文件产物 | `dtt-stellaris/output` |
| 9 | `DONE` | 100% | 汇总运行结果与状态 | `RunOutcome` | `dtt-application` |

---

## 4. 解析层

### 4.1 `.sav` 容器读取与格式识别

Stellaris 的存档本质上是一个 **ZIP 压缩包**，内部包含两个 Clausewitz 格式的文件：`meta`（存储名称、版本和日期等元数据）和 `gamestate`（存储完整的游戏状态）。

在非铁人模式下，这两个文件均为纯文本格式；而在铁人模式下，`gamestate` 文件采用**二进制**格式。二进制 `gamestate` 中的键被替换为数字 token，这些 token 会随游戏版本更新而改变，且受限于 Paradox 的授权协议，本工具无法分发 token 映射表。因此，**本工具不解析二进制存档**。CLI 遇到二进制存档时会返回错误；库调用方如已掌握帝国属性，可以通过 `RunGenerationRequest::from_snapshot` 构造请求并调用 `run_generation`（详情参见 §6.4）。

### 4.2 Clausewitz 文本解析

需要读取的游戏数据均采用 Clausewitz 文本语法，使用 Jomini 作为 Clausewitz 文本解析引擎。

### 4.3 `@` 变量解析

游戏定义文件中大量使用了诸如 `@tier1cost = 1000` 定义变量，随后通过 `cost = @tier1cost` 进行引用的语法。Clausewitz 语法解析本身不会完成这种引用的数值求解，因此系统在数据摄取阶段先扫描文件顶层的 `@KEY = value` 结构建立变量映射表，再在读取具体数值时解析 `@ref`。如果缺少这一步骤，类似 `tier = @tier1` 的字段将被识别为普通字符串，导致数据丢失。

### 4.4 本地化 `.yml` 文件解析

Paradox 游戏使用的 `.yml` 文件**并非标准的 YAML 格式**，其**强制要求使用 UTF-8 with BOM 编码**（若缺少 BOM 签名，游戏引擎将无法正确解析，出现乱码）。典型的结构如下：

```yaml
l_english:
 tech_corvettes: "Corvettes"
 tech_corvettes_desc: "..."
```

系统在读取本地化文件时，会解析 Paradox 本地化索引格式（如 `KEY:0 "值"`）和无索引格式（如 `KEY: "值"`）。

---

## 5. 数据摄取层

### 5.1 加载顺序与文件覆盖

加载顺序读取 Paradox 启动器数据库（`launcher-v2.sqlite`）中当前激活 playset 的已启用 MOD，按 `position` 升序排列。索引 0 固定为游戏本体；其后 MOD 从 1 起按该顺序递增编号。若没有激活 playset，则仅保留本体。

在扫描合并文件时，系统严格依据该索引号遵循最后加载优先原则进行覆盖。特殊处理的引擎原生覆盖机制包括：
- **`replace_path`**：当 MOD 声明了对特定路径的 `replace_path`，系统会直接剔除所有较小索引号数据源中对应前缀的文件。
- **`localisation/replace` 延迟生效**：位于此目录下的本地化文件享有独立的生效阶段（phase=1），这确保了它们一定会在基础本地化词条合并完毕之后，再依据索引号进行最终覆盖。

### 5.2 科技定义的提取与合并

系统只读取各数据源 `common/technology` 目录直属的 `.txt` 文件；`category/`、`tier/` 等子目录属于独立元数据，不作为科技定义递归摄取。

跳过 `@` 变量后，系统将提取每个顶层 `key = { … }` 作为独立的定义片段。科技字段分为以下三类：

| 字段分类 | 典型字段 | 处理行为 | 状态说明 |
|---|---|---|---|
| **基础元数据** | `area`、`tier` | 存入领域模型 | 决定科技领域与阶层；`start_tech` 不参与静态依赖图计算 |
| **依赖前置** | `prerequisites` | 构建科技依赖边 | 解析必须满足项与 `OR` 可选依赖组 |
| **资格门槛** | `potential` | 编译为条件 AST | 用于帝国身份三值逻辑资格判定 |
| **渲染特征** | `is_dangerous`、`is_rare` | 映射为显示标记 | 控制 ASCII 树中的危险/稀有颜色 |
| **条件变体** | `technology_swap` | 提取变体与触发器 | 控制特定帝国风格下的变体替换 |
| **研发与权重 (忽略)** | `cost`、`weight`、`levels` 等 | 识别后直接跳过 | 合法字段但不影响图谱，不报警告 |
| **AI 策略 (忽略)** | `ai_weight`、`ai_update_type` | 识别后直接跳过 | 仅供游戏 AI 决策使用，静态分析忽略 |
| **机制与标记 (忽略)** | `gateway`、`is_repeatable` 等 | 识别后直接跳过 | 特殊机制标记，当前分析不需要 |
| **UI 与修饰 (忽略)** | `icon`、`category`、`modifier` 等 | 识别后直接跳过 | 界面与数值修饰，不影响拓扑关系 |
| **未识别字段** | 上述以外的任意顶层字段 | 记录字段名与来源 | 计入诊断报告，防止遗漏新扩展语法 |

> [!NOTE]
> 完整忽略字段列表包括：`start_tech`、`category`、`cost`、`cost_per_level`、`weight`、`weight_modifier`、`weight_groups`、`mod_weight_if_group_picked`、`ai_weight`、`ai_update_type`、`modifier`、`prereqfor_desc`、`gateway`、`is_reverse_engineerable`、`feature_flags`、`levels`、`starting_potential`、`is_insight`、`icon`、`is_repeatable`。

其中，关于核心的结构依赖与变体：
- `prerequisites`：无论是否声明 `start_tech = yes`，均保留显式前置关系。系统必须保留其逻辑结构：直接列出的科技 ID 是必须满足的**前置条件（all-of）**，而包围在 `OR = { ... }` 块内的科技则是**可选组（any-of）**。同一个前置条件块中可以同时包含多个必须满足的 ID 以及多个可选组。绝不能将 `OR` 组展平为必须全部满足的条件，也不能将其丢弃。
- `technology_swaps[]`：系统必须保留其声明的先后顺序，并按顺序抽取 `name`（作为 `active_id`）与 `trigger`（详见 §7.7）。缺少 `trigger` 时视为空 `AND`（恒为 True）。提取可选 `area`，匹配成功后覆盖显示领域；未声明时沿用基础科技领域，非法值产生诊断并跳过该变体。`inherit_*` 与其它覆盖字段不入库。
- 重复的已消费单值字段会产生诊断，当前仍只读取第一处，不假定游戏会合并或覆盖。与已识别字段仅大小写不同的名称（例如 `Tier`）单独报告，不自动转换；缺少有效 `tier` 的定义仍跳过。

### 5.3 本地化文本摄取

系统仅保留路径中匹配 `l_<目标语言>.yml` 或 `*_l_<目标语言>.yml` 的文件，并跳过文件名以 `zztechtree` 开头的已生成输出。合并时同样遵循后加载覆盖先加载原则，且 `localisation/replace` 作为独立的第二阶段覆盖。此外，只保留键名以 `_desc` 结尾，且去除后缀后的 ID 属于已知科技或**已知替换变体 ID**（详见 §7）的词条。在摄取后，系统会对描述文本进行空白字符的规范化处理。对于没有独立 `{active_id}_desc` 的 `technology_swap` 变体，系统会沿用其基础科技的 `{base_id}_desc`，与游戏数据中的实际回退行为保持一致。

---

## 6. 存档快照与帝国识别

### 6.1 安全性与校验机制

- 系统将 `.sav` 文件作为 ZIP 压缩包打开，并强制要求内部必须包含 `meta` 与 `gamestate` 文件。
- 铁人与二进制识别：系统检查 `gamestate` 条目的前 128 个字节。若包含 NUL 字节，或以 `bin` / `sav` 开头，即判定为二进制格式。**由于无法解析二进制存档**，基于存档的生成调用会返回错误。

### 6.2 玩家国家识别

系统扫描顶层 `player` 块（若为数组则遍历其元素），收集其中的 `country = <int>` 引用并去重，不会递归搜索更深的嵌套结构。
- 如果只找到一个候选国家，系统将自动选定。
- 如果找到多个候选，默认流程会返回「帝国选择歧义」错误并附上候选列表；库调用方可在 `GenerationSource::Save` 或 `inspect_save` 中指定 `country_id`。`dtt-cli` 不暴露该参数。

### 6.3 帝国快照字段

`empire::Snapshot` 是规则引擎求值的**唯一底物**——内建求值器直接读取其字段，脚本化触发器展开后也最终落到这些字段。其结构包含从所选国家中提取的关键属性（按照 Stellaris 存档的惯例结构提取）：

| 快照字段 | 数据结构路径 | 存档提取来源 | 支持的触发器 / 作用 |
|---|---|---|---|
| 思潮 | `ethics` | `ethos = { ethic }` | `has_ethic`（及派生脚本触发器） |
| 国民理念 | `government.civics` | `government = { civics }` | `has_civic`、`has_valid_civic` |
| 政体 | `government.authority` | `government = { authority }` | `has_authority`（及帝国形态派生） |
| 起源 | `government.origin` | `government = { origin }` | `has_origin`（及起源派生） |
| 政府类型 | `government.government_type` | `government = { type }` | `has_government` |
| 物种特性 | `founder_species.traits` | `founder_species_ref` 物种块 | `has_trait` |
| 物种原型 | `founder_species.archetype` | 由 traits 推断（机器/石质/生物） | `is_archetype`（及物种派生） |
| 舰船风格 | `graphical_culture` | 国家块 `graphical_culture` | `uses_ship_category`（查 ship_kinds） |
| 国家类型 | `country_type` | 国家块 `type`（`colony` 归一为 `default`） | `is_country_type`、`is_ai`（玩家恒为 `no`） |
| 游牧标志 | `is_nomadic` | 国家块 `is_nomadic` | `is_nomadic` |
| 飞升天赋 | `ascension_perks` | 国家块 `ascension_perks` 数组 | 仅供前端展示；对应条件按过程性策略保留可能性 |
| 传统列表 | `traditions` | 国家块 `traditions` 数组 | 仅供前端展示；对应条件按过程性策略保留可能性 |

> [!NOTE]
> 物种 Archetype 由特性推断：`trait_machine_unit` / `trait_mechanical` 推断为 `MACHINE`，`trait_lithoid` 推断为 `LITHOID`，其余为 `BIOLOGICAL`。
> 动态游戏状态（如 `has_country_flag`、`has_policy_flag`、`is_subject`、`has_federation` 等）不提取到快照中，由过程性策略保留未来可能性（详见 §7.5）。

### 6.4 快照的生成方式：自动提取与程序化输入

`empire::Snapshot` 是一个独立于数据来源的结构体。**规则引擎在进行逻辑判定时只依赖这个结构体，而不关心数据是如何获取的**。应用层 `run_generation` 支持两种输入方式：

- **自动提取（CLI 默认方式）**：从非铁人模式的文本存档中自动读取并生成。
- **程序化快照**：Rust 调用方构造标准 `empire::Snapshot`，通过 `RunGenerationRequest::from_snapshot` 创建请求，再调用 `run_generation`。这种方式适用于调用方已从其他可信来源获得帝国属性，或进行无存档的假设推演。

程序化快照与自动提取的快照**结构完全一致**，规则引擎对二者一视同仁：统一展开脚本触发器并读取相同字段。应用层另提供 `inspect_save`，只解析存档并返回快照，不执行生成。`dtt-cli` 当前只暴露基于存档的生成流程，不提供交互式快照录入或 `inspect_save` 子命令。

---

## 7. 长期科技树分析

目标是在开局生成一次可长期参考的科技路线图。生成时的帝国身份是固定分析基准，联邦、外交、事件、研究进度等过程性门槛保留未来满足的可能。系统不模拟整局状态转移，也不承诺每个被纳入的科技都能在本局实际获得。选择较晚存档时，身份基准就是该存档中的身份，不反推历史开局。

### 7.1 语义与策略的模块边界

- `clausewitz/script` 保留布尔结构、`if` 分支、作用域引用、`any_*` 集合体及脚本调用参数。
- `game_data/lower` 负责变量绑定、脚本展开和循环检测，输出 `CompiledCondition`；不读取快照、不删除过程条件、不把作用域降为国家或物种标签。
- `dtt-stellaris/analysis` 提供 `World` 和 `AnalysisContext`。`scope` 解析对象关系，`policy` 解释触发器并决定开局身份与过程条件的处理方式。
- `dtt-core/condition` 保留通用条件模型，以 `EvaluationContext` 为对象及谓词接口，只计算逻辑边界，不内置 Stellaris 触发器或快照字段映射。
- `technology/eligibility` 按可能性筛选并计算前置闭包；`technology/swap` 按确定性和声明顺序选择显示变体；图谱与渲染不解释脚本。

### 7.2 保留结构的条件模型

`Condition` 包含 `All`、`Any`、`Not`、`If`、`Scope`、`AnyObject`、`Scripted`、`Predicate` 和 `Unknown`。作用域保留原始引用字符串；谓词保留名称、运算符与参数，参数无法识别时产生明确诊断。脚本展开保留调用节点，诊断可以说明调用链。

相邻 `if` 是独立条件，只有 `else_if` 与 `else` 延续当前条件链。分支按控制流求值，不把同一 limit 的正反状态当成互不相关的条件。显式空 `AND` 为真，空 `OR` 为假，`always = no` 保留确定的禁用含义。

### 7.3 对象身份、作用域和数据完整性

分析上下文区分选定国家、其创始物种、未来可能对象以及无法解析的对象。对象类型并不等于对象身份：其他国家的身份条件不会读取玩家国家数据。

- `this` 保留当前对象，`root` 指向科技条件入口的国家，`prev` / `prevprev` 等按作用域栈解析。脚本触发器调用本身不压入作用域栈。
- `from` / `fromfrom` 等保留独立引用含义。当前科技分析入口没有事件来源对象，因此返回带原因的 Unknown，不能映射为玩家国家。
- 从选定国家解析 `species`、`owner_species`、`founder_species` 时使用其创始物种；从未来国家得到未来物种，其他未经确认的关系返回 Unknown。
- 联邦、首都、领袖及已识别的 `any_*` 集合使用未来可能对象。已知所属关系保留，例如从玩家的 `any_owned_planet` 元素返回 `owner` 时仍指向玩家国家。
- `any_country`、`any_relation`、`any_neighbor_country`、`any_subject`、`any_owned_species`、`any_owned_planet`、`any_owned_leader`、`any_situation` 在合适上下文中有明确对象类型。未知集合保留块结构和不确定性，不把元素冒充玩家对象。
- 集合不按开局空集合判为假。集合体中的明确限制仍有效，例如体内 `always = no` 或通过 `root` 得到的身份排除条件。
- 可选作用域后缀 `?` 保留“对象存在才检查块体”的条件结构，无法确认存在时综合存在与不存在两条路径。

所有开局身份谓词先校验目标类型。快照中的身份字段、创始物种及物种特性使用 `Option` 区分缺失和已知空值；未知起源、缺失物种和未解析舰船文化都不能默认为不满足。存档玩家表确认的国家可判为非 AI；其他国家不凭选择行为假定为玩家。

### 7.4 可能性与确定性

每个条件输出 `TruthBounds { guaranteed, possible }`，两个分量均使用 `True / False / Unknown`。

| 事实 | guaranteed | possible |
|---|---|---|
| 确定满足的开局身份 | True | True |
| 确定不满足的开局身份 | False | False |
| 放宽的过程性门槛 | False | True |
| 无法解释或数据不足 | Unknown | Unknown |

`AND`、`OR` 分别组合两个边界；`NOT` 交换边界再取反。条件分支保留同一 limit 的互斥关系。只有实际可达的分支参与求值，运行时诊断记录已访问的条件、作用域路径和脚本调用链。

过程性条件既不被删除，也不固定为 True。例如 `OR { 机械物种, 完成考古事件 }` 对非机械帝国仍有满足的可能；`AND { 机械物种, 完成考古事件 }` 对非机械帝国仍确定不满足；否定事件条件同样保留可能性。

这是保守的局部边界分析，不追踪不同节点之间的事件互斥、重复谓词相关性或完整事件可达性，因此可能保留实际无法同时满足的组合。

### 7.5 开局身份与过程性门槛

`policy` 在选定国家及其物种上评估已有身份条件：思潮、国民理念、政体、起源、政府类型、国家类型、AI 标记、游牧标记、物种特性、物种原型和舰船类别。对未来对象的这些属性保留可能性，不借用玩家事实。本阶段不模拟飞升或物种转化后的身份。

过程性策略涵盖时间、通信、危机等级、DNA、联邦特性、全局及国家标记、危机天赋、遗珍、议案、资源开销、DLC、科技进度、飞升天赋、传统、联邦、附属关系、对象存在、局势类型和政策标记。DLC 按产品的全路线展示策略放宽，不声明其会在游戏中随时间改变。合法目标与参数结构仍须校验，未绑定变量和畸形参数进入 Unknown。

过程性诊断使用 `deferred_triggers`，与 `unknown_triggers` 分开。桌面端显示“过程性条件”，不把这些节点称为已忽略。

### 7.6 科技资格与前置关系

科技的 `potential` 从选定国家入口分析：

- `possible = False`：排除并记录身份限制。
- `possible = True`：纳入；若 `guaranteed != True`，标记为非确定资格。
- `possible = Unknown`：按 `unknown_strategy` 处理，默认 `include_flagged` 保留，`exclude_strict` 排除，`error` 中断。

严格未知策略只处理无法解释的可能性，不会因为纯过程条件尚未完成而排除科技。随后按显式 `prerequisites` 计算必要前置和 OR 组的闭包；初始科技同样保留定义中的依赖。过程条件中的 `has_technology` 不额外产生科技图连线。

### 7.7 科技变体

变体使用与资格相同的作用域和逻辑引擎，但要求开局身份足以确定匹配：

1. 按声明顺序分析；`possible = False` 时继续。
2. `guaranteed = True` 时选择当前变体，应用名称及可选研究领域，描述缺失时回退基础科技。
3. 若当前变体仍可能匹配但不确定，默认 `keep_base` 立即保留基础显示，结果记为 `Uncertain`。不能越过它选择后续变体，因为那会违反声明优先级。严格 `error` 策略中断。
4. 所有变体均确定不匹配时记录 `NoMatch`；确定选中时记录 `Matched`。报告和界面分别统计三种结果。
5. 不确定变体不修改基础科技的依赖、显示 ID 或研究领域；不同科技不能解析为相同显示 ID。

---

## 8. 科技图谱构建

在完成资格判定和替换解析后，系统开始构建内部数据图：

- **节点**：经过身份判定、可能出现的全部科技。
- **边**：连接关系**仅来源于规范化后的 `prerequisites` 字段**（含 §5.2 所述的 swap 变体 ID 别名）。
- **跨领域连线**：系统支持跨科学领域（物理、社会、工程）的前置依赖和连线。
- **循环依赖检测**：系统会自动扫描循环依赖，并区分“自我引用”和“复杂成环依赖”进行报告。
- **子树拆解**：系统会将图谱拆解，为每一个可用科技生成一棵以它为根节点、包含其后续解锁科技的子树。当某个节点被多个前置依赖时，额外的要求会以附注（如「Requires …」）呈现；同一棵子树中重复出现的节点只会展开一次，以防止图谱过长。

---

## 9. ASCII 依赖树渲染

最终的科技树需要嵌入到游戏内的文本框中。`dtt-core/render` 只负责 ASCII 连接线、层级、截断与节点位置，不感知 Stellaris 标记或输出语言；`dtt-stellaris/output` 通过节点格式化回调补充游戏链接、图标、颜色和本地化提示，再完成换行转义。这样既保持严格排版，也不让具体游戏格式泄漏到核心层。

### 9.1 字形定义

| 常量名称 | 字符序列 | 视觉构成 | 排版用途 |
|---|---|---|---|
| `TREE_BAR` | `"\|   "` | 竖线 + 3 空格 | 垂直延伸连接线（当前层有后续兄弟节点） |
| `TREE_EMPTY` | `"    "` | 4 空格 | 空白缩进填充（当前层已无后续兄弟节点） |
| `TREE_BRANCH` | `"\|-"` | 竖线 + 横杠 | 节点行首分支转折符 |
| `ELLIPSIS` | `"..."` | 3 个点号 | 树深度或同级数量超限时的截断标识 |

### 9.2 节点数据格式

Stellaris 输出适配层提供给核心渲染器的节点文本格式为：
```text
({tier})['technology:{base_id}', {area_icon}{color}${display_id}$§!]
```
`technology:` 链接使用基础科技 ID；可见名称使用替换后的 `display_id`。领域图标采用首个匹配变体声明的 `area`，否则沿用基础科技领域：物理 `£physics£`、社会 `£society£`、工程 `£engineering£`。颜色：危险科技 `§R`，稀有科技 `§M`，常规 `§W`。附加前置条件将在节点末尾追加如 `[§RRequires§! <Tech1> ...]` 的提示（`Requires` 按输出语言本地化；`any_of` 组写成 `(A OR B)`）。

### 9.3 头部尾部与特殊转义

生成的 `.yml` 本地化文本不能存在物理上的换行符，否则游戏引擎会报错。因此，在内存中拼装好树状图后，在输出前，**必须将所有物理换行符 `\n` 整体替换为字面的 `\` 和 `n` 两个字符组成的转义序列**。

### 9.4 显示上限防爆机制

为了防止庞大的科技树导致游戏卡死，系统配置了多重安全阈值（`RenderLimits`，CLI 使用默认值）：
1. 达到最大深度（默认 16）时停止下探。
2. 同级子节点超过限制（默认 64）时截断。
3. 树的总节点数达到全局限制（默认 4096）时终止渲染。
4. 超长根拦截：当直接相连的后续科技数量超过限制（默认 128）时，放弃渲染该树，仅保留错误提示。


---

## 10. 文件输出层

生成时，`dtt-application` 以当前可执行文件所在目录作为输出根目录，`dtt-stellaris/output` 针对用户指定的每种语言在本地化子目录下生成两个 `.yml` 文件，并在输出根目录生成一份诊断报告：

| 输出文件路径 | 文件类型 | 核心内容与用途 |
|---|---|---|
| `localisation/{lang}/zztechtreemain_l_{lang}.yml` | 主本地化 | 存储渲染生成的全量科技树 ASCII 文本结构 |
| `localisation/{lang}/replace/zztechtreereplaced_l_{lang}.yml` | 覆盖本地化 | 覆盖科技描述，追加科技树展示引用 |
| `dtt-save-report.txt` | 诊断报告 | 汇总资格判定、未知/过程性条件、循环与替换统计 |

**输出核心要求**：
- `.yml` 文件必须严格保存为带 BOM 的 UTF-8 编码（`utf-8-sig`），换行符为 LF。
- 生成文件直接写入当前可执行文件所在目录的 `localisation` 子目录，不再写入群星用户目录下的临时输出目录。
- 摄取本地化时会跳过文件名以 `zztechtree` 开头的已生成文件，避免把自身输出再读回去。
- 部分文件写入失败将返回 `incomplete` 状态，而非直接崩溃。

---

## 11. 工程代码结构

```text
project/
├── Cargo.toml                     # Rust workspace 与共享依赖
├── apps/
│   └── dtt-cli/                   # 薄命令行适配器
│       └── src/
│           ├── main.rs
│           ├── args.rs
│           └── commands/          # 参数映射、应用入口调用与终端展示
├── crates/
│   ├── dtt-core/                  # 纯领域模型与纯算法
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs           # 领域错误
│   │       ├── empire.rs          # 帝国快照
│   │       ├── condition.rs      # 通用语义模型、求值上下文接口与真假边界
│   │       ├── condition/
│   │       │   └── evaluation.rs # 可能性、确定性与控制流求值
│   │       ├── technology.rs     # 科技定义、目录与前置关系
│   │       ├── technology/
│   │       │   ├── eligibility.rs
│   │       │   └── swap.rs
│   │       ├── graph.rs           # 科技依赖图与循环算法
│   │       └── render.rs         # ASCII 科技树渲染
│   ├── dtt-stellaris/             # Stellaris 适配：按外部系统能力分组的公共库
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs            # 带路径与 #[source] 的结构化错误
│   │       ├── clausewitz.rs       # 私有 Jomini/Clausewitz 边界
│   │       ├── clausewitz/
│   │       │   ├── document.rs
│   │       │   ├── reader.rs
│   │       │   ├── value.rs       # 自有 Clausewitz 值树
│   │       │   └── script.rs      # Script IR（All/Any/Not/If/Scope/Call）
│   │       ├── paths.rs            # 路径发现公共 API
│   │       ├── paths/
│   │       │   ├── documents.rs
│   │       │   └── steam.rs
│   │       ├── save.rs             # 存档公共 API（Archive / Gamestate）
│   │       ├── save/
│   │       │   ├── container.rs
│   │       │   └── snapshot.rs
│   │       ├── load_order.rs       # 加载顺序公共 API 与 Manifest
│   │       ├── load_order/
│   │       │   ├── launcher.rs
│   │       │   ├── descriptor.rs
│   │       │   └── files.rs
│   │       ├── analysis.rs         # 世界视图与求值上下文
│   │       ├── analysis/
│   │       │   ├── scope.rs        # 对象身份、关系及作用域解析
│   │       │   └── policy.rs       # 触发器解释与长期展示策略
│   │       ├── game_data.rs        # load_game_data 公共 API
│   │       ├── game_data/
│   │       │   ├── diagnostic.rs
│   │       │   ├── graphical_culture.rs
│   │       │   ├── inline_script.rs
│   │       │   ├── lower.rs       # Script → CompiledCondition
│   │       │   ├── scripted_trigger.rs
│   │       │   ├── scripted_variable.rs
│   │       │   ├── technology.rs
│   │       │   └── technology/
│   │       │       └── definition.rs
│   │       ├── localisation.rs     # 本地化摄取公共 API
│   │       ├── localisation/
│   │       │   └── parser.rs
│   │       ├── output.rs           # 输出公共 API 与结果模型
│   │       └── output/
│   │           ├── language.rs
│   │           ├── render.rs
│   │           └── writer.rs
│   └── dtt-application/           # 共享应用入口与执行编排
│       └── src/
│           ├── lib.rs
│           ├── environment.rs     # 环境请求、结果、检测与配置发现
│           ├── save.rs            # 存档请求、结果与检查
│           ├── generation.rs      # 生成公共模型、设置与子模块
│           ├── generation/
│           │   ├── pipeline.rs    # 生成流水线编排
│           │   └── report.rs      # 报告模型、汇总与文本格式化
│           ├── progress.rs
│           ├── cancellation.rs
│           └── error.rs
└── docs/
    └── architecture.md            # 系统架构设计文档（本文）
```

依赖方向固定为 `dtt-cli -> dtt-application -> { dtt-core, dtt-stellaris }` 和 `dtt-stellaris -> dtt-core`。CLI 不直接依赖领域或 Stellaris 适配 crate；`dtt-core` 不依赖其他内部 package。

---

## 12. 命令行入口

`dtt-cli` 生成可执行文件 `dtt`，提供以下子命令：

- `dtt detect-paths`：打印自动发现的游戏目录、文档目录、启动器数据库和 Steam 库。
- `dtt generate <SAVE>`：读取非铁人文本存档并生成本地化文件；可通过 `--stellaris-root`、`--documents-dir` 与 `--launcher-db` 覆盖自动发现结果。`--documents-dir` 只用于定位 `launcher-v2.sqlite`，不作为输出目录。

输出语言通过可重复的 `--language <LANG>` 指定，默认 `english`。未知触发器与科技替换的处理策略分别由 `--unknown-strategy`（默认 `include-flagged`）和 `--swap-unknown-strategy`（默认 `keep-base`）控制。CLI 不暴露帝国 `country_id` 或渲染阈值覆盖。


## 国际化边界

`dtt-i18n` 是独立叶子 crate，提供 `AppLocale`、任务级 `Translator` 与类型化消息接口。CLI、应用层报告、Stellaris 输出适配器依赖它；纯领域层不依赖翻译。领域渲染返回 `RenderOutcome`，未知条件、摄取诊断、写出失败均传递枚举和参数。

产品语言是 `en` / `zh-Hans`，游戏输出是 `GameLanguage::{English, SimpChinese}`，两者独立。`RunGenerationRequest.presentation.report_locale` 在创建任务时捕获；磁盘报告经 `render_report` 渲染，后续界面语言变更不重写报告。游戏文案只使用 game 域，不注入 Fluent bidi 隔离符。

Tauri DTO 传递稳定错误码和带标签的诊断联合，技术细节单独展示。桌面设置仅接受当前 schema，不保留旧字段、旧语言枚举或迁移逻辑。维护规范见 [翻译贡献说明](i18n-translating.md)。
