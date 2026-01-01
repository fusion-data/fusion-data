# Fusion-Data LLM 索引

**Rust 数据融合平台** | v0.1.0 | [CLAUDE.md](./CLAUDE.md) | [docs](./docs/)

## 核心库速查

| 库 | 路径 | 用途 |
|---|---|---|
| [fusion-common](./crates/fusions/fusion-common/llm.md) | `fusion-common/` | 基础工具: Ctx, 加密, UUID, 时间 |
| [fusion-core](./crates/fusions/fusion-core/llm.md) | `fusion-core/` | 核心框架: Application, DI, Plugin, 配置 |
| [fusion-db](./crates/fusions/fusion-db/llm.md) | `fusion-db/` | 数据库层: ModelManager, DbPlugin |
| [fusion-web](./crates/fusions/fusion-web/llm.md) | `fusion-web/` | Web 层: Axum, Router, WebError |
| [fusion-ai](./crates/fusions/fusion-ai/llm.md) | `fusion-ai/` | AI 集成: LLM Provider, Agent, GraphFlow |
| [fusionsql](./crates/fusions/fusionsql/llm.md) | `fusionsql/` | ORM: BMC/Service, Filter, Pagination |

## 启动入口

```
hetuflow-server:  hetuflow/hetuflow-server/bin/hetuflow-server.rs
jieyuan-server:   jieyuan/jieyuan-server/bin/jieyuan-server.rs
hetumind-studio:  hetumind/hetumind-studio/bin/hetumind-studio.rs
```

## 通用模式

```rust
// 应用启动
Application::builder()
  .add_plugin(LogforthPlugin)
  .add_plugin(DbPlugin)
  .run().await?;

// 获取组件
let mm: ModelManager = Application::global().component();

// Web 服务
WebServerBuilder::new(router).with_shutdown(shutdown_rx).build().await?;

// AI Agent
let factory = ClientFactory::new();
let agent = factory.openai_agent(&config, &client)?;
```

## 文件命名规范

```
model/           # 实体定义 (User, UserForCreate, UserForUpdate)
bmc/             # 数据访问 (UserBmc)
service/         # 业务逻辑 (UserSvc)
endpoint/        # API 路由 (api/mod.rs, api/v1/users.rs)
```

## 快速导航

- 配置: `fusion-core/src/configuration/model/`
- 组件: `fusion-core/src/component/mod.rs`
- 插件: `fusion-core/src/plugin/mod.rs`
- 安全: `fusion-core/src/security/`
- 工作流: `fusion-ai/src/graph_flow/`
