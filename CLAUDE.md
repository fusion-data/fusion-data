# CLAUDE.md

## Repository Overview

**fusion-data** platform - comprehensive data fusion platform built with Rust:

- **fusionsql**: Database abstraction layer with sea-query/sqlx ORM
- **hetuflow**: Distributed task scheduling and workflow orchestration ("河图流动")
- **hetumind**: AI Agent/Flow platform with LLM integration ("河图智思")
- **jieyuan**: Access control and authentication with modular IAM ("界垣")
- **fusions**: Core library suite

**Version**: 0.1.0 | **Rust Edition**: 2024 | **License**: Apache-2.0

## Development Environment

### Prerequisites

- **Rust**: ≥1.90 (rustup, rsproxy for Chinese users)
- **Node.js**: ≥22 (nvm)
- **pnpm**: Package manager
- **Docker & Docker Compose**: Services
- **Git**: Version control

### Common Commands

```bash
# Build & Check
cargo check
cargo build
cargo fmt
cargo clippy --workspace --all-targets --all-features --no-deps -- -D warnings
cargo test

# Run Applications
cargo run --bin hetuflow-server
cargo run --bin hetuflow-agent
cargo run --bin hetumind-studio
cargo run --bin hetumind-cli
cargo run --bin jieyuan-server

# Database Services
docker-compose up -d      # Start services
docker-compose ps         # Check status
docker-compose logs -f    # Follow logs
docker-compose down       # Stop services
```

## Architecture Overview

### Workspace Structure

**Core Libraries (`crates/fusions/`)**:

- `fusion-core`: Application framework, components, config, async runtime
- `fusion-web`: HTTP server (Axum-based)
- `fusion-db`: Database access layer
- `fusion-grpc`: gRPC utilities
- `fusion-security`: Security components
- `fusion-common`: Shared utilities
- `fusion-ai`: AI integration
- `fusion-core-macros`: Derive macros

**Applications**:

- **hetuflow**: Distributed task scheduling (server, agent, web)
- **hetumind**: AI Agent platform (core, nodes, studio, cli)
- **fusionsql**: Database ORM (core, main, macros)
- **jieyuan**: Access control (core, server with OAuth + IAM)

### Key Patterns

**Configuration**: TOML-based, `FUSION_CONFIG_FILE` env var, structured types

**Async Runtime**: Tokio-based, multi-threaded, graceful shutdown

**Database**: sea-query + SQLx, model-based ORM, PostgreSQL/SQLite, vector support, multi-tenant

**Error Handling**: Custom `Result<T>` with `DataError`, structured types, anyhow integration

**Component System**: Dependency injection, lifecycle management, service registration

**API Development**: Axum + Tower, `FromRequestParts` extraction, multi-tenant middleware, fine-grained permissions

### Cluster Node Architecture

**Core Components**:

- **SubNodeProvider**: Unified interface for AI capabilities (LLM, Memory, Agent)
- **ClusterNodeManager**: Centralized management
- **NodeRegistry**: Unified registration system
- **GraphFlow**: Task execution framework

**Usage**:

```rust
let registry = NodeRegistry::new();
registry.register_subnode_provider(node_kind, provider)?;
let manager = ClusterNodeManager::new_default().await?;
let response = manager.execute_task("deepseek_llm", input).await?;
```

## Development Practices

### Code Style

- Rust 2024 edition, 1.90+ syntax
- 2-space soft tabs, 120 char max width
- Use modern syntax: let-chains, collapsible ifs, if-let guards
- `cargo clippy` for syntax modernization

### Development Rules

- **No audit functionality**: No user action tracking
- **No migration logic**: No historical compatibility (system not released)
- **Test-driven fixing**: Match test expectations to actual code behavior
- **API consistency**: Use correct method names from actual APIs
- **String replacement completeness**: Replace all problematic characters in identifiers

### Rust Best Practices

**Arc and Clone**:

```rust
// ✅ Clone before moving
let provider = Arc::new(MyProvider::new());
let provider_for_registry = provider.clone();
registry.register(provider_for_registry).await?;

// ❌ Don't move and reuse
registry.register(provider).await?; // provider moved
let result = provider.do_something(); // Error!
```

**Option/Result Handling**:

```rust
// ✅ Proper Option to Result conversion
let registered_node = node_registry.get_executor(&node_kind)
    .ok_or_else(|| NodeExecutionError::ConfigurationError("Node not found".to_string()))?;

// ✅ Use ? operator
let result = some_operation()?;

// ❌ Don't unwrap in production
let result = some_operation().unwrap(); // Can panic!
```

**String Sanitization**:

```rust
// ✅ Comprehensive replacement
fn sanitize_identifier(input: &str) -> String {
    input.replace('.', "_")
         .replace('@', "_")
         .replace('#', "_")
         .replace(' ', "_")
         .to_lowercase()
}
```

## FusionSQL ORM Patterns

### Model Structure (3-file pattern)

1. `{entity}_entity.rs` - Database entity with `Fields` derive
2. `{entity}_model.rs` - Request/response models, filters
3. `{entity}_bmc.rs` - Database model controller (CRUD)
4. `{entity}_svc.rs` - Business logic service

**Crate Rules**:

- Entity/Model files: Can be in `xxx-core` crates
- BMC/Service files: Only in binary application crates
- Exception: Dedicated `xxx-db` crates for shared database code

### Query Filters & Pagination

```rust
use fusionsql::{
  filter::{OpValString, OpValInt32, OpValBool, OpValDateTime},
  FilterNodes, page::Page,
};

#[derive(Debug, Clone, Default, Deserialize, FilterNodes)]
pub struct CredentialFilter {
  pub namespace_id: Option<OpValString>,
  pub name: Option<OpValString>,
  pub kind: Option<OpValInt32>,
  pub is_managed: Option<OpValBool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CredentialForQuery {
  #[serde(default)]
  pub page: Page,
  #[serde(default)]
  pub filters: Vec<CredentialFilter>,
}
```

### Entity Definition

```rust
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

## Authentication & Authorization

### IAM System Architecture

**Centralized Identity Management**:

- **Jieyuan**: OAuth 2.0 + PKCE + policy-based authorization
- **Hetumind**: Authentication proxy redirecting to Jieyuan
- **Multi-tenant Security**: Tenant-based data isolation
- **IAM Resource Mapping**: Zero-configuration permission control via path patterns

**Modular Access Control**:

- `access_control` module: Core auth, policy, resource mapping
- `oauth` module: Dedicated OAuth 2.0 + PKCE service
- **即时解析 + 严格渲染**: Direct policy loading for each evaluation

### Authentication Flow

1. User request → Hetumind redirects to Jieyuan
2. Jieyuan OAuth flow with PKCE
3. JWT token issuance → Hetumind validation
4. Tenant middleware injection → Permission-based access
5. Remote authorization API → Fine-grained decisions

### Usage Patterns

```rust
use jieyuan::access_control::{AuthSvc, PolicySvc, IamResourceMappingSvc};
use jieyuan::oauth::OAuthSvc;
use jieyuan_core::model::iam_api::render_resource;

// Authentication
let auth_svc = AuthSvc::new(user_svc);
let response = auth_svc.signin(request).await?;

// OAuth
let oauth_svc = OAuthSvc::new(model_manager, app);
let auth_response = oauth_svc.authorize(oauth_request).await?;

// Resource template rendering
let resource = render_resource("iam:hetumind:workflow/{id}", &ctx, Some(&extras));
```

## Database & Multi-Tenancy

### Multi-Tenant Architecture

**Tenant ID Guidelines**:

- **Primary Tables** (Require tenant_id): `user_entity`, `project`, `workflow_entity`, `execution_entity`, `credential_entity`, `sched_job`, `sched_task`
- **Secondary Tables** (No tenant_id): `sched_task_instance`, `sched_agent`, `execution_data`, `execution_annotation`, `migrations`, `settings`
- **IAM Tables**: `iam_resource_mapping`, `permission_audit_logs`, `policy_entity` (tenant-scoped)

**Indexing Strategy**:

- Single tenant: `idx_table_tenant_id`
- Composite: `idx_table_tenant_status`
- Path mappings: `idx_path_mappings_service_pattern`

### Database Naming Conventions

**Constraints**:

- Primary Key: `{table_name}_pk` (e.g., `iam_user_pk`)
- Foreign Key: `{table_name}_fk_{column_name}` (e.g., `iam_user_fk_tenant_id`)

**Indexes**:

- Single Column: `{table_name}_idx_{column_name}`
- Composite: `{table_name}_idx_{column1}_{column2}`
- Unique: `{table_name}_uidx_{column_name}`

**Example**:

```sql
CREATE TABLE iam_namespace (
  id BIGSERIAL NOT NULL,
  tenant_id BIGINT NOT NULL,
  name VARCHAR(255) NOT NULL,
  created_by BIGINT NOT NULL,
  CONSTRAINT iam_namespace_pk PRIMARY KEY (id),
  CONSTRAINT iam_namespace_fk_tenant FOREIGN KEY (tenant_id) REFERENCES iam_tenant(id),
  CONSTRAINT iam_namespace_uk_tenant_name UNIQUE (tenant_id, name)
);

CREATE INDEX iam_namespace_idx_tenant_id ON iam_namespace(tenant_id);
CREATE UNIQUE INDEX iam_namespace_idx_tenant_name ON iam_namespace(tenant_id, name);
```

### BMC Layer Patterns

```rust
// ✅ Correct: No &self, first param is &ModelManager
impl UserBmc {
  pub fn get_by_id(mm: &ModelManager, id: UserId) -> Result<Option<UserEntity>, SqlError> {
    // implementation
  }
}

// ❌ Incorrect: Using &self
impl UserBmc {
  pub fn get_by_id(&self, id: UserId) -> Result<Option<UserEntity>, SqlError> {
    // implementation
  }
}
```

## API Development

### Service Layer Pattern

```rust
impl FromRequestParts<Application> for CredentialSvc {
  type Rejection = WebError;

  async fn from_request_parts(parts: &mut Parts, state: &Application) -> core::result::Result<Self, Self::Rejection> {
    let ctx = extract_ctx(parts, state.fusion_setting().security())?;
    let mm = state.component::<ModelManager>().with_ctx(ctx);
    let key_manager = state.component();
    Ok(CredentialSvc { mm, key_manager })
  }
}
```

### Service Requirements

- All service structs must implement `Clone`
- Use direct references, not Arc wrapping for cloneable types

```rust
// ✅ Correct: Direct reference
#[derive(Clone)]
pub struct WorkflowSvc {
  mm: ModelManager,
  auth_svc: AuthSvc,
}

// ❌ Incorrect: Unnecessary Arc
pub struct WorkflowSvc {
  mm: Arc<ModelManager>,
  auth_svc: Arc<AuthSvc>,
}
```

### Middleware Integration

```rust
.route("/workflows", workflows::routes()
  .layer(AsyncRequireAuthorizationLayer::new(WebAuth::default())))
  .layer(tenant_middleware_layer())
```

## Logging Guidelines

**Use `log` crate, not `tracing`**:

```rust
use log::{debug, error, info, warn};

debug!("Processing request for workflow: {}", workflow_id);
info!("User {} authenticated successfully", username);
warn!("Connection to Redis is slow: {}ms", duration);
error!("Failed to save message to memory: {}", error);
```

**Configuration**:

- Development: `RUST_LOG=debug`
- Production: `RUST_LOG=info`

## Performance Optimization

### Hash Collections - Use ahash

```rust
use fusion_common::ahash::{HashMap, HashSet};

// ✅ Prefer ahash for performance (2-3x faster)
let mut map = HashMap::default();
let mut set = HashSet::default();
```

**Benefits**: Faster data processing, improved cache hit rates, reduced latency, better throughput

## Testing

### Test-Driven Debugging

1. Run tests with `--nocapture` to see output
2. Verify test expectations match actual behavior
3. Fix root cause in code or correct tests
4. Validate all related tests pass

### Common Commands

```bash
cargo test -p <crate-name> <test_name> -- --nocapture
cargo test -p <crate-name>
cargo test -p <crate-name> --lib --no-run
```

### Anti-Patterns

- Don't change working code to pass incorrect tests
- Don't ignore test failures without understanding
- Don't use outdated API methods in tests
- Don't assume serialization formats - verify against serde attributes

## Final Checks

Always run compilation checks:

```bash
cargo check -p hetumind-studio
cargo check -p fusionsql
cargo clippy --workspace --all-targets --all-features --no-deps -- -D warnings
```

## Important Notes

- **Zero Unsafe Code**: Workspace lint enforcement
- **Security First**: Vetted dependencies
- **Type Safety**: Strong compile-time guarantees
- **Async Throughout**: Consistent async/await with Tokio
- **Configuration-Driven**: TOML configuration files
- **Modular Architecture**: Clean separation of concerns
- **Performance Optimized**: Release builds with LTO
- **Production Ready**: Health checks, metrics, graceful shutdown
