use modelsql::ModelManager;
use ultimate_core::component::Component;
use ultimate_core::{application::Application, component::ComponentArc};
use ultimate_db::DbPlugin;

#[derive(Clone, Component)]
struct TestService {
  #[component]
  mm: ModelManager,
}

impl TestService {
  pub async fn test(&self) -> ultimate_core::Result<String> {
    let _mm = &self.mm;
    Ok(String::from("test service"))
  }
}

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();
  let mut ab = Application::builder();
  ab.add_plugin(DbPlugin);
  ab.run().await;
  let app = Application::global();
  let mm: ComponentArc<ModelManager> = app.get_component_arc().unwrap();

  let addr: *const ModelManager = &*mm;
  println!("ModelManager address: {:p}", addr);

  let test_serv = app.get_component_arc::<TestService>().unwrap();
  let ret = test_serv.test().await.unwrap();
  assert_eq!(ret, "test service");
}
