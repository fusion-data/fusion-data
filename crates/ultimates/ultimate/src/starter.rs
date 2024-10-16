use crate::{configuration::ConfigurationState, trace};

pub fn load_and_init() -> ConfigurationState {
  let config_state = config_load();
  let ultimate_config = config_state.configuration();
  trace::init_trace(ultimate_config);
  config_state
}

pub fn config_load() -> ConfigurationState {
  // 配置文件载入失败应提前终止程序
  ConfigurationState::load().unwrap()
}
