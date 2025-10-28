use axum::http::request::Parts;

use crate::core::application::Application;
use crate::db::ModelManager;
use crate::web::WebError;

pub fn extract_model_manager(parts: &Parts, state: &Application) -> Result<ModelManager, WebError> {
  use fusion_web::extensions_2_ctx;

  let ctx = extensions_2_ctx(parts)?;
  let mm = state
    .get_component::<fusion_db::ModelManager>()
    .map_err(|_| WebError::new_with_code(500, "Failed to get ModelManager"))?
    .with_ctx(ctx.clone());
  Ok(mm)
}
