use fusion_core::application::Application;
use fusion_corelib::ctx::Ctx;
use modelsql::ModelManager;

#[derive(Clone)]
pub struct CtxW {
  ctx: Ctx,
  mm: ModelManager,
}

impl CtxW {
  /// 创建新的上下文包装器实例
  pub fn new(mm: ModelManager) -> Self {
    Self {
      ctx: Ctx::new_root(),
      mm,
    }
  }

  pub fn app(&self) -> Application {
    Application::global()
  }

  pub fn mm(&self) -> &ModelManager {
    &self.mm
  }
}
