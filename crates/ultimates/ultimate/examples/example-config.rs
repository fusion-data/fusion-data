use ultimate::application::Application;

#[tokio::main]
async fn main() {
  let app = Application::builder().build().await.unwrap();

  let config = app.ultimate_config();
  println!("Loaded config[app] is {:?}", config.app());
}
