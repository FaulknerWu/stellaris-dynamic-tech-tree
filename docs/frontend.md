# 前端 UI 基线与依赖说明

本文档只描述技术选型与依赖。页面结构、交互规则与视觉规范见 [`docs/frontend-design.md`](frontend-design.md)。

## 1. 技术选型

- 组件基线：shadcn/ui；
- 组件原语：Base UI；
- 图标库：Lucide；
- 样式引擎：Tailwind CSS 4；

### 1.1 开发约束

- 新增通用 UI 组件时，必须先从仓库根目录执行 `pnpm --filter dtt-desktop exec shadcn add <组件名>`，再按产品需要修改生成的本地源码。
- 不得引入其他成套组件库，也不得复制其他组件库的组件作为并行基线。业务组件应组合 `src/components/ui` 中的 shadcn/ui 组件。
- `components.json` 只描述生成规则，不代表组件已经安装。未被产品使用的组件源码不应保留。
- 图标统一使用 Lucide，避免引入视觉语言不一致的并行图标集。

## 2. shadcn/ui 基础设施

| 文件 | 作用 |
| --- | --- |
| `apps/dtt-desktop/components.json` | 固定 Base UI、Nova、Lucide、CSS 变量、组件目录和导入别名等生成规则。 |
| `apps/dtt-desktop/src/index.css` | 引入 Tailwind CSS、shadcn 主题工具、动画工具与 Geist 字体，并定义亮色、暗色设计令牌及全局基础样式。 |
| `apps/dtt-desktop/src/lib/utils.ts` | 提供 shadcn/ui 组件统一使用的 `cn` 类名合并函数。 |
| `apps/dtt-desktop/tsconfig.json` | 为 TypeScript 声明 `@/*` 到 `src/*` 的路径映射。 |
| `apps/dtt-desktop/vite.config.ts` | 为 Vite 声明相同的运行时路径别名，并接入 Tailwind CSS 插件。 |

## 3. 运行时直接依赖

以下清单对应 `apps/dtt-desktop/package.json` 的 `dependencies`。shadcn CLI 后续添加不同组件时，可能按组件源码的实际需要增加依赖；新增项必须同步补充到本文档。

| 依赖 | 作用 |
| --- | --- |
| `@base-ui/react` | 提供 shadcn/ui 组件所需的无样式无障碍交互原语（Base UI）。 |
| `@fontsource-variable/geist` | 在应用包内提供 Nova 预设使用的 Geist 可变字体，避免依赖远程字体服务。 |
| `@fontsource-variable/geist-mono` | 在应用包内提供 Nova 预设使用的 Geist Mono 可变等宽字体，用于路径、游戏 ID 与诊断展示。 |
| `@tauri-apps/api` | 提供 Tauri WebView 前端访问窗口、事件和应用能力的 JavaScript API。 |
| `@tauri-apps/plugin-dialog` | 调用 Tauri 原生文件选择、保存和消息对话框。 |
| `@tauri-apps/plugin-store` | 在桌面端持久化轻量键值配置。 |
| `class-variance-authority` | 提供类型安全且可组合的组件变体样式变体管理（cva）。 |
| `clsx` | 按条件组合 CSS 类名，是 `cn` 工具的输入归一化层。 |
| `i18next` | 国际化核心，负责资源、语言切换、插值和翻译解析。 |
| `lucide-react` | 提供 shadcn/ui 配置指定的 React 图标组件。 |
| `react` | 前端组件与状态模型的运行时。 |
| `react-dom` | 将 React 组件树挂载到 WebView DOM。 |
| `react-i18next` | 将 i18next 接入 React，提供 Hook、组件更新和上下文集成。 |
| `shadcn` | 提供组件 CLI，并通过 `shadcn/tailwind.css` 提供当前 shadcn/ui 所需的 Tailwind 主题工具和状态变体。 |
| `tailwind-merge` | 按 Tailwind 规则消解冲突的工具类，是 `cn` 工具的冲突处理层。 |
| `tw-animate-css` | 提供 shadcn/ui 组件状态切换所需的 Tailwind 动画工具类。 |

## 4. 开发与构建直接依赖

以下清单对应 `apps/dtt-desktop/package.json` 的 `devDependencies`。

| 依赖 | 作用 |
| --- | --- |
| `@tailwindcss/vite` | 在 Vite 开发与生产构建中扫描源码并编译 Tailwind CSS 4。 |
| `@tauri-apps/cli` | 启动、构建和打包 Tauri 桌面应用。 |
| `@types/node` | 为 Vite 配置中使用的 Node.js API 和环境变量提供 TypeScript 类型。 |
| `@types/react` | 提供 React 的 TypeScript 类型声明。 |
| `@types/react-dom` | 提供 React DOM 的 TypeScript 类型声明。 |
| `@vitejs/plugin-react` | 将 React Fast Refresh 与 JSX 转换接入 Vite。 |
| `tailwindcss` | 解析工具类、设计令牌和样式指令，生成 shadcn/ui 最终使用的 CSS。 |
| `typescript` | 对前端源码执行静态类型检查；项目构建时不由 TypeScript 输出文件。 |
| `vite` | 提供前端开发服务器和生产资源构建。 |

## 5. 权威资料

- [shadcn/ui 简介](https://ui.shadcn.com/docs)
- [shadcn/ui 的 Vite 安装说明](https://ui.shadcn.com/docs/installation/vite)
- [Tailwind CSS 的 Vite 安装说明](https://tailwindcss.com/docs/installation/using-vite)
- [Base UI 快速开始](https://base-ui.com/react/overview/quick-start)
- [Tauri JavaScript API](https://tauri.app/reference/javascript/api/)
