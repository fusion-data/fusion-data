use crate::filter::{IntoSeaError, OpValsValue, SeaResult};
use serde::{Deserialize, Serialize};
use ultimate_common::time::{Duration, UtcDateTime};

#[cfg(feature = "with-uuid")]
use uuid::Uuid;

pub fn try_into_op_vals_value_opt<V: Serialize>(value: V) -> Result<Option<OpValsValue>, serde_json::Error> {
  let value = serde_json::to_value(value)?;

  let values: OpValsValue = serde_json::from_value(value)?;

  Ok(if values.0.is_empty() { None } else { Some(values) })
}

#[cfg(feature = "with-uuid")]
pub fn uuid_to_sea_value(json_value: serde_json::Value) -> SeaResult<sea_query::Value> {
  Ok(Uuid::deserialize(json_value)?.into())
}

pub fn datetime_to_sea_value(v: serde_json::Value) -> SeaResult<sea_query::Value> {
  if v.as_str().is_some() {
    Ok(UtcDateTime::deserialize(v)?.into())
  } else if let Some(i) = v.as_i64() {
    let d = UtcDateTime::UNIX_EPOCH + Duration::milliseconds(i);
    Ok(sea_query::Value::ChronoDateTimeUtc(Some(Box::new(d))))
  } else {
    Err(IntoSeaError::Custom(format!("Invalid value: incoming is {:?}", v)))
  }
}

pub(crate) fn as_positive_u64(num: i64) -> u64 {
  if num < 0 { 0 } else { num as u64 }
}
