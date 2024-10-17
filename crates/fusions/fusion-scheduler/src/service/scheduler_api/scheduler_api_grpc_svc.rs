use fusion_scheduler_api::v1::{
  scheduler_api_server::{SchedulerApi, SchedulerApiServer},
  CreateProcessRequest, CreateProcessResponse, CreateTriggerRequest, CreateTriggerResponse, EventRequest,
  EventResponse, PullJobRequest, PullJobResponse, RegisterWorkerRequest, RegisterWorkerResponse, UpdateTriggerRequest,
  UpdateTriggerResponse,
};
use fusion_server::{ctx::CtxW, grpc::interceptor::auth_interceptor};
use std::pin::Pin;
use tokio_stream::Stream;
use tonic::{Request, Response, Status, Streaming};
use tracing::debug;
use ultimate_grpc::GrpcServiceIntercepted;

use crate::service::process_definition::ProcessDefinitionSvc;

pub fn scheduler_api_grpc_svc() -> GrpcServiceIntercepted<SchedulerApiServer<SchedulerApiGrpcSvc>> {
  SchedulerApiServer::with_interceptor(SchedulerApiGrpcSvc, auth_interceptor)
}

pub struct SchedulerApiGrpcSvc;

#[tonic::async_trait]
impl SchedulerApi for SchedulerApiGrpcSvc {
  async fn register_worker(
    &self,
    request: Request<RegisterWorkerRequest>,
  ) -> Result<Response<RegisterWorkerResponse>, Status> {
    todo!()
  }

  async fn create_process(
    &self,
    request: Request<CreateProcessRequest>,
  ) -> Result<Response<CreateProcessResponse>, Status> {
    let (_meta, exts, request) = request.into_parts();
    let ctx: &CtxW = (&exts).try_into()?;

    let link_trigger_ids = request.trigger_ids.clone();

    let entity_c = request.into();

    let process_id = ProcessDefinitionSvc::create(ctx, entity_c, link_trigger_ids).await?;
    let process = ProcessDefinitionSvc::find_by_id(ctx, process_id).await?;

    Ok(Response::new(CreateProcessResponse { process: Some(process.into()) }))
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

  type EventListenerStream = Pin<Box<dyn Stream<Item = Result<EventResponse, Status>> + Send>>;

  async fn event_listener(
    &self,
    request: Request<Streaming<EventRequest>>,
  ) -> Result<Response<Self::EventListenerStream>, Status> {
    todo!()
  }

  async fn pull_job(&self, request: Request<PullJobRequest>) -> Result<Response<PullJobResponse>, Status> {
    todo!()
  }
}
