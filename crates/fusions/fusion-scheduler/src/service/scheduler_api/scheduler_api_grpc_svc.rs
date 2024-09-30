use fusion_scheduler_api::v1::{
  scheduler_api_server::{SchedulerApi, SchedulerApiServer},
  CreateJobRequest, CreateJobResponse, CreateTriggerRequest, CreateTriggerResponse, PullJobRequest, PullJobResponse,
  RegisterNodeRequest, RegisterNodeResponse, UpdateJobRequest, UpdateJobResponse, UpdateTriggerRequest,
  UpdateTriggerResponse,
};
use fusion_server::{ctx::CtxW, grpc::interceptor::auth_interceptor};
use tonic::{Request, Response, Status};
use tracing::debug;
use ultimate_grpc::GrpcServiceIntercepted;
use uuid::Uuid;

use crate::service::JobSvc;

pub fn scheduler_api_grpc_svc() -> GrpcServiceIntercepted<SchedulerApiServer<SchedulerApiGrpcSvc>> {
  SchedulerApiServer::with_interceptor(SchedulerApiGrpcSvc, auth_interceptor)
}

pub struct SchedulerApiGrpcSvc;

#[tonic::async_trait]
impl SchedulerApi for SchedulerApiGrpcSvc {
  async fn create_job(&self, request: Request<CreateJobRequest>) -> Result<Response<CreateJobResponse>, Status> {
    let (_meta, exts, request) = request.into_parts();
    let ctx: &CtxW = (&exts).try_into()?;

    let tigger_ids = if request.trigger_ids.is_empty() {
      None
    } else {
      let mut trigger_ids = Vec::with_capacity(request.trigger_ids.len());
      for id in request.trigger_ids.iter() {
        let uuid =
          Uuid::parse_str(id).map_err(|_| Status::invalid_argument(format!("Invalid trigger id: '{}'", id)))?;
        trigger_ids.push(uuid);
      }
      Some(trigger_ids)
    };

    let entity_c = request.into();

    let job_id = JobSvc::create(ctx, entity_c, tigger_ids).await?.to_string();

    Ok(Response::new(CreateJobResponse { job_id }))
  }

  async fn update_job(&self, request: Request<UpdateJobRequest>) -> Result<Response<UpdateJobResponse>, Status> {
    debug!("update_job: {:?}", request.into_inner());
    todo!()
  }

  async fn create_trigger(
    &self,
    request: Request<CreateTriggerRequest>,
  ) -> Result<Response<CreateTriggerResponse>, Status> {
    debug!("create_trigger: {:?}", request.into_inner());
    todo!()
  }

  async fn update_trigger(
    &self,
    request: Request<UpdateTriggerRequest>,
  ) -> Result<Response<UpdateTriggerResponse>, Status> {
    debug!("update_trigger: {:?}", request.into_inner());
    todo!()
  }

  async fn pull_job(&self, request: Request<PullJobRequest>) -> Result<Response<PullJobResponse>, Status> {
    todo!()
  }

  async fn register_node(
    &self,
    request: Request<RegisterNodeRequest>,
  ) -> Result<Response<RegisterNodeResponse>, Status> {
    todo!()
  }
}
