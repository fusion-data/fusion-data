use ultimate::starter;
use ultimate_common::runtime;

fn main() {
  // for (key, value) in std::env::vars() {
  //     println!("{}: {}", key, value);
  // }

  println!("cargo_manifest_dir: {:?}", runtime::cargo_manifest_dir());

  let config_state = starter::load_and_init();

  println!("Config content is:\n{}", toml::to_string(config_state.configuration()).unwrap());
}
