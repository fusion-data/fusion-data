use crate::service::{process_definition::ProcessDefinition, process_task::ProcessTask};

pub struct SchedProcessTask {
  pub process_definition: ProcessDefinition,
  pub process_task: ProcessTask,
}
