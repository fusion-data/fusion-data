use std::collections::HashMap;

use chrono::{Duration, Utc};
use fusion_scheduler_api::v1::JobFilterRequest;
use modql::filter::{FilterGroups, FilterNodes, OpValsValue};
use sea_query::Condition;
use ultimate_db::{time_to_sea_value, try_into_op_vals_value_opt, uuid_to_sea_value};
use uuid::Uuid;

fn main() {
  let mut job_id = HashMap::new();
  job_id.insert("$eq".into(), Uuid::now_v7().to_string());
  let mut ctime = HashMap::new();
  let begin = Utc::now();
  let end = begin + Duration::days(1);
  println!("ctime: [{}, {})", begin, end);
  ctime.insert("$gte".into(), begin.timestamp_millis());
  ctime.insert("$lt".into(), end.timestamp_millis());
  let filter_request = JobFilterRequest { job_id, ctime };

  println!("\n{:?}", filter_request);

  let job_filter: JobFilter = filter_request.try_into().unwrap();
  println!("\n{:?}", job_filter);

  let filters: FilterGroups = job_filter.into();
  println!("\n{:?}", filters);

  let cond: Condition = filters.try_into().unwrap();
  println!("\n{:?}", cond);
}

#[derive(Debug, Default, FilterNodes)]
struct JobFilter {
  #[modql(to_sea_value_fn = "uuid_to_sea_value")]
  job_id: Option<OpValsValue>,

  #[modql(to_sea_value_fn = "time_to_sea_value")]
  ctime: Option<OpValsValue>,
}

impl TryFrom<JobFilterRequest> for JobFilter {
  type Error = serde_json::Error;
  fn try_from(value: JobFilterRequest) -> Result<Self, Self::Error> {
    Ok(Self { job_id: try_into_op_vals_value_opt(value.job_id)?, ctime: try_into_op_vals_value_opt(value.ctime)? })
  }
}
