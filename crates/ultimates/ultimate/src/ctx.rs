use std::{ops::Deref, sync::Arc, time::SystemTime};

use josekit::jwt::JwtPayload;
use tracing::{span::EnteredSpan, Span};
use ulid::Ulid;
use ultimate_common::time::{self, Duration, UtcDateTime};
use uuid::Uuid;

use crate::DataError;

#[derive(Debug)]
pub struct InnerCtx {
  payload: JwtPayload,
  req_time: UtcDateTime,
  request_id: String,
}

/// 会话上下文。
/// 此处 clone 的成本很低，若后续数据多的话可以使用 Arc 加 Wrapper 模式来降低数据复制的成本
#[derive(Clone, Debug)]
pub struct Ctx {
  inner: Arc<InnerCtx>,
  // request_span: Span,
  // request_entered: Arc<EnteredSpan>,
}

impl Ctx {
  pub fn new(payload: JwtPayload, req_time: Option<UtcDateTime>, request_id: Option<String>) -> Self {
    let req_time = req_time.unwrap_or_else(time::now_utc);
    let request_id = request_id.unwrap_or_else(|| Uuid::now_v7().to_string());

    tracing::Span::current().record("sub", payload.subject().unwrap_or_default());

    // let request_span = tracing::info_span!("Ctx", tid = %request_id, sub = %payload.subject().unwrap_or_default());
    // let request_entered = Arc::new(request_span.clone().entered());
    // let _request_span_guard = Arc::new(Box::new(request_span.enter()));

    Self { inner: Arc::new(InnerCtx { payload, req_time, request_id }) /*, request_span*/ }
  }

  pub fn new_root() -> Self {
    let req_time = time::now_utc();
    let expires_at = req_time + Duration::minutes(30);
    let mut payload = JwtPayload::new();
    payload.set_expires_at(&expires_at.into());
    payload.set_subject("0");

    Self::new(payload, Some(req_time), None)
  }

  pub fn new_super_admin() -> Self {
    let req_time = time::now_utc();
    let expires_at = req_time + Duration::minutes(30);
    let mut payload = JwtPayload::new();
    payload.set_expires_at(&expires_at.into());
    payload.set_subject("1");
    Self::new(payload, Some(req_time), None)
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

  pub fn request_id(&self) -> &str {
    &self.request_id
  }

  // pub fn request_span(&self) -> Span {
  //   self.request_span.clone()
  // }

  pub fn try_from_jwt_payload(
    mut payload: JwtPayload,
    req_time: Option<UtcDateTime>,
    request_id: Option<String>,
  ) -> Result<Self, DataError> {
    if let Some(st) = payload.expires_at() {
      if st < SystemTime::now() {
        return Err(DataError::unauthorized("The token expired"));
      }
    } else {
      let exp: SystemTime = UtcDateTime::MAX_UTC.into();
      payload.set_expires_at(&exp);
    };

    Ok(Ctx::new(payload, req_time, request_id))
  }
}

impl Deref for Ctx {
  type Target = InnerCtx;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl TryFrom<JwtPayload> for Ctx {
  type Error = DataError;

  fn try_from(payload: JwtPayload) -> std::result::Result<Self, Self::Error> {
    Ctx::try_from_jwt_payload(payload, Some(time::now_utc()), Some(Ulid::new().to_string()))
  }
}
