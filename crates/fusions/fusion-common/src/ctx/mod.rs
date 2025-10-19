use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{
  ops::Deref,
  sync::Arc,
  time::{Duration, SystemTime, UNIX_EPOCH},
};
use thiserror::Error;

use crate::time::now_utc;

#[derive(Debug, Error)]
pub enum CtxError {
  #[error("Invalid payload, need a json object")]
  InvalidPayload,

  #[error("Unauthorized: {0}")]
  Unauthorized(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(transparent)]
pub struct CtxPayload(Map<String, Value>);

impl CtxPayload {
  pub fn new(payload: Map<String, Value>) -> Self {
    Self(payload)
  }

  pub fn into_inner(self) -> Map<String, Value> {
    self.0
  }

  pub fn set_subject(&mut self, value: impl Into<String>) {
    self.set_string(Ctx::SUB, value);
  }

  /// Set the expiration time with `OffsetDateTime`.
  pub fn set_expires_at(&mut self, value: impl Into<SystemTime>) {
    let value: SystemTime = value.into();
    self.set_exp(value.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64);
  }

  /// Set the expiration time in seconds since the Unix epoch.
  pub fn set_exp(&mut self, epoch_seconds: i64) {
    self.set_i64(Ctx::EXP, epoch_seconds);
  }

  pub fn set_string(&mut self, key: &str, value: impl Into<String>) {
    self.0.insert(key.to_string(), Value::String(value.into()));
  }

  pub fn set_i64(&mut self, key: &str, value: i64) {
    self.0.insert(key.to_string(), Value::Number(value.into()));
  }

  pub fn set_i32(&mut self, key: &str, value: i32) {
    self.0.insert(key.to_string(), Value::Number(value.into()));
  }

  pub fn set_system_time(&mut self, key: &str, value: impl Into<SystemTime>) {
    self
      .0
      .insert(key.to_string(), Value::Number(value.into().duration_since(UNIX_EPOCH).unwrap().as_secs().into()));
  }

  pub fn set_bool(&mut self, key: &str, value: bool) {
    self.0.insert(key.to_string(), Value::Bool(value));
  }

  pub fn set_strings<I, S>(&mut self, key: &str, value: I)
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    self
      .0
      .insert(key.to_string(), Value::Array(value.into_iter().map(|s| Value::String(s.into())).collect()));
  }

  pub fn get_subject(&self) -> Option<&str> {
    self.get_str(Ctx::SUB)
  }

  pub fn get_expires_at(&self) -> Option<SystemTime> {
    self.get_exp().map(|exp| UNIX_EPOCH + std::time::Duration::from_secs(exp as u64))
  }

  pub fn get_exp(&self) -> Option<i64> {
    self.get_i64(Ctx::EXP)
  }

  pub fn get_str(&self, key: &str) -> Option<&str> {
    self.0.get(key).and_then(|s| s.as_str())
  }

  pub fn get_i64(&self, key: &str) -> Option<i64> {
    self.0.get(key).and_then(|s| s.as_i64())
  }

  pub fn get_i32(&self, key: &str) -> Option<i32> {
    self.0.get(key).and_then(|s| s.as_i64()).map(|v| v as i32)
  }

  pub fn get_strings(&self, key: &str) -> Option<Vec<&str>> {
    self
      .0
      .get(key)
      .and_then(|s| s.as_array())
      .map(|v| v.iter().filter_map(|s| s.as_str()).map(|s| s.trim()).collect())
  }

  pub fn get_system_time(&self, key: &str) -> Option<SystemTime> {
    self.get_i64(key).map(|v| UNIX_EPOCH + std::time::Duration::from_secs(v as u64))
  }

  pub fn get_bool(&self, key: &str) -> Option<bool> {
    self.0.get(key).and_then(|s| s.as_bool())
  }
}

impl From<Map<String, Value>> for CtxPayload {
  fn from(payload: Map<String, Value>) -> Self {
    Self::new(payload)
  }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct CtxInner {
  payload: CtxPayload,
  req_time: DateTime<Utc>,
  req_id: String,
}

/// 会话上下文。此处 clone 的成本很低
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(transparent)]
pub struct Ctx(Arc<CtxInner>);

impl Ctx {
  pub const SUB: &str = "sub";
  pub const EXP: &str = "exp";
  pub const TENANT_ID: &str = "tenant_id";

  pub(crate) fn new(payload: CtxPayload, req_time: DateTime<Utc>, req_id: String) -> Self {
    Self(Arc::new(CtxInner { payload, req_time, req_id }))
  }

  /// Create a new context
  pub fn try_new(
    mut payload: CtxPayload,
    req_time: Option<DateTime<Utc>>,
    req_id: Option<String>,
  ) -> Result<Self, CtxError> {
    let now = now_utc();
    if let Some(st) = payload.get_expires_at().map(|st| DateTime::<Utc>::from(st)) {
      if st < now {
        return Err(CtxError::Unauthorized("The token expired".to_string()));
      }
    } else {
      let exp = now + Duration::from_secs(60 * 60 * 24 * 60); // 60 days
      payload.set_system_time(Self::EXP, exp);
    }

    Ok(Self::new(payload, req_time.unwrap_or(now), req_id.unwrap_or_default()))
  }

  pub fn new_root() -> Self {
    let req_time = now_utc();
    let expires_at = req_time + Duration::from_secs(60 * 30); // 30 minutes
    let mut payload = CtxPayload::default();
    payload.set_string(Self::SUB, "0");
    payload.set_system_time(Self::EXP, expires_at);
    Self::new(payload, req_time, "".to_string())
  }

  pub fn new_super_admin() -> Self {
    let req_time = now_utc();
    let expires_at = req_time + Duration::from_secs(60 * 30); // 30 minutes
    let mut payload = CtxPayload::default();
    payload.set_string(Self::SUB, "1");
    payload.set_system_time(Self::EXP, expires_at);
    Self::new(payload, req_time, "".to_string())
  }

  pub fn get_user_id(&self) -> Option<i64> {
    self.payload.get_i64(Self::SUB)
  }

  pub fn user_id(&self) -> i64 {
    self.get_user_id().unwrap_or(0)
  }

  pub fn get_tenant_id(&self) -> Option<i64> {
    self.payload.get_i64(Self::TENANT_ID)
  }

  pub fn tenant_id(&self) -> i64 {
    self.get_tenant_id().unwrap_or(0)
  }

  pub fn req_time(&self) -> &DateTime<Utc> {
    &self.req_time
  }

  pub fn req_datetime(&self) -> DateTime<Utc> {
    self.req_time.into()
  }

  pub fn req_epoch_secs(&self) -> i64 {
    self.req_time.timestamp()
  }

  pub fn req_epoch_millis(&self) -> i64 {
    self.req_time.timestamp_millis()
  }

  pub fn req_id(&self) -> &str {
    &self.req_id
  }

  pub fn expires_at(&self) -> Option<SystemTime> {
    self.payload.get_system_time(Self::EXP)
  }

  pub fn payload(&self) -> &CtxPayload {
    &self.payload
  }
}

impl Deref for Ctx {
  type Target = CtxInner;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
