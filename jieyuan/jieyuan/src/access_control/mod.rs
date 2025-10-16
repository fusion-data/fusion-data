//! 访问控制服务
mod config;
mod policy_attachment_bmc;
mod policy_bmc;
mod policy_repo;
mod policy_svc;

pub use config::IamConfig;
pub use jieyuan_core::model::{
  AuthContext, Decision, DecisionEffect, PolicyDocument, build_auth_context, build_auth_context_with_timezone,
};
pub use jieyuan_core::web::middleware::{
  AuthorizationService, AuthorizationServiceExt, RouteMeta, authz_guard, inject_route_meta,
};
pub use policy_repo::PolicyRepo;
pub use policy_svc::PolicySvc;
