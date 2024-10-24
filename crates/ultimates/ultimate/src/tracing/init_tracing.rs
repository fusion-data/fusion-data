use tracing::{info, Subscriber};
use tracing_subscriber::{
  filter::EnvFilter,
  fmt::{
    self,
    format::{FmtSpan, Format, Pretty},
    time::ChronoLocal,
  },
  layer::SubscriberExt,
  registry::LookupSpan,
};

use crate::{
  configuration::{
    model::{LogWriterType, TracingConfig},
    Configuration,
  },
  Result,
};

pub fn init_tracing(c: &Configuration) {
  if !c.tracing().enable {
    return;
  }

  init_subscribers(c).expect("Init tracing error. Please check your configuration");
}

fn init_subscribers(c: &Configuration) -> Result<()> {
  //setup a temporary subscriber to log output during setup
  let subscriber =
    tracing_subscriber::registry().with(build_loglevel_filter_layer(c.tracing())).with(stdout_fmt_layer(c));

  #[cfg(feature = "tracing-appender")]
  let subscriber = subscriber.with(file_fmt_layer(c));

  let _guard = tracing::subscriber::set_default(subscriber);
  info!("init logging & tracing");
  info!("Loaded the Configuration is:\n{}", toml::to_string(c).unwrap());

  #[cfg(feature = "opentelemetry")]
  let otel_layer = if let Some(otel) = &c.tracing().otel {
    std::env::set_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", &otel.exporter_otlp_endpoint);
    std::env::set_var("OTEL_TRACES_SAMPLER", &otel.traces_sample);
    std::env::set_var("OTEL_SERVICE_NAME", c.app().name());
    let layer = init_tracing_opentelemetry::tracing_subscriber_ext::build_otel_layer()
      .map_err(|e| crate::DataError::server_error(e.to_string()))?;
    Some(layer)
  } else {
    None
  };
  #[cfg(not(feature = "opentelemetry"))]
  let otel_layer: Option<Box<dyn tracing_subscriber::Layer<_> + Send + Sync + 'static>> = Default::default();

  let subscriber = tracing_subscriber::registry()
    .with(otel_layer)
    .with(build_loglevel_filter_layer(c.tracing()))
    .with(stdout_fmt_layer(c));

  #[cfg(feature = "tracing-appender")]
  let subscriber = subscriber.with(file_fmt_layer(c));

  tracing::subscriber::set_global_default(subscriber)?;
  Ok(())
}

#[must_use]
fn build_loglevel_filter_layer(conf: &TracingConfig) -> EnvFilter {
  // filter what is output on log (fmt)
  // std::env::set_var("RUST_LOG", "warn,otel::tracing=info,otel=debug");
  std::env::set_var(
    "RUST_LOG",
    format!(
      // `otel::tracing` should be a level info to emit opentelemetry trace & span
      // `otel::setup` set to debug to log detected resources, configuration read and infered
      "{},otel::tracing=trace,otel=debug",
      std::env::var("RUST_LOG")
        .or_else(|_| std::env::var("OTEL_LOG_LEVEL"))
        .unwrap_or_else(|_| conf.log_level.to_string())
    ),
  );
  EnvFilter::from_default_env()
}

fn stdout_fmt_layer<S>(c: &Configuration) -> Option<fmt::Layer<S, Pretty, Format<Pretty, ChronoLocal>>>
where
  S: Subscriber,
  for<'a> S: LookupSpan<'a>,
{
  if c.tracing().enable
    && (c.tracing().log_writer == LogWriterType::Stdout || c.tracing().log_writer == LogWriterType::Both)
  {
    let l = fmt::layer::<S>()
      .pretty()
      .with_ansi(true)
      .with_file(true)
      .with_line_number(true)
      .with_thread_ids(true)
      .with_thread_names(true)
      .with_target(c.tracing().target)
      .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
      .with_timer(fmt::time::ChronoLocal::rfc_3339());
    Some(l)
  } else {
    None
  }
}

#[cfg(feature = "tracing-appender")]
fn file_fmt_layer<S>(
  c: &Configuration,
) -> Option<
  fmt::Layer<
    S,
    fmt::format::JsonFields,
    Format<fmt::format::Json, ChronoLocal>,
    tracing_appender::rolling::RollingFileAppender,
  >,
>
where
  S: Subscriber,
  for<'a> S: LookupSpan<'a>,
{
  use std::path::Path;
  if c.tracing().enable
    && (c.tracing().log_writer == LogWriterType::File || c.tracing().log_writer == LogWriterType::Both)
  {
    //.with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
    let path = Path::new(&c.tracing().log_dir);
    let file_appender = tracing_appender::rolling::daily(path, format!("{}.log", c.app().name()));
    let l = fmt::layer::<S>()
      .json()
      .with_file(true)
      .with_line_number(true)
      .with_thread_ids(true)
      .with_thread_names(true)
      .with_target(c.tracing().target)
      .with_timer(fmt::time::ChronoLocal::rfc_3339())
      .with_writer(file_appender);
    Some(l)
  } else {
    None
  }
}
