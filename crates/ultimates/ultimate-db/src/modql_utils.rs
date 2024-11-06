use crate::modql::filter::{IntoSeaError, OpValsValue, SeaResult};
use serde::{Deserialize, Serialize};
use ultimate_common::time::{Duration, UtcDateTime};

pub fn try_into_op_vals_value_opt<V: Serialize>(value: V) -> Result<Option<OpValsValue>, serde_json::Error> {
  let value = serde_json::to_value(value)?;

  let values: OpValsValue = serde_json::from_value(value)?;

  Ok(if values.0.is_empty() { None } else { Some(values) })
}

pub fn uuid_to_sea_value(json_value: serde_json::Value) -> SeaResult<sea_query::Value> {
  Ok(uuid::Uuid::deserialize(json_value)?.into())
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

// #[cfg(feature = "utoipa")]
// pub fn op_vals_integer_schema() -> utoipa::openapi::Object {
//   utoipa::openapi::ObjectBuilder::new()
//     .schema_type(utoipa::openapi::Type::Object)
//     .description(Some("opvalfloat64"))
//     .build()
// }

// #[cfg(feature = "utoipa")]
// pub fn op_vals_string_schema() -> utoipa::openapi::Object {
//   utoipa::openapi::ObjectBuilder::new()
//     .schema_type(utoipa::openapi::schema::Type::String)
//     .description(Some("https://github.com/jeremychone/rust-modql?tab=readme-ov-file#opvalstring-operators"))
//     .build()
// }

// #[cfg(feature = "utoipa")]
// pub fn op_vals_bool_schema() -> utoipa::openapi::Object {
//   utoipa::openapi::ObjectBuilder::new()
//     .schema_type(utoipa::openapi::schema::Type::Boolean)
//     .description(Some("https://github.com/jeremychone/rust-modql?tab=readme-ov-file#opvalbool-operators"))
//     .build()
// }
