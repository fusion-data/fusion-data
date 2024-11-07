use ultimate::component::Component;
use ultimate::{application::Application, component::ComponentRef};
use ultimate_db::{DbPlugin, ModelManager};

#[derive(Clone, Component)]
struct TestService {
  #[component]
  mm: ModelManager,
}

impl TestService {
  pub async fn test(&self) -> ultimate::Result<String> {
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
  let mm: ComponentRef<ModelManager> = app.get_component_ref().unwrap();

  let addr: *const ModelManager = &*mm;
  println!("ModelManager address: {:p}", addr);

  let test_serv = app.get_component_ref::<TestService>().unwrap();
  let ret = test_serv.test().await.unwrap();
  assert_eq!(ret, "test service");
}
