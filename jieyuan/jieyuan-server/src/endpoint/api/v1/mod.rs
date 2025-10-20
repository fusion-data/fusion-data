use fusion_core::application::Application;
use fusion_web::middleware::WebAuth;
use utoipa_axum::router::OpenApiRouter;

mod iams;
mod namespaces;
mod permissions;
mod policies;
mod resource_mappings;
mod roles;
mod tenant_users;
mod tenants;
mod users;

pub fn routes() -> OpenApiRouter<Application> {
  OpenApiRouter::new()
    .nest("/iam", iams::routes())
    .nest("/resource-mappings", resource_mappings::routes())
    .nest("/namespaces", namespaces::routes())
    .nest("/tenants", tenants::routes())
    .nest("/users", users::routes())
    .nest("/roles", roles::routes())
    .nest("/permissions", permissions::routes())
    .nest("/policies", policies::routes())
    .nest("/tenant-users", tenant_users::routes())
    .route_layer(WebAuth::default().into_layer())
}
