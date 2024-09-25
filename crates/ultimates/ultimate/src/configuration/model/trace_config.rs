use serde::{
  de::{Unexpected, Visitor},
  Deserialize, Deserializer, Serialize,
};
use std::fmt::Display;
use tracing::log::Level;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceConfig {
  pub enable: bool,

  pub target: bool,

  pub log_level: LogLevel,

  pub log_writer: LogWriterType,

  /// 目录输出目录
  pub log_dir: String,

  /// 目标文件名，默认为 <app name>.log
  pub log_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum LogWriterType {
  Console,
  File,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct LogLevel(Level);

impl Display for LogLevel {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.pad(self.0.as_str())
  }
}

impl<'de> Deserialize<'de> for LogLevel {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    const MSG: &str = "attempted to convert a string that doesn't match an existing log level";
    struct StrToLogLevel;
    impl<'d> Visitor<'d> for StrToLogLevel {
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
    const MSG: &str = "expect in ('console', 'file').";

    struct StrToLogWriterType;
    impl<'d> Visitor<'d> for StrToLogWriterType {
      type Value = LogWriterType;

      fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(MSG)
      }

      fn visit_str<E>(self, v: &str) -> core::result::Result<Self::Value, E>
      where
        E: serde::de::Error,
      {
        let writer = if v.eq_ignore_ascii_case("console") {
          LogWriterType::Console
        } else if v.eq_ignore_ascii_case("file") {
          LogWriterType::File
        } else {
          return Err(serde::de::Error::invalid_value(Unexpected::Str(v), &MSG));
        };
        Ok(writer)
      }
    }

    deserializer.deserialize_str(StrToLogWriterType)
  }
}
