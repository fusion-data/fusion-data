use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::configuration::{model::LogWriterType, UltimateConfig};

pub fn init_trace(c: &UltimateConfig) {
  if !c.trace().enable {
    return;
  }

  let rust_log = std::env::var("RUST_LOG");
  if rust_log.is_err() || rust_log.is_ok_and(|s| s.is_empty()) {
    std::env::set_var("RUST_LOG", c.trace().log_level.to_string());
  }

  let formater = tracing_subscriber::fmt::format()
    .compact()
    .with_file(true)
    .with_line_number(true)
    .with_thread_ids(true)
    .with_thread_names(true)
    .with_target(c.trace().target);

  let registry = tracing_subscriber::registry().with(EnvFilter::from_default_env());
  let fmt_layer = fmt::layer().event_format(formater);

  if LogWriterType::Console == c.trace().log_writer {
    registry.with(fmt_layer).init();
  } else {
    #[cfg(feature = "tracing-appender")]
    {
      let r = registry.with(fmt_layer.with_writer(init_log_appender(c)));
      r.init();
    }
    #[cfg(not(feature = "tracing-appender"))]
    {
      let r = registry.with(fmt_layer);
      r.init();
    }
  }

  // if c.trace().log_level {
  info!("Loaded the UltimateConfig is:\n{}", toml::to_string(c).unwrap());
  // }
}

#[cfg(feature = "tracing-appender")]
pub fn init_log_appender(c: &UltimateConfig) -> tracing_appender::rolling::RollingFileAppender {
  use std::path::Path;

  let path = Path::new(&c.trace().log_dir);
  let file_appender = tracing_appender::rolling::daily(path, c.app().name());

  // let (non_blocking, _guard1) = tracing_appender::non_blocking(file_appender);

  // non_blocking
  file_appender
}
