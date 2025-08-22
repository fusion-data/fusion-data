use std::sync::Arc;

use hetumind_core::task::TaskQueue;
use log::{error, info};
use uuid::Uuid;

use crate::runtime::worker::TaskProcessor;

use super::WorkerConfig;

pub struct GenericWorker {
  worker_id: Uuid,
  queue: Arc<dyn TaskQueue>,
  processor: Arc<dyn TaskProcessor>,
  config: WorkerConfig,
  shutdown: tokio::sync::watch::Receiver<bool>,
}

impl GenericWorker {
  pub async fn run(&self) {
    info!("Worker started");

    let mut poll_interval = tokio::time::interval(self.config.poll_interval);
    let mut shutdown = self.shutdown.clone();

    loop {
      tokio::select! {
          _ = poll_interval.tick() => {
              if let Err(e) = self.process_batch().await {
                  error!("Failed to process batch");
              }
          }
          _ = shutdown.changed() => {
              info!("Worker shutting down");
              break;
          }
      }
    }
  }

  async fn process_batch(&self) -> Result<(), Box<dyn std::error::Error>> {
    let tasks = self.queue.dequeue(&self.worker_id, self.config.batch_size).await?;

    for (task_key, task) in tasks {
      let queue = self.queue.clone();
      let processor = self.processor.clone();
      let timeout = self.config.max_processing_time;

      tokio::spawn(async move {
        let result = tokio::time::timeout(timeout, processor.process(&task)).await;

        match result {
          Ok(Ok(task_result)) => {
            if let Err(e) = queue.ack(&task_key, task_result).await {
              error!("Failed to ack task");
            }
          }
          Ok(Err(e)) => {
            let should_retry = task.retry_count < task.max_retries;
            if let Err(e) = queue.nack(&task_key, &e.to_string(), should_retry).await {
              error!("Failed to nack task");
            }
          }
          Err(_) => {
            if let Err(e) = queue.nack(&task_key, "Timeout", true).await {
              error!("Failed to nack timeout task");
            }
          }
        }
      });
    }

    Ok(())
  }
}
