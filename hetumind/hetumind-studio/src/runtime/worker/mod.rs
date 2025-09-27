use std::time::Duration;

mod generic_worker;
mod task_processor;

pub use generic_worker::*;
pub use task_processor::*;

#[derive(Debug, Clone)]
pub struct WorkerConfig {
  pub batch_size: usize,
  pub poll_interval: Duration,
  pub heartbeat_interval: Duration,
  pub max_processing_time: Duration,
}
