mod agents;
mod auth;
mod gateway;
mod jobs;
mod system;
mod task_instances;
mod tasks;

use utoipa_axum::router::OpenApiRouter;

use crate::application::ServerApplication;

pub fn routes() -> OpenApiRouter<ServerApplication> {
  OpenApiRouter::new()
    .nest("/agents", agents::routes())
    .nest("/jobs", jobs::routes())
    .nest("/tasks", tasks::routes())
    .nest("/task-instances", task_instances::routes())
    .nest("/system", system::routes())
    .nest("/gateway", gateway::routes())
    .nest("/auth", auth::routes())
}
