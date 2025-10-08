use std::{collections::HashMap, env, path::Path};

use config::{Config, ConfigBuilder, Environment, File, FileFormat, builder::DefaultState};
use fusion_common::{
  env::{get_env, get_envs},
  runtime,
};
use log::{debug, trace};

use super::ConfigureResult;

/// 加载配置
///
/// [crate::RunModel]
pub fn load_config() -> ConfigureResult<Config> {
  load_config_with(None)
}

pub fn load_config_with(custom_config: Option<Config>) -> ConfigureResult<Config> {
  let mut b = Config::builder().add_source(load_default_source());

  // load from default files, if exists
  b = load_from_files(&["app.toml".to_string(), "app.yaml".to_string(), "app.yml".to_string()], b);

  // load from profile files, if exists
  let profile_files = if let Ok(profiles_active) = env::var("FUSION__PROFILES__ACTIVE") {
    vec![
      format!("app-{profiles_active}.toml"),
      format!("app-{profiles_active}.yaml"),
      format!("app-{profiles_active}.yml"),
    ]
  } else {
    vec![]
  };
  debug!("Load profile files: {:?}", profile_files);
  b = load_from_files(&profile_files, b);

  // load from file of env, if exists
  if let Ok(file) = get_env("FUSION_CONFIG_FILE") {
    let path = Path::new(&file);
    if path.exists() {
      b = b.add_source(File::from(path));
    }
  }

  b = add_environment(b);

  if let Some(custom_config) = custom_config {
    b = b.add_source(custom_config);
  }

  let c = b.build()?;

  trace!("Load config file: {}", c.cache);

  Ok(c)
}

fn load_from_files(files: &[String], mut b: ConfigBuilder<DefaultState>) -> ConfigBuilder<DefaultState> {
  for file in files {
    if let Ok(path) = runtime::cargo_manifest_dir().map(|dir| dir.join("resources").join(file))
      && path.exists()
    {
      b = b.add_source(File::from(path));
      break;
    }
  }

  for file in files {
    let path = Path::new(file);
    if path.exists() {
      b = b.add_source(File::from(path));
      break;
    }
  }

  b
}

pub fn load_default_source() -> File<config::FileSourceString, FileFormat> {
  let text = include_str!("default.toml");
  File::from_str(text, FileFormat::Toml)
}

pub fn add_environment(b: ConfigBuilder<DefaultState>) -> ConfigBuilder<DefaultState> {
  // Load all latest variables from current environment
  let envs = get_envs();
  let env = Environment::default().separator("__").source(Some(envs.into_iter().collect::<HashMap<_, _>>()));
  b.add_source(env)
}
