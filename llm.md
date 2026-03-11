# Fusion-Data LLM 索引

**Rust 数据融合平台** | v0.1.0 | [CLAUDE.md](./CLAUDE.md) | [docs](./docs/)

## 核心库速查

| 库                                         | 路径           | 用途                                    |
| ------------------------------------------ | -------------- | --------------------------------------- |
| [hetu-common](./crates/hetu-common/llm.md) | `hetu-common/` | 基础工具: Ctx, 加密, UUID, 时间         |
| [hetu-core](./crates/hetu-core/llm.md)     | `hetu-core/`   | 核心框架: Application, DI, Plugin, 配置 |
| [hetu-db](./crates/hetu-db/llm.md)         | `hetu-db/`     | 数据库层: ModelManager, DbPlugin        |
| [hetu-web](./crates/hetu-web/llm.md)       | `hetu-web/`    | Web 层: Axum, Router, WebError          |
| [hetu-ai](./crates/hetu-ai/llm.md)         | `hetu-ai/`     | AI 集成: LLM Provider, Agent, GraphFlow |
| [hetusql](./crates/hetusql/llm.md)         | `hetusql/`     | ORM: BMC/Service, Filter, Pagination    |

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

- 配置: `hetu-core/src/configuration/model/`
- 组件: `hetu-core/src/component/mod.rs`
- 插件: `hetu-core/src/plugin/mod.rs`
- 安全: `hetu-core/src/security/`
- 工作流: `hetu-ai/src/graph_flow/`
