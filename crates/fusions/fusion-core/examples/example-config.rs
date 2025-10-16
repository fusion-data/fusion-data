use fusion_core::application::Application;

#[tokio::main]
async fn main() {
  let app = Application::builder().build().await.unwrap();

  let config = app.fusion_setting();
  println!("Loaded config[app] is {:?}", config.app());
}
