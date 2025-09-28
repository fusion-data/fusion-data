use std::time::Duration;

use fusion_core::DataError;
use log::{debug, info, warn};
use modelsql::ModelManager;
use uuid::Uuid;

use croner::Cron;
use fusion_common::time::{OffsetDateTime, now_offset};
use serde_json::json;
use std::str::FromStr;

use hetuflow_core::models::{SchedJob, SchedSchedule, SchedTask, TaskForCreate, TaskInstanceForCreate};
use hetuflow_core::types::{ScheduleKind, ScheduleStatus, TaskInstanceStatus, TaskStatus};

use crate::infra::bmc::{JobBmc, ScheduleBmc, TaskBmc, TaskInstanceBmc};

/// 调度服务
#[derive(Clone)]
pub struct SchedulerSvc {
  mm: ModelManager,
}

impl SchedulerSvc {
  /// 创建任务生成服务
  pub fn new(mm: ModelManager) -> Self {
    Self { mm }
  }

  /// 基于外部事件生成即时任务
  ///
  /// 行为：
  /// - 事件驱动任务无关联 Schedule
  /// - 默认较高优先级（100）
  /// - 计划时间为当前（立即触发）
  pub async fn generate_event_task(
    &self,
    job_id: Uuid,
    params: Option<serde_json::Value>,
    priority: Option<i32>,
  ) -> Result<Uuid, DataError> {
    // 读取 Job 以获取 namespace_id
    let job = JobBmc::find_by_id(&self.mm, &job_id).await?;

    let parameters = params.unwrap_or_else(|| json!({}));

    let mm = self.mm.get_txn_clone();
    mm.dbx().begin_txn().await?;
    let task_id = Self::create_task_and_instance(&mm, &job, None, now_offset(), parameters, priority).await?;
    mm.dbx().commit_txn().await?;

    info!("Generated event-driven task {} for job {}", task_id, job_id);
    Ok(task_id)
  }

  /// 为指定时间范围预生成任务
  ///
  /// 参数：
  /// - from_time: 起始时间，包含
  /// - to_time: 截止时间，排除
  ///
  /// 返回：已生成任务的 ID 列表
  pub async fn generate_tasks_for_schedule(
    &self,
    from_time: OffsetDateTime,
    to_time: OffsetDateTime,
  ) -> Result<Vec<Uuid>, DataError> {
    let mut generated_task_ids = Vec::new();
    let mm = self.mm.get_txn_clone();
    mm.dbx().begin_txn().await?;

    // 1. 读取 schedule_kind 为 (Cron, Time) 且有效的 SchedSchedule
    let schedule_entities = ScheduleBmc::find_schedulable_entities(&mm).await?;

    // 2. 遍历 schedule_entities，找到对应的有效 SchedJob
    for schedule in schedule_entities {
      // 3. 检查是否过期
      if schedule.end_time.is_some_and(|end_time| end_time < from_time) {
        info!("Schedule {} is expired, end time is {:?}", schedule.id, schedule.end_time);
        ScheduleBmc::update_status_by_id(&mm, schedule.id, ScheduleStatus::Expired).await?;
        continue;
      }

      // 获取关联的 Job
      let job = match JobBmc::find_enabled_by_id(&mm, schedule.job_id).await? {
        Some(job) => job,
        None => {
          info!("Job {} not found for schedule {}, skipping", schedule.job_id, schedule.id);
          continue;
        }
      };

      // 4. 根据 schedule_entity.schedule_kind 调用不同的生成方法
      let task_ids = match schedule.schedule_kind {
        ScheduleKind::Cron => Self::generate_cron_tasks(&mm, &schedule, &job, from_time, to_time).await?,
        ScheduleKind::Interval => Self::generate_interval_tasks(&mm, &schedule, &job, from_time, to_time).await?,
        _ => continue, // 其他类型暂不处理
      };

      generated_task_ids.extend(task_ids);
    }

    mm.dbx().commit_txn().await?;
    debug!("Generated {} tasks for time range {:?} to {:?}", generated_task_ids.len(), from_time, to_time);
    Ok(generated_task_ids)
  }

  /// 为 Cron 类型的 Schedule 生成任务
  async fn generate_cron_tasks(
    mm: &ModelManager,
    schedule: &SchedSchedule,
    job: &SchedJob,
    from_time: OffsetDateTime,
    to_time: OffsetDateTime,
  ) -> Result<Vec<Uuid>, DataError> {
    let mut task_ids = Vec::new();

    // 1. 解析 schedule.cron_expression
    let cron_expression = schedule
      .cron_expression
      .as_deref()
      .ok_or_else(|| DataError::server_error(format!("Not set cron expression for schedule {}", schedule.id)))?;
    let cron = match Cron::from_str(cron_expression) {
      Ok(cron) => cron,
      Err(e) => {
        warn!("Failed to parse cron expression '{}' for schedule {}: {}", cron_expression, schedule.id, e);
        return Err(DataError::server_error(format!("Invalid cron expression: {}, for schedule: {}", e, schedule.id)));
      }
    };

    // 2. 计算在 [from_time, to_time) 范围内的执行时间点
    let mut current_time = from_time;
    let max_iterations = 1000; // 防止无限循环
    let mut iteration_count = 0;

    while current_time < to_time && iteration_count < max_iterations {
      iteration_count += 1;

      // 计算下一次执行时间
      let scheduled_at = match cron.find_next_occurrence(&current_time, true) {
        Ok(next) => {
          if next >= to_time {
            break;
          }
          next
        }
        Err(e) => {
          warn!(
            "Failed to find next occurrence for schedule {}, cron expression: {}, error: {}",
            schedule.id, cron_expression, e
          );
          break;
        }
      };

      // 3. 检查任务是否已存在（去重）
      let existing_task = TaskBmc::find_task_by_schedule_and_time(mm, &schedule.id, scheduled_at).await?;
      if existing_task.is_some() {
        current_time = scheduled_at + Duration::from_secs(1);
        continue;
      }

      let task_id = Self::create_task_and_instance(mm, job, Some(schedule.id), scheduled_at, json!({}), None).await?;
      task_ids.push(task_id);

      info!("Generated cron task {} for schedule {} at {}", task_id, schedule.id, scheduled_at);

      // 移动到下一个时间点
      current_time = scheduled_at + Duration::from_secs(1);
    }

    if iteration_count >= max_iterations {
      warn!("Reached maximum iterations ({}) when generating cron tasks for schedule {}", max_iterations, schedule.id);
    }

    Ok(task_ids)
  }

  /// 为 Interval 类型的 Schedule 生成任务
  async fn generate_interval_tasks(
    mm: &ModelManager,
    schedule: &SchedSchedule,
    job: &SchedJob,
    from_time: OffsetDateTime,
    to_time: OffsetDateTime,
  ) -> Result<Vec<Uuid>, DataError> {
    // Get interval seconds from schedule config
    let interval_secs = schedule.interval_secs.unwrap_or(0);
    if interval_secs == 0 && schedule.max_count.is_some_and(|max_count| max_count > 1) {
      return Err(DataError::bad_request(format!(
        "Not set interval seconds for schedule {}, and max_count > 1, so not generate tasks",
        schedule.id
      )));
    }

    let mut task_ids = Vec::new();

    // Calculate execution times within the range
    let mut schedule_at = schedule.start_time.unwrap_or(from_time);
    while schedule_at < to_time {
      // Check for existing tasks at this time
      let existing_task = TaskBmc::find_task_by_schedule_and_time(mm, &schedule.id, schedule_at).await?;
      if existing_task.is_some() {
        continue;
      }

      let task_id = Self::create_task_and_instance(mm, job, Some(schedule.id), schedule_at, json!({}), None).await?;
      task_ids.push(task_id);

      // Move to next interval
      schedule_at += Duration::from_secs(interval_secs as u64);
    }

    Ok(task_ids)
  }

  /// 生成重试任务
  ///
  /// 为失败的任务创建重试任务实例
  pub async fn generate_retry_tasks(&self) -> Result<Vec<Uuid>, DataError> {
    let mut retry_task_ids = Vec::new();

    // 查找需要重试的任务
    let retryable_tasks = TaskBmc::find_retryable_tasks(&self.mm).await?;

    for task in retryable_tasks {
      // 获取最大重试次数配置
      let max_retries = task.config.max_retries as i32;

      // 检查是否已达到最大重试次数
      if task.retry_count >= max_retries {
        continue;
      }

      // 获取关联的 Job
      let job = JobBmc::find_by_id(&self.mm, &task.job_id).await?;

      let mm = self.mm.get_txn_clone();
      mm.dbx().begin_txn().await?;

      // 创建重试任务
      let retry_task_id = self.create_retry_task_and_instance(&mm, &job, &task, now_offset()).await?;

      mm.dbx().commit_txn().await?;

      retry_task_ids.push(retry_task_id);
      info!("Generated retry task {} for failed task {}", retry_task_id, task.id);
    }

    Ok(retry_task_ids)
  }

  async fn create_retry_task_and_instance(
    &self,
    mm: &ModelManager,
    job: &SchedJob,
    original_task: &SchedTask,
    scheduled_at: OffsetDateTime,
  ) -> Result<Uuid, DataError> {
    let task_id = Uuid::now_v7();
    let task = TaskForCreate {
      id: Some(task_id),
      job_id: job.id,
      namespace_id: Some(job.namespace_id.clone()),
      schedule_id: original_task.schedule_id,
      schedule_kind: Some(ScheduleKind::Event),
      priority: original_task.priority,
      scheduled_at: Some(scheduled_at),
      status: Some(TaskStatus::Pending),
      parameters: original_task.parameters.clone(),
      environment: job.environment.clone(),
      config: Some(job.config.clone()),
      retry_count: original_task.retry_count + 1,
      dependencies: original_task.dependencies.clone(),
    };
    let task_instance = TaskInstanceForCreate {
      id: Some(Uuid::now_v7()),
      job_id: job.id,
      task_id,
      agent_id: None,
      status: TaskInstanceStatus::Pending,
      started_at: Some(scheduled_at),
    };

    TaskBmc::insert(mm, task).await.map_err(DataError::from)?;
    TaskInstanceBmc::insert(mm, task_instance).await.map_err(DataError::from)?;

    Ok(task_id)
  }

  async fn create_task_and_instance(
    mm: &ModelManager,
    job: &SchedJob,
    schedule_id: Option<Uuid>,
    scheduled_at: OffsetDateTime,
    parameters: serde_json::Value,
    priority: Option<i32>,
  ) -> Result<Uuid, DataError> {
    let task_id = Uuid::now_v7();
    let task = TaskForCreate {
      id: Some(task_id),
      job_id: job.id,
      namespace_id: Some(job.namespace_id.clone()),
      schedule_id,
      schedule_kind: Some(ScheduleKind::Event),
      priority: priority.unwrap_or_default(),
      scheduled_at: Some(scheduled_at),
      status: Some(TaskStatus::Pending),
      parameters,
      environment: job.environment.clone(),
      config: Some(job.config.clone()),
      retry_count: 0,
      dependencies: None,
    };
    let task_instance = TaskInstanceForCreate {
      id: Some(Uuid::now_v7()),
      job_id: job.id,
      task_id,
      agent_id: None,
      status: TaskInstanceStatus::Pending,
      started_at: None,
    };

    TaskBmc::insert(mm, task).await.map_err(DataError::from)?; // 入库。等待 Agent 主动 poll 任务执行
    TaskInstanceBmc::insert(mm, task_instance).await.map_err(DataError::from)?;

    Ok(task_id)
  }
}
