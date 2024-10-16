use ultimate_db::generate_enum_i32_to_sea_query_value;

use super::{
  instance_task::TaskKind,
  process_definition::ProcessStatus,
  process_instance::InstanceStatus,
  sched_node::{NodeKind, NodeStatus},
  trigger_definition::{TriggerKind, TriggerStatus},
};

generate_enum_i32_to_sea_query_value!(
  Enum: ProcessStatus,
  Enum: TriggerKind,
  Enum: TriggerStatus,
  Enum: InstanceStatus,
  Enum: TaskKind,
  Enum: NodeStatus,
  Enum: NodeKind,
);
