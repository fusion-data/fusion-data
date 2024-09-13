use axum::{
  async_trait,
  body::Body,
  extract::{
    rejection::{FormRejection, JsonRejection},
    FromRequest,
  },
  http::{header, Request},
  Form,
};
use axum_extra::headers::ContentType;
use mime::Mime;
use serde::de::DeserializeOwned;

use crate::AppError;

pub struct JsonOrForm<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for JsonOrForm<T>
where
  S: Send + Sync,
  T: DeserializeOwned,
{
  type Rejection = AppError;

  async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
    let header_value = req.headers().get(header::CONTENT_TYPE).ok_or(AppError::new("'Content-Type' not found"))?;

    let content_type: ContentType = header_value
      .to_str()
      .map_err(|ex| AppError::new(ex.to_string()))?
      .parse()
      .map_err(|_ex| AppError::new("'Content-Type' invalid"))?;

    let m: Mime = content_type.into();

    let res = if mime::APPLICATION_JSON == m {
      let axum::Json(res): axum::Json<T> =
        FromRequest::<S>::from_request(req, state).await.map_err(|ex: JsonRejection| AppError::new(ex.body_text()))?;
      res
    } else if mime::APPLICATION_WWW_FORM_URLENCODED == m {
      let Form(res): Form<T> =
        FromRequest::<S>::from_request(req, state).await.map_err(|ex: FormRejection| AppError::new(ex.body_text()))?;
      res
    } else {
      return Err(AppError::new("Extract TokenReq from HttpRequest error."));
    };
    Ok(JsonOrForm(res))
  }
}
