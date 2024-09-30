use tonic::{Request, Response, Status};
use ultimate_grpc::utils::parse_uuid;

use crate::{
  pb::fusion_scheduler::v1::{
    scheduler_server::Scheduler, PageJobRequest, PageJobResponse, PageJobTaskRequest, PageJobTaskResponse,
    PageTriggerRequest, PageTriggerResponse, TriggerJobRequest, TriggerJobResponse,
  },
  service::{JobSvc, JobTaskSvc, TriggerSvc},
};

use super::scheduler_svc::SchedulerSvc;

pub struct SchedulerGrpcSvc;

#[tonic::async_trait]
impl Scheduler for SchedulerGrpcSvc {
  async fn page_job(&self, request: Request<PageJobRequest>) -> Result<Response<PageJobResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;

    let page = JobSvc::page(ctx, request.try_into()?).await?;
    Ok(Response::new(page.into()))
  }

  async fn page_trigger(&self, request: Request<PageTriggerRequest>) -> Result<Response<PageTriggerResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;

    let page = TriggerSvc::page(ctx, request.try_into()?).await?;
    Ok(Response::new(page.into()))
  }

  async fn trigger_job(&self, request: Request<TriggerJobRequest>) -> Result<Response<TriggerJobResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;

    let job_task_id = SchedulerSvc::trigger_job(ctx, parse_uuid(&request.job_id)?).await?;
    Ok(Response::new(TriggerJobResponse { job_task_id: job_task_id.to_string() }))
  }

  async fn page_job_task(&self, request: Request<PageJobTaskRequest>) -> Result<Response<PageJobTaskResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;

    let page = JobTaskSvc::page(ctx, request.try_into()?).await?;
    Ok(Response::new(page.into()))
  }
}
