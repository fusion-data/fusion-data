use tonic::{Request, Response, Status};

use crate::{
  pb::fusion_flow::v1::{
    HealthCheckRequest, HealthCheckResponse, PageProcessRequest, PageProcessResponse, PageProcessTaskRequest,
    PageProcessTaskResponse, PageTriggerRequest, PageTriggerResponse, TriggerProcessRequest, TriggerProcessResponse,
    scheduler_server::Scheduler,
  },
  service::{
    process_definition::ProcessDefinitionSvc, process_task::ProcessTaskSvc, trigger_definition::TriggerDefinitionSvc,
  },
};

use super::scheduler_svc::SchedulerSvc;

pub struct SchedulerGrpcSvc;

#[tonic::async_trait]
impl Scheduler for SchedulerGrpcSvc {
  async fn page_process(&self, request: Request<PageProcessRequest>) -> Result<Response<PageProcessResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;

    let page = ProcessDefinitionSvc::page(ctx, request.try_into()?).await?;
    Ok(Response::new(page.into()))
  }

  async fn page_trigger(&self, request: Request<PageTriggerRequest>) -> Result<Response<PageTriggerResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;

    let page = TriggerDefinitionSvc::page(ctx, request.try_into()?).await?;
    Ok(Response::new(page.into()))
  }

  async fn trigger_process(
    &self,
    request: Request<TriggerProcessRequest>,
  ) -> Result<Response<TriggerProcessResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;

    let job_task_id = SchedulerSvc::trigger_process(
      ctx,
      request.process_id.parse().map_err(|e| Status::invalid_argument(format!("Invalid UUID: {}", e)))?,
    )
    .await?;
    Ok(Response::new(TriggerProcessResponse { job_task_id: job_task_id.to_string() }))
  }

  async fn page_process_task(
    &self,
    request: Request<PageProcessTaskRequest>,
  ) -> Result<Response<PageProcessTaskResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;

    let page = ProcessTaskSvc::page(ctx, request.try_into()?).await?;
    Ok(Response::new(page.into()))
  }

  async fn healthy_check(
    &self,
    _request: Request<HealthCheckRequest>,
  ) -> Result<Response<HealthCheckResponse>, Status> {
    todo!()
  }
}
