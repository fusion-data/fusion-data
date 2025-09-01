use axum::{
  Form,
  body::Body,
  extract::{
    FromRequest,
    rejection::{FormRejection, JsonRejection},
  },
  http::{Request, header},
};
use headers::ContentType;
use mime::Mime;
use serde::de::DeserializeOwned;

use crate::WebError;

pub struct JsonOrForm<T>(pub T);

impl<S, T> FromRequest<S> for JsonOrForm<T>
where
  S: Send + Sync,
  T: DeserializeOwned,
{
  type Rejection = WebError;

  async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
    let header_value = req
      .headers()
      .get(header::CONTENT_TYPE)
      .ok_or(WebError::new_with_code(400, "'Content-Type' not found"))?;

    let content_type: ContentType = header_value
      .to_str()
      .map_err(|ex| WebError::new_with_msg(ex.to_string()))?
      .parse()
      .map_err(|_ex| WebError::new_with_code(400, "'Content-Type' invalid"))?;

    let m: Mime = content_type.into();

    let res = if mime::APPLICATION_JSON == m {
      let axum::Json(res) = axum::Json::<T>::from_request(req, state)
        .await
        .map_err(|ex: JsonRejection| WebError::new_with_code(400, ex.body_text()))?;
      res
    } else if mime::APPLICATION_WWW_FORM_URLENCODED == m {
      let Form(res) = Form::<T>::from_request(req, state)
        .await
        .map_err(|ex: FormRejection| WebError::new_with_code(400, ex.body_text()))?;
      res
    } else {
      return Err(WebError::new_with_code(400, "Extract incoming data from HttpRequest error."));
    };
    Ok(JsonOrForm(res))
  }
}
