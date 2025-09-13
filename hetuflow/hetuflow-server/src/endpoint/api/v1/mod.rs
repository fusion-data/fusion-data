mod agents;
mod auth;
mod gateway;
mod jobs;
mod system;
mod task_instances;
mod tasks;

use fusion_web::Router;

use crate::application::ServerApplication;

pub fn routes() -> Router<ServerApplication> {
  Router::new()
    .nest("/agents", agents::routes())
    .nest("/auth", auth::routes())
    .nest("/jobs", jobs::routes())
    .nest("/tasks", tasks::routes())
    .nest("/task-instances", task_instances::routes())
    .nest("/system", system::routes())
    .nest("/gateway", gateway::routes())
}
