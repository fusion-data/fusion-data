# AGENTS

## Repository Overview

This is a **fusion-data** platform - a comprehensive data fusion platform built with Rust, featuring:

- **fusionsql**: Database abstraction layer with sea-query(and sqlx) based ORM
- **hetuflow**: Distributed task scheduling and workflow orchestration system ("河图流动")
- **hetumind**: AI Agent/Flow platform with LLM integration ("河图智思")
- **jieyuan**: Access control and authentication utilities with modular IAM system ("界垣")
- **fusions**: Core library suite providing foundational components

**Version**: 0.1.0
**Rust Edition**: 2024
**License**: Apache-2.0

## Development Environment

### Prerequisites

- **Rust**: Version ≥ 1.90 (managed via rustup, rsproxy recommended for Chinese users)
- **Node.js**: Version ≥ 22 (managed via nvm)
- **pnpm**: Package manager for Node.js dependencies
- **Docker & Docker Compose**: For database and service management
- **Git**: For version control

### Common Commands

#### Build & Test

```bash
# Basic build and check
cargo check
cargo build

# Format and linting
cargo fmt
cargo clippy --workspace --all-targets --all-features --no-deps -- -D warnings

# Run tests
cargo test

# Specific test scenarios
cargo test -p <crate-name> --lib              # Library and doc tests only
cargo test -p <crate-name> --bins             # Binary tests only
cargo test -p <crate-name> --lib --bins       # Library and binary tests (no integration)

# Build with specific profile
cargo build --release                         # Optimized release build
cargo build --profile bench                   # Benchmark profile

# Update dependencies
cargo update
cargo tree                                   # View dependency tree
```

#### Running Applications

```bash
# hetuflow services (with config)
cargo run --bin hetuflow-server
cargo run --bin hetuflow-agent

# hetumind applications
cargo run --bin hetumind-studio
cargo run --bin hetumind-cli

# jieyuan access control
cargo run --bin jieyuan-server

# Development tools
cargo run --bin <binary-name> -- --help          # Show help for any binary
```

#### Database Services (Docker)

```bash
docker-compose up -d      # Start services
docker-compose ps         # Check status
docker-compose logs -f    # Follow logs
docker-compose down       # Stop services
docker-compose down -v    # Stop and clean volumes

# Service-specific commands
docker-compose restart postgres
```

## Architecture Overview

### Workspace Structure

The project uses Cargo workspace with the following main components:

**Core Libraries (`crates/fusions/`)**:

- `fusion-core`: Application framework with component system, configuration, async runtime
- `fusion-web`: HTTP server framework (Axum-based)
- `fusion-db`: Database access layer
- `fusion-grpc`: gRPC utilities
- `fusion-security`: Security components
- `fusion-common`: Shared utilities
- `fusion-ai`: AI integration utilities
- `fusion-core-macros`: Core derive macros
- `fusions`: Meta-package for all fusion libraries

**Application Projects**:

- `hetuflow`: Distributed task scheduling and workflow orchestration system
  - `hetuflow-core`: Shared models, protocols, and job definitions
  - `hetuflow-server`: Central scheduling server with gRPC/Web API
  - `hetuflow-agent`: Distributed execution agent with task runner
  - `hetuflow-test`: Integration tests and test utilities
  - `hetuflow-web`: Web interface and dashboard
  - `hetuflow-docs`: Documentation and examples
- `hetumind`: AI Agent/Flow platform with LLM integration
  - `hetumind-core`: Core AI functionality and agent orchestration
  - `hetumind-nodes`: Node execution framework with comprehensive workflow nodes
  - `hetumind-context`: Context management and state persistence
  - `hetumind-studio`: Web studio interface for agent design with multi-tenant support
  - `hetumind-cli`: Command-line tools for agent management
  - `hetumind-docs`: Documentation and tutorials
- `fusionsql`: Database abstraction layer with sea-query ORM
  - `fusionsql-core`: Core types, traits, and database abstractions
  - `fusionsql`: Main database ORM with field-level operations
  - `fusionsql-macros`: Derive macros for model definitions
- `jieyuan`: Access control and authentication utilities with modular IAM system
  - `jieyuan-core`: Core access control models, OAuth authentication, policy engine, and IAM Resource Mapping
  - Centralized IAM system with OAuth 2.0 + PKCE support, remote authorization API, and modular architecture
  - **IAM Resource Mapping**: Zero-configuration permission control through managed API path mappings
  - **Modular Access Control**: Separated authentication and authorization into specialized modules
  - **OAuth Module**: Independent OAuth 2.0 + PKCE authentication service
  - **Policy Engine**: Role-based access control with fine-grained permissions
  - **Remote Authorization**: Centralized policy evaluation with automatic tenant injection

### Key Architecture Patterns

**Configuration Management**:

- Uses `FUSION_CONFIG_FILE` environment variable for config file path
- TOML-based configuration with structured types in `resources/` directories
- Environment variable override support
- Default configs provided in core libraries, overridden by applications

**Async Runtime**:

- Tokio-based async throughout
- Multi-threaded runtime configuration
- Graceful shutdown handling

**Database Layer**:

- sea-query for query building
- SQLx for database connectivity
- Model-based ORM with field-level operations
- Support for PostgreSQL, SQLite
- Vector database support with pgvector
- Multi-tenant architecture with tenant-based data isolation

**Error Handling**:

- Custom `Result<T>` types with `DataError` in fusion-core
- Structured error types with context
- anyhow integration for error chaining

**Component System**:

- Dependency injection patterns with builder pattern
- Component lifecycle management with graceful shutdown
- Application-wide service registration and access
- Async-first component initialization

**API Development Patterns**:

- Axum-based web framework with Tower middleware
- Service layer extraction using `FromRequestParts` trait
- Multi-tenant middleware for automatic context injection
- Permission-based access control with fine-grained permissions
- Error handling with `WebResult` and structured error types
- OpenAPI documentation support

**Workflow Node Architecture**:

- Comprehensive node execution framework with various node types
- Conditional branching, data manipulation, and flow control nodes
- HTTP webhook and time-based trigger nodes
- Async execution with proper error handling and data lineage tracking

### Development Practices

**Code Style**:

- Rust 2024 edition with 1.90+ syntax features
- 2-space soft tabs
- 120 character max line width
- Visual indent style
- `rustfmt` configuration in `rustfmt.toml`

**Development Rules**:

- **No audit functionality**: Do not design or implement audit features for tracking user actions or data changes
- **No migration logic**: Do not consider historical version compatibility or database migration logic (system not yet released)

**Modern Rust Syntax (1.90+)**:

Always utilize the latest Rust syntax features for cleaner, more maintainable code:

- **Collapsible if statements**: Use `&&` operators instead of nested if blocks

  ```rust
  // ❌ Avoid nested if blocks
  if condition1 {
      if condition2 {
          // code
      }
  }

  // ✅ Prefer combined conditions
  if condition1 && condition2 {
      // code
  }
  ```

- **Let-chains**: Use `let` in conditions for pattern matching

  ```rust
  // ✅ Modern let-chain syntax
  if let Some(value) = optional_value && value > 0 {
      // code using value
  }
  ```

- **If-let guards**: Use guards in if-let expressions

  ```rust
  // ✅ Guard patterns
  if let Some(value) = optional_value if value > 0 {
      // code using value
  }
  ```

- **Else-if let chains**: Chain let expressions with else-if
  ```rust
  // ✅ Chained let expressions
  if let Some(a) = option_a {
      // handle a
  } else if let Some(b) = option_b {
      // handle b
  }
  ```

Always run `cargo clippy` to catch opportunities for modernizing syntax.

**Testing Strategy**:

- Unit tests in lib.rs and source files
- Integration tests in `tests/` directories
- Use `insta` for snapshot testing
- Test context management with `test-context`

**Dependencies**:

- Workspace-level dependency management in `Cargo.toml`
- Careful feature flag management with security focus
- Zero unsafe code policy (workspace lint enforcement)
- AI/LLM integration capabilities
- gRPC and HTTP API support with OpenAPI documentation
- Cloud storage integration with OpenDAL (S3, OSS, OBS, local)
- Advanced web framework with Axum and Tower middleware
- High-performance async runtime with Tokio
- Database connectivity with SQLx and sea-query (FusionSQL ORM)
- Vector database support with pgvector
- Message queuing with Redis/Valkey (fred client)
- Template engine with MiniJinja
- Data processing capabilities
- Cryptographic operations with modern Rust crates
- Metrics and logging capabilities with `log` crate
- Memory optimization with tikv-jemallocator (optional)

### Configuration Files

**Rust Toolchain**: `rust-toolchain.toml`

- Stable channel with rustfmt and clippy components

**Environment**: `.env` file for local development

- Database connection strings
- Service endpoints
- Feature flags

**Docker**: `docker-compose.yml`

- PostgreSQL database with pgvector extension
- Redis/Valkey cache and message broker
- MinIO/S3-compatible storage
- Development services with health checks
- Environment configuration via `.env` file

### Database Design and Multi-Tenancy

**Multi-Tenant Architecture**:

The platform implements a robust multi-tenant architecture to ensure data isolation and proper access control across different organizations or tenants.

**Tenant ID Field Guidelines**:

- **Primary Tables (Require tenant_id)**: Core business entities that are tenant-specific

  - `user_entity`, `project`, `workflow_entity`, `execution_entity`
  - `credential_entity`, `sched_job`, `sched_task` (job definitions and task schedules)
  - These tables store tenant-owned data and require strict isolation

- **Secondary/Dimension Tables (No tenant_id required)**: Supporting tables that store execution data, logs, or system-wide metadata

  - `sched_task_instance`, `sched_agent` (execution instances and runtime components)
  - `execution_data`, `execution_annotation` (execution metadata and logs)
  - `migrations`, `settings` (system configuration tables)
  - These tables either reference tenant data through foreign keys or store system-wide information

- **IAM and Authorization Tables (Special handling)**:
  - `iam_resource_mapping`: Path mapping configurations for Resource-Path optimization (tenant-scoped)
  - `permission_audit_logs`: Authorization audit logging (tenant_id included for filtering)
  - `policy_entity`, `policy_attachment`: Policy definitions and attachments (tenant-scoped)

**Indexing Strategy**:

- Single tenant queries: `idx_table_tenant_id` on tenant_id columns
- Composite queries: `idx_table_tenant_status` on (tenant_id, status) combinations
- Performance optimization for multi-tenant filtering scenarios
- Path mapping indexes: `idx_path_mappings_service_pattern` for efficient path lookups

**Data Isolation**:

- All tenant-specific queries must include `WHERE tenant_id = ?` filters
- Application layer enforces tenant boundaries through tenant middleware
- Database constraints ensure referential integrity across tenant boundaries

## FusionSQL ORM Patterns

**Database Model Architecture**:

The platform uses FusionSQL, a sea-query and SQLx based ORM that provides type-safe database operations with automatic query generation.

### Model Structure Pattern

Follow the established three-file pattern for database entities:

1. **`{entity}_entity.rs`** - Database entity definition with `Fields` derive
2. **`{entity}_model.rs`** - Request/response models and filter types
3. **`{entity}_bmc.rs`** - Database model controller (BMC) with CRUD operations
4. **`{entity}_svc.rs`** - Business logic service layer

**Crate Placement Rules**:

- **Entity and Model files** (`{entity}_entity.rs`, `{entity}_model.rs`) can be placed in reusable `xxx-core` crates
- **BMC and Service files** (`{entity}_bmc.rs`, `{entity}_svc.rs`) should **only** be defined in binary application crates, not in reusable `xxx-core` crates
- **Exception**: Projects with special requirements may create dedicated `xxx-db` crates to store reusable database access code that can be shared across multiple projects

### Query Filter Types

Use `OpValXxx` types for type-safe query filtering:

```rust
use fusionsql::{
  filter::{OpValString, OpValInt32, OpValInt64, OpValBool, OpValDateTime},
  FilterNodes,
};

#[derive(Debug, Clone, Default, Deserialize, FilterNodes)]
pub struct CredentialFilter {
  pub namespace_id: Option<OpValString>,
  pub name: Option<OpValString>,
  pub kind: Option<OpValInt32>,
  pub is_managed: Option<OpValBool>,
  pub created_at: Option<OpValDateTime>,
  pub created_by: Option<OpValInt64>,
}
```

### Pagination Patterns

Use structured query models for paginated results:

```rust
use fusionsql::page::Page;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CredentialForQuery {
  #[serde(default)]
  pub page: Page,
  #[serde(default)]
  pub filters: Vec<CredentialFilter>,
}
```

### Entity and Field Definitions

Use `Fields` derive macro for automatic field mapping:

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

**Feature-Conditional Derives for Core Crates**:

When generating code in reusable crates (like `xxx-core`) that can be referenced by other projects, use conditional compilation for database-related derives when the project has a `with-db` feature flag:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(sqlx::FromRow, fusionsql::Fields))]
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

This pattern allows the core crate to be used in contexts where database functionality is not needed, while still providing full ORM capabilities when the `with-db` feature is enabled.

### Request/Response Models

Separate models for different operations:

```rust
#[derive(Debug, Clone, Deserialize, Fields)]
pub struct CredentialForUpdate {
  pub namespace_id: Option<String>,
  pub name: Option<String>,
  pub data: Option<String>,
  pub kind: Option<CredentialKind>,
  pub is_managed: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Fields)]
pub struct CredentialForInsert {
  pub namespace_id: String,
  pub name: String,
  pub data: String,
  pub kind: CredentialKind,
  pub is_managed: Option<bool>,
  pub id: Option<Uuid>,
}
```

## Authentication and Authorization

**Centralized Identity Management**:

The platform implements a sophisticated IAM system through Jieyuan with comprehensive authentication and authorization capabilities:

- **Jieyuan**: Provides unified IAM capabilities with OAuth 2.0 + PKCE authentication and policy-based authorization
- **Hetumind**: Acts as authentication proxy, redirecting to Jieyuan for centralized identity management
- **User Synchronization**: Event-driven synchronization of user changes across the platform
- **Multi-tenant Security**: Tenant-based data isolation with fine-grained permission middleware
- **IAM Resource Mapping**: Zero-configuration permission control through managed API path mappings
- **Modular Access Control Architecture**: Authentication and authorization functionality distributed across specialized modules
  - `access_control`: Core authentication services, policy management, and resource mapping
  - `oauth`: Independent OAuth 2.0 + PKCE authentication module with comprehensive PKCE support
  - **即时解析 + 严格渲染**: 每次权限评估都直接从策略配置文件加载并解析，不生成持久化表示，缺失必须参数直接拒绝

**Authentication Flow**:

1. User requests authentication → Hetumind redirects to Jieyuan
2. Jieyuan handles OAuth flow with PKCE security
3. Jieyuan issues JWT tokens → Hetumind validates and processes
4. Tenant middleware injects tenant context → Permission-based access control
5. Remote authorization API evaluates policies → Fine-grained access decisions

**Authorization System**:

The IAM system implements a sophisticated policy-based authorization with:

- **Modular Access Control Architecture**: IAM functionality distributed across specialized modules
  - **access_control module** (`jieyuan/jieyuan/src/access_control/`):
    - `AuthSvc`: User authentication service with signin/signup/token validation
    - `PolicySvc`: Policy evaluation and authorization decisions
    - `IamResourceMappingSvc`: Resource path mapping and management (即时解析 + 严格渲染)
  - **oauth module** (`jieyuan/jieyuan/src/oauth/`):
    - `OAuthSvc`: Dedicated OAuth 2.0 + PKCE authentication flow service
- **Unified Context**: Direct use of `fusion_common::ctx::Ctx` throughout the system, eliminating complex intermediate types
- **Remote Authorization API**: Centralized policy evaluation at `/iam/authorize` endpoint with unified request structure
- **Resource Template Rendering**: Automatic tenant_id injection with double-layer format support
  - Policy format: `iam:hetumind:{tenant_id}:workflow/123` (complete, unambiguous)
  - API format: `iam:hetumind:workflow/123` (simplified, auto-injected)
- **IAM Resource Mapping**: Zero-configuration permission control through managed API path mappings
  - Path pattern matching: `/api/v1/workflows/{id}` → action/resource mapping
  - Centralized configuration via jieyuan management interface
  - Simplified client integration with automatic parameter extraction
  - Path code support for direct mapping lookup
  - **即时解析 + 严格渲染**: 每次权限评估都直接从策略配置文件加载并解析，不生成持久化表示，缺失必须参数直接拒绝
- **Policy Engine**: "Explicit deny 优先 → allow 命中 → 边界/会话裁剪" evaluation flow
- **Role-based Access Control**: Predefined roles (viewer, editor, admin) with hierarchical permissions
- **Resource-level Permissions**: Fine-grained control over specific resources and actions

**Security Features**:

- OAuth 2.0 with Authorization Code + PKCE
- JWT token validation and refresh with sequence-based replay protection
- Tenant isolation middleware with automatic context injection
- Fine-grained permission system with policy-based access control
- User change synchronization via event polling
- Comprehensive audit logging for all authorization decisions
- Zero unsafe code policy with security-first dependency management

**Implementation Details**:

- **Access Control Modules**: Distributed IAM functionality across specialized modules
  - `jieyuan/jieyuan/src/access_control/`: Core authentication and policy services
  - `jieyuan/jieyuan/src/oauth/`: Dedicated OAuth 2.0 + PKCE authentication
- **Remote Authorization Endpoint**: `jieyuan/jieyuan/src/endpoint/api/v1/iams.rs`
- **IAM Resource Mapping**: `jieyuan/jieyuan-core/src/model/iam_resource_mapping.rs`
- **Resource Template Rendering**: `jieyuan/jieyuan-core/src/model/iam_api.rs` with unified `render_resource` function
- **Path-based Authorization Middleware**: `jieyuan-core/src/web/middleware/path_authz.rs`
- **Policy Configuration**: JSON-based policies with condition-based access control
- **Integration Documentation**: `documents/hetumind-jieyuan-integration.md` and `documents/iam.md`

### IAM Development Best Practices

**Modular Access Control Integration**:

Use the modular access control architecture for all authentication and authorization needs:

```rust
use jieyuan::access_control::{AuthSvc, PolicySvc, IamResourceMappingSvc};
use jieyuan::oauth::OAuthSvc;

// Application setup with modular access control
pub fn app() -> Router {
    Router::new()
        .route("/api/v1/auth/signin", post(signin))
        .route("/api/v1/auth/signup", post(signup))
        .route("/oauth/authorize", post(oauth_authorize))
        .route("/oauth/token", post(oauth_token))
        .route("/api/v1/workflows", post(create_workflow))
        .route("/api/v1/workflows/:id", get(get_workflow))
        // Integrated permission middleware
        .layer(path_authz_middleware())
}

// Authentication handler using AuthSvc from access_control module
pub async fn signin(
    State(app): State<Application>,
    auth_svc: AuthSvc,  // From access_control module
    Json(req): Json<SigninRequest>,
) -> WebResult<SigninResponse> {
    let response = auth_svc.signin(req).await?;
    Ok(Json(response))
}

// OAuth handler using OAuthSvc from dedicated oauth module
pub async fn oauth_authorize(
    State(app): State<Application>,
    oauth_svc: OAuthSvc,  // From oauth module
    Json(req): Json<OAuthAuthorizeRequest>,
) -> WebResult<OAuthAuthorizeResponse> {
    let response = oauth_svc.authorize(req).await?;
    Ok(Json(response))
}

// Handler with direct context access
pub async fn get_workflow(
    workflow_svc: WorkflowSvc,
    ctx: Ctx,  // Directly from middleware - no complex auth logic needed
    Path(workflow_id): Path<WorkflowId>,
) -> WebResult<Workflow> {
    // Business logic only - authorization handled automatically
    let res = workflow_svc.get_workflow(&workflow_id).await?;
    Ok(Json(res))
}
```

**Resource Template Rendering**:

Use the unified `render_resource` function for resource template rendering with optional extras parameter:

```rust
use jieyuan_core::model::iam_api::render_resource;

// Automatic tenant_id injection and parameter replacement
let resource = render_resource("iam:hetumind:workflow/{id}", &ctx, Some(&extras));
```

**Modular Access Control Usage**:

IAM functionality is distributed across specialized modules for better organization:

```rust
use jieyuan::access_control::{AuthSvc, PolicySvc, IamResourceMappingSvc};
use jieyuan::oauth::OAuthSvc;

// Core authentication from access_control module
let auth_svc = AuthSvc::new(user_svc);
let signin_response = auth_svc.signin(signin_request).await?;

// OAuth 2.0 + PKCE from dedicated oauth module
let oauth_svc = OAuthSvc::new(model_manager, app);
let auth_response = oauth_svc.authorize(oauth_request).await?;

// Policy evaluation from access_control module
let policy_svc = PolicySvc::new(model_manager);
let authz_response = policy_svc.evaluate_policy(policy_request).await?;

// Resource mapping from access_control module
let mapping_svc = IamResourceMappingSvc::new(model_manager);
let lookup_result = mapping_svc.lookup_by_path(&lookup_request).await?;
```

- **Double-layer Format Support**: Use simplified format for APIs, complete format for policies
- **Path Code Integration**: Use path codes for direct mapping lookup when available
- **Centralized Configuration**: Manage path mappings through jieyuan management interface
- **Automatic Parameter Extraction**: No need to manually configure action/resource templates in code

**Integration Approaches**:

1. **Modular Access Control (Recommended for all projects)**:

   - Distributed functionality across specialized modules
   - Core authentication in `access_control` module
   - OAuth functionality in dedicated `oauth` module
   - Zero configuration permission control
   - Implementation: `jieyuan/jieyuan/src/access_control/` and `jieyuan/jieyuan/src/oauth/`

2. **Direct Service Integration**:
   - Direct use of AuthSvc, PolicySvc from `access_control` module
   - Direct use of OAuthSvc from `oauth` module
   - Fine-grained control over specific endpoints
   - Implementation: Import from respective modules

### Important Notes

- **No Unsafe Code**: Workspace lints enforce zero unsafe code policy
- **Security First**: All dependencies vetted for security vulnerabilities
- **Type Safety**: Strong compile-time guarantees with extensive use of Rust's type system
- **Async Throughout**: Consistent async/await patterns with Tokio runtime
- **Configuration-Driven**: Behavior controlled via TOML configuration files
- **Modular Architecture**: Clean separation of concerns with well-defined APIs
- **Performance Optimized**: Release builds with LTO, panic=abort, and size optimizations
- **Memory Efficient**: Optional tikv-jemallocator for better memory management
- **Logging**: Use `log` crate for all logging operations with structured, context-aware messages
- **Developer Friendly**: Comprehensive tooling with formatting, linting, and testing
- **Production Ready**: Includes health checks, metrics, and graceful shutdown

## Development Guidelines

### Logging Guidelines

**Use `log` crate, not `tracing`**:

- **Preferred crate**: Use `log` crate for all logging operations
- **Dependency**: Already included in workspace dependencies as `log = "0.4"`
- **Import pattern**: `use log::{debug, error, info, warn};`
- **Rationale**: `log` provides stable, mature logging with sufficient features for our needs

**Logging Levels Usage**:

```rust
use log::{debug, error, info, warn};

// Debug level - Detailed diagnostic information
debug!("Processing request for workflow: {}", workflow_id);

// Info level - General operational information  
info!("User {} authenticated successfully", username);

// Warn level - Potentially problematic situations
warn!("Connection to Redis is slow: {}ms", duration);

// Error level - Failed operations and errors
error!("Failed to save message to memory: {}", error);
```

**Logging Best Practices**:

- **Structured logging**: Include relevant context in log messages
- **Minimal performance impact**: Use appropriate log levels
- **No sensitive data**: Avoid logging passwords, tokens, or personal information
- **Consistent formatting**: Use clear, descriptive message formats

**Configuration**:

Logging is controlled via environment variables or configuration files:
- `RUST_LOG=debug` for development (verbose)
- `RUST_LOG=info` for production (default)
- Component-specific log level configuration available

**Examples from codebase**:

```rust
// Service layer logging
pub async fn get_workflow(
    workflow_svc: WorkflowSvc,
    ctx: Ctx,
    Path(workflow_id): Path<WorkflowId>,
) -> WebResult<Workflow> {
    // Business logic only - authorization handled automatically
    let res = workflow_svc.get_workflow(&workflow_id).await?;
    info!("Retrieved workflow: {}", workflow_id);
    Ok(Json(res))
}

// Error handling with logging
match risky_operation() {
    Ok(result) => {
        info!("Operation completed successfully");
        result
    }
    Err(err) => {
        error!("Operation failed: {}", err);
        Err(WebError::InternalError(err.to_string()))
    }
}
```

### API Development

**Service Layer Pattern**:

Use `FromRequestParts` trait for service extraction:

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

**Middleware Integration**:

Apply tenant and permission middleware to routes:

```rust
.route("/workflows", workflows::routes()
  .layer(AsyncRequireAuthorizationLayer::new(WebAuth::default())))
  .layer(tenant_middleware_layer())
```

### Service Layer Development

**Service Clone Requirements**:

All service layer structs must implement Clone macro to ensure cheap cloning operations:

```rust
#[derive(Clone)]
pub struct UserSvc {
  mm: ModelManager,
}
```

**Arc Wrapping Rules**:

- Components and services that support cheap cloning should NOT be wrapped in Arc
- Use direct references instead of Arc wrapping for cloneable types

```rust
// ✅ Correct: Direct reference
pub struct WorkflowSvc {
  mm: ModelManager,
  auth_svc: AuthSvc,
}

// ❌ Incorrect: Unnecessary Arc wrapping
pub struct WorkflowSvc {
  mm: Arc<ModelManager>,
  auth_svc: Arc<AuthSvc>,
}
```

### Database Development

**Database Naming Conventions**:

Follow these naming conventions for all database objects to ensure consistency across the platform:

**Constraints Naming**:

- **Primary Key**: `{table_name}_pk` (e.g., `iam_user_pk`, `iam_namespace_pk`)
- **Foreign Key**: `{table_name}_fk_{column_name}` (e.g., `iam_user_fk_tenant_id`, `iam_namespace_fk_created_by`)

**Indexes Naming**:

- **Single Column Index**: `{table_name}_idx_{column_name}` (e.g., `iam_user_idx_email`, `iam_namespace_idx_tenant_id`)
- **Foreign Key Index**: `{table_name}_idx_{foreign_key_column_name}` (e.g., `iam_user_idx_created_by`, `iam_namespace_fk_updated_by`)
- **Composite Index**: `{table_name}_idx_{column1}_{column2}` (e.g., `iam_user_idx_tenant_status`, `iam_namespace_idx_tenant_name`)
- **Unique Index**: `{table_name}_uidx_{column_name}` (e.g., `iam_user_uidx_email`, `iam_tenant_user_uidx_user_tenant`)

**Example Implementation**:

```sql
-- Table with proper naming conventions
CREATE TABLE iam_namespace (
  id BIGSERIAL NOT NULL,
  tenant_id BIGINT NOT NULL,
  name VARCHAR(255) NOT NULL,
  created_by BIGINT NOT NULL,
  -- ... other columns

  -- Constraints
  CONSTRAINT iam_namespace_pk PRIMARY KEY (id),
  CONSTRAINT iam_namespace_fk_tenant FOREIGN KEY (tenant_id) REFERENCES iam_tenant(id),
  CONSTRAINT iam_namespace_fk_created_by FOREIGN KEY (created_by) REFERENCES iam_user(id),
  CONSTRAINT iam_namespace_uk_tenant_name UNIQUE (tenant_id, name)
);

-- Indexes
CREATE INDEX iam_namespace_idx_tenant_id ON iam_namespace(tenant_id);
CREATE INDEX iam_namespace_idx_created_by ON iam_namespace(created_by);
CREATE UNIQUE INDEX iam_namespace_idx_tenant_name ON iam_namespace(tenant_id, name);
```

**FusionSQL Best Practices**:

- Use `OpValXxx` types for type-safe filtering
- Separate entity, model, BMC, and service layers
- Follow pagination patterns with `Page` and `PageResult`
- Use `Fields` derive macro for automatic mapping
- Implement proper error handling with `DataError`
- **Always create indexes for foreign key columns** to optimize JOIN queries and cascade operations
- **Use partial indexes** for conditional data (e.g., logical deletion: `WHERE logical_deletion IS NOT NULL`)

**BMC Layer Function Patterns**:

- BMC functions should NOT hold `&self` as a parameter
- First parameter should be `mm: &ModelManager`

```rust
// ✅ Correct: BMC function signature
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

**IAM Development Best Practices**:

- Use the unified `render_resource` function for resource template rendering with optional extras parameter
- Leverage double-layer resource format: simplified format for APIs, complete format for policies
- **IAM Resource Mapping**: Use path-based authorization for zero-configuration permission control
  - Configure path mappings through jieyuan management interface
  - Use simplified middleware: `path_authz_middleware` for automatic permission checking
  - No need to manually configure RouteMeta or action/resource templates in code
  - **即时解析 + 严格渲染**: 每次权限评估都直接从策略配置文件加载并解析，不生成持久化表示，缺失必须参数直接拒绝
- Implement role-based access control with hierarchical permissions (viewer → editor → admin)
- Use remote authorization middleware for centralized policy evaluation
- Follow resource naming convention: `iam:{service}:{type}/{id}` for API calls
- Define policies using complete format: `iam:{service}:{tenant_id}:{type}/{id}` for unambiguous evaluation

**IAM Integration Approaches**:

1. **IAM Resource Mapping (Recommended approach)**:

   - Zero configuration permission control through path pattern matching
   - Centralized path mapping management via jieyuan admin interface
   - Simplified development experience with automatic parameter extraction
   - Implementation: `jieyuan-core/src/model/iam_resource_mapping.rs`

2. **Direct Resource Template Authorization**:
   - Manual action and resource template configuration
   - Fine-grained control over specific endpoints
   - Use when complex authorization logic is required
   - Implementation: `jieyuan-core/src/model/iam_api.rs`

### Testing

Always run compilation checks after implementation:

```bash
cargo check -p hetumind-studio
cargo check -p fusionsql
cargo clippy --workspace --all-targets --all-features --no-deps -- -D warnings
```

## Performance Optimization Guidelines

### Hash Map and Set Usage

**Performance-First Approach**: The platform prioritizes performance by using `ahash` instead of standard library hash collections.

**Why ahash?**:

- **2-3x faster** than `std::collections::HashMap` for typical workloads
- **Better hash distribution** reducing collisions
- **Memory efficient** with optimized layouts
- **Zero runtime overhead** with compile-time optimizations

**Usage Guidelines**:

```rust
// ✅ Prefer ahash for all hash collections
use fusion_common::ahash::{HashMap, HashSet};

// ✅ Use default() constructor for optimal performance
let mut map = HashMap::default();
let mut set = HashSet::default();

// ✅ Type annotations when needed
let mut source_ports: HashSet<&ConnectionKind> = HashSet::default();
```

**Performance Benefits**:

- Faster data processing in workflow engines
- Improved cache hit rates in resource management
- Reduced latency in API request handling
- Better throughput in concurrent scenarios

**Migration Completed**: All platform modules have been migrated to ahash:

- `jieyuan`: Authentication and authorization services
- `hetumind-core`: Core workflow execution engine
- `hetumind-studio`: Web interface and management tools
- `hetumind-nodes`: Workflow node execution framework
- `fusion-core`: Core application framework

## Documentation and Code Generation

### Project Documentation

The project maintains comprehensive technical documentation:

- **`documents/iam.md`**: Complete IAM technical specifications and Resource-Path optimization mechanisms
  - Unified authorization system using `fusion_common::ctx::Ctx`
  - Remote authorization API with simplified request structure
  - Resource template rendering with double-layer format support
  - Policy engine evaluation flow and role-based access control
- **`documents/hetumind-jieyuan-integration.md`**: Integration guide for hetumind-studio with jieyuan IAM system
  - Resource-Path optimization for zero-configuration permission control
  - Simplified integration approach with unified middleware
  - Development experience comparisons and migration guidance
  - Role-based access control design and policy examples
