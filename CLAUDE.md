# CLAUDE.md

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

**集群节点架构**:
```rust
// 节点注册模式
let node_registry = NodeRegistry::new();
let deepseek_node = Arc::new(DeepseekModelNode::new()?);
node_registry.register_node(deepseek_node.clone())?;

// 注册 SubNodeProvider
for s in deepseek_node.node_suppliers() {
  node_registry.register_subnode_provider(deepseek_node.kind(), s.clone())?;
}

// 类型化供应器注册
let typed_supplier: LLMSubNodeProviderRef = Arc::new(DeepseekModelSupplier::new());
node_registry.register_llm_supplier(deepseek_node.kind(), typed_supplier)?;

// 工作流执行中的使用
let engine = DefaultWorkflowEngine::new(node_registry, execution_store);
let result = engine.execute_workflow(workflow_id, context).await?;
```

**核心组件**:
- **NodeRegistry**: 统一节点注册表，支持类型化供应器 (LLM/Memory/Tool/Agent)
- **SubNodeProvider**: 类型化的 AI 能力接口 (LLMSubNodeProvider, MemorySubNodeProvider)
- **DefaultWorkflowEngine**: 工作流执行引擎，集成 SubNodeProvider 调用
- **EngineRouter**: 统一请求路由器，处理节点工具请求
- **GraphFlow**: 任务执行框架与集群节点执行器集成

**实际使用模式**:
```rust
// 类型安全的供应器获取
pub fn get_llm_supplier_typed(registry: &NodeRegistry, kind: &NodeKind) -> Option<LLMSubNodeProviderRef>
pub fn get_simple_memory_supplier_typed(registry: &NodeRegistry) -> Option<MemorySubNodeProviderRef>

// 工作流执行中的内存注入
if let Some(mem_supplier) = get_simple_memory_supplier_typed(&self.node_registry) {
  let mem_msgs = mem_supplier.retrieve_messages(session_id, history_count).await.unwrap_or_default();
  // 注入历史上下文到输入数据
}

// 节点产生的 EngineRequest 路由
engine.route_engine_requests(&mut output_data, &context).await?;
```

## 开发规范

### 编码风格
- Rust 2024 edition，现代语法 (let-chains, if-let guards)
- 2 空格缩进，120 字符行宽
- 使用 `cargo clippy` 确保代码现代化

### 开发原则
- **代码复用优先**: 扩展现有代码而非重写
- **可执行性**: 避免伪代码，确保代码可直接运行
- **模块化实现**: 按模块逐步实现，避免一次性大段代码
- **注释精简**: 对关键实现添加简短注释
- **需求确认**: 遇到模糊需求时先确认再编码
- **测试验证**: 用简洁可执行的测试样例验证功能
- **重构空间**: 保留重构空间，指出未来优化点

### Rust 最佳实践

```rust
// Arc 和 Clone - 先克隆再移动
let provider = Arc::new(MyProvider::new());
let provider_for_registry = provider.clone();
registry.register(provider_for_registry).await?;

// Option/Result 处理 - 使用 ? 操作符
let registered_node = node_registry.get_executor(&node_kind)
    .ok_or_else(|| NodeExecutionError::ConfigurationError("Node not found".to_string()))?;

// 字符串清理 - 完整替换特殊字符
fn sanitize_identifier(input: &str) -> String {
    input.replace('.', "_").replace('@', "_").replace('#', "_")
         .replace(' ', "_").to_lowercase()
}

// BMC 层模式 - 第一个参数是 &ModelManager，不用 &self
impl UserBmc {
    pub fn get_by_id(mm: &ModelManager, id: UserId) -> Result<Option<UserEntity>, SqlError> { /* */ }
}

// 服务层 - 必须实现 Clone，使用直接引用
#[derive(Clone)]
pub struct WorkflowSvc {
    mm: ModelManager,  // ✅ 直接引用
    auth_svc: AuthSvc, // ✅ 直接引用
}
```

## FusionSQL ORM 模式

**文件结构** (4文件模式):
1. `{entity}_entity.rs` - 数据库实体 (Fields derive)
2. `{entity}_model.rs` - 请求/响应模型、过滤器
3. `{entity}_bmc.rs` - 数据库模型控制器 (CRUD)
4. `{entity}_svc.rs` - 业务逻辑服务

**规则**: Entity/Model 可在 `xxx-core` 中，BMC/Service 仅在二进制应用中

```rust
// 查询过滤器和分页
use fusionsql::{filter::{OpValString, OpValInt32}, FilterNodes, page::Page};

#[derive(Debug, Clone, Default, Deserialize, FilterNodes)]
pub struct CredentialFilter {
    pub namespace_id: Option<OpValString>,
    pub name: Option<OpValString>,
    pub kind: Option<OpValInt32>,
}

// 实体定义
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "credential_entity")]
pub struct CredentialEntity {
    pub id: CredentialId,
    pub namespace_id: String,
    pub name: String,
    pub data: String,
    pub kind: CredentialKind,
    pub is_managed: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: Option<OffsetDateTime>,
    pub created_by: i64,
    pub updated_by: Option<i64>,
    pub logical_deletion: Option<OffsetDateTime>,
}
```

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

## 数据库与多租户

**租户 ID 指南**:
- **主要表** (需要 tenant_id): `user_entity`, `project`, `workflow_entity`, `execution_entity`, `credential_entity`
- **次要表** (无 tenant_id): `sched_task_instance`, `sched_agent`, `execution_data`, `migrations`
- **IAM 表**: `iam_resource_mapping`, `policy_entity` (租户范围)

**命名约定**:
- 主键: `{table_name}_pk`
- 外键: `{table_name}_fk_{column_name}`
- 索引: `{table_name}_idx_{column_name}`
- 唯一: `{table_name}_uidx_{column_name}`

## API 开发

```rust
// 服务层模式
impl FromRequestParts<Application> for CredentialSvc {
    type Rejection = WebError;
    async fn from_request_parts(parts: &mut Parts, state: &Application) -> core::result::Result<Self, Self::Rejection> {
        let ctx = extract_ctx(parts, state.fusion_setting().security())?;
        let mm = state.component::<ModelManager>().with_ctx(ctx);
        Ok(CredentialSvc { mm, key_manager })
    }
}

// 中间件集成
.route("/workflows", workflows::routes()
  .layer(AsyncRequireAuthorizationLayer::new(WebAuth::default())))
  .layer(tenant_middleware_layer())
```

## 性能优化

```rust
// 使用 ahash 提升性能 (2-3x)
use fusion_common::ahash::{HashMap, HashSet};
let mut map = HashMap::default();
let mut set = HashSet::default();
```

## 测试与调试

```bash
# 测试命令
cargo test -p <crate-name> <test_name> -- --nocapture
cargo test -p <crate-name> --lib --no-run

# 编译检查
cargo check -p hetumind-studio
cargo clippy --workspace --all-targets --all-features --no-deps -- -D warnings
```

**日志**: 使用 `log` crate，开发环境 `RUST_LOG=debug`，生产环境 `RUST_LOG=info`

## 核心原则

- **零 Unsafe 代码**: 工作空间 lint 强制
- **安全优先**: 审查依赖
- **类型安全**: 强编译时保证
- **异步一致**: Tokio async/await
- **配置驱动**: TOML 配置文件
- **模块化架构**: 关注点分离
- **性能优化**: LTO 构建优化
