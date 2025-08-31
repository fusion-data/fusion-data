use chrono::{DateTime, Duration, Utc};
use serde_json::{Map, Value};
use std::{ops::Deref, sync::Arc};
use thiserror::Error;

use crate::time::{OffsetDateTime, UtcDateTime, now_offset};

#[derive(Debug, Error)]
pub enum CtxError {
  #[error("Invalid payload, need a json object")]
  InvalidPayload,

  #[error("Unauthorized: {0}")]
  Unauthorized(String),
}

#[derive(Debug, Clone, Default)]
pub struct CtxPayload {
  payload: Map<String, Value>,
}

impl CtxPayload {
  pub fn new(payload: Map<String, Value>) -> Self {
    Self { payload }
  }

  pub fn into_inner(self) -> Map<String, Value> {
    self.payload
  }

  pub fn set_subject(&mut self, value: impl Into<String>) {
    self.set_string(Ctx::SUB, value);
  }

  /// Set the expiration time with `OffsetDateTime`.
  pub fn set_expires_at(&mut self, value: impl Into<OffsetDateTime>) {
    let value = value.into();
    self.set_exp(value.timestamp());
  }

  /// Set the expiration time in seconds since the Unix epoch.
  pub fn set_exp(&mut self, value: i64) {
    self.set_i64(Ctx::EXP, value);
  }

  pub fn set_string(&mut self, key: &str, value: impl Into<String>) {
    self.payload.insert(key.to_string(), Value::String(value.into()));
  }

  pub fn set_i64(&mut self, key: &str, value: i64) {
    self.payload.insert(key.to_string(), Value::Number(value.into()));
  }

  pub fn set_i32(&mut self, key: &str, value: i32) {
    self.payload.insert(key.to_string(), Value::Number(value.into()));
  }

  pub fn set_datetime(&mut self, key: &str, value: impl Into<OffsetDateTime>) {
    self.payload.insert(key.to_string(), Value::String(value.into().to_rfc3339()));
  }

  pub fn set_bool(&mut self, key: &str, value: bool) {
    self.payload.insert(key.to_string(), Value::Bool(value));
  }

  pub fn get_subject(&self) -> Option<&str> {
    self.get_str(Ctx::SUB)
  }

  pub fn get_expires_at(&self) -> Option<OffsetDateTime> {
    self.get_exp().and_then(|exp| UtcDateTime::from_timestamp(exp, 0)).map(Into::into)
  }

  pub fn get_exp(&self) -> Option<i64> {
    self.get_i64(Ctx::EXP)
  }

  pub fn get_str(&self, key: &str) -> Option<&str> {
    self.payload.get(key).and_then(|s| s.as_str())
  }

  pub fn get_i64(&self, key: &str) -> Option<i64> {
    self.payload.get(key).and_then(|s| s.as_i64())
  }

  pub fn get_i32(&self, key: &str) -> Option<i32> {
    self.payload.get(key).and_then(|s| s.as_i64()).map(|v| v as i32)
  }

  pub fn get_datetime(&self, key: &str) -> Option<OffsetDateTime> {
    get_datetime_from_value(&self.payload, key)
  }

  pub fn get_bool(&self, key: &str) -> Option<bool> {
    self.payload.get(key).and_then(|s| s.as_bool())
  }
}

impl From<Map<String, Value>> for CtxPayload {
  fn from(payload: Map<String, Value>) -> Self {
    Self::new(payload)
  }
}

#[derive(Debug)]
pub struct CtxInner {
  payload: CtxPayload,
  req_time: OffsetDateTime,
  req_id: String,
}

/// 会话上下文。此处 clone 的成本很低
#[derive(Clone, Debug)]
pub struct Ctx(Arc<CtxInner>);

impl Ctx {
  pub const SUB: &str = "sub";
  pub const EXP: &str = "exp";

  pub(crate) fn new(payload: CtxPayload, req_time: OffsetDateTime, req_id: String) -> Self {
    Self(Arc::new(CtxInner { payload, req_time, req_id }))
  }

  /// Create a new context
  pub fn try_new(
    mut payload: CtxPayload,
    req_time: Option<OffsetDateTime>,
    req_id: Option<String>,
  ) -> Result<Self, CtxError> {
    let now = now_offset();
    if let Some(st) = payload.get_expires_at() {
      if st < now {
        return Err(CtxError::Unauthorized("The token expired".to_string()));
      }
    } else {
      let exp = DateTime::<Utc>::MAX_UTC;
      payload.set_datetime(Self::EXP, exp);
    }

    Ok(Self::new(payload, req_time.unwrap_or(now), req_id.unwrap_or_default()))
  }

  pub fn new_root() -> Self {
    let req_time = now_offset();
    let expires_at = req_time + Duration::minutes(30);
    let mut payload = CtxPayload::default();
    payload.set_string(Self::SUB, "0");
    payload.set_datetime(Self::EXP, expires_at);
    Self::new(payload, req_time, "".to_string())
  }

  pub fn new_super_admin() -> Self {
    let req_time = now_offset();
    let expires_at = req_time + Duration::minutes(30);
    let mut payload = CtxPayload::default();
    payload.set_string(Self::SUB, "1");
    payload.set_datetime(Self::EXP, expires_at);
    Self::new(payload, req_time, "".to_string())
  }

  pub fn uid(&self) -> i64 {
    match self.payload.get_str(Self::SUB) {
      Some(sub) => sub.parse::<i64>().unwrap_or(0),
      None => 0,
    }
  }

  pub fn req_time(&self) -> &OffsetDateTime {
    &self.req_time
  }
  pub fn req_id(&self) -> &str {
    &self.req_id
  }

  pub fn expires_at(&self) -> Option<OffsetDateTime> {
    self.payload.get_datetime(Self::EXP)
  }
}

impl Deref for Ctx {
  type Target = CtxInner;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

pub fn get_datetime_from_value(payload: &Map<String, Value>, key: &str) -> Option<OffsetDateTime> {
  let value = payload.get(key)?;
  if let Some(value) = value.as_str() {
    return DateTime::parse_from_rfc3339(value).ok();
  }

  if let Some(millis) = value.as_i64() {
    return DateTime::from_timestamp_millis(millis).map(|dt| dt.into());
  }

  if let Some(f) = value.as_f64() {
    let secs = f as i64;
    let nsecs = ((f - secs as f64) * 1_000_000_000.0) as u32;
    return DateTime::from_timestamp(secs, nsecs).map(|dt| dt.into());
  }

  None
}
