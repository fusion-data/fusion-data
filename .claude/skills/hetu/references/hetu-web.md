# hetu-web

Web 框架：Router、Middleware、Extract、Error 处理、服务器构建。

## Imports

```rust
use hetus::web::{
    Router, WebError, WebResult, WebConfig, WebServerBuilder,
    extract::JsonOrForm,
    middleware::WebAuth,
    ok_json,
};
use axum::{
    routing::{get, post, put, delete},
    extract::{Path, State, Json, Query},
    middleware,
};
use tower_http::{
    trace::TraceLayer,
    cors::{CorsLayer, cors},
    compression::CompressionLayer,
    sensitive_headers::SetSensitiveRequestHeadersLayer,
};
use http::header::AUTHORIZATION;
```

## Router 构建模式

### 基本路由

```rust
use axum::Router;
use axum::routing::{get, post, put, delete};

let router = Router::new()
    .route("/api/users", get(list_users))
    .route("/api/users", post(create_user))
    .route("/api/users/:id", get(get_user))
    .route("/api/users/:id", put(update_user))
    .route("/api/users/:id", delete(delete_user))
    .with_state(app.clone());
```

### 嵌套路由

```rust
let router = Router::new()
    .nest("/api/v1", api_v1_routes())
    .nest("/api/v2", api_v2_routes())
    .with_state(app);

fn api_v1_routes() -> Router<App> {
    Router::new()
        .route("/users", get(list_users))
        .route("/users/:id", get(get_user))
}
```

### OpenAPI 路由 (utoipa-axum)

```rust
use utoipa_axum::router::OpenApiRouter;
use utoipa::ToSchema;

pub fn routes() -> OpenApiRouter<Application> {
    OpenApiRouter::new()
        .routes(utoipa_axum::routes!(list_users))
        .routes(utoipa_axum::routes!(create_user))
        .routes(utoipa_axum::routes!(get_user))
}

#[utoipa::path(get, path = "/item", tag = "Users")]
async fn list_users(user_svc: UserSvc) -> WebResult<PageResult<User>> {
    let users = user_svc.list(None, None).await?;
    ok_json!(users)
}
```

## Handler 模式

### 基本结构

```rust
use hetus::web::{WebResult, WebError, ok_json};
use axum::Json;

// 简单响应
async fn list_users(mm: ModelManager) -> WebResult<Vec<User>> {
    let users = UserBmc::list(&mm, None, None).await?;
    ok_json!(users)
}

// 带路径参数
async fn get_user(
    Path(id): Path<i64>,
    mm: ModelManager,
) -> WebResult<User> {
    let user = UserBmc::get(&mm, id).await?
        .ok_or_else(|| WebError::not_found("User not found"))?;
    ok_json!(user)
}

// 带请求体
async fn create_user(
    mm: ModelManager,
    Json(req): Json<UserForCreate>,
) -> WebResult<IdI64Result> {
    let id = UserBmc::create(&mm, req).await?;
    ok_json!(IdI64Result::from(id))
}

// 带查询参数
async fn search_users(
    Query(filter): Query<UserFilter>,
    Query(page): Query<Page>,
    mm: ModelManager,
) -> WebResult<PageResult<User>> {
    let result = UserBmc::list(&mm, Some(filter), Some(page)).await?;
    ok_json!(result)
}
```

### 使用 Service 层

```rust
// Component 作为 extractor
async fn create_user(
    user_svc: UserSvc,  // 自动注入
    Json(req): Json<UserForCreate>,
) -> WebResult<IdI64Result> {
    let id = user_svc.create(req).await?;
    ok_json!(IdI64Result::from(id))
}

// 需要实现 FromRef
impl FromRef<Application> for UserSvc {
    fn from_ref(app: &Application) -> Self {
        app.component()
    }
}
```

## Middleware

### 认证中间件

```rust
use hetus::web::middleware::WebAuth;

let auth = WebAuth::default()
    .with_includes(vec!["/api".to_string()])
    .with_excludes(vec![
        "/api/public".to_string(),
        "/api/health".to_string(),
    ])
    .with_api_base_url("https://auth.example.com")
    .into_layer();

let router = Router::new()
    .route("/api/users", get(list_users))
    .layer(auth);
```

### 自定义中间件

```rust
use axum::middleware::{from_fn, from_fn_with_state};
use axum::{Request, Next, Response};

async fn logging_middleware(
    req: Request,
    next: Next,
) -> Result<Response, WebError> {
    let start = std::time::Instant::now();

    let response = next.run(req).await;

    let duration = start.elapsed();
    tracing::info!("Request took {:?}", duration);

    Ok(response)
}

// 使用
.layer(from_fn(logging_middleware))
```

### 常用 Tower Layer

```rust
use tower_http::{
    trace::TraceLayer,
    cors::{CorsLayer, cors},
    compression::CompressionLayer,
    sensitive_headers::SetSensitiveRequestHeadersLayer,
};

let router = Router::new()
    .route("/api/users", get(list_users))
    .layer(TraceLayer::new_for_http())
    .layer(CorsLayer::new()
        .allow_methods(cors::Any)
        .allow_origin(cors::Any))
    .layer(SetSensitiveRequestHeadersLayer::new(vec![AUTHORIZATION]))
    .layer(CompressionLayer::new());
```

## Extract 提取器

### JsonOrForm

```rust
use hetus::web::extract::JsonOrForm;

// 支持 JSON 和 Form 两种请求
async fn create_user(
    JsonOrForm(user): JsonOrForm<UserForCreate>,
) -> WebResult<User> {
    // ...
}
```

### 常用提取器

```rust
use axum::extract::{
    Path, Query, State, Json, Extension,
    OriginalUri, ConnectInfo, SocketAddr,
};

async fn handler(
    Path(id): Path<i64>,              // 路径参数
    Query(filter): Query<UserFilter>, // 查询参数
    State(mm): State<ModelManager>,   // 应用状态
    Json(body): Json<CreateUser>,     // JSON 请求体
    Extension(ctx): Extension<Ctx>,   // 扩展数据
) -> WebResult<Response> {
    // ...
}
```

## Error 处理

### WebError

```rust
use hetus::web::WebError;

// HTTP 状态码对应
WebError::bad_request("Invalid input")      // 400
WebError::unauthorized("Token expired")     // 401
WebError::forbidden("Access denied")        // 403
WebError::not_found("User not found")       // 404
WebError::conflicted("Resource exists")     // 409
WebError::server_error("Internal error")    // 500

// 带详细信息
WebError::with_detail(
    400,
    "Validation failed",
    json!({ "field": "email", "error": "invalid format" })
)
```

### 错误转换

```rust
use hetus::core::DataError;

// 自动转换 DataError -> WebError
impl From<DataError> for WebError {
    fn from(err: DataError) -> Self {
        WebError::from_data_error(err)
    }
}

// 在 handler 中使用 ?
async fn get_user(id: Path<i64>, mm: ModelManager) -> WebResult<User> {
    let user = UserBmc::get(&mm, *id).await?
        .ok_or_else(|| DataError::not_found("User not found"))?;
    ok_json!(user)
}
```

## WebServerBuilder

```rust
use hetus::web::WebServerBuilder;

// 基本构建
WebServerBuilder::new(router)
    .build()
    .await?;

// 带优雅关闭
WebServerBuilder::new(router)
    .with_shutdown(app.shutdown_recv().await)
    .build()
    .await?;

// 自定义配置
WebServerBuilder::new(router)
    .with_addr("0.0.0.0:8080")
    .with_shutdown(shutdown_rx)
    .build()
    .await?;
```

## 响应类型

```rust
use hetus::web::ok_json;
use hetus::common::model::{WrapperResult, IdI64Result, IdUuidResult};

// 使用 ok_json! 宏
ok_json!(user)                              // 直接返回
ok_json!(WrapperResult { data: users })     // 包装结果
ok_json!(IdI64Result::from(id))             // ID 结果

// 带状态码
async fn create_user(...) -> Result<(StatusCode, Json<IdI64Result>), WebError> {
    let id = user_svc.create(req).await?;
    Ok((StatusCode::CREATED, Json(IdI64Result::from(id))))
}
```

## Best Practices

1. **错误处理**: 使用 `?` 操作符和 `From<DataError> for WebError` 自动转换
2. **中间件顺序**: 注意 Layer 顺序是从下往上执行的
3. **认证**: 使用 `WebAuth` 处理认证，支持远程验证
4. **OpenAPI**: 使用 `utoipa-axum` 自动生成文档
5. **优雅关闭**: 使用 `with_shutdown()` 实现优雅关闭

## Examples from Codebase

- `crates/hetu-web/src/server.rs` - WebServerBuilder 实现
- `hetuflow/hetuflow-server/src/endpoint/api/v1/jobs.rs` - API 端点示例
- `jieyuan/jieyuan-server/src/endpoint/api/v1/users.rs` - 用户 API 示例
