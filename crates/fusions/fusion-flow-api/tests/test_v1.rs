use uuid::Uuid;

use fusion_flow_api::v1::task::TaskKind;
use fusion_flow_api::v1::{
  update_trigger_request, CronSchedule, ProcessDefinition, SimpleSchedule, UpdateTriggerRequest,
};

#[test]
fn test_process_definition() {
  let process_definition = ProcessDefinition { id: Uuid::now_v7().to_string(), ..Default::default() };
  println!("process_definition: {:?}", process_definition);
  println!("process_definition json: {}", serde_json::to_string_pretty(&process_definition).unwrap());
  let task_kind = TaskKind::Http;
  println!("task_kind json: {}", serde_json::to_string_pretty(&task_kind).unwrap());
}

#[test]
fn test_update_trigger_request() {
  let update_trigger_request = UpdateTriggerRequest {
    trigger_id: 1,
    data: Some(vec![]),
    tags: Some(vec!["tag1".to_string(), "tag2".to_string()].into()),
    schedule: Some(update_trigger_request::Schedule::Simple(SimpleSchedule {
      interval: "1s".to_string(),
      first_delay: "0".to_string(),
      execution_count: Some(5),
    })),
    ..Default::default()
  };
  println!("{}", serde_json::to_string_pretty(&update_trigger_request).unwrap());
}

#[test]
fn test_schedule_json() {
  let simple_schedule =
    SimpleSchedule { interval: "1s".to_string(), first_delay: "0".to_string(), execution_count: Some(5) };
  println!("simple_schedule json: {}", serde_json::to_string_pretty(&simple_schedule).unwrap());

  let cron_schedule = CronSchedule { cron: "0 5 * * *".to_string(), ..Default::default() };
  println!("cron_schedule json: {}", serde_json::to_string_pretty(&cron_schedule).unwrap());
}
