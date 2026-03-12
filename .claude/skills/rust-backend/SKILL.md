---
name: rust-backend
description: Fusion-Data Rust backend patterns, hetus library, Component/Application, API patterns, performance optimization
---

# Rust Backend Development

## Quick Reference

| Pattern   | Usage                                                       |
| --------- | ----------------------------------------------------------- |
| Imports   | `use hetus::{core::*, common::*}`                           |
| Component | `#[derive(Component)]` struct with `#[component]` fields    |
| Config    | `#[derive(Configuration)]` with `#[config(prefix = "...")]` |
| Error     | `hetu_core::DataError` + `thiserror`                        |

## Core Patterns

### Component Architecture

```rust
use hetu_core_macros::Component;
use hetus::core::{Application, ApplicationBuilder};

#[derive(Clone, Component)]
pub struct MyService {
    #[component]
    mm: ModelManager,
    config: ConfigArc<MyConfig>,
}

impl MyService {
    pub async fn do_something(&self) -> Result<()> {
        // use self.mm for DB operations
        Ok(())
    }
}
```

### Application Setup

```rust
use hetus::core::{Application, DataError};

#[tokio::main]
async fn main() -> Result<(), DataError> {
    let app = Application::new().await?;
    app.start().await?;
    // shutdown on signals
    Ok(())
}
```

### API Development (Axum)

```rust
use hetus::web::{Router, WebError, ok_json};
use axum::{routing::post, extract::Json};

pub fn routes() -> Router {
    Router::new()
        .route("/api/endpoint", post(handler))
}

async fn handler(Json(req): Json<Request>) -> Result<Json<Response>, WebError> {
    ok_json(Response::from(req))
}
```

### Error Handling

```rust
use hetu_core::DataError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Validation: {0}")]
    Validation(String),
}

// Convert to DataError
impl From<ServiceError> for DataError {
    fn from(e: ServiceError) -> Self {
        DataError::ServiceError(e.to_string())
    }
}
```

## hetus Library Structure

```
hetus::
├── core::       Application, Component, Configuration, DataError
├── common::     ahash, ctx, env, time, model, serde
├── web::        Router, WebError, ok_json
├── db::         ModelManager, DbPlugin
├── ai::         LLM providers, agents
└── macros::     Component, Builder, Configuration
```

## Performance

- **HashMap**: `hetus::common::ahash::HashMap` (2-3x faster)
- **Async**: Tokio runtime, proper `await` chaining
- **Config**: `ConfigArc<T>` for thread-safe config access

## Build & Test

```bash
cargo check -p <crate>
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUST_LOG=debug cargo run --bin <service>
```

## Related Skills

- `sql-database`: BMC/Service patterns, transactions
- `cluster-node`: NodeRegistry, AI providers
