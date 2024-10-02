use chrono::{Duration, Utc};
use fusion_scheduler::pb::fusion_scheduler::v1::JobFilterRequest;
use modql::filter::{FilterGroups, FilterNodes, OpValsValue};
use sea_query::Condition;
use ultimate::DataError;
use ultimate_api::v1::{OpNumber, OpString, ValInt64, ValString};
use ultimate_db::{
  datetime_to_sea_value, try_into_op_vals_value_opt_with_filter_int64, try_into_op_vals_value_opt_with_filter_string,
  uuid_to_sea_value,
};
use uuid::Uuid;

fn main() {
  let id = vec![ValString::new_value(OpString::Eq, Uuid::now_v7())];
  let begin = Utc::now();
  let end = begin + Duration::days(1);
  println!("ctime: [{}, {})", begin, end);
  let ctime = vec![
    ValInt64::new_value(OpNumber::Gte, begin.timestamp_millis()),
    ValInt64::new_value(OpNumber::Lt, end.timestamp_millis()),
  ];
  let filter_request = JobFilterRequest { id, ctime, ..Default::default() };

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

  #[modql(to_sea_value_fn = "datetime_to_sea_value")]
  ctime: Option<OpValsValue>,
}

impl TryFrom<JobFilterRequest> for JobFilter {
  type Error = DataError;
  fn try_from(value: JobFilterRequest) -> Result<Self, Self::Error> {
    Ok(Self {
      job_id: try_into_op_vals_value_opt_with_filter_string(value.id)?,
      ctime: try_into_op_vals_value_opt_with_filter_int64(value.ctime)?,
    })
  }
}
