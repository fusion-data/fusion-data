use modql::filter::{IntoSeaError, SeaResult};
use serde::Deserialize;
use ultimate_common::time::{local_offset, Duration, OffsetDateTime, UtcDateTime};

pub fn time_to_sea_value(json_value: serde_json::Value) -> SeaResult<sea_query::Value> {
  Ok(UtcDateTime::deserialize(json_value)?.into())
}

pub fn to_sea_chrono_utc(v: serde_json::Value) -> SeaResult<sea_query::Value> {
  if v.as_str().is_some() {
    Ok(UtcDateTime::deserialize(v)?.into())
  } else if let Some(i) = v.as_i64() {
    let d = UtcDateTime::MIN_UTC + Duration::milliseconds(i);
    Ok(sea_query::Value::ChronoDateTimeUtc(Some(Box::new(d))))
  } else {
    Err(IntoSeaError::Custom(format!("Invalid value: incoming is {:?}", v)))
  }
}
pub fn to_sea_chrono_offset(v: serde_json::Value) -> SeaResult<sea_query::Value> {
  if v.as_str().is_some() {
    Ok(OffsetDateTime::deserialize(v)?.into())
  } else if let Some(i) = v.as_i64() {
    let d = (OffsetDateTime::MIN_UTC + Duration::milliseconds(i)).with_timezone(local_offset());
    Ok(sea_query::Value::ChronoDateTimeWithTimeZone(Some(Box::new(d))))
  } else {
    Err(IntoSeaError::Custom(format!("Invalid value: incoming is {:?}", v)))
  }
}

#[cfg(feature = "utoipa")]
pub fn op_vals_integer_schema() -> utoipa::openapi::Object {
  utoipa::openapi::ObjectBuilder::new()
    .schema_type(utoipa::openapi::Type::Object)
    .description(Some("opvalfloat64"))
    .build()
}

#[cfg(feature = "utoipa")]
pub fn op_vals_string_schema() -> utoipa::openapi::Object {
  utoipa::openapi::ObjectBuilder::new()
    .schema_type(utoipa::openapi::schema::Type::String)
    .description(Some("https://github.com/jeremychone/rust-modql?tab=readme-ov-file#opvalstring-operators"))
    .build()
}

#[cfg(feature = "utoipa")]
pub fn op_vals_bool_schema() -> utoipa::openapi::Object {
  utoipa::openapi::ObjectBuilder::new()
    .schema_type(utoipa::openapi::schema::Type::Boolean)
    .description(Some("https://github.com/jeremychone/rust-modql?tab=readme-ov-file#opvalbool-operators"))
    .build()
}
