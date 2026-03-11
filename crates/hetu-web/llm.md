# hetu-web LLM 使用指南

> hetu-web 是 Fusion-Data 项目的 Web 层封装，基于 Axum 框架，提供 Router、WebServerBuilder、中间件等工具。

## 架构概览

```
hetu-web/
├── config.rs        # WebConfig 配置
├── error.rs         # WebError 错误类型
├── extract.rs       # 请求提取器 (JsonOrForm)
├── lib.rs           # 主模块导出
├── middleware/      # 中间件
│   ├── mod.rs
│   ├── utils.rs
│   └── web_auth.rs  # WebAuth 认证中间件
├── server.rs        # WebServerBuilder
└── util.rs          # 工具函数
```

---

## Router (Axum)

```rust
use hetu_web::Router;

pub fn routes() -> Router {
  Router::new()
    .route("/health", get(health_handler))
    .route("/api/v1/users", get(list_users).post(create_user))
    .with_state(app_state)
}
```

---

## WebServerBuilder

```rust
use hetu_web::server::WebServerBuilder;
use hetu_core::application::Application;

let router = crate::endpoint::routes();
let shutdown_rx = Application::global().shutdown_recv().await;

WebServerBuilder::new(router)
  .with_shutdown(shutdown_rx)
  .build()
  .await?;
```

### 配置项 (fusion.web.\*)

```toml
[hetu.web]
enable = true
server_addr = "0.0.0.0:8080"
enable_remote_addr = true  # 启用远程地址获取
```

---

## WebError 错误处理

### 创建错误

```rust
use hetu_web::{WebError, WebResult};

// 4xx 客户端错误
WebError::bad_request("参数错误");
WebError::unauthorized("未登录");
WebError::forbidden("权限不足");
WebError::not_found("资源不存在");
WebError::conflict("资源冲突");
WebError::unprocessable_entity("验证失败");

// 5xx 服务器错误
WebError::server_error("内部错误");
WebError::not_implemented("功能未实现");
WebError::service_unavailable("服务不可用");

// 自定义
WebError::new_with_code(418, "我是茶壶");
WebError::new_with_msg("自定义消息");
WebError::server_error_with_detail("错误", serde_json::json!({ "field": "detail" }));
```

### 错误响应

```rust
// WebResult 类型
type WebResult<T> = Result<Json<T>, WebError>;

// 转换为 HTTP 响应
impl IntoResponse for WebError {
  fn into_response(self) -> Response {
    let status = StatusCode::from_u16(self.err_code as u16).unwrap_or(INTERNAL_SERVER_ERROR);
    let mut res = Json(self).into_response();
    *res.status_mut() = status;
    res
  }
}
```

### From 转换

```rust
impl From<DataError> for WebError {
  fn from(err: DataError) -> Self {
    Self::new_with_msg(err.msg.clone())
      .with_err_code(err.code)
      .with_details(err.data.clone().unwrap_or_default())
  }
}

impl From<hyper::Error> for WebError {}
impl From<serde_json::Error> for WebError {}
```

---

## 提取器 (Extractors)

### JsonOrForm

```rust
use hetu_web::extract::JsonOrForm;

#[derive(Deserialize)]
struct CreateUserRequest {
  name: String,
  email: String,
}

async fn create_user(JsonOrForm(req): JsonOrForm<CreateUserRequest>) -> WebResult<Json<User>> {
  // 自动解析 JSON 或 Form 表单
  let user = user_service.create(req).await?;
  Ok(Json(user))
}
```

### Ctx 提取

```rust
use hetus::common::ctx::Ctx;
use hetu_core::application::Application;

async fn get_user(
  Path(id): Path<i64>,
  State(state): State<AppState>,
) -> WebResult<Json<User>> {
  let ctx = Application::global().ctx();
  let user = user_bmc.get_by_id(&state.mm, id)
    .await?
    .ok_or(WebError::not_found("用户不存在"))?;
  Ok(Json(user))
}
```

---

## 中间件

### WebAuth 认证中间件

```rust
use hetu_web::middleware::WebAuth;

let auth = WebAuth::default()
  .with_includes(vec!["/api".to_string()])  // 需要认证的路径
  .with_excludes(vec!["/health".to_string()])  // 排除认证的路径
  .with_api_base_url("http://auth-server")  // 远程认证服务器
  .into_layer();

let router = Router::new()
  .layer(auth)
  .route(...)
```

---

## 工具函数

### extract_ctx (从请求提取 Ctx)

```rust
use hetu_web::extract_ctx;
use hetu_core::application::Application;

let ctx = extract_ctx(&parts, security_setting)?;
parts.extensions.insert(ctx);
```

---

## 使用模式

### 完整 HTTP 服务启动

```rust
use hetu_core::{DataError, application::Application, logforth::LogforthPlugin};
use hetu_db::DbPlugin;
use hetu_web::server::WebServerBuilder;

async fn start_http_server() -> Result<(), DataError> {
  let app = Application::global();
  let shutdown_rx = app.shutdown_recv().await;

  let router = crate::endpoint::routes();

  WebServerBuilder::new(router)
    .with_shutdown(shutdown_rx)
    .build()
    .await
}
```

---

**版本**: hetu-web v0.1.0+
