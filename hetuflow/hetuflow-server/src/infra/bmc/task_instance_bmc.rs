use fusion_common::time::now_offset;
use modelsql::{
  ModelManager, SqlError,
  base::DbBmc,
  field::FieldMask,
  filter::{OpValsDateTime, OpValsInt32},
  generate_pg_bmc_common, generate_pg_bmc_filter,
};
use sqlx::Row;
use uuid::Uuid;

use hetuflow_core::{
  protocol::AcquireTaskRequest,
  types::{TaskInstanceStatus, TaskStatus},
};

use hetuflow_core::models::{SchedTaskInstance, TaskInstanceFilter, TaskInstanceForCreate, TaskInstanceForUpdate};

/// TaskInstanceBmc 实现
pub struct TaskInstanceBmc;

impl DbBmc for TaskInstanceBmc {
  const TABLE: &str = "sched_task_instance";
  const ID_GENERATED_BY_DB: bool = false;
}

generate_pg_bmc_common!(
  Bmc: TaskInstanceBmc,
  Entity: SchedTaskInstance,
  ForUpdate: TaskInstanceForUpdate,
  ForInsert: TaskInstanceForCreate,
);

generate_pg_bmc_filter!(
  Bmc: TaskInstanceBmc,
  Entity: SchedTaskInstance,
  Filter: TaskInstanceFilter,
);

impl TaskInstanceBmc {
  /// 开始执行任务实例
  pub async fn start_instance(mm: &ModelManager, instance_id: &Uuid, agent_id: String) -> Result<(), SqlError> {
    let update = TaskInstanceForUpdate {
      agent_id: Some(agent_id),
      status: Some(TaskInstanceStatus::Running),
      started_at: Some(now_offset()),
      update_mask: Some(FieldMask::new(vec!["agent_id", "status", "started_at", "updated_at"])),
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
  ) -> Result<Vec<SchedTaskInstance>, SqlError> {
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
  pub async fn find_zombie_instances(_mm: &ModelManager) -> Result<Vec<SchedTaskInstance>, SqlError> {
    // TODO: 找到所有僵尸任务实例，Server 端需要判断僵尸任务实例吗？
    Ok(vec![])
  }

  /// 拉取到 SchedTaskInstance 后，将 request.agent_id 绑定到 SchedTaskInstance.agent_id 上
  pub async fn find_many_by_poll(
    mm: &ModelManager,
    request: &AcquireTaskRequest,
  ) -> Result<Vec<SchedTaskInstance>, SqlError> {
    // SQL 语句：
    // 1. 查询符合条件的 task_instance
    // 2. 更新对应 task 状态
    // 3. 更新对应 task_instance 状态 和 agent_id
    // 4. 返回更新后的 task_instance
    let mm = mm.get_txn_clone();
    mm.dbx().begin_txn().await?;

    // 构建标签匹配条件：使用 JSON 包含查询，类似 k8s label selector
    // Agent 的 labels 必须是 Task config.labels 的子集
    let where_labels =
      if request.labels.is_empty() { "" } else { "and (t.config->'labels' is null or t.config->'labels' <@ $4)" };

    let sql = format!(
      r#"with sti as (select ti.*
                  from sched_task_instance ti
                          inner join sched_task t on t.id = ti.task_id
                  where ti.status = $1 and t.status = any($2) and t.scheduled_at <= $3 {}
                  limit $5),
              update_task as (update sched_task set status = $6 where id in (select task_id from sti) and status = $7)
      update sched_task_instance
      set status   = $8,
          agent_id = $9
      where id in (select id from sti)
      returning sched_task_instance.*"#,
      where_labels
    );

    let mut query = sqlx::query_as::<_, SchedTaskInstance>(&sql)
      .bind(TaskInstanceStatus::Pending)
      .bind(TaskStatus::runnables())
      .bind(request.max_scheduled_at);
    if !request.labels.is_empty() {
      let agent_labels_json = serde_json::to_value(&request.labels).map_err(|e| SqlError::InvalidArgument {
        message: format!("Failed to serialize Labels to JSON, error: {}", e),
      })?;
      query = query.bind(agent_labels_json);
    };
    query = query
      .bind(request.acquire_count as i32)
      .bind(TaskStatus::Doing)
      .bind(TaskStatus::Pending)
      .bind(TaskInstanceStatus::Dispatched)
      .bind(&request.agent_id);

    let task_instances = mm.dbx().db_postgres()?.fetch_all(query).await?;

    mm.dbx().commit_txn().await?;
    Ok(task_instances)
  }
}
