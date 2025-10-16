pub mod permissions;
pub mod policies;
pub mod roles;
pub mod users;

use fusion_core::application::Application;
use utoipa_axum::router::OpenApiRouter;

pub fn routes() -> OpenApiRouter<Application> {
  OpenApiRouter::new()
    .nest("/users", users::routes())
    .nest("/roles", roles::routes())
    .nest("/permissions", permissions::routes())
    .nest("/policies", policies::routes())
}
