use axum::Router;
use ultimate_core::application::Application;

mod executions;
mod users;
mod workflows;

pub fn v1_routes() -> Router<Application> {
  Router::new()
    .nest("/executions", executions::routes())
    .nest("/workflows", workflows::routes())
    .nest("/users", users::routes())
}
