use crate::service::{
  process_definition::ProcessDefinition, process_task::ProcessTask, sched_namespace::SchedNamespace,
};

pub struct SchedProcessTask {
  pub process_definition: ProcessDefinition,
  pub process_task: ProcessTask,
}

pub(in crate::master) enum DbCmd {
  Stop,
  ListenNamespaces(Vec<SchedNamespace>),
  UnlistenNamespaces(Vec<SchedNamespace>),
  Heartbeat(&'static str),
}
