# 前端开发说明

## 技术选型与组件规范

桌面端采用 React 19、TypeScript、Vite 和 Tauri 2。界面组件以 shadcn/ui 为基础，使用 Base UI 交互原语、Tailwind CSS 4 样式和 Lucide 图标。

新增通用组件时，从仓库根目录执行 `pnpm --filter dtt-desktop exec shadcn add <组件名>`，再按产品需要修改生成的本地源码。业务组件统一组合 `src/components/ui` 中的组件，图标统一使用 Lucide。仓库只保留产品实际使用的组件源码；`components.json` 用于配置生成规则，已安装组件以源码目录为准。

## 目录与配置

以下路径均相对于 `apps/dtt-desktop`。

| 路径 | 作用 |
| --- | --- |
| `components.json` | 配置 Base UI、Nova 样式、Lucide、CSS 变量和导入别名。 |
| `src/index.css` | 引入 Tailwind、shadcn 主题工具、动画和 Geist 字体，定义亮色、暗色主题及全局样式。 |
| `src/lib/utils.ts` | 提供统一的 `cn` 类名合并函数。 |
| `tsconfig.json`、`vite.config.ts` | 配置 `@/*` 到 `src/*` 的别名、类型检查和构建；Vite 同时接入 Tailwind 插件。 |
| `src/app` | 应用入口、页面布局、会话状态、主题和设置持久化。 |
| `src/features` | 设置、存档与帝国、生成、结果四个页面。 |
| `src/components` | 页面共用组件及 `ui` 基础组件。 |
| `src/ipc/commands.ts`、`src/ipc/errors.ts` | 封装 Tauri 命令、进度通道和错误归一化。 |
| `src/ipc/bindings` | 从 Rust DTO 导出的 TypeScript 类型。 |
| `src/i18n` | 翻译资源、语言协商、状态控制、格式器和相关测试。 |
| `src-tauri/src` | Rust 命令、DTO、错误转换和生成任务状态。 |

## 页面状态与 IPC

`src/app/session.ts` 管理设置、存档扫描、帝国检查和生成结果。环境解析、存档扫描和检查请求通过序号识别最新响应。生成状态涵盖空闲、执行、取消中、完成、已取消和失败。

`src/ipc/commands.ts` 封装七个命令：`get_bootstrap_data`、`resolve_environment`、`scan_save_library`、`inspect_save`、`run_generation`、`cancel_generation`、`open_output_directory`。生成进度由 Tauri `Channel` 传递。Rust 侧通过应用层完成业务操作，同一时刻仅运行一个生成任务；取消在生成阶段切换时生效。

Rust DTO 是 IPC 类型的维护入口。修改 DTO 后，在仓库根目录运行 `pnpm bindings` 更新 `src/ipc/bindings`。

## 设置、语言与初始化

`src/app/store.ts` 将桌面设置保存到 `dtt-desktop.json` 的 `preferences` 项；浏览器预览使用同名 localStorage 项。当前设置版本为 `schemaVersion: 2`，其他版本回退到默认设置。路径覆盖、输出语言、分析策略、界面语言、主题和存档来源均经过字段校验后保存，写入请求按顺序执行。

默认设置使用英语游戏输出、`include_flagged` 未知策略、`keep_base` 变体策略、本地存档来源，以及跟随系统的界面语言和主题。

`src/main.tsx` 先读取并校验设置，再调用 `bootstrapLocale` 初始化翻译、设置文档语言和窗口标题，随后尝试保存校验结果并挂载 React。保存失败状态会传给应用界面。

界面使用 i18next / react-i18next 的 selector API 和生成的资源字面量类型。`src/i18n/controller.ts` 集中维护 `system | en | zh-Hans | ja | ru` 偏好、实际语言和保存失败状态，同时响应系统语言变化。界面语言与游戏输出语言分别保存和选择。

`Intl` 格式器按显式 locale 和 options 缓存。错误与诊断以结构化数据保存在界面状态中，切换语言时直接重新翻译现有数据。磁盘报告采用任务创建时选定的产品语言。

翻译资源位于 `src/i18n/locales/` 下的 `en`、`zh-Hans`、`ja` 和 `ru` 目录。修改资源后，运行 `pnpm i18n:generate` 更新类型，再运行 `pnpm i18n:check` 检查各语言的键、参数、复数、空值、重复键、闲置键和硬编码标签。

## 运行时直接依赖

下表对应 `package.json` 的 `dependencies`。添加组件或调整依赖时，同步维护用途说明。

| 依赖 | 用途 |
| --- | --- |
| `@base-ui/react` | 提供组件交互和无障碍基础能力。 |
| `@fontsource-variable/geist` | 随应用提供 Geist 可变字体。 |
| `@fontsource-variable/geist-mono` | 随应用提供等宽字体，用于路径、游戏 ID 和诊断。 |
| `@tauri-apps/api` | 访问 Tauri 窗口、IPC 和应用能力。 |
| `@tauri-apps/plugin-dialog` | 调用原生文件和目录选择对话框。 |
| `@tauri-apps/plugin-store` | 持久化桌面设置。 |
| `class-variance-authority` | 管理组件样式变体。 |
| `clsx` | 按条件组合 CSS 类名。 |
| `i18next` | 管理翻译资源、语言切换和插值。 |
| `lucide-react` | 提供统一的 React 图标组件。 |
| `react` | 提供组件和状态模型。 |
| `react-dom` | 将 React 组件挂载到 WebView DOM。 |
| `react-i18next` | 将翻译能力接入 React，并随语言变化更新组件。 |
| `shadcn` | 提供组件 CLI，以及 `shadcn/tailwind.css` 中的主题工具和状态变体。 |
| `tailwind-merge` | 合并 Tailwind 类名并处理样式冲突。 |
| `tw-animate-css` | 提供组件状态切换动画。 |

## 开发与构建直接依赖

下表对应 `package.json` 的 `devDependencies`。

| 依赖 | 用途 |
| --- | --- |
| `@tailwindcss/vite` | 扫描源码并编译 Tailwind CSS。 |
| `@tauri-apps/cli` | 启动、构建和打包桌面应用。 |
| `@types/node` | 提供 Node.js API 类型。 |
| `@types/react` | 提供 React 类型。 |
| `@types/react-dom` | 提供 React DOM 类型。 |
| `@vitejs/plugin-react` | 接入 React Fast Refresh 和 JSX 转换。 |
| `jsonc-parser` | 在翻译检查中解析 JSON 并检查重复键。 |
| `tailwindcss` | 编译工具类、主题变量和样式指令。 |
| `typescript` | 执行静态类型检查，产物构建交给 Vite。 |
| `vite` | 提供开发服务器和生产资源构建。 |
| `vitest` | 运行前端测试。 |

## 常用命令

仓库通过根目录 `package.json` 固定 pnpm 版本。以下命令均从仓库根目录执行。

| 命令 | 用途 |
| --- | --- |
| `pnpm install` | 安装 workspace 的前端依赖。 |
| `pnpm desktop:dev` | 启动 Tauri 开发应用及前端开发服务器。 |
| `pnpm --filter dtt-desktop dev` | 启动 Vite，用于浏览器预览界面。 |
| `pnpm desktop:build` | 运行 `tsc --noEmit` 并构建前端资源。 |
| `pnpm desktop:tauri build` | 构建和打包 Tauri 桌面应用。 |
| `pnpm bindings` | 从 Rust DTO 重新导出 TypeScript 类型。 |
| `pnpm i18n:generate` | 更新翻译资源类型。 |
| `pnpm i18n:check` | 执行翻译资源和文案检查。 |
| `pnpm desktop:test` | 运行 Vitest 测试。 |

当前前端测试主要覆盖语言初始化、并发切换、保存失败、设置校验、格式化，以及错误和诊断的重新翻译。系统流程与 Rust 验证命令见 [系统架构](architecture.md)。
