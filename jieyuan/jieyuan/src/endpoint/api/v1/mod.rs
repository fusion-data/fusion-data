pub mod iam;
pub mod permissions;
pub mod policies;
pub mod roles;
pub mod tenant_users;
pub mod users;

use fusion_core::application::Application;
use fusion_web::middleware::WebAuth;
use utoipa_axum::router::OpenApiRouter;

pub fn routes() -> OpenApiRouter<Application> {
  OpenApiRouter::new()
    .nest("/iam", iam::routes())
    .nest("/users", users::routes())
    .nest("/roles", roles::routes())
    .nest("/permissions", permissions::routes())
    .nest("/policies", policies::routes())
    .nest("/tenant-users", tenant_users::routes())
    .route_layer(WebAuth::default().into_layer())
}
