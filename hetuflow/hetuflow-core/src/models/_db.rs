use fusionsql::postgres::PgRowType;
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef};
use sqlx::types::Json;
use sqlx::{Decode, Encode, Postgres, Type};

use crate::types::Labels;

use super::*;

impl From<Labels> for sea_query::Value {
  fn from(value: Labels) -> Self {
    Self::Json(Some(Box::new(serde_json::to_value(value).unwrap())))
  }
}
impl sea_query::Nullable for Labels {
  fn null() -> sea_query::Value {
    sea_query::Value::Json(None)
  }
}
impl Type<Postgres> for Labels {
  fn type_info() -> PgTypeInfo {
    <Json<Self> as Type<Postgres>>::type_info()
  }
}
impl PgHasArrayType for Labels {
  fn array_type_info() -> PgTypeInfo {
    <Json<Self> as PgHasArrayType>::array_type_info()
  }
}
impl Encode<'_, Postgres> for Labels {
  fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
    Json(self).encode_by_ref(buf)
  }
}
impl<'r> Decode<'r, Postgres> for Labels {
  fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
    Ok(Json::<Self>::decode(value)?.0)
  }
}

impl From<TaskConfig> for sea_query::Value {
  fn from(value: TaskConfig) -> Self {
    Self::Json(Some(Box::new(serde_json::to_value(value).unwrap())))
  }
}
impl sea_query::Nullable for TaskConfig {
  fn null() -> sea_query::Value {
    sea_query::Value::Json(None)
  }
}
impl Type<Postgres> for TaskConfig {
  fn type_info() -> PgTypeInfo {
    <Json<Self> as Type<Postgres>>::type_info()
  }
}
impl PgHasArrayType for TaskConfig {
  fn array_type_info() -> PgTypeInfo {
    <Json<Self> as PgHasArrayType>::array_type_info()
  }
}
impl Encode<'_, Postgres> for TaskConfig {
  fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
    Json(self).encode_by_ref(buf)
  }
}
impl<'r> Decode<'r, Postgres> for TaskConfig {
  fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
    Ok(Json::<Self>::decode(value)?.0)
  }
}

impl From<AgentCapabilities> for sea_query::Value {
  fn from(value: AgentCapabilities) -> Self {
    sea_query::Value::Json(Some(Box::new(serde_json::to_value(value).unwrap())))
  }
}
impl sea_query::Nullable for AgentCapabilities {
  fn null() -> sea_query::Value {
    sea_query::Value::Json(None)
  }
}
impl Type<Postgres> for AgentCapabilities {
  fn type_info() -> PgTypeInfo {
    <Json<Self> as Type<Postgres>>::type_info()
  }
}
impl PgHasArrayType for AgentCapabilities {
  fn array_type_info() -> PgTypeInfo {
    <Json<Self> as PgHasArrayType>::array_type_info()
  }
}
impl Encode<'_, Postgres> for AgentCapabilities {
  fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
    Json(self).encode_by_ref(buf)
  }
}
impl<'r> Decode<'r, Postgres> for AgentCapabilities {
  fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
    Ok(Json::<Self>::decode(value)?.0)
  }
}

impl From<AgentStatistics> for sea_query::Value {
  fn from(value: AgentStatistics) -> Self {
    Self::Json(Some(Box::new(serde_json::to_value(value).unwrap())))
  }
}
impl sea_query::Nullable for AgentStatistics {
  fn null() -> sea_query::Value {
    sea_query::Value::Json(None)
  }
}
impl Type<Postgres> for AgentStatistics {
  fn type_info() -> PgTypeInfo {
    <Json<Self> as Type<Postgres>>::type_info()
  }
}
impl PgHasArrayType for AgentStatistics {
  fn array_type_info() -> PgTypeInfo {
    <Json<Self> as PgHasArrayType>::array_type_info()
  }
}
impl Encode<'_, Postgres> for AgentStatistics {
  fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
    Json(self).encode_by_ref(buf)
  }
}
impl<'r> Decode<'r, Postgres> for AgentStatistics {
  fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
    Ok(Json::<Self>::decode(value)?.0)
  }
}

impl From<TaskMetrics> for sea_query::Value {
  fn from(value: TaskMetrics) -> Self {
    Self::Json(Some(Box::new(serde_json::to_value(value).unwrap())))
  }
}
impl sea_query::Nullable for TaskMetrics {
  fn null() -> sea_query::Value {
    sea_query::Value::Json(None)
  }
}
impl Type<Postgres> for TaskMetrics {
  fn type_info() -> PgTypeInfo {
    <Json<Self> as Type<Postgres>>::type_info()
  }
}
impl PgHasArrayType for TaskMetrics {
  fn array_type_info() -> PgTypeInfo {
    <Json<Self> as PgHasArrayType>::array_type_info()
  }
}
impl Encode<'_, Postgres> for TaskMetrics {
  fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
    Json(self).encode_by_ref(buf)
  }
}
impl<'r> Decode<'r, Postgres> for TaskMetrics {
  fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
    Ok(Json::<Self>::decode(value)?.0)
  }
}

impl PgRowType for SchedServer {}
impl PgRowType for SchedAgent {}
impl PgRowType for SchedJob {}
impl PgRowType for SchedSchedule {}
impl PgRowType for SchedTask {}
impl PgRowType for SchedTaskInstance {}
