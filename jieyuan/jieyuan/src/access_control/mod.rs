//! 访问控制服务
pub mod auth_ctx;
mod config;
mod middleware;
mod policy_attachment_bmc;
mod policy_bmc;
mod policy_engine;
mod policy_repo;
mod policy_svc;

pub use config::IamConfig;
pub use middleware::{RouteMeta, authz_guard, inject_route_meta};
pub use policy_engine::Decision;
pub use policy_repo::PolicyRepo;
pub use policy_svc::PolicySvc;
