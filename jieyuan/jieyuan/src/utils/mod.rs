use axum::http::request::Parts;
use fusion_common::ctx::Ctx;
use fusion_core::application::Application;
use fusion_web::WebError;
use fusionsql::ModelManager;

pub fn model_manager_from_parts(parts: &Parts, app: &Application) -> Result<ModelManager, WebError> {
  let ctx: &Ctx = parts.extensions.get().ok_or_else(|| WebError::new_with_code(401, "Unauthorized"))?;
  let mm = app
    .get_component::<ModelManager>()
    .map_err(|_e| WebError::new_with_code(401, "ModelManager not exists"))?;
  Ok(mm.with_ctx(ctx.clone()))
}
