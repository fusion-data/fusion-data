//! 访问控制服务
mod config;
mod policy_attachment_bmc;
mod policy_bmc;
mod policy_repo;
mod policy_svc;

pub use config::IamConfig;
pub use policy_repo::PolicyRepo;
pub use policy_svc::PolicySvc;
