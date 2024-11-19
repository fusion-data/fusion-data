use fusion_iam::role::RoleSvc;
use fusion_iam::user::UserSvc;
use ultimate::component::Component;
use ultimate_macros::PrintTypePaths;

#[derive(Clone, Component, PrintTypePaths)]
struct Example {
  #[component]
  user_svc: UserSvc,

  #[component]
  role_svc: RoleSvc,
}

impl Example {
  fn dependencies(&self) -> Vec<&str> {
    vec![std::any::type_name::<UserSvc>()]
  }
}

#[tokio::main]
async fn main() {
  let name = std::any::type_name::<Example>();
  // let app = Application::builder().build().await.unwrap();
  // let example = app.get_component::<Example>().unwrap();
  Example::print_type_paths();
}
