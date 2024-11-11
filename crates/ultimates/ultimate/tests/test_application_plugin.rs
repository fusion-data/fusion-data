use async_trait::async_trait;
use config::{File, FileFormat};
use serde::Deserialize;
use ultimate::configuration::{ConfigRegistry, Configuration};
use ultimate::{
  application::{Application, ApplicationBuilder},
  plugin::Plugin,
};

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

#[derive(Debug, Configuration, Deserialize)]
#[config_prefix = "my-plugin"]
struct Config {
  a: u32,
  b: bool,
  c: ConfigInner,
  d: String,
  e: ConfigEnum,
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
  Application::builder().add_config_source(cs).add_plugin(MyPlugin).run().await;
  let app = Application::global();
  assert_eq!(app.ultimate_config().app().name(), "ultimate");
}
