// use crate::filter::OpValValue;
#[cfg(feature = "with-sea-query")]
use crate::filter::{IntoSeaError, SeaResult};
#[allow(unused_imports)]
use chrono::{DateTime, Duration, FixedOffset, Utc};
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

// pub fn try_into_op_vals_value_opt<V: Serialize>(value: V) -> Result<Option<OpValValue>, serde_json::Error> {
//   let value = serde_json::to_value(value)?;

//   let values: OpValValue = serde_json::from_value(value)?;

//   Ok(if values.0.is_empty() { None } else { Some(values) })
// }

#[cfg(all(feature = "with-uuid", feature = "with-sea-query"))]
pub fn uuid_to_sea_value(json_value: serde_json::Value) -> SeaResult<sea_query::Value> {
  Ok(uuid::Uuid::deserialize(json_value)?.into())
}

#[cfg(feature = "with-sea-query")]
pub fn datetime_to_sea_value(v: serde_json::Value) -> SeaResult<sea_query::Value> {
  if v.as_str().is_some() {
    let dt = DateTime::<FixedOffset>::deserialize(v)?.into();
    Ok(dt)
  } else if let Some(i) = v.as_i64() {
    let d = DateTime::<Utc>::UNIX_EPOCH + Duration::milliseconds(i);
    Ok(sea_query::Value::ChronoDateTimeUtc(Some(Box::new(d))))
  } else {
    Err(IntoSeaError::Custom(format!("Invalid value, need OffsetDateTime. incoming value is {:?}", v)))
  }
}
