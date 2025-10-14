use axum::Router;
use fusion_core::application::Application;
use fusion_web::middleware::web_auth::WebAuth;
use tower_http::auth::AsyncRequireAuthorizationLayer;

mod credentials;
mod executions;
mod users;
mod workflows;

pub fn v1_routes() -> Router<Application> {
  Router::new()
    .nest("/credentials", credentials::routes())
    .nest("/executions", executions::routes())
    .nest("/workflows", workflows::routes())
    .nest("/users", users::routes())
    .layer(AsyncRequireAuthorizationLayer::new(WebAuth::default()))
}
