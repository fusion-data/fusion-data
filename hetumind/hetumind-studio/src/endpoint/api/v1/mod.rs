use axum::Router;
use ultimates::core::application::Application;
use ultimates::web::middleware::WebAuth;

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
    .layer(WebAuth::default().with_api_base_url("http://localhost:8080").into_layer())
}
