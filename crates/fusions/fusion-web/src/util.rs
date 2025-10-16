use std::time::SystemTime;

use axum::Json;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::http::request::Parts;
use fusion_common::ctx::Ctx;
use fusion_common::model::IdI64Result;
use fusion_core::configuration::SecuritySetting;
use fusion_core::log::get_trace_id;
use fusion_core::security::{AccessToken, SecurityUtils};
use headers::authorization::Bearer;
use headers::{Authorization, HeaderMapExt};
use serde::de::DeserializeOwned;
#[cfg(feature = "with-ulid")]
use ulid::Ulid;

use crate::WebResult;
use crate::error::WebError;

/// ok_json! 宏：支持无参数（返回 Ok(Json(()))）或一个参数（返回 Ok(Json(v))）
#[macro_export]
macro_rules! ok_json {
  () => {
    Ok(axum::Json(().into()))
  };
  ($v:expr) => {
    Ok(axum::Json($v))
  };
}

#[inline]
pub fn ok_id(id: i64) -> WebResult<IdI64Result> {
  Ok(IdI64Result::new(id).into())
}

#[cfg(feature = "with-ulid")]
#[inline]
pub fn ok_ulid(id: Ulid) -> WebResult<fusion_common::model::IdUlidResult> {
  Ok(fusion_common::model::IdUlidResult::new(id).into())
}

#[cfg(feature = "with-uuid")]
#[inline]
pub fn ok_uuid(id: uuid::Uuid) -> WebResult<fusion_common::model::IdUuidResult> {
  Ok(fusion_common::model::IdUuidResult::new(id).into())
}

pub fn unauthorized_app_error(msg: impl Into<String>) -> (StatusCode, Json<WebError>) {
  (StatusCode::UNAUTHORIZED, Json(WebError::new_with_msg(msg).with_err_code(401)))
}

/// 从 Http Request Authorization Header 或 access_token query 中获取 [Ctx]
pub fn extract_ctx(parts: &Parts, sc: &SecuritySetting) -> Result<Ctx, WebError> {
  let req_time = SystemTime::now();

  let token = if let Some(Authorization(bearer)) = parts.headers.typed_get::<Authorization<Bearer>>() {
    bearer.token().to_string()
  } else if let Ok(at) = Query::<AccessToken>::try_from_uri(&parts.uri) {
    at.0.access_token
  } else {
    return Err(WebError::new_with_code(401, "Missing token"));
  };

  let (payload, _) =
    SecurityUtils::decrypt_jwt(sc.pwd(), &token).map_err(|_e| WebError::new_with_code(401, "Failed decode jwt"))?;

  let ctx =
    Ctx::try_new(payload, Some(req_time), get_trace_id()).map_err(|e| WebError::new_with_code(401, e.to_string()))?;
  Ok(ctx)
}

pub fn opt_to_app_result<T>(opt: Option<T>) -> WebResult<T>
where
  T: DeserializeOwned,
{
  if let Some(v) = opt { Ok(Json(v)) } else { Err(WebError::new_with_code(404, "Not found.")) }
}
