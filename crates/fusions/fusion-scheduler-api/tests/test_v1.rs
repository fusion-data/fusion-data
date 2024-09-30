use uuid::Uuid;

use fusion_scheduler_api::v1::{
  job_definition::JobType, update_trigger_request, CronSchedule, JobDefinition, SimpleSchedule, UpdateTriggerRequest,
};

#[test]
fn test_job_definition() {
  let job_definition =
    JobDefinition { job_id: Uuid::now_v7().to_string(), job_type: JobType::Script as i32, ..Default::default() };
  println!("job_definition: {:?}", job_definition);
  println!("job_definition json: {}", serde_json::to_string_pretty(&job_definition).unwrap());
  let job_type = JobType::Http;
  println!("job_type json: {}", serde_json::to_string_pretty(&job_type).unwrap());
}

#[test]
fn test_update_trigger_request() {
  let update_trigger_request = UpdateTriggerRequest {
    trigger_id: Uuid::now_v7().to_string(),
    data: Some(vec![]),
    tags: Some(vec!["tag1".to_string(), "tag2".to_string()].into()),
    schedule: Some(update_trigger_request::Schedule::Simple(SimpleSchedule {
      repeat_count: 1,
      interval_seconds: 1,
      delay_seconds: 1,
    })),
    ..Default::default()
  };
  println!("{}", serde_json::to_string_pretty(&update_trigger_request).unwrap());
}

#[test]
fn test_schedule_json() {
  let schedule_simple = SimpleSchedule { repeat_count: 3, interval_seconds: 60 * 60, delay_seconds: 0 };
  println!("schedule_simple json: {}", serde_json::to_string_pretty(&schedule_simple).unwrap());

  let schedule_cron = CronSchedule { cron: "0 5 * * *".to_string(), ..Default::default() };
  println!("schedule_cron json: {}", serde_json::to_string_pretty(&schedule_cron).unwrap());
}
