use axum::Router;
use tower_http::auth::AsyncRequireAuthorizationLayer;
use ultimate_core::application::Application;
use ultimate_web::middleware::web_auth::WebAuth;

pub mod auth;
pub mod v1;

pub fn routes() -> Router<Application> {
  Router::new()
    .nest("/v1", v1::v1_routes().layer(AsyncRequireAuthorizationLayer::new(WebAuth::default())))
    .nest("/auth", auth::auth_routes())
}
