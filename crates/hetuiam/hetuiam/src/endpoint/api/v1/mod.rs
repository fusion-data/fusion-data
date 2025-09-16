pub mod auth;
pub mod permissions;
pub mod policies;
pub mod roles;
pub mod users;

use fusion_core::application::Application;
use utoipa_axum::router::OpenApiRouter;

pub fn routes() -> OpenApiRouter<Application> {
  OpenApiRouter::new()
    .merge(auth::routes())
    .merge(users::routes())
    .merge(roles::routes())
    .merge(permissions::routes())
    .merge(policies::routes())
}
