# CLAUDE.md

**Fusion-Data** - Rust 数据融合平台 v0.1.0 (Rust 2024, Apache-2.0)

## 项目架构

**核心模块**:
- **fusionsql**: sea-query/sqlx ORM 数据库抽象层
- **hetuflow**: 分布式任务调度和工作流编排 ("河图流动")
- **hetumind**: AI Agent/Flow 平台 ("河图智思")
- **jieyuan**: 模块化 IAM 访问控制 ("界垣")
- **fusions**: 核心库套件

**技术栈**: Rust 1.90+, Tokio, Axum, PostgreSQL, React 19, TypeScript, pnpm workspace

## 快速开始

```bash
# 环境要求
Rust >=1.90, Node.js >=22, pnpm, Docker Compose

# 后端构建
cargo check && cargo build && cargo fmt
cargo clippy --workspace --all-targets --all-features --no-deps -- -D warnings

# 前端构建
pnpm install && pnpm build

# 运行服务
cargo run --bin hetuflow-server      # 工作流引擎
cargo run --bin hetumind-studio      # AI Studio
cargo run --bin jieyuan-server        # IAM 服务

# 启动数据库
docker-compose up -d && docker-compose logs -f
```

## Claude Skills 使用指南

### 何时使用 Skills

**skill: rust-backend** - Rust 后端开发
- 编写新的 Rust 服务、API 接口、业务逻辑
- 内存管理、错误处理、性能优化
- 组件架构、依赖注入、配置管理
- 使用时机: 修改 `crates/` 下任何 Rust 代码

**skill: sql-database** - 数据库 ORM 开发
- 创建新的数据库实体、查询接口
- 实现 BMC/Service 层数据操作
- 多租户数据隔离、分页查询
- 使用时机: 修改数据库 schema 或数据访问代码

**skill: web-frontend** - React/TypeScript 前端开发
- 开发 React 19 组件、状态管理
- Ant Design UI 集成、工作流画布
- API 客户端、实时数据同步
- 使用时机: 修改 `hetuflow-web/` 或 `hetumind-web/`

**skill: cluster-node** - AI 节点集群架构
- 实现 LLM/Memory/Tool 供应商
- 工作流引擎集成、请求路由
- 集群节点注册与发现
- 使用时机: 开发 AI 节点或工作流执行引擎

### 认证授权架构

**Jieyuan**: OAuth 2.0 + PKCE + 基于策略的授权
**Hetumind**: 认证代理重定向到 Jieyuan
**多租户安全**: 基于租户的数据隔离
**IAM 资源映射**: 基于路径模式的零配置权限控制

认证流程: 用户请求 → Hetumind 重定向 → Jieyuan OAuth → JWT 令牌 → 租户中间件 → 权限访问

## 核心原则

- **零 Unsafe 代码**: 工作空间 lint 强制
- **安全优先**: 审查依赖
- **类型安全**: 强编译时保证
- **异步一致**: Tokio async/await
- **配置驱动**: TOML 配置文件
- **模块化架构**: 关注点分离
- **Web 搜索**: 使用 web-search-prime MCP 进行网络搜索

**重要**: 总是先使用对应的 skill 获取详细的技术指导，确保代码符合项目规范和最佳实践。
