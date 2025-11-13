use fusion_common::time::{OffsetDateTime, now_offset};
use fusionsql::page::OrderBys;
use fusionsql::{
  ModelManager, SqlError,
  base::DbBmc,
  field::FieldMask,
  filter::{OpValDateTime, OpValInt32, OpValString, OpValUuid},
  generate_pg_bmc_common, generate_pg_bmc_filter,
};
use uuid::Uuid;

use hetuflow_core::types::TaskStatus;

use hetuflow_core::models::{SchedTask, TaskFilter, TaskForCreate, TaskForUpdate};

/// TaskBmc 实现
pub struct TaskBmc;

impl DbBmc for TaskBmc {
  const TABLE: &str = "sched_task";
  const ID_GENERATED_BY_DB: bool = false;
  fn _default_order_bys() -> Option<OrderBys> {
    Some("!id".into())
  }
}

generate_pg_bmc_common!(
  Bmc: TaskBmc,
  Entity: SchedTask,
  ForUpdate: TaskForUpdate,
  ForInsert: TaskForCreate,
);

generate_pg_bmc_filter!(
  Bmc: TaskBmc,
  Entity: SchedTask,
  Filter: TaskFilter,
);

impl TaskBmc {
  /// 查找待处理的任务
  pub async fn find_pending_tasks(mm: &ModelManager, namespace_id: &str) -> Result<Vec<SchedTask>, SqlError> {
    let filter = TaskFilter {
      status: Some(OpValInt32::eq(TaskStatus::Pending as i32)),
      scheduled_at: Some(OpValDateTime::lte(now_offset())),
      namespace_id: Some(OpValString::eq(namespace_id.to_string())),
      ..Default::default()
    };

    Self::find_many(mm, vec![filter], None).await
  }

  /// 批量更新任务状态（优化版本，使用单个SQL语句）
  pub async fn update_tasks_status(mm: &ModelManager, task_ids: Vec<Uuid>, status: TaskStatus) -> Result<(), SqlError> {
    if task_ids.is_empty() {
      return Ok(());
    }

    mm.dbx()
      .use_postgres(|dbx| async move {
        let query = "UPDATE sched_task SET status = $1, updated_at = NOW() WHERE id = ANY($2)";
        sqlx::query(query).bind(status as i32).bind(&task_ids).execute(dbx.db()).await?;
        Ok(())
      })
      .await
      .map_err(SqlError::from)
  }

  /// 重置失败任务为待处理状态
  pub async fn reset_failed_tasks(mm: &ModelManager) -> Result<Vec<SchedTask>, SqlError> {
    let filter = TaskFilter { status: Some(OpValInt32::eq(TaskStatus::Doing as i32)), ..Default::default() };

    let tasks = Self::find_many(mm, vec![filter], None).await?;

    for task in &tasks {
      let update = TaskForUpdate {
        status: Some(TaskStatus::Pending),
        locked_at: None,
        lock_version: Some(task.lock_version + 1),
        update_mask: Some(FieldMask::new(vec![
          "status".to_string(),
          "agent_id".to_string(),
          "server_id".to_string(),
          "locked_at".to_string(),
          "lock_version".to_string(),
          "updated_at".to_string(),
        ])),
        ..Default::default()
      };
      Self::update_by_id(mm, task.id, update).await?;
    }

    Ok(tasks)
  }

  /// 根据命名空间过滤获取任务
  pub async fn acquire_task_for_execution(mm: &ModelManager, task_id: Uuid) -> Result<Option<SchedTask>, SqlError> {
    mm.dbx()
      .use_postgres(|dbx| async move {
        // TODO 是否添加 FOR UPDATE SKIP LOCKED
        let query = r#"
          UPDATE sched_task
          SET status = $1, locked_at = NOW(), lock_version = lock_version + 1, updated_at = NOW()
          WHERE id = $2 AND status = $3
          -- FOR UPDATE SKIP LOCKED
          RETURNING *
        "#;

        let task = sqlx::query_as::<_, SchedTask>(query)
          .bind(TaskStatus::Doing as i32)
          .bind(task_id)
          .bind(TaskStatus::Pending as i32)
          .fetch_optional(dbx.db())
          .await?;

        Ok(task)
      })
      .await
      .map_err(SqlError::from)
  }

  pub async fn find_task_by_schedule_and_time(
    mm: &ModelManager,
    schedule_id: &Uuid,
    scheduled_at: OffsetDateTime,
  ) -> Result<Option<SchedTask>, SqlError> {
    let filter = TaskFilter {
      schedule_id: Some(OpValUuid::eq(*schedule_id)),
      scheduled_at: Some(OpValDateTime::eq(scheduled_at)),
      ..Default::default()
    };

    Self::find_unique(mm, vec![filter]).await
  }

  pub async fn find_retryable_tasks(mm: &ModelManager) -> Result<Vec<SchedTask>, SqlError> {
    mm.dbx()
      .use_postgres(|dbx| async move {
        let query = r#"
          SELECT * FROM sched_task WHERE status = $1 AND retry_count < max_retries AND updated_at <= NOW() - INTERVAL '5 minutes'
        "#;
        let rows = sqlx::query_as::<_, SchedTask>(query)
          .bind(TaskStatus::Failed as i32)
          .fetch_all(dbx.db())
          .await?;
        Ok(rows)
      })
      .await
      .map_err(SqlError::from)
  }
}
