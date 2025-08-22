use std::time::Duration;

use async_trait::async_trait;
use hetumind_core::task::{QueueError, QueueStats, QueueTask, TaskQueue, TaskResult, TaskStatus};
use modelsql::ModelManager;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::TaskQueueEntity;

pub struct PostgresQueue {
  mm: ModelManager,
  config: PostgresQueueConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PostgresQueueConfig {
  pub max_pool_size: u32,
  pub visibility_timeout: Duration,
  pub enable_listen_notify: bool,
}

impl Default for PostgresQueueConfig {
  fn default() -> Self {
    Self { max_pool_size: 10, visibility_timeout: Duration::from_secs(30), enable_listen_notify: false }
  }
}

impl PostgresQueue {
  pub fn new(mm: ModelManager, config: PostgresQueueConfig) -> Self {
    Self { mm, config }
  }

  fn db(&self) -> Result<PgPool, QueueError> {
    self.mm.dbx().db_postgres().map_err(|e| QueueError::InternalError(e.to_string()))
  }

  fn row_to_task(&self, row: TaskQueueEntity) -> Result<(Uuid, QueueTask), QueueError> {
    let task = QueueTask {
      id: row.id.to_string(),
      task_type: row.task_kind.to_string(),
      execution_id: row.execution_id,
      workflow_id: row.workflow_id,
      priority: row.priority,
      payload: row.payload,
      retry_count: row.retry_count as u32,
      max_retries: row.max_retries as u32,
      created_at: row.created_at,
      scheduled_at: Some(row.scheduled_at),
      metadata: serde_json::from_value(row.metadata).unwrap_or_default(),
    };
    Ok((row.id, task))
  }
}

#[async_trait]
impl TaskQueue for PostgresQueue {
  async fn initialize(&self) -> Result<(), QueueError> {
    // 初始化操作 - 表已通过迁移创建
    Ok(())
  }

  async fn enqueue(&self, task: QueueTask) -> Result<Uuid, QueueError> {
    let db = self.db()?;

    let id: Uuid = sqlx::query_scalar(
      r#"INSERT INTO task_queue (
          id, task_kind, execution_id, workflow_id,
          priority, payload, max_retries, metadata
      ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
      RETURNING id"#,
    )
    .bind(Uuid::now_v7())
    .bind(&task.task_type)
    .bind(task.execution_id.as_ref())
    .bind(task.workflow_id.as_ref())
    .bind(task.priority as i32)
    .bind(&task.payload)
    .bind(task.max_retries as i32)
    .bind(serde_json::to_value(task.metadata).unwrap())
    .fetch_one(&db)
    .await
    .map_err(|e| QueueError::InternalError(e.to_string()))?;

    // 如果启用了 LISTEN/NOTIFY，发送通知
    if self.config.enable_listen_notify {
      sqlx::query("NOTIFY task_queue_channel, $1")
        .bind(id)
        .execute(&db)
        .await
        .map_err(|e| QueueError::InternalError(e.to_string()))?;
    }

    Ok(id)
  }

  async fn enqueue_batch(&self, tasks: Vec<QueueTask>) -> Result<Vec<Uuid>, QueueError> {
    let mut ids = Vec::new();
    for task in tasks {
      let id = self.enqueue(task).await?;
      ids.push(id);
    }
    Ok(ids)
  }

  async fn dequeue(&self, worker_id: &Uuid, batch_size: usize) -> Result<Vec<(Uuid, QueueTask)>, QueueError> {
    let db = self.db()?;
    let rows = sqlx::query_as::<_, TaskQueueEntity>(
      r#"
            UPDATE task_queue
            SET
                status = 2,
                worker_id = $1,
                started_at = CURRENT_TIMESTAMP,
                heartbeat_at = CURRENT_TIMESTAMP
            WHERE id IN (
                SELECT id
                FROM task_queue
                WHERE
                    status = 1
                    AND scheduled_at <= CURRENT_TIMESTAMP
                ORDER BY priority DESC, scheduled_at ASC
                LIMIT $2
                FOR UPDATE SKIP LOCKED
            )
            RETURNING *
            "#,
    )
    .bind(worker_id)
    .bind(batch_size as i64)
    .fetch_all(&db)
    .await
    .map_err(|e| QueueError::InternalError(e.to_string()))?;

    rows.into_iter().map(|row| self.row_to_task(row)).collect()
  }

  async fn ack(&self, task_id: &Uuid, result: TaskResult) -> Result<(), QueueError> {
    let db = self.db()?;

    sqlx::query(
      r#"UPDATE task_queue
      SET status = 3, result = $1, completed_at = CURRENT_TIMESTAMP
      WHERE id = $2 "#,
    )
    .bind(result.result)
    .bind(task_id)
    .execute(&db)
    .await
    .map_err(|e| QueueError::InternalError(e.to_string()))?;

    Ok(())
  }

  async fn nack(&self, task_id: &Uuid, error: &str, retry: bool) -> Result<(), QueueError> {
    let db = self.db()?;

    if retry {
      sqlx::query(
        r#"UPDATE task_queue
        SET status = 1, retry_count = retry_count + 1, error_message = $1
        WHERE id = $2 AND retry_count < max_retries"#,
      )
      .bind(error)
      .bind(task_id)
      .execute(&db)
      .await
      .map_err(|e| QueueError::InternalError(e.to_string()))?;
    } else {
      sqlx::query(
        r#"UPDATE task_queue
        SET status = 4, error_message = $1, completed_at = CURRENT_TIMESTAMP
        WHERE id = $2"#,
      )
      .bind(error)
      .bind(task_id)
      .execute(&db)
      .await
      .map_err(|e| QueueError::InternalError(e.to_string()))?;
    }

    Ok(())
  }

  async fn delay(&self, task_id: &Uuid, delay: Duration) -> Result<(), QueueError> {
    let db = self.db()?;

    sqlx::query(
      r#"UPDATE task_queue
      SET scheduled_at = CURRENT_TIMESTAMP + INTERVAL '1 second' * $1
      WHERE id = $2"#,
    )
    .bind(delay.as_secs() as i64)
    .bind(task_id)
    .execute(&db)
    .await
    .map_err(|e| QueueError::InternalError(e.to_string()))?;

    Ok(())
  }

  async fn cancel(&self, task_id: &Uuid) -> Result<(), QueueError> {
    let db = self.db()?;

    sqlx::query(
      r#"UPDATE task_queue
      SET status = 5, completed_at = CURRENT_TIMESTAMP
      WHERE id = $1"#,
    )
    .bind(task_id)
    .execute(&db)
    .await
    .map_err(|e| QueueError::InternalError(e.to_string()))?;

    Ok(())
  }

  async fn get_task_status(&self, task_id: &Uuid) -> Result<Option<TaskStatus>, QueueError> {
    let db = self.db()?;

    let status: Option<TaskStatus> = sqlx::query_scalar("SELECT status FROM task_queue WHERE id = $1")
      .bind(task_id)
      .fetch_optional(&db)
      .await
      .map_err(|e| QueueError::InternalError(e.to_string()))?;

    Ok(status)
  }

  async fn get_stats(&self) -> Result<QueueStats, QueueError> {
    let db = self.db()?;

    let (pending, processing, completed, failed, delayed) = sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(
      r#"SELECT
          COUNT(CASE WHEN status = 1 THEN 1 END) as pending,
          COUNT(CASE WHEN status = 2 THEN 1 END) as processing,
          COUNT(CASE WHEN status = 3 THEN 1 END) as completed,
          COUNT(CASE WHEN status = 4 THEN 1 END) as failed,
          COUNT(CASE WHEN scheduled_at > CURRENT_TIMESTAMP THEN 1 END) as delayed
      FROM task_queue"#,
    )
    .fetch_one(&db)
    .await
    .map_err(|e| QueueError::InternalError(e.to_string()))?;

    Ok(QueueStats {
      pending: pending as u64,
      processing: processing as u64,
      completed: completed as u64,
      failed: failed as u64,
      delayed: delayed as u64,
    })
  }

  async fn cleanup(&self, retention: Duration) -> Result<u64, QueueError> {
    let db = self.db()?;

    let result = sqlx::query(
      r#"DELETE FROM task_queue
      WHERE status IN (3, 4, 5)
      AND completed_at < CURRENT_TIMESTAMP - INTERVAL '1 second' * $1"#,
    )
    .bind(retention.as_secs() as i64)
    .execute(&db)
    .await
    .map_err(|e| QueueError::InternalError(e.to_string()))?;

    Ok(result.rows_affected())
  }
}
