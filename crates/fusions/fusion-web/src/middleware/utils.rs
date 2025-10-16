use axum::{body::Body, response::Response};
use http::{StatusCode, header::CONTENT_TYPE};

use crate::WebError;

pub fn web_error_2_body(e: WebError) -> Response<Body> {
  let body = serde_json::to_vec(&e).unwrap();
  Response::builder()
    .status(StatusCode::UNAUTHORIZED)
    .header(CONTENT_TYPE, "application/json; charset=utf-8")
    .body(Body::from(body))
    .unwrap()
}
