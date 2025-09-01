use hetumind_core::task::{QueueError, WorkerError};
use fusion_core::DataError;

pub fn queue_error_to_data_error(error: QueueError) -> DataError {
  DataError::server_error(error.to_string())
}

pub fn worker_error_to_data_error(error: WorkerError) -> DataError {
  DataError::server_error(error.to_string())
}
