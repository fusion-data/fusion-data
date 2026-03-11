use axum::http::request::Parts;
use hetus::common::ctx::Ctx;
use hetus::core::application::Application;
use hetus::web::WebError;
use hetusql::ModelManager;

pub fn model_manager_from_parts(parts: &Parts, app: &Application) -> Result<ModelManager, WebError> {
  let ctx: &Ctx = parts.extensions.get().ok_or_else(|| WebError::new_with_code(401, "Unauthorized"))?;
  let mm = app
    .get_component::<ModelManager>()
    .map_err(|_e| WebError::new_with_code(401, "ModelManager not exists"))?;
  Ok(mm.with_ctx(ctx.clone()))
}
