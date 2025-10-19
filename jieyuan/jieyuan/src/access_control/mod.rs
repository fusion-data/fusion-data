//! 访问控制服务
//!
//! 提供 IAM (Identity and Access Management) 相关的完整功能，包括：
//! - 认证服务：OAuth 2.0 + PKCE 流程、令牌生成和验证
//! - 授权服务：基于策略的访问控制、资源路径管理
//! - 用户管理：用户认证、租户隔离、权限验证
//! - 策略引擎：策略评估、策略附件管理

mod auth_svc;
mod auth_utils;
mod config;
mod policy_attachment_bmc;
mod policy_bmc;
mod policy_repo;
mod policy_svc;
mod resource_mapping_bmc;
mod resource_mapping_cache_bmc;
mod resource_mapping_svc;
mod types;

pub use auth_svc::AuthSvc;
pub use auth_utils::*;
pub use config::IamConfig;
pub use policy_repo::PolicyRepo;
pub use policy_svc::PolicySvc;
pub use resource_mapping_bmc::ResourceMappingBmc;
pub use resource_mapping_cache_bmc::{CacheStats, ResourceMappingCacheBmc};
pub use resource_mapping_svc::ResourceMappingSvc;
pub use types::{ResolvedResourceMapping, ResourceMappingLookupRequest, ResourceMappingLookupResponse};
