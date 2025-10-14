use axum::extract::FromRequestParts;
use fusion_common::time::now;
use fusion_core::{DataError, application::Application};
use fusion_web::WebError;
use fusionsql::{ModelManager, page::PageResult};
use hetumind_context::utils::get_mm_from_parts;
use hetumind_core::workflow::{
  ExecuteWorkflowRequest, ExecutionGraph, ExecutionId, ExecutionIdResponse, ExecutionMode, ExecutionStatus,
  ValidateWorkflowRequest, ValidateWorkflowResponse, ValidationError, Workflow, WorkflowForCreate, WorkflowForQuery,
  WorkflowForUpdate, WorkflowId, WorkflowStatus,
};
use http::request::Parts;

use super::{ExecutionBmc, ExecutionDataBmc, ExecutionDataEntity, ExecutionEntity, WorkflowBmc};

pub struct WorkflowSvc {
  mm: ModelManager,
}

impl WorkflowSvc {
  pub async fn get_workflow(&self, id: &WorkflowId) -> Result<Workflow, DataError> {
    let entity = WorkflowBmc::find_by_id(&self.mm, id.clone()).await?;
    Workflow::try_from(entity)
  }

  pub async fn delete_workflow(&self, workflow_id: &WorkflowId) -> Result<(), DataError> {
    WorkflowBmc::delete_by_id(&self.mm, workflow_id.clone()).await?;
    Ok(())
  }

  pub async fn duplicate_workflow(&self, workflow_id: &WorkflowId) -> Result<WorkflowId, DataError> {
    let mut wf = self.get_workflow(workflow_id).await?;
    wf.id = WorkflowId::now_v7(); // 复制时，生成新 id
    wf.name = format!("{} (Copy)", wf.name); // 复制时，修改名称
    wf.version = None; // 复制时，版本号清空
    let entity_c = WorkflowForCreate::try_from(wf).map_err(|e| DataError::bad_request(e.to_string()))?;
    let id = self.create(entity_c).await?;
    Ok(id)
  }

  pub async fn query_workflows(&self, input: WorkflowForQuery) -> Result<PageResult<Workflow>, DataError> {
    let res = WorkflowBmc::page(&self.mm, vec![input.filter], input.options).await?;
    let result = res.result.into_iter().map(Workflow::try_from).collect::<Result<Vec<_>, _>>()?;
    Ok(PageResult::new(res.page.total, result))
  }

  pub async fn create(&self, mut entity_c: WorkflowForCreate) -> Result<WorkflowId, DataError> {
    if entity_c.id.is_none() {
      entity_c.id = Some(WorkflowId::now_v7());
    }
    let workflow_id = entity_c.id.clone().unwrap();

    let wf = Workflow::try_from(entity_c.clone()).map_err(|e| DataError::bad_request(e.to_string()))?;
    let errors = self.validate_base(&wf);
    if !errors.is_empty() {
      return Err(DataError::biz_error(400, "工作流不合法", Some(serde_json::to_value(errors).unwrap())));
    }

    WorkflowBmc::insert(&self.mm, entity_c).await?;
    Ok(workflow_id)
  }

  /// 更新工作流
  pub async fn update(&self, id: &WorkflowId, mut input: WorkflowForUpdate) -> Result<WorkflowId, DataError> {
    let wf = Workflow::try_from(input.clone())?;
    let errors = self.validate_base(&wf);
    if !errors.is_empty() {
      return Err(DataError::biz_error(400, "工作流不合法", Some(serde_json::to_value(errors).unwrap())));
    }

    let mm = self.mm.get_txn_clone();
    mm.dbx().begin_txn().await?;

    // 不更新状态，只更新其他字段。如果状态发生变化，后面单独进行更新
    let update_status = input.status.take();

    WorkflowBmc::update_by_id(&mm, id.clone(), input).await?;

    if let Some(status) = update_status {
      // 如果状态为活跃，则校验工作流是否合法
      if status == WorkflowStatus::Active {
        let wf = self.get_workflow(id).await?;
        self
          .validate_activate(wf)
          .map_err(|errors| DataError::biz_error(400, "工作流不合法", Some(serde_json::to_value(errors).unwrap())))?;
      }
      WorkflowBmc::update_by_id(&mm, id.clone(), WorkflowForUpdate { status: Some(status), ..Default::default() })
        .await?;
    }

    mm.dbx().commit_txn().await?;

    Ok(id.clone())
  }

  // 基础校验
  // - 工作流是否存在循环依赖
  fn validate_base(&self, workflow: &Workflow) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    let graph = ExecutionGraph::new(workflow);
    if graph.has_cycles() {
      errors.push(ValidationError::WorkflowHasCycles);
    }

    errors
  }

  // 校验工作流是否可激活
  // - 基础校验 [validate_base]
  // - 工作流是否存在未连接的必需输入端口 [validate_connectivity]
  fn validate_activate(&self, workflow: Workflow) -> Result<(), Vec<ValidationError>> {
    let mut errors = self.validate_base(&workflow);
    if let Err(errs) = workflow.validate_connectivity(Application::global().component()) {
      errors.extend(errs);
    }

    // TODO 其它校验

    if errors.is_empty() { Ok(()) } else { Err(errors) }
  }

  pub async fn validate_workflow_from_request(
    &self,
    input: ValidateWorkflowRequest,
  ) -> Result<ValidateWorkflowResponse, DataError> {
    let workflow = if let Some(wf) = input.workflow {
      wf
    } else if let Some(id) = input.id.as_ref() {
      self.get_workflow(id).await?
    } else {
      return Err(DataError::bad_request("Either 'workflow' or 'id' must be provided"));
    };

    match self.validate_activate(workflow) {
      Ok(_) => Ok(ValidateWorkflowResponse { is_valid: true, errors: None }),
      Err(errors) => Ok(ValidateWorkflowResponse { is_valid: false, errors: Some(errors) }),
    }
  }

  pub async fn execute_workflow(
    &self,
    workflow_id: &WorkflowId,
    input: ExecuteWorkflowRequest,
  ) -> Result<ExecutionIdResponse, DataError> {
    // 1. 获取并验证工作流
    let workflow = self.get_workflow(workflow_id).await?;

    // 检查工作流状态
    if workflow.status != WorkflowStatus::Active {
      return Err(DataError::bad_request("工作流未激活，无法执行"));
    }

    // 2. 创建执行记录
    let created_by = self.mm.ctx_ref()?.uid();
    let execution_id = ExecutionId::now_v7();
    let execution_entity = ExecutionEntity {
      id: execution_id.clone(),
      workflow_id: workflow_id.clone(),
      mode: ExecutionMode::Local,
      status: ExecutionStatus::New,
      started_at: None,
      finished_at: None,
      triggered_by: Some(created_by),
      wait_till: None,
      retry_of: None,
      retry_success_id: None,
      logical_deletion: None,
      created_at: now(),
      created_by,
      updated_at: None,
      updated_by: None,
    };

    let mm = self.mm.get_txn_clone();
    mm.dbx().begin_txn().await?; // 开启事务

    // 插入执行记录
    ExecutionBmc::insert(&mm, execution_entity).await?;

    // 3. 创建执行数据记录
    let workflow_data = serde_json::to_value(&workflow)?;
    let execution_data_entity = ExecutionDataEntity {
      execution_id: execution_id.clone(),
      workflow_data,
      data: "{}".to_string(), // 初始为空
    };
    ExecutionDataBmc::insert(&mm, execution_data_entity).await?;

    // 4. TODO: 触发异步执行
    // 在实际生产环境中，这里应该：
    // - 将执行任务放入消息队列（如 Redis、RabbitMQ）
    // - 或使用专门的任务调度系统
    // - 由后台 worker 进程处理实际的工作流执行

    mm.dbx().commit_txn().await?; // 提交事务

    Ok(ExecutionIdResponse { execution_id })
  }
}

impl WorkflowSvc {
  pub fn new(mm: ModelManager) -> Self {
    Self { mm }
  }
}

impl FromRequestParts<Application> for WorkflowSvc {
  type Rejection = WebError;

  async fn from_request_parts(parts: &mut Parts, state: &Application) -> Result<Self, Self::Rejection> {
    get_mm_from_parts(parts, state).map(Self::new)
  }
}
