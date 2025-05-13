//! 调度作业聚合服务
//!
mod scheduler_grpc_svc;
mod scheduler_svc;

#[allow(unused_imports)]
pub use scheduler_grpc_svc::SchedulerGrpcSvc;
#[allow(unused_imports)]
pub use scheduler_svc::SchedulerSvc;
