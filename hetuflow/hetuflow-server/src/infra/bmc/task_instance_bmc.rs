use modelsql::{
  ModelManager, SqlError,
  base::DbBmc,
  field::FieldMask,
  filter::{OpValsDateTime, OpValsInt32, OpValsUuid},
  generate_pg_bmc_common, generate_pg_bmc_filter,
};
use sqlx::Row;
use ultimate_common::time::now_offset;
use uuid::Uuid;

use hetuflow_core::types::TaskInstanceStatus;

use hetuflow_core::models::{TaskInstanceEntity, TaskInstanceFilter, TaskInstanceForCreate, TaskInstanceForUpdate};

/// TaskInstanceBmc 实现
pub struct TaskInstanceBmc;

impl DbBmc for TaskInstanceBmc {
  const TABLE: &str = "sched_task_instance";
}

generate_pg_bmc_common!(
  Bmc: TaskInstanceBmc,
  Entity: TaskInstanceEntity,
  ForUpdate: TaskInstanceForUpdate,
  ForInsert: TaskInstanceForCreate,
);

generate_pg_bmc_filter!(
  Bmc: TaskInstanceBmc,
  Entity: TaskInstanceEntity,
  Filter: TaskInstanceFilter,
);

impl TaskInstanceBmc {
  /// 统计 Agent 运行中的任务实例数量
  pub async fn count_running_tasks_by_agent(mm: &ModelManager, agent_id: &Uuid) -> Result<u64, SqlError> {
    let filter = TaskInstanceFilter {
      agent_id: Some(OpValsUuid::eq(*agent_id)),
      status: Some(OpValsInt32::eq(TaskInstanceStatus::Running as i32)),
      ..Default::default()
    };

    Self::count(mm, vec![filter]).await
  }

  /// 查找运行中的任务实例
  pub async fn find_running_instances(mm: &ModelManager) -> Result<Vec<TaskInstanceEntity>, SqlError> {
    let filter =
      TaskInstanceFilter { status: Some(OpValsInt32::eq(TaskInstanceStatus::Running as i32)), ..Default::default() };

    Self::find_many(mm, vec![filter], None).await
  }

  /// 根据任务 ID 查找实例
  pub async fn find_many_by_task_id(mm: &ModelManager, task_id: Uuid) -> Result<Vec<TaskInstanceEntity>, SqlError> {
    let filter = TaskInstanceFilter { task_id: Some(OpValsUuid::eq(task_id)), ..Default::default() };

    Self::find_many(mm, vec![filter], None).await
  }

  /// 根据 Agent ID 查找实例
  pub async fn find_many_by_agent_id(mm: &ModelManager, agent_id: &Uuid) -> Result<Vec<TaskInstanceEntity>, SqlError> {
    let filter = TaskInstanceFilter { agent_id: Some(OpValsUuid::eq(*agent_id)), ..Default::default() };

    Self::find_many(mm, vec![filter], None).await
  }

  /// 开始执行任务实例
  pub async fn start_instance(
    mm: &ModelManager,
    instance_id: &Uuid,
    server_id: &Uuid,
    agent_id: &Uuid,
  ) -> Result<(), SqlError> {
    let update = TaskInstanceForUpdate {
      server_id: Some(*server_id),
      agent_id: Some(*agent_id),
      status: Some(TaskInstanceStatus::Running),
      started_at: Some(now_offset()),
      update_mask: Some(FieldMask::new(vec![
        "server_id".to_string(),
        "agent_id".to_string(),
        "status".to_string(),
        "started_at".to_string(),
        "updated_at".to_string(),
      ])),
      ..Default::default()
    };

    Self::update_by_id(mm, instance_id, update).await.map(|_| ())
  }

  /// 完成任务实例
  pub async fn complete_instance(
    mm: &ModelManager,
    instance_id: Uuid,
    status: TaskInstanceStatus,
    output: Option<String>,
    error_message: Option<String>,
    exit_code: Option<i32>,
    metrics: Option<serde_json::Value>,
  ) -> Result<(), SqlError> {
    let update = TaskInstanceForUpdate {
      status: Some(status),
      completed_at: Some(now_offset()),
      output,
      error_message,
      exit_code,
      metrics,
      update_mask: Some(FieldMask::new(vec![
        "status".to_string(),
        "completed_at".to_string(),
        "output".to_string(),
        "error_message".to_string(),
        "exit_code".to_string(),
        "metrics".to_string(),
        "updated_at".to_string(),
      ])),
      ..Default::default()
    };

    Self::update_by_id(mm, instance_id, update).await.map(|_| ())
  }

  /// 取消任务实例
  pub async fn cancel_instance(mm: &ModelManager, instance_id: Uuid) -> Result<(), SqlError> {
    let update = TaskInstanceForUpdate {
      status: Some(TaskInstanceStatus::Cancelled),
      completed_at: Some(now_offset()),
      update_mask: Some(FieldMask::new(vec![
        "status".to_string(),
        "completed_at".to_string(),
        "updated_at".to_string(),
      ])),
      ..Default::default()
    };

    Self::update_by_id(mm, instance_id, update).await.map(|_| ())
  }

  /// 查找超时的任务实例
  pub async fn find_timeout_instances(
    mm: &ModelManager,
    timeout_seconds: i64,
  ) -> Result<Vec<TaskInstanceEntity>, SqlError> {
    let cutoff_time = now_offset() - chrono::Duration::seconds(timeout_seconds);

    let filter = TaskInstanceFilter {
      status: Some(OpValsInt32::eq(TaskInstanceStatus::Running as i32)),
      started_at: Some(OpValsDateTime::lt(cutoff_time)),
      ..Default::default()
    };

    Self::find_many(mm, vec![filter], None).await
  }

  /// 批量取消超时的任务实例
  pub async fn cancel_timeout_instances(mm: &ModelManager, instance_ids: &[Uuid]) -> Result<(), SqlError> {
    if instance_ids.is_empty() {
      return Ok(());
    }

    mm.dbx()
      .use_postgres(|dbx| async move {
        let query = r#"
          UPDATE sched_task_instance
          SET status = 'cancelled',
              completed_at = NOW(),
              error_message = 'Task timeout',
              updated_at = NOW()
          WHERE id = ANY($1) AND status = 'running'
        "#;

        sqlx::query(query).bind(instance_ids).execute(dbx.db()).await?;

        Ok(())
      })
      .await
      .map_err(SqlError::from)
  }

  /// 获取任务实例统计信息
  pub async fn get_instance_stats(mm: &ModelManager, task_id: Option<Uuid>) -> Result<serde_json::Value, SqlError> {
    mm.dbx()
      .use_postgres(|dbx| async move {
        let query = if task_id.is_some() {
          r#"
            SELECT
              status,
              COUNT(*) as count,
              AVG(EXTRACT(EPOCH FROM (completed_at - started_at))) as avg_duration
            FROM sched_task_instance
            WHERE task_id = $1
            GROUP BY status
          "#
        } else {
          r#"
            SELECT
              status,
              COUNT(*) as count,
              AVG(EXTRACT(EPOCH FROM (completed_at - started_at))) as avg_duration
            FROM sched_task_instance
            GROUP BY status
          "#
        };

        let rows = if let Some(task_id) = task_id {
          sqlx::query(query).bind(task_id).fetch_all(dbx.db()).await?
        } else {
          sqlx::query(query).fetch_all(dbx.db()).await?
        };

        let mut stats = serde_json::Map::new();
        for row in rows {
          let status: String = row.get("status");
          let count: i64 = row.get("count");
          let avg_duration: Option<f64> = row.get("avg_duration");

          let mut stat = serde_json::Map::new();
          stat.insert("count".to_string(), serde_json::Value::Number(count.into()));
          if let Some(duration) = avg_duration {
            stat.insert(
              "avg_duration".to_string(),
              serde_json::Value::Number(serde_json::Number::from_f64(duration).unwrap_or(serde_json::Number::from(0))),
            );
          }

          stats.insert(status, serde_json::Value::Object(stat));
        }

        Ok(serde_json::Value::Object(stats))
      })
      .await
      .map_err(SqlError::from)
  }

  /// 找到所有僵尸任务实例
  pub async fn find_zombie_instances(_mm: &ModelManager) -> Result<Vec<TaskInstanceEntity>, SqlError> {
    todo!()
  }
}
