# Hetumind Web

Hetumind Web 是 Hetumind Studio 的前端项目，为 AI Agent 开发和工作流编排提供可视化的 Web 界面。

## 技术栈

- **前端框架**: React 19.1.1
- **开发语言**: TypeScript 5.9.2
- **构建工具**: Vite 7.1.7
- **UI 框架**: Ant Design 5.27.4
- **路由管理**: React Router DOM 7.9.3
- **状态管理**: Zustand (轻量级状态管理)
- **拖拽系统**: @dnd-kit/core + @dnd-kit/sortable
- **可视化引擎**: React Flow (工作流画布)
- **代码编辑器**: @monaco-editor/react
- **样式方案**: CSS Modules + CSS Variables
- **开发工具**: ESLint + Prettier + TypeScript
- **数据获取与缓存**: @tanstack/react-query

## 开发环境要求

- **Node.js**: Version ≥ 22 (推荐使用 nvm 管理)
- **pnpm**: 包管理工具

## 快速开始

### 安装依赖

```bash
pnpm install
```

### 启动开发服务器

```bash
pnpm dev
```

应用将在 http://localhost:3001 启动

### 构建生产版本

```bash
pnpm build
```

### 预览生产版本

```bash
pnpm preview
```

### 代码检查和格式化

```bash
# 代码检查
pnpm lint

# 代码修复
pnpm lint:fix

# 类型检查
pnpm type-check
```

## 项目结构

```
src/
├── components/           # 通用组件
│   ├── layout/          # 布局组件
│   ├── workflow/        # 工作流相关组件
│   ├── agent/           # AI Agent 相关组件
│   ├── ui/              # 基础 UI 组件
│   └── common/          # 通用业务组件
├── pages/               # 页面组件
│   ├── Dashboard/       # 仪表板
│   ├── Workflows/       # 工作流管理
│   ├── Agents/          # AI 智能体管理
│   ├── Settings/        # 设置页面
│   └── Login/           # 登录页面
├── hooks/               # 自定义 Hooks
├── stores/              # 状态管理 (Zustand)
├── contexts/            # React Context
├── types/               # TypeScript 类型定义
├── utils/               # 工具函数
├── services/            # API 服务
├── styles/              # 样式文件
└── assets/              # 静态资源
```

## 核心功能

### 1. 可视化工作流编辑器
- 基于 React Flow 的强大画布系统
- 支持拖拽式节点编辑
- 实时连接和配置
- 多种节点类型支持

### 2. AI Agent 开发平台
- 直观的智能体配置界面
- 支持多种 AI 模型
- 提示词编辑和测试
- 执行历史记录

### 3. 现代化主题系统
- 支持亮色/暗色主题切换
- 多种色彩方案
- 系统主题自动跟随
- 平滑过渡动画

### 4. 模块化架构
- 清晰的组件分层
- TypeScript 类型安全
- 可扩展的插件系统
- 高性能渲染

## 开发指南

### 添加新的节点类型

1. 在 `src/types/node.ts` 中定义节点数据类型
2. 在 `src/components/workflow/Canvas/NodeTypes/` 中创建节点组件
3. 在节点注册中心注册新节点类型

### 添加新页面

1. 在 `src/pages/` 中创建页面组件
2. 在 `src/App.tsx` 中添加路由配置
3. 更新导航菜单（如果需要）

### 主题定制

1. 在 `src/styles/themes.css` 中添加新的颜色方案
2. 在 `src/contexts/ThemeContext.tsx` 中更新主题配置

## API 集成

项目通过代理配置与后端 API 集成：

- 开发环境：`http://localhost:9501` (hetumind-studio)
- 生产环境：根据部署配置

## 贡献指南

1. Fork 项目
2. 创建功能分支
3. 提交更改
4. 推送到分支
5. 创建 Pull Request

## 许可证

Apache-2.0 License