use serde::{
  Deserialize, Deserializer, Serialize,
  de::{Unexpected, Visitor},
};
use std::fmt::Display;
use tracing::log::Level;

fn default_log_dir() -> String {
  std::option_env!("ULTIMATE__LOG_DIR").unwrap_or_else(|| "./var/logs/").to_string()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogConfig {
  pub with_target: bool,
  pub with_file: bool,
  pub with_thread_ids: bool,
  pub with_thread_names: bool,
  pub with_line_number: bool,
  pub with_span_events: Vec<String>,
  pub time_format: String,
  pub log_level: LogLevel,
  pub log_targets: Vec<String>,
  pub log_writer: LogWriterType,

  /// 目录输出目录
  #[serde(default = "default_log_dir")]
  pub log_dir: String,

  /// 目标文件名，默认为 <app name>.log
  pub log_name: Option<String>,

  pub otel: OtelConfig,
}

impl LogConfig {
  pub fn otel(&self) -> &OtelConfig {
    &self.otel
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelConfig {
  pub enable: bool,
  pub traces_sample: String,
  pub exporter_otlp_endpoint: String,
}

impl Default for OtelConfig {
  fn default() -> Self {
    Self {
      enable: Default::default(),
      traces_sample: String::from("always_on"),
      exporter_otlp_endpoint: String::from("http://localhost:4317"),
    }
  }
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum LogWriterType {
  #[default]
  Stdout,
  File,
  Both,
}

impl LogWriterType {
  pub fn is_stdout(&self) -> bool {
    matches!(self, LogWriterType::Stdout | LogWriterType::Both)
  }

  pub fn is_file(&self) -> bool {
    matches!(self, LogWriterType::File | LogWriterType::Both)
  }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct LogLevel(pub(crate) Level);

impl Display for LogLevel {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.pad(self.0.as_str())
  }
}

impl Default for LogLevel {
  fn default() -> Self {
    LogLevel(Level::Info)
  }
}

impl<'de> Deserialize<'de> for LogLevel {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    const MSG: &str = "attempted to convert a string that doesn't match an existing log level";
    struct StrToLogLevel;
    impl Visitor<'_> for StrToLogLevel {
      type Value = LogLevel;

      fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(MSG)
      }

      fn visit_str<E>(self, v: &str) -> core::result::Result<Self::Value, E>
      where
        E: serde::de::Error,
      {
        let level = if v.eq_ignore_ascii_case("error") {
          Level::Error
        } else if v.eq_ignore_ascii_case("warn") {
          Level::Warn
        } else if v.eq_ignore_ascii_case("info") {
          Level::Info
        } else if v.eq_ignore_ascii_case("debug") {
          Level::Debug
        } else if v.eq_ignore_ascii_case("trace") {
          Level::Trace
        } else {
          return Err(serde::de::Error::invalid_value(Unexpected::Str(v), &MSG));
        };
        Ok(LogLevel(level))
      }
    }

    deserializer.deserialize_str(StrToLogLevel)
  }
}

impl Serialize for LogLevel {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(self.0.as_str())
  }
}

impl<'de> Deserialize<'de> for LogWriterType {
  fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    const MSG: &str = "expect in ('stdout', 'file', 'both').";

    struct StrToLogWriterType;
    impl Visitor<'_> for StrToLogWriterType {
      type Value = LogWriterType;

      fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(MSG)
      }

      fn visit_str<E>(self, v: &str) -> core::result::Result<Self::Value, E>
      where
        E: serde::de::Error,
      {
        let writer = if v.eq_ignore_ascii_case("stdout") {
          LogWriterType::Stdout
        } else if v.eq_ignore_ascii_case("file") {
          LogWriterType::File
        } else if v.eq_ignore_ascii_case("both") {
          LogWriterType::Both
        } else {
          return Err(serde::de::Error::invalid_value(Unexpected::Str(v), &MSG));
        };
        Ok(writer)
      }
    }

    deserializer.deserialize_str(StrToLogWriterType)
  }
}
