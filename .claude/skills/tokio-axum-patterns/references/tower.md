# Tower Middleware & Service Patterns

基于 [Tower 官方指南](https://github.com/tower-rs/tower/blob/master/guides/building-a-middleware-from-scratch.md)。

## Core Concepts

### Service Trait
```rust
use tower::Service;
use std::future::Future;

pub trait Service<Request> {
    type Response;
    type Error;
    type Future: Future<Output = Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;
    fn call(&mut self, req: Request) -> Self::Future;
}
```

### Layer Trait
```rust
use tower::Layer;

pub trait Layer<S> {
    type Service;

    fn layer(&self, inner: S) -> Self::Service;
}
```

## Building Middleware

### 基本结构
```rust
use tower::{Service, Layer};
use std::task::{Context, Poll};

// Middleware 包装器
pub struct TimeoutMiddleware<S> {
    inner: S,
    timeout: Duration,
}

impl<S> TimeoutMiddleware<S> {
    pub fn new(inner: S, timeout: Duration) -> Self {
        Self { inner, timeout }
    }
}

// 实现 Service
impl<S, Request> Service<Request> for TimeoutMiddleware<S>
where
    S: Service<Request>,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = TimeoutError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|_| TimeoutError)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let future = self.inner.call(req);
        let timeout = self.timeout;

        Box::pin(async move {
            tokio::time::timeout(timeout, future)
                .await
                .map_err(|_| TimeoutError)?
        })
    }
}

// 实现 Layer
pub struct TimeoutLayer {
    timeout: Duration,
}

impl TimeoutLayer {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl<S> Layer<S> for TimeoutLayer {
    type Service = TimeoutMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TimeoutMiddleware::new(inner, self.timeout)
    }
}
```

## ServiceBuilder

### 链式组合
```rust
use tower::ServiceBuilder;
use tower_http::{
    trace::TraceLayer,
    cors::CorsLayer,
    compression::CompressionLayer,
};

let service = ServiceBuilder::new()
    .layer(TraceLayer::new_for_http())
    .layer(CorsLayer::new().allow_origin(Any))
    .layer(CompressionLayer::new())
    .service(handler);
```

### 与 Axum 集成
```rust
let router = Router::new()
    .route("/api/users", get(list_users))
    .layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(TimeoutLayer::new(Duration::from_secs(30)))
            .layer(CorsLayer::new().allow_origin(Any))
    );
```

## Built-in Middleware

### Timeout
```rust
use tower::timeout::TimeoutLayer;

.layer(TimeoutLayer::new(Duration::from_secs(30)))
```

### Rate Limit
```rust
use tower::load_shed::LoadShedLayer;
use tower::limit::RateLimitLayer;

.layer(RateLimitLayer::new(100, Duration::from_secs(1)))
.layer(LoadShedLayer::new())  // 超载时丢弃请求
```

### Concurrency Limit
```rust
use tower::limit::ConcurrencyLimitLayer;

.layer(ConcurrencyLimitLayer::new(100))  // 最多 100 并发
```

### Buffer
```rust
use tower::buffer::BufferLayer;

.layer(BufferLayer::new(1024))  // 缓冲 1024 个请求
```

### Retry
```rust
use tower::retry::{RetryLayer, Policy};

struct RetryPolicy {
    max_retries: usize,
}

impl<Req, Res, E> Policy<Req, Res, E> for RetryPolicy {
    type Future = futures::future::Ready<()>;

    fn retry(&self, req: &Req, result: Result<&Res, &E>) -> Option<Self::Future> {
        match result {
            Ok(_) => None,
            Err(_) => {
                if self.max_retries > 0 {
                    Some(futures::future::ready(()))
                } else {
                    None
                }
            }
        }
    }
}

.layer(RetryLayer::new(RetryPolicy { max_retries: 3 }))
```

## Tower-HTTP Middleware

### Trace
```rust
use tower_http::trace::TraceLayer;

.layer(TraceLayer::new_for_http())
```

### CORS
```rust
use tower_http::cors::{CorsLayer, Any};

.layer(CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any))
```

### Compression
```rust
use tower_http::compression::CompressionLayer;

.layer(CompressionLayer::new())
```

### Request Body Limit
```rust
use tower_http::limit::RequestBodyLimitLayer;

.layer(RequestBodyLimitLayer::new(1024 * 1024))  // 1MB
```

### Sensitive Headers
```rust
use tower_http::sensitive_headers::SetSensitiveRequestHeadersLayer;
use http::header::AUTHORIZATION;

.layer(SetSensitiveRequestHeadersLayer::new(vec![
    AUTHORIZATION,
]))
```

### Request ID
```rust
use tower_http::request_id::{SetRequestIdLayer, XRequestIdHeader};

.layer(SetRequestIdLayer::new(XRequestIdHeader, UuidGenerator))
```

## Custom Middleware Patterns

### 状态注入
```rust
pub struct AuthLayer {
    api_url: String,
}

impl AuthLayer {
    pub fn new(api_url: String) -> Self {
        Self { api_url }
    }
}

impl<S> Layer<S> for AuthLayer {
    type Service = AuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthMiddleware {
            inner,
            api_url: self.api_url.clone(),
        }
    }
}
```

### 异步中间件
```rust
impl<S, Request> Service<Request> for AuthMiddleware<S>
where
    S: Service<Request>,
    S::Future: Send + 'static,
{
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn call(&mut self, req: Request) -> Self::Future {
        let api_url = self.api_url.clone();
        let future = self.inner.call(req);

        Box::pin(async move {
            // 异步验证
            let client = reqwest::Client::new();
            let response = client.get(&api_url).send().await?;

            if !response.status().is_success() {
                return Err(AuthError);
            }

            future.await
        })
    }
}
```

## Best Practices

1. **Layer 顺序**: 从外到内执行，注意顺序
2. **错误处理**: 中间件错误要能转换为应用错误
3. **性能**: 使用 `BufferLayer` 处理背压
4. **可观测性**: 添加 `TraceLayer` 进行调试
5. **安全**: 使用 `SetSensitiveRequestHeadersLayer` 隐藏敏感信息

## References

- [Tower Guide](https://github.com/tower-rs/tower/blob/master/guides/building-a-middleware-from-scratch.md)
- [Tower Docs](https://docs.rs/tower)
- [Tower-HTTP Docs](https://docs.rs/tower-http)
