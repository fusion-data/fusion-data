use fusion_core::application::Application;
use fusion_corelib::ctx::Ctx;
use modelsql::ModelManager;

#[derive(Clone)]
pub struct CtxW {
  ctx: Ctx,
  mm: ModelManager,
}

impl CtxW {
  pub fn app(&self) -> Application {
    Application::global()
  }

  pub fn mm(&self) -> &ModelManager {
    &self.mm
  }
}
