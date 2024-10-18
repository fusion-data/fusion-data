use crate::{configuration::ConfigurationState, tracing};

pub fn load_and_init() -> ConfigurationState {
  let config_state = config_load();
  let ultimate_config = config_state.configuration();
  tracing::init_tracing(ultimate_config);
  config_state
}

pub fn config_load() -> ConfigurationState {
  // 配置文件载入失败应提前终止程序
  match ConfigurationState::load() {
    Ok(c) => c,
    Err(err) => panic!("Failed to load configuration: {}", err),
  }
}
