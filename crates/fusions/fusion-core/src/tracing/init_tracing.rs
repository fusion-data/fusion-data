use std::sync::Arc;

use ::log::Level;
use fusion_common::time::{self, FixedOffset};
use init_tracing_opentelemetry::{Guard, TracingConfig};
use tracing::{Subscriber, debug, info, subscriber::DefaultGuard};
use tracing_subscriber::{
  Registry,
  filter::EnvFilter,
  fmt::{
    self,
    format::{DefaultFields, FmtSpan, Format, Full, Pretty},
    time::FormatTime,
  },
  layer::SubscriberExt,
  registry::LookupSpan,
};

use crate::{
  DataError, Result,
  configuration::{FusionSetting, LogLevel, LogSetting, LogWriterType},
};

// setup a temporary subscriber to log output during setup
pub(crate) fn init_tracing_guard() -> (DefaultGuard, Option<String>) {
  let c = LogSetting {
    with_target: true,
    log_level: LogLevel(Level::Trace),
    log_writers: vec![LogWriterType::Stdout],
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

pub fn init_subscribers(setting: &FusionSetting) -> Result<Option<Guard>> {
  let (_tmp_guard, _) = init_tracing_guard();

  let c = setting.log();
  info!("init logging & tracing");
  info!("Loaded the LogSetting is:\n{}", toml::to_string(c).unwrap());

  if c.otel().enable {
    unsafe {
      std::env::set_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", &c.otel().exporter_otlp_endpoint);
      std::env::set_var("OTEL_TRACES_SAMPLER", &c.otel().traces_sample);
      std::env::set_var("OTEL_SERVICE_NAME", setting.app().name());
    }

    let guard = TracingConfig::default()
      .with_log_directives("info,tokio::task=trace,tokio::task::waker=warn")
      .with_span_events(tracing_subscriber::fmt::format::FmtSpan::NONE)
      .init_subscriber_ext(|subscriber| {
        subscriber.with(file_fmt_layer(setting.app().name(), *setting.app().time_offset(), c))
      })
      .map_err(|e| DataError::server_error(format!("Init tracing & otel failed, error: {}", e)))?;
    Ok(Some(guard))
  } else {
    let subscriber = transform_identity(setting, tracing_subscriber::registry());
    tracing::subscriber::set_global_default(subscriber)
      .map_err(|e| DataError::server_error(format!("Set global default traceing subscriber failed, error: {}", e)))?;
    Ok(None)
  }
}

fn transform_identity(
  setting: &FusionSetting,
  subscriber: Registry,
) -> impl Subscriber + for<'a> LookupSpan<'a> + Send + Sync {
  let c = setting.log();
  subscriber
    .with(build_loglevel_filter_layer(c).0)
    .with(stdout_fmt_layer(*setting.app().time_offset(), c))
    .with(file_fmt_layer(setting.app().name(), *setting.app().time_offset(), c))
}

#[must_use]
pub fn build_loglevel_filter_layer(c: &LogSetting) -> (EnvFilter, Option<String>) {
  let rust_log = std::env::var("RUST_LOG").or_else(|_| std::env::var("OTEL_LOG_LEVEL"));
  let original_rust_log = rust_log.clone().ok();

  let value = [
    if c.log_targets.is_empty() { None } else { Some(c.log_targets.join(",")) },
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
  c: &LogSetting,
) -> Option<fmt::Layer<S, Pretty, Format<Pretty, Chrono>>>
where
  S: Subscriber,
  for<'a> S: LookupSpan<'a>,
{
  if c.log_writers.iter().any(|lw| lw.is_stdout()) {
    let l = _fmt_layer(fixed_offset, c).pretty().with_ansi(true);
    Some(l)
  } else {
    None
  }
}

pub fn file_fmt_layer<S>(
  app_name: &str,
  fixed_offset: FixedOffset,
  c: &LogSetting,
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
  if c.log_writers.iter().any(|lw| lw.is_file()) {
    //.with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
    let path = Path::new(&c.log_dir);
    let file_appender = tracing_appender::rolling::daily(path, format!("{}.log", app_name));
    let l = _fmt_layer(fixed_offset, c).json().with_writer(file_appender);
    Some(l)
  } else {
    None
  }
}

fn _fmt_layer<S>(fixed_offset: FixedOffset, c: &LogSetting) -> fmt::Layer<S, DefaultFields, Format<Full, Chrono>>
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
