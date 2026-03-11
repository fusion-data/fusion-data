/// Extract Ctx from Parts, then set Ctx to the ModelManager, and finally return the ModelManager.
#[cfg(feature = "web")]
pub fn extract_model_manager(
  parts: &axum::http::request::Parts,
  state: &crate::core::application::Application,
) -> Result<crate::db::ModelManager, crate::web::WebError> {
  use crate::web::{WebError, extensions_2_ctx};
  let ctx = extensions_2_ctx(parts)?;
  let mm = state
    .get_component::<hetu_db::ModelManager>()
    .map_err(|_| WebError::new_with_code(500, "Failed to get ModelManager"))?
    .with_ctx(ctx.clone());
  Ok(mm)
}
