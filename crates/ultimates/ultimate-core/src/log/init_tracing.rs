use std::sync::Arc;

use ::log::Level;
use tracing::{Subscriber, debug, info, subscriber::DefaultGuard};
use tracing_subscriber::{
  filter::EnvFilter,
  fmt::{
    self,
    format::{DefaultFields, FmtSpan, Format, Full, Pretty},
    time::FormatTime,
  },
  layer::SubscriberExt,
  registry::LookupSpan,
};
use ultimate_common::time::{self, FixedOffset};

use crate::{
  Result,
  configuration::{LogConfig, LogLevel, LogWriterType, UltimateConfig},
};

pub fn init_tracing(c: &UltimateConfig) {
  init_subscribers(c).expect("Init tracing error. Please check your configuration");
}

// setup a temporary subscriber to log output during setup
pub(crate) fn init_tracing_guard() -> (DefaultGuard, Option<String>) {
  let c = LogConfig {
    with_target: true,
    log_level: LogLevel(Level::Trace),
    log_writer: LogWriterType::Stdout,
    log_dir: std::option_env!("ULTIMATE__LOG_DIR").unwrap_or_else(|| "./var/logs/").to_string(),
    ..Default::default()
  };
  let fixed_offset = time::local_offset();
  let (layer, original_rust_log) = build_loglevel_filter_layer(&c);
  let subscriber = tracing_subscriber::registry()
    .with(layer)
    .with(stdout_fmt_layer(*fixed_offset, &c))
    .with(file_fmt_layer(&temporary_app_name(), *fixed_offset, &c));

  (::tracing::subscriber::set_default(subscriber), original_rust_log)
}

fn temporary_app_name() -> String {
  std::env::var("ULTIMATE__APP__NAME")
    .or_else(|_| std::env::var("ULTIMATE_APP_NAME"))
    .unwrap_or_else(|_| "ultimate".to_string())
}

fn init_subscribers(conf: &UltimateConfig) -> Result<()> {
  let c = conf.log();
  info!("init logging & tracing");
  info!("Loaded the LogConfig is:\n{}", toml::to_string(c).unwrap());

  let otel_layer = if cfg!(feature = "opentelemetry") && c.otel().enable {
    unsafe {
      std::env::set_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", &c.otel().exporter_otlp_endpoint);
      std::env::set_var("OTEL_TRACES_SAMPLER", &c.otel().traces_sample);
      std::env::set_var("OTEL_SERVICE_NAME", conf.app().name());
    }
    let (layer, _guard) = init_tracing_opentelemetry::tracing_subscriber_ext::build_otel_layer()
      .map_err(|e| crate::DataError::server_error(e.to_string()))?;
    Some(Box::new(layer))
  } else {
    None
  };

  let subscriber = tracing_subscriber::registry()
    .with(otel_layer)
    .with(build_loglevel_filter_layer(c).0)
    .with(stdout_fmt_layer(*conf.app().time_offset(), c))
    .with(file_fmt_layer(conf.app().name(), *conf.app().time_offset(), c));

  tracing::subscriber::set_global_default(subscriber)?;
  Ok(())
}

#[must_use]
pub fn build_loglevel_filter_layer(c: &LogConfig) -> (EnvFilter, Option<String>) {
  let rust_log = std::env::var("RUST_LOG").or_else(|_| std::env::var("OTEL_LOG_LEVEL"));
  let original_rust_log = rust_log.clone().ok();

  let value = [
    if c.log_targets.is_empty() { None } else { Some(c.log_targets.join(",")) },
    if c.otel().enable {
      // `otel::tracing` should be a level info to emit opentelemetry trace & span
      // `otel::setup` set to debug to log detected resources, configuration read and infered
      Some("otel::tracing=trace,otel::setup=debug".to_string())
    } else {
      None
    },
    rust_log.ok().or_else(|| Some(c.log_level.to_string())),
  ]
  .into_iter()
  .flatten()
  .collect::<Vec<_>>()
  .join(",");

  // let value = format!("{},{},{}", rust_log.unwrap_or_else(|_| c.log_level.to_string()), otel, libraries);
  let log_value = if value.ends_with(',') { &value[..value.len() - 1] } else { &value[..] };

  debug!("ORIGINAL RUST_LOG: {:?}; NEW RUST_LOG: {}", original_rust_log, log_value);
  unsafe {
    std::env::set_var("RUST_LOG", log_value);
  }
  (EnvFilter::from_default_env(), original_rust_log)
}

pub fn stdout_fmt_layer<S>(
  fixed_offset: FixedOffset,
  c: &LogConfig,
) -> Option<fmt::Layer<S, Pretty, Format<Pretty, Chrono>>>
where
  S: Subscriber,
  for<'a> S: LookupSpan<'a>,
{
  if c.log_writer.is_stdout() {
    let l = _fmt_layer(fixed_offset, c).pretty().with_ansi(true);
    Some(l)
  } else {
    None
  }
}

pub fn file_fmt_layer<S>(
  app_name: &str,
  fixed_offset: FixedOffset,
  c: &LogConfig,
) -> Option<
  fmt::Layer<
    S,
    fmt::format::JsonFields,
    Format<fmt::format::Json, Chrono>,
    tracing_appender::rolling::RollingFileAppender,
  >,
>
where
  S: Subscriber,
  for<'a> S: LookupSpan<'a>,
{
  use std::path::Path;
  if c.log_writer.is_file() {
    //.with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
    let path = Path::new(&c.log_dir);
    let file_appender = tracing_appender::rolling::daily(path, format!("{}.log", app_name));
    let l = _fmt_layer(fixed_offset, c).json().with_writer(file_appender);
    Some(l)
  } else {
    None
  }
}

fn _fmt_layer<S>(fixed_offset: FixedOffset, c: &LogConfig) -> fmt::Layer<S, DefaultFields, Format<Full, Chrono>>
where
  S: Subscriber,
  for<'a> S: LookupSpan<'a>,
{
  let fmt_span = c.with_span_events.iter().fold(FmtSpan::NONE, |span, s| span | parse_to_fmt_span(s));
  let format = if c.time_format.is_empty() {
    Arc::new(ChronoFmtType::Rfc3339)
  } else {
    Arc::new(ChronoFmtType::Custom(c.time_format.clone()))
  };
  let fmt_time = Chrono { format, fixed_offset };
  fmt::layer::<S>()
    .with_file(c.with_file)
    .with_line_number(c.with_line_number)
    .with_thread_ids(c.with_thread_ids)
    .with_thread_names(c.with_thread_names)
    .with_target(c.with_target)
    .with_span_events(fmt_span)
    .with_timer(fmt_time)
}

fn parse_to_fmt_span(s: &str) -> FmtSpan {
  if "new".eq_ignore_ascii_case(s) {
    FmtSpan::NEW
  } else if "enter".eq_ignore_ascii_case(s) {
    FmtSpan::ENTER
  } else if "exit".eq_ignore_ascii_case(s) {
    FmtSpan::EXIT
  } else if "close".eq_ignore_ascii_case(s) {
    FmtSpan::CLOSE
  } else if "none".eq_ignore_ascii_case(s) {
    FmtSpan::NONE
  } else if "active".eq_ignore_ascii_case(s) {
    FmtSpan::ACTIVE
  } else if "full".eq_ignore_ascii_case(s) {
    FmtSpan::FULL
  } else {
    panic!("Invalid FmtSpan value: {}", s)
  }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Chrono {
  format: Arc<ChronoFmtType>,
  fixed_offset: FixedOffset,
}

impl FormatTime for Chrono {
  fn format_time(&self, w: &mut fmt::format::Writer<'_>) -> std::fmt::Result {
    let t = time::now().with_timezone(&self.fixed_offset);
    match self.format.as_ref() {
      ChronoFmtType::Rfc3339 => w.write_str(&t.to_rfc3339()),
      ChronoFmtType::Custom(fmt) => w.write_str(&format!("{}", t.format(fmt))),
    }
  }
}

#[derive(Debug, Clone, Eq, PartialEq)]

enum ChronoFmtType {
  /// Format according to the RFC 3339 convention.
  Rfc3339,
  /// Format according to a custom format string.
  Custom(String),
}
