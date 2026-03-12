# Fusion-Data

Rust 数据融合平台 v0.1.0 (Rust 2024, Apache-2.0)

## 项目模块

| 模块         | 职责                               | 技术栈             |
| ------------ | ---------------------------------- | ------------------ |
| **hetus**    | 核心库套件 (common/core/web/db/ai) | Tokio, Axum        |
| **hetuflow** | 分布式任务调度和工作流编排         | 任务队列, 调度器   |
| **hetumind** | AI Agent/Flow 平台                 | LLM providers, rig |
| **jieyuan**  | 模块化 IAM 访问控制                | OAuth 2.0, JWT     |

## 技术栈

- **后端**: Rust 1.90+, Tokio, Axum, sqlx, sea-query
- **前端**: React 19, TypeScript 5.x, Ant Design, @xyflow/react
- **数据库**: PostgreSQL, Redis (任务队列)
- **构建**: pnpm workspace, Vite

## 快速开始

```bash
# 环境要求
Rust >=1.90, Node.js >=22, pnpm, Docker Compose

# 后端构建
cargo check && cargo build && cargo fmt
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 前端构建
pnpm install && pnpm build

# 运行服务
cargo run --bin hetuflow-server   # 工作流引擎
cargo run --bin hetumind-studio   # AI Studio
cargo run --bin jieyuan-server    # IAM 服务

# 数据库
docker-compose up -d
```

## 核心原则

- **零 Unsafe**: workspace lint 强制
- **类型安全**: 强编译时保证
- **异步一致**: Tokio async/await
- **配置驱动**: TOML 配置文件
- **模块化**: 关注点分离

## Claude Skills

使用 Skills 获取详细技术指导：

| Skill          | 用途                    |
| -------------- | ----------------------- |
| `rust-backend` | Rust 后端开发、组件架构 |
| `sql-database` | ORM、BMC/Service 模式   |
| `web-frontend` | React 前端、工作流画布  |
| `cluster-node` | AI 节点、集群架构       |

**重要**: 修改代码前先使用对应 Skill 获取模式指导。

## 认证架构

- **Jieyuan**: OAuth 2.0 + PKCE + 基于策略的授权
- **Hetumind**: 认证代理重定向到 Jieyuan
- **多租户**: 基于 `tenant_id` 数据隔离
