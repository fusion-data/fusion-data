use std::sync::Arc;

use ultimate::{application::Application, component::Component};

#[derive(Component, Clone)]
pub struct AuthSvc {
  #[component]
  user_svc: UserSvc,
  #[component]
  pwd_svc: PwdSvc,
}

#[derive(Clone, Component)]
pub struct UserSvc {
  #[component]
  db: Db,
}

#[derive(Clone, Component)]
pub struct PwdSvc {
  pwd_generator: Arc<PwdGenerator>,
}

// #[derive(Debug, Clone)]
// pub struct PwdSvc2 {
//   pwd_generator: Arc<PwdGenerator>,
// }

#[derive(Debug, Default)]
pub struct PwdGenerator {}

#[derive(Clone, Component)]
pub struct Db {}

#[tokio::main]
async fn main() -> ultimate::Result<()> {
  let app = Application::builder().build().await?;

  let _auth_svc: AuthSvc = app.component();

  // let _pwd_svc = PwdSvc2 { pwd_generator: Arc::default() };
  // println!("{:?}", _pwd_svc);

  Ok(())
}
