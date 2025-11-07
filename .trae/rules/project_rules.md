# AGENTS

**Fusion-Data** - Rust 数据融合平台 v0.1.0 (Rust 2024, Apache-2.0)

## 核心架构

**应用模块**:
- **fusionsql**: sea-query/sqlx ORM 数据库抽象层
- **hetuflow**: 分布式任务调度和工作流编排 ("河图流动")
- **hetumind**: AI Agent/Flow 平台 ("河图智思")
- **jieyuan**: 模块化 IAM 访问控制 ("界垣")
- **fusions**: 核心库套件

**技术栈**: Rust 1.90+, Tokio, Axum, PostgreSQL/SQLite, React 19, TypeScript, pnpm workspace

## 快速开始

```bash
# 环境
Rust >=1.90, Node.js >=22, pnpm, Docker Compose

# 构建
cargo check && cargo build && cargo fmt
cargo clippy --workspace --all-targets --all-features --no-deps -- -D warnings

# 运行
cargo run --bin hetuflow-server
cargo run --bin hetumind-studio
cargo run --bin jieyuan-server

# 数据库
docker-compose up -d && docker-compose logs -f
```

## 核心设计模式

**配置**: TOML 文件, `FUSION_CONFIG_FILE` 环境变量
**异步**: Tokio 多线程运行时，优雅关闭
**数据库**: sea-query + SQLx ORM，多租户支持
**错误处理**: 自定义 Result<T> + DataError
**组件系统**: 依赖注入，生命周期管理
**API**: Axum + Tower，FromRequestParts 提取，多租户中间件

## Claude Skills

开发过程中使用以下专门的 Claude Skills 获取详细指导：

- **rust-backend-development**: Rust 核心开发模式、内存管理、错误处理、API 开发、性能优化
- **fusion-sql-orm**: FusionSQL ORM 四文件结构、实体定义、查询过滤器、BMC/Service 模式、多租户支持
- **cluster-node-architecture**: 集群节点架构、NodeRegistry、SubNodeProvider 模式、类型化供应商、工作流引擎集成

## 认证授权

**IAM 架构**:
- **Jieyuan**: OAuth 2.0 + PKCE + 基于策略的授权
- **Hetumind**: 认证代理重定向到 Jieyuan
- **多租户安全**: 基于租户的数据隔离
- **IAM 资源映射**: 基于路径模式的零配置权限控制

```rust
use jieyuan::access_control::{AuthSvc, PolicySvc};
use jieyuan::oauth::OAuthSvc;

// 认证
let auth_svc = AuthSvc::new(user_svc);
let response = auth_svc.signin(request).await?;

// OAuth
let oauth_svc = OAuthSvc::new(model_manager, app);
let auth_response = oauth_svc.authorize(oauth_request).await?;
```

**认证流程**: 用户请求 → Hetumind 重定向 → Jieyuan OAuth → JWT 令牌 → 租户中间件 → 权限访问

## 核心原则

- **零 Unsafe 代码**: 工作空间 lint 强制
- **安全优先**: 审查依赖
- **类型安全**: 强编译时保证
- **异步一致**: Tokio async/await
- **配置驱动**: TOML 配置文件
- **模块化架构**: 关注点分离
- **性能优化**: LTO 构建优化
- **Web 搜索**: 总是使用 web-search-prime MCP 进行网络搜索和访问网页
