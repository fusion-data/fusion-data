use ::tracing::log::Level;
use tracing_subscriber::layer::SubscriberExt;

use crate::{
  configuration::{
    model::{LogLevel, LogWriterType, TracingConfig},
    ConfigurationState,
  },
  tracing::{self, build_loglevel_filter_layer, file_fmt_layer, stdout_fmt_layer},
};

pub fn load_and_init() -> ConfigurationState {
  //setup a temporary subscriber to log output during setup
  let _guard = {
    let c = TracingConfig {
      enable: true,
      target: true,
      log_level: LogLevel(Level::Trace),
      log_writer: LogWriterType::Stdout,
      log_dir: "./logs/".to_string(),
      ..Default::default()
    };
    let subscriber =
      tracing_subscriber::registry().with(build_loglevel_filter_layer(c.log_level)).with(stdout_fmt_layer(&c));

    #[cfg(feature = "tracing-appender")]
    let subscriber = subscriber.with(file_fmt_layer(&temporary_app_name(), &c));

    ::tracing::subscriber::set_default(subscriber)
  };

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

fn temporary_app_name() -> String {
  std::env::var("ULTIMATE__APP__NAME")
    .or_else(|_| std::env::var("ULTIMATE_APP_NAME"))
    .unwrap_or_else(|_| "ultimate".to_string())
}
