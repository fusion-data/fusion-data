use tracing::{info, Subscriber};
use tracing_subscriber::{filter::EnvFilter, layer::SubscriberExt, registry::LookupSpan, Layer};

use crate::configuration::Configuration;

pub fn init_trace(c: &Configuration) {
  if !c.trace().enable {
    return;
  }

  init_subscribers(c).expect("Init tracing error. Please check your configuration");
}

fn init_subscribers(c: &Configuration) -> Result<(), init_tracing_opentelemetry::Error> {
  use init_tracing_opentelemetry::tracing_subscriber_ext::build_otel_layer;

  //setup a temporary subscriber to log output during setup
  let subscriber = tracing_subscriber::registry().with(build_loglevel_filter_layer()).with(build_logger_text(c));
  let _guard = tracing::subscriber::set_default(subscriber);
  info!("init logging & tracing");
  info!("Loaded the Configuration is:\n{}", toml::to_string(c).unwrap());

  let subscriber = tracing_subscriber::registry()
    .with(build_otel_layer()?)
    .with(build_loglevel_filter_layer())
    .with(build_logger_text(c));

  tracing::subscriber::set_global_default(subscriber)?;
  Ok(())
}

#[must_use]
fn build_loglevel_filter_layer() -> tracing_subscriber::filter::EnvFilter {
  // filter what is output on log (fmt)
  // std::env::set_var("RUST_LOG", "warn,otel::tracing=info,otel=debug");
  std::env::set_var(
    "RUST_LOG",
    format!(
      // `otel::tracing` should be a level info to emit opentelemetry trace & span
      // `otel::setup` set to debug to log detected resources, configuration read and infered
      "{},otel::tracing=trace,otel=debug",
      std::env::var("RUST_LOG").or_else(|_| std::env::var("OTEL_LOG_LEVEL")).unwrap_or_else(|_| "info".to_string())
    ),
  );
  EnvFilter::from_default_env()
}

#[must_use]
fn build_logger_text<S>(c: &Configuration) -> Box<dyn Layer<S> + Send + Sync + 'static>
where
  S: Subscriber + for<'a> LookupSpan<'a>,
{
  use tracing_subscriber::fmt::format::FmtSpan;

  #[cfg(feature = "tracing-appender")]
  {
    Box::new(
      tracing_subscriber::fmt::layer()
      .json()
      //.with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
      .with_timer(tracing_subscriber::fmt::time::time()).with_writer(init_log_appender(c)),
    )
  }
  #[cfg(not(feature = "tracing-appender"))]
  {
    if cfg!(debug_assertions) {
      Box::new(
        tracing_subscriber::fmt::layer()
          .pretty()
          .with_ansi(true)
          .with_file(true)
          .with_line_number(true)
          .with_thread_ids(true)
          .with_thread_names(true)
          .with_target(c.trace().target)
          .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
          .with_timer(tracing_subscriber::fmt::time::ChronoLocal::rfc_3339()),
      )
    } else {
      Box::new(
        tracing_subscriber::fmt::layer()
                .json()
                //.with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_timer(tracing_subscriber::fmt::time::ChronoLocal::rfc_3339()),
      )
    }
  }
}

#[cfg(feature = "tracing-appender")]
fn init_log_appender(c: &Configuration) -> tracing_appender::rolling::RollingFileAppender {
  use std::path::Path;

  let path = Path::new(&c.trace().log_dir);
  let file_appender = tracing_appender::rolling::daily(path, c.app().name());

  // let (non_blocking, _guard1) = tracing_appender::non_blocking(file_appender);

  // non_blocking
  file_appender
}
