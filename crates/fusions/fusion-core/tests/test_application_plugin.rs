use async_trait::async_trait;
use config::{File, FileFormat};
use fusion_core::configuration::{ConfigRegistry, Configurable};
use fusion_core::{
  application::{Application, ApplicationBuilder},
  plugin::Plugin,
};
use serde::Deserialize;

struct MyPlugin;

#[async_trait]
impl Plugin for MyPlugin {
  async fn build(&self, app: &mut ApplicationBuilder) {
    match app.get_config::<Config>() {
      Ok(config) => {
        println!("{:#?}", config);
        assert_eq!(config.a, 1);
        assert!(config.b);
        assert_eq!(config.c.g, "hello");
        assert_eq!(config.d, "world");
        assert_eq!(config.e, ConfigEnum::EA);
        println!("c.f: {}", config.c.f);
      }
      Err(e) => println!("{:?}", e),
    }
  }
}

#[derive(Debug, Deserialize)]
struct Config {
  a: u32,
  b: bool,
  c: ConfigInner,
  d: String,
  e: ConfigEnum,
}

impl Configurable for Config {
  fn config_prefix() -> &'static str {
    "my-plugin"
  }
}

#[derive(PartialEq, Debug, Deserialize)]
enum ConfigEnum {
  EA,
  EB,
  EC,
  ED,
}

#[derive(Debug, Deserialize)]
struct ConfigInner {
  f: u32,
  g: String,
}

#[tokio::test]
async fn test_application_plugin() {
  let cs = File::from_str(
    r#"[my-plugin]
a = 1
b = true
c.f = 11
c.g = "hello"
d = "world"
e = "EA"
"#,
    FileFormat::Toml,
  );
  Application::builder().add_config_source(cs).add_plugin(MyPlugin).run().await.unwrap();
  let app = Application::global();
  assert_eq!(app.fusion_setting().app().name(), "fusion");
  let c: Config = app.get_config().unwrap();
  assert_eq!(c.c.g, "hello");
}
