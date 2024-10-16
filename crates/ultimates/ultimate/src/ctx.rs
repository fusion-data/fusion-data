use std::{ops::Deref, sync::Arc, time::SystemTime};

use josekit::jwt::JwtPayload;
use ultimate_common::time::{self, Duration, UtcDateTime};

use crate::DataError;

#[derive(Debug, Default)]
pub struct InnerCtx {
  payload: JwtPayload,
  req_time: UtcDateTime,
}

/// 会话上下文。
/// 此处 clone 的成本很低，若后续数据多的话可以使用 Arc 加 Wrapper 模式来降低数据复制的成本
#[derive(Clone, Debug, Default)]
pub struct Ctx(Arc<InnerCtx>);

impl Ctx {
  pub fn new(payload: JwtPayload, req_time: UtcDateTime) -> Self {
    Self(Arc::new(InnerCtx { payload, req_time }))
  }

  pub fn new_root() -> Self {
    let req_time = time::now_utc();
    let expires_at = req_time + Duration::minutes(30);
    let mut payload = JwtPayload::new();
    payload.set_expires_at(&expires_at.into());
    payload.set_subject("0");
    Self::new(payload, req_time)
  }

  pub fn new_super_admin() -> Self {
    let req_time = time::now_utc();
    let expires_at = req_time + Duration::minutes(30);
    let mut payload = JwtPayload::new();
    payload.set_expires_at(&expires_at.into());
    payload.set_subject("1");
    Self::new(payload, req_time)
  }

  pub fn uid(&self) -> i64 {
    self.payload.subject().and_then(|s| s.parse::<i64>().ok()).unwrap_or(0)
  }

  pub fn req_time(&self) -> &UtcDateTime {
    &self.req_time
  }

  pub fn expires_at(&self) -> Option<UtcDateTime> {
    self.payload.expires_at().as_ref().map(|exp| (*exp).into())
  }

  pub fn try_from_jwt_payload(mut payload: JwtPayload, req_time: Option<UtcDateTime>) -> Result<Self, DataError> {
    let req_time = req_time.unwrap_or_else(time::now_utc);

    if let Some(st) = payload.expires_at() {
      if st < SystemTime::now() {
        return Err(DataError::unauthorized("The token expired"));
      }
    } else {
      let exp: SystemTime = UtcDateTime::MAX_UTC.into();
      payload.set_expires_at(&exp);
    };

    Ok(Ctx::new(payload, req_time))
  }
}

impl Deref for Ctx {
  type Target = InnerCtx;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl TryFrom<JwtPayload> for Ctx {
  type Error = DataError;

  fn try_from(payload: JwtPayload) -> std::result::Result<Self, Self::Error> {
    Ctx::try_from_jwt_payload(payload, Some(time::now_utc()))
  }
}
