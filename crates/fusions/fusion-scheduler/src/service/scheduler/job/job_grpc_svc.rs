use fusion_server::grpc::interceptor::auth_interceptor;
use tonic::{Request, Response, Status};
use ultimate_grpc::GrpcServiceIntercepted;

use crate::{
  pb::fusion_scheduler::v1::{
    job_server::{self, JobServer},
    CreateJobRequest, CreateJobResponse,
  },
  service::scheduler::JobSvc,
};

pub fn job_grpc_svc() -> GrpcServiceIntercepted<job_server::JobServer<JobGrpcSvc>> {
  JobServer::with_interceptor(JobGrpcSvc, auth_interceptor)
}

pub struct JobGrpcSvc;
#[tonic::async_trait]
impl job_server::Job for JobGrpcSvc {
  async fn create(&self, request: Request<CreateJobRequest>) -> Result<Response<CreateJobResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;

    let entity_c = request.into();
    let job_id = JobSvc::create(ctx, entity_c, None).await?;

    Ok(Response::new(CreateJobResponse { id: job_id.to_string() }))
  }
}
