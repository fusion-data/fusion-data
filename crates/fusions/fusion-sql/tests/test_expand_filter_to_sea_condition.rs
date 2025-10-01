//! Should compile. No test functions yet.
use chrono::{DateTime, Utc};
use fusion_sql::filter::{FilterNodes, OpValsInt64, OpValsString, OpValsValue, SeaResult};

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

#[derive(FilterNodes, Default)]
pub struct ProjectFilter {
  id: Option<OpValsInt64>,
  name: Option<OpValsString>,

  #[modelsql(to_sea_value_fn = "my_to_sea_value")]
  created_at: Option<OpValsValue>,
}

fn my_to_sea_value(original: serde_json::Value) -> SeaResult<sea_query::Value> {
  let dt: DateTime<Utc> = serde_json::from_value(original)?;
  Ok(sea_query::Value::from(dt))
}

#[test]
fn test_expand_filter_nodes() -> Result<()> {
  let _filter = ProjectFilter {
    id: Some(123.into()),
    created_at: Some(OpValsValue::eq(serde_json::Value::from("some-date"))),
    ..Default::default()
  };

  Ok(())
}
