# Axum Web Framework Patterns

基于 [Axum 官方文档](https://docs.rs/axum) 和 [Shuttle 指南](https://www.shuttle.dev/blog/2023/12/06/using-axum-rust)。

## Router

### 基本路由
```rust
use axum::{
    Router,
    routing::{get, post, put, delete},
};

let router = Router::new()
    .route("/users", get(list_users))
    .route("/users", post(create_user))
    .route("/users/:id", get(get_user))
    .route("/users/:id", put(update_user))
    .route("/users/:id", delete(delete_user));
```

### 嵌套路由
```rust
let router = Router::new()
    .nest("/api/v1", v1_routes())
    .nest("/api/v2", v2_routes());

fn v1_routes() -> Router {
    Router::new()
        .route("/users", get(list_users))
        .route("/posts", get(list_posts))
}
```

### 路由组
```rust
let router = Router::new()
    .merge(users_routes())
    .merge(posts_routes());

fn users_routes() -> Router {
    Router::new()
        .route("/users", get(list_users))
        .route("/users/:id", get(get_user))
}
```

## Handlers

### 基本结构
```rust
async fn list_users() -> Json<Vec<User>> {
    Json(vec![])
}

async fn get_user(Path(id): Path<i64>) -> Json<User> {
    Json(User { id, name: "Test".into() })
}
```

### 多个 Extractor
```rust
async fn create_user(
    Path(tenant_id): Path<i64>,           // 路径参数
    Query(filter): Query<UserFilter>,     // 查询参数
    State(mm): State<ModelManager>,       // 应用状态
    Json(body): Json<CreateUser>,         // JSON 请求体
) -> Result<Json<User>, WebError> {
    let user = UserSvc::create(&mm, tenant_id, body).await?;
    Ok(Json(user))
}
```

### 返回类型
```rust
// 简单 JSON
async fn get_user() -> Json<User> { ... }

// 带状态码
async fn create_user() -> (StatusCode, Json<User>) {
    (StatusCode::CREATED, Json(user))
}

// 错误处理
async fn get_user() -> Result<Json<User>, WebError> { ... }

// 原始响应
async fn get_user() -> Response {
    Json(user).into_response()
}

// 空
async fn delete_user() -> StatusCode {
    StatusCode::NO_CONTENT
}
```

## Extractors

### Path
```rust
// 单个参数
Path(id): Path<i64>

// 多个参数
Path(params): Path<(i64, String)>

// 结构体
#[derive(Deserialize)]
struct UserPath {
    tenant_id: i64,
    user_id: i64,
}
Path(params): Path<UserPath>
```

### Query
```rust
#[derive(Deserialize)]
struct UserQuery {
    name: Option<String>,
    page: Option<u32>,
}
Query(query): Query<UserQuery>
```

### Json
```rust
Json(body): Json<CreateUser>

// 可选 JSON
Json(body): Option<Json<CreateUser>>
```

### State
```rust
// 注册状态
let router = Router::new()
    .route("/users", get(list_users))
    .with_state(app);

// 提取
State(app): State<Application>

// 多个状态（使用 tuple）
State((mm, config)): State<(ModelManager, Config)>
```

### Extension
```rust
// 中间件注入
.layer(Extension(ctx))

// 提取
Extension(ctx): Extension<Ctx>
```

### 自定义 Extractor
```rust
use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};

struct UserContext(Ctx);

impl<S> FromRequestParts<S> for UserContext
where
    S: Send + Sync,
{
    type Rejection = WebError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let ctx = parts.extensions.get::<Ctx>()
            .ok_or_else(|| WebError::unauthorized("No context"))?;

        Ok(UserContext(ctx.clone()))
    }
}

// 使用
async fn handler(UserContext(ctx): UserContext) -> Json<User> {
    // ...
}
```

## Error Handling

### IntoResponse
```rust
use axum::response::{IntoResponse, Response};

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        let status = match self.code {
            400 => StatusCode::BAD_REQUEST,
            401 => StatusCode::UNAUTHORIZED,
            403 => StatusCode::FORBIDDEN,
            404 => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(json!({
            "code": self.code,
            "message": self.message,
            "detail": self.detail,
        }));

        (status, body).into_response()
    }
}
```

### Result Handler
```rust
// 自动转换错误
async fn handler() -> Result<Json<User>, WebError> {
    let user = find_user().await?;  // 错误自动转换
    Ok(Json(user))
}

// 需要实现 From
impl From<DataError> for WebError {
    fn from(err: DataError) -> Self {
        WebError::from_data_error(err)
    }
}
```

## Middleware

### from_fn
```rust
use axum::middleware::{from_fn, from_fn_with_state};

async fn auth_middleware(
    State(settings): State<SecuritySetting>,
    req: Request,
    next: Next,
) -> Result<Response, WebError> {
    // 验证逻辑
    let response = next.run(req).await;
    Ok(response)
}

// 无状态
.layer(from_fn(logging_middleware))

// 有状态
.layer(from_fn_with_state(settings, auth_middleware))
```

### 常用 Tower Layer
```rust
use tower_http::{
    trace::TraceLayer,
    cors::{CorsLayer, cors},
    compression::CompressionLayer,
    limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer,
    sensitive_headers::SetSensitiveRequestHeadersLayer,
};

.layer(TraceLayer::new_for_http())
.layer(CorsLayer::new()
    .allow_origin(cors::Any)
    .allow_methods(cors::Any)
    .allow_headers(cors::Any))
.layer(CompressionLayer::new())
.layer(RequestBodyLimitLayer::new(1024 * 1024))
.layer(TimeoutLayer::new(Duration::from_secs(30)))
.layer(SetSensitiveRequestHeadersLayer::new(vec![AUTHORIZATION]))
```

## Best Practices

1. **状态管理**: 使用 `Arc` 包装共享状态
2. **错误处理**: 为所有错误实现 `IntoResponse`
3. **路由组织**: 使用 `nest` 和 `merge` 组织路由
4. **中间件顺序**: 注意 Layer 是从下往上执行的
5. **类型安全**: 使用自定义 Extractor 封装通用逻辑

## References

- [Axum Docs](https://docs.rs/axum)
- [Tower HTTP Docs](https://docs.rs/tower-http)
- [Shuttle Axum Guide](https://www.shuttle.dev/blog/2023/12/06/using-axum-rust)
