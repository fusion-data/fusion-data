//! 调度任务 API。用于调度作业 Worker 集群间和 Client 通信。
//!
mod scheduler_api_grpc_svc;

pub use scheduler_api_grpc_svc::scheduler_api_grpc_svc;
