use serde::{Deserialize, Serialize};
use strum::AsRefStr;

pub type JsonValue = serde_json::Value;

/// 支持的数据类型
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DataType {
  #[default]
  String,
  Number,
  Boolean,
  DateTime {
    offset: Option<i32>,
  },
  Date,
  Time,
  Object,
  Array {
    kind: Box<DataType>,
  },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IconColor {
  Gray,
  Black,
  Blue,
  LightBlue,
  DarkBlue,
  Orange,
  OrangeRed,
  PinkRed,
  Red,
  LightGreen,
  Green,
  DarkGreen,
  Azure,
  Purple,
  Crimson,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, AsRefStr)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum CodeLanguage {
  JavaScript = 1,
  Python = 2,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BinaryFileKind {
  Text,
  Json,
  Image,
  Video,
  Audio,
  Pdf,
  Html,
  Excel,
  Word,
  Ppt,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextFileType {
  Csv,
  Markdown,
  Xml,
  Yaml,
  Toml,
  Properties,
  Ini,
}
