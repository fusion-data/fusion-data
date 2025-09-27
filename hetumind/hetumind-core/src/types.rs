use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
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

#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr, AsRefStr)]
#[repr(u8)]
pub enum CodeLanguage {
  JavaScript = 1,
  Python = 2,
}
