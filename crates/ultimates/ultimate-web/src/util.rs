use axum::Json;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::{Authorization, HeaderMapExt};
use serde::Serialize;
use serde::de::DeserializeOwned;
use ulid::Ulid;
use ultimate_common::ctx::Ctx;
use ultimate_common::time;
use ultimate_core::configuration::SecurityConfig;
use ultimate_core::security::{AccessToken, SecurityUtils};
use ultimate_core::{DataError, IdI64Result, IdUlidResult};

use crate::AppResult;
use crate::error::AppError;

#[inline]
pub fn ok<T: Serialize>(v: T) -> AppResult<T> {
  Ok(Json(v))
}

#[inline]
pub fn ok_id(id: i64) -> AppResult<IdI64Result> {
  Ok(IdI64Result::new(id).into())
}

#[inline]
pub fn ok_ulid(id: Ulid) -> AppResult<IdUlidResult> {
  Ok(IdUlidResult::new(id).into())
}

#[inline]
pub fn ok_uuid(id: uuid::Uuid) -> AppResult<ultimate_core::IdUuidResult> {
  Ok(ultimate_core::IdUuidResult::new(id).into())
}

pub fn unauthorized_app_error(msg: impl Into<String>) -> (StatusCode, Json<AppError>) {
  (StatusCode::UNAUTHORIZED, Json(AppError::new(msg).with_err_code(401)))
}

/// 从 Http Request Parts 中获取 [SessionCtx]
pub fn extract_session(parts: &Parts, sc: &SecurityConfig) -> Result<Ctx, DataError> {
  let req_time = time::now();

  let token = if let Some(Authorization(bearer)) = parts.headers.typed_get::<Authorization<Bearer>>() {
    bearer.token().to_string()
  } else if let Ok(at) = Query::<AccessToken>::try_from_uri(&parts.uri) {
    at.0.access_token
  } else {
    return Err(DataError::unauthorized("Missing token"));
  };

  let (payload, _) =
    SecurityUtils::decrypt_jwt(sc.pwd(), &token).map_err(|_e| DataError::unauthorized("Failed decode jwt"))?;

  let ctx = Ctx::new(payload, Some(req_time), None);
  Ok(ctx)
}

pub fn opt_to_app_result<T>(opt: Option<T>) -> AppResult<T>
where
  T: DeserializeOwned,
{
  if let Some(v) = opt { Ok(Json(v)) } else { Err(Box::new(AppError::new_with_code(404, "Not found."))) }
}
