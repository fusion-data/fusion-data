---
name: tokio-axum-patterns
description: Tokio async runtime patterns, Axum web framework best practices, Tower middleware layers, async programming, handler patterns, 异步编程, Web 开发最佳实践
globs:
  - "**/*.rs"
---

# Tokio + Axum + Tower Patterns

## Quick Reference

| Category | Pattern | Usage |
|----------|---------|-------|
| Tokio | Runtime | `#[tokio::main]`, `spawn`, `select!` |
| Axum | Handler | `async fn handler(Extractors) -> Result<Response>` |
| Tower | Middleware | `.layer(ServiceBuilder::new().layer(...))` |
| Error | Propagation | `?` operator with `From<T> for WebError` |

## Core Patterns

### Tokio Runtime
```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Multi-threaded runtime (default)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // Single-threaded runtime (for I/O-bound)
}
```

### Axum Handler
```rust
use axum::{
    extract::{Path, State, Json, Query},
    response::{IntoResponse, Response},
    http::StatusCode,
};

async fn get_user(
    Path(id): Path<i64>,
    State(mm): State<ModelManager>,
) -> Result<Json<User>, WebError> {
    let user = UserBmc::get(&mm, id).await?
        .ok_or_else(|| WebError::not_found("User not found"))?;
    Ok(Json(user))
}
```

### Tower Middleware Stack
```rust
use tower::ServiceBuilder;
use tower_http::{
    trace::TraceLayer,
    cors::{CorsLayer, cors},
    compression::CompressionLayer,
    limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer,
};

let router = Router::new()
    .route("/api/users", get(list_users))
    .layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(CorsLayer::new().allow_origin(cors::Any))
            .layer(CompressionLayer::new())
            .layer(RequestBodyLimitLayer::new(1024 * 1024))  // 1MB
            .layer(TimeoutLayer::new(Duration::from_secs(30)))
    );
```

## Common Mistakes

| Mistake | Correct |
|---------|---------|
| `Rc<T>` in State | Use `Arc<T>` |
| Blocking in async | Use `spawn_blocking` |
| `.clone()` everywhere | Share `Arc` reference |
| `unwrap()` in handlers | Use `?` with error conversion |
| Missing `Send + Sync` | Use thread-safe types |

## References (按需加载)

- [tokio](references/tokio.md) - Runtime, spawn, channels, select!
- [axum](references/axum.md) - Handlers, extractors, state, routing
- [tower](references/tower.md) - Middleware, layers, services

## Related Skills
- `hetu`: 核心库模式
- `rust-skills:m07-concurrency`: 并发模式
- `rust-skills:domain-web`: Web 开发
