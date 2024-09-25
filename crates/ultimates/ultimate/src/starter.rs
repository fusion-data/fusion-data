use crate::{configuration::ConfigState, trace};

pub fn load_and_init() -> ConfigState {
  let config_state = config_load();
  let ultimate_config = config_state.ultimate_config();
  trace::init_trace(ultimate_config);
  config_state
}

pub fn config_load() -> ConfigState {
  // 配置文件载入失败应提前终止程序
  ConfigState::load().unwrap()
}
