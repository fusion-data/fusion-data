use std::sync::Arc;

use fusion_core::{application::Application, component::Component};

#[derive(Component, Clone)]
#[allow(unused)]
pub struct AuthSvc {
  #[component]
  user_svc: UserSvc,
  #[component]
  pwd_svc: PwdSvc,
}

#[derive(Clone, Component)]
#[allow(unused)]
pub struct UserSvc {
  #[component]
  db: Db,
}

#[derive(Clone, Component)]
#[allow(unused)]
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
async fn main() -> fusion_core::Result<()> {
  Application::builder().build().await?;

  let _auth_svc: AuthSvc = Application::global().component();

  // let _pwd_svc = PwdSvc2 { pwd_generator: Arc::default() };
  // println!("{:?}", _pwd_svc);

  Ok(())
}
