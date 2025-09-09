use fusion_core::DataError;
use modelsql::ModelManager;
use modelsql::field::FieldMask;
use modelsql::filter::OpValsUuid;
use uuid::Uuid;

use hetuflow_core::models::{
  SchedTask, SchedTaskInstance, TaskFilter, TaskForCreate, TaskForQuery, TaskForUpdate, TaskInstanceFilter,
  TaskInstanceForCreate, TaskInstanceForQuery, TaskInstanceForUpdate,
};
use hetuflow_core::types::{TaskInstanceStatus, TaskStatus};
use modelsql::page::PageResult;

use crate::infra::bmc::{JobBmc, ScheduleBmc, TaskBmc, TaskInstanceBmc};
pub struct TaskSvc {
  mm: ModelManager,
}

impl TaskSvc {
  pub fn new(mm: ModelManager) -> Self {
    Self { mm }
  }

  /// 查找待处理任务
  pub async fn find_pending_tasks(&self, namespace_id: &Uuid) -> Result<Vec<SchedTask>, DataError> {
    TaskBmc::find_pending_tasks(&self.mm, namespace_id).await.map_err(DataError::from)
  }

  /// 创建新任务
  pub async fn create_task(&self, task_data: TaskForCreate) -> Result<Uuid, DataError> {
    let id = task_data.id;
    TaskBmc::insert(&self.mm, task_data).await?;
    Ok(id)
  }

  /// 更新任务状态
  pub async fn update_task_status(&self, task_id: Uuid, status: TaskStatus) -> Result<(), DataError> {
    let update = TaskForUpdate {
      status: Some(status),
      update_mask: Some(FieldMask::new(vec!["status".to_string(), "updated_at".to_string()])),
      ..Default::default()
    };

    TaskBmc::update_by_id(&self.mm, task_id, update).await.map_err(DataError::from).map(|_| ())
  }

  /// 批量分发任务
  pub async fn dispatch_tasks(&self, task_ids: Vec<Uuid>, agent_id: &Uuid) -> Result<(), DataError> {
    for task_id in task_ids {
      let update = TaskForUpdate {
        status: Some(TaskStatus::Dispatched),
        agent_id: Some(*agent_id),
        server_id: None,
        lock_version: Some(1),
        ..Default::default()
      };

      TaskBmc::update_by_id(&self.mm, task_id, update).await.map_err(DataError::from)?;
    }
    Ok(())
  }

  /// 根据作业 ID 查找任务
  pub async fn find_tasks_by_job(&self, job_id: Uuid) -> Result<Vec<SchedTask>, DataError> {
    let filter = TaskFilter { job_id: Some(OpValsUuid::eq(job_id)), ..Default::default() };

    TaskBmc::find_many(&self.mm, vec![filter], None).await.map_err(DataError::from)
  }

  /// 创建任务实例
  pub async fn create_task_instance(&self, mut instance_data: TaskInstanceForCreate) -> Result<Uuid, DataError> {
    let id = if let Some(id) = instance_data.id {
      id
    } else {
      let id = Uuid::now_v7();
      instance_data.id = Some(id);
      id
    };
    TaskInstanceBmc::insert(&self.mm, instance_data).await.map_err(DataError::from)?;
    Ok(id)
  }

  /// 更新任务实例状态
  pub async fn update_task_instance_status(
    &self,
    instance_id: Uuid,
    status: TaskInstanceStatus,
    output: Option<String>,
    error_message: Option<String>,
    exit_code: Option<i32>,
  ) -> Result<(), DataError> {
    let update = TaskInstanceForUpdate { status: Some(status), output, error_message, exit_code, ..Default::default() };
    TaskInstanceBmc::update_by_id(&self.mm, instance_id, update).await.map_err(DataError::from)
  }

  /// 查找任务的所有实例
  pub async fn find_task_instances(&self, task_id: Uuid) -> Result<Vec<SchedTaskInstance>, DataError> {
    let filter = TaskInstanceFilter { task_id: Some(OpValsUuid::eq(task_id)), ..Default::default() };

    TaskInstanceBmc::find_many(&self.mm, vec![filter], None).await.map_err(DataError::from)
  }

  /// 分页查询任务
  pub async fn page(&self, input: TaskForQuery) -> Result<PageResult<SchedTask>, DataError> {
    TaskBmc::page(&self.mm, vec![input.filter], input.page).await.map_err(DataError::from)
  }

  /// 根据 ID 获取任务
  pub async fn get_by_id(&self, id: Uuid) -> Result<Option<SchedTask>, DataError> {
    TaskBmc::get_by_id(&self.mm, &id).await.map_err(DataError::from)
  }

  /// 更新任务
  pub async fn update_task(&self, id: Uuid, task_data: TaskForUpdate) -> Result<(), DataError> {
    TaskBmc::update_by_id(&self.mm, &id, task_data).await.map_err(DataError::from).map(|_| ())
  }

  /// 删除任务
  pub async fn delete_task(&self, id: Uuid) -> Result<(), DataError> {
    TaskBmc::delete_by_id(&self.mm, &id).await.map_err(DataError::from).map(|_| ())
  }

  /// 重试任务
  pub async fn retry_task(&self, id: Uuid) -> Result<(), DataError> {
    let update = TaskForUpdate {
      status: Some(TaskStatus::Pending),
      retry_count: None, // 重试次数会在任务执行时自动增加
      update_mask: Some(FieldMask::new(vec!["status".to_string(), "updated_at".to_string()])),
      ..Default::default()
    };

    TaskBmc::update_by_id(&self.mm, &id, update).await.map_err(DataError::from).map(|_| ())
  }

  /// 取消任务
  pub async fn cancel_task(&self, id: Uuid) -> Result<(), DataError> {
    let update = TaskForUpdate {
      status: Some(TaskStatus::Cancelled),
      update_mask: Some(FieldMask::new(vec!["status".to_string(), "updated_at".to_string()])),
      ..Default::default()
    };

    TaskBmc::update_by_id(&self.mm, &id, update).await.map_err(DataError::from).map(|_| ())
  }

  /// 级联删除作业及其相关数据
  pub async fn delete_job_cascade(&self, job_id: Uuid) -> Result<(), DataError> {
    // 删除任务实例
    let instances = self.find_task_instances(job_id).await?;
    for instance in instances {
      TaskInstanceBmc::delete_by_id(&self.mm, instance.id).await?;
    }

    // 删除任务
    let tasks = self.find_tasks_by_job(job_id).await?;
    for task in tasks {
      TaskBmc::delete_by_id(&self.mm, task.id).await?;
    }

    // 删除调度
    let schedules = ScheduleBmc::find_by_job_id(&self.mm, job_id).await?;
    for schedule in schedules {
      ScheduleBmc::delete_by_id(&self.mm, schedule.id).await?;
    }

    // 删除作业
    JobBmc::delete_by_id(&self.mm, job_id).await?;

    Ok(())
  }

  pub(crate) async fn find_task_instances_page(
    &self,
    input: TaskInstanceForQuery,
  ) -> Result<PageResult<SchedTaskInstance>, DataError> {
    let result = TaskInstanceBmc::page(&self.mm, vec![input.filter], input.page).await?;
    Ok(result)
  }

  /// 根据 ID 获取任务实例
  pub async fn find_task_instance(&self, id: Uuid) -> Result<Option<SchedTaskInstance>, DataError> {
    TaskInstanceBmc::get_by_id(&self.mm, &id).await.map_err(DataError::from)
  }

  /// 删除任务实例
  pub async fn delete_task_instance(&self, id: Uuid) -> Result<(), DataError> {
    TaskInstanceBmc::delete_by_id(&self.mm, &id).await.map_err(DataError::from).map(|_| ())
  }

  /// 更新任务实例
  pub async fn update_task_instance(&self, id: Uuid, instance_data: TaskInstanceForUpdate) -> Result<(), DataError> {
    TaskInstanceBmc::update_by_id(&self.mm, &id, instance_data)
      .await
      .map_err(DataError::from)
      .map(|_| ())
  }
}
