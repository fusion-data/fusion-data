# Packages

这个目录用于存放可复用的库和组件包，是 Fusion Data 平台的 TypeScript/JavaScript SDK 生态系统。

每个子目录是一个独立的 npm 包，具有自己的 `package.json` 文件，支持 TypeScript 和现代模块系统。

## 当前包结构

```
packages/
├── fusion-core/      # 核心共享工具库
├── fusionsql/        # FusionSQL 数据库工具库
├── hetuflow-sdk/     # Hetuflow 工作流调度 SDK
└── hetumind-sdk/     # Hetumind AI 代理/流程 SDK
```

## 包详情

### @fusion-data/fusion-core
- **描述**: Fusion Data 项目的核心共享工具库
- **版本**: 1.0.0
- **主要功能**:
  - 基础工具函数
  - 时间处理工具 (`./time` 导出)
  - 类型定义
- **依赖**: 无外部依赖

### @fusion-data/fusionsql
- **描述**: FusionSQL 数据库工具库
- **版本**: 1.0.0
- **主要功能**:
  - 数据库操作工具
  - 分页工具 (`./page` 导出)
  - 操作符定义 (`./op` 导出)
- **依赖**:
  - `dayjs` - 时间处理
  - `@fusion-data/fusion-core` - 核心工具

### @fusion-data/hetuflow-sdk
- **描述**: Hetuflow TypeScript SDK，用于工作流调度 API 访问
- **版本**: 1.0.0
- **主要功能**:
  - Hetuflow API 客户端
  - 工作流管理
  - 任务调度接口
- **依赖**:
  - `axios` - HTTP 客户端
  - `@fusion-data/fusion-core`
  - `@fusion-data/fusionsql`
- **开发工具**: OpenAPI 类型生成

### @fusion-data/hetumind-sdk
- **描述**: Hetumind AI 代理/流程平台 TypeScript SDK
- **版本**: 1.0.0
- **主要功能**:
  - AI 代理管理
  - 工作流设计接口
  - LLM 集成工具
- **依赖**:
  - `axios` - HTTP 客户端
  - `dayjs` - 时间处理
  - `@fusion-data/fusion-core`
  - `@fusion-data/fusionsql`
- **开发工具**: OpenAPI 类型生成、ESLint 配置

## 开发指南

### 构建命令
所有包都支持以下标准命令：
```bash
pnpm build        # 构建 TypeScript
pnpm dev          # 开发模式（监听文件变化）
pnpm test         # 运行测试
pnpm lint         # 代码检查
pnpm format       # 代码格式化
pnpm clean        # 清理构建产物
```

### 创建新包

要创建新的包，请运行：

```bash
mkdir packages/your-package-name
cd packages/your-package-name
pnpm init
```

### 包命名规范
- 使用 `@fusion-data/` 命名空间
- 包名使用 kebab-case 格式
- 示例: `@fusion-data/my-new-sdk`

### 依赖管理
- 内部包依赖使用 `workspace:*` 版本
- 外部依赖指定具体版本号
- 开发依赖统一管理在 devDependencies 中

### 构建输出
- 所有包输出到 `dist/` 目录
- 支持 ESM 模块格式
- 包含 TypeScript 类型定义文件
- 配置适当的 `exports` 字段支持子路径导入
