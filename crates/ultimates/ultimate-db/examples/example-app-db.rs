use modelsql::ModelManager;
use ultimate_core::component::Component;
use ultimate_core::{application::Application, component::ComponentArc};
use ultimate_db::DbPlugin;

#[derive(Clone, Component)]
struct TestSvc {
  #[component]
  mm: ModelManager,
}

impl TestSvc {
  pub async fn test(&self) -> ultimate_core::Result<String> {
    let _mm = &self.mm;
    Ok(String::from("test service"))
  }
}

#[tokio::main]
async fn main() {
  logforth::stdout().apply();

  let mut ab = Application::builder();
  ab.add_plugin(DbPlugin);
  ab.run().await.unwrap();
  let app = Application::global();
  let mm: ComponentArc<ModelManager> = app.get_component_arc().unwrap();

  let addr: *const ModelManager = &*mm;
  println!("ModelManager address: {:p}", addr);

  let test_svc = app.get_component_arc::<TestSvc>().unwrap();
  let ret = test_svc.test().await.unwrap();
  assert_eq!(ret, "test service");
}
