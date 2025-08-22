// src/value.rs
use ahash::HashMap;
use chrono::Duration;
use serde::{Deserialize, Serialize};
use ultimate_common::time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
  Null,
  Bool(bool),
  Number(f64),
  String(String),
  Array(Vec<Value>),
  Object(HashMap<String, Value>),
  DateTime(OffsetDateTime),
  Binary(BinaryData),
}

impl Value {
  pub fn new_string(s: impl Into<String>) -> Self {
    Self::String(s.into())
  }

  pub fn new_number(n: impl Into<f64>) -> Self {
    Self::Number(n.into())
  }

  pub fn new_bool(b: bool) -> Self {
    Self::Bool(b)
  }

  pub fn new_null() -> Self {
    Self::Null
  }

  pub fn new_array(arr: impl Into<Vec<Value>>) -> Self {
    Self::Array(arr.into())
  }

  pub fn new_object(obj: impl Into<HashMap<String, Value>>) -> Self {
    Self::Object(obj.into())
  }

  pub fn new_datetime(dt: impl Into<OffsetDateTime>) -> Self {
    Self::DateTime(dt.into())
  }

  pub fn new_binary(
    data: impl Into<Vec<u8>>,
    mime_type: impl Into<String>,
    filename: Option<impl Into<String>>,
  ) -> Self {
    Self::Binary(BinaryData {
      data: data.into(),
      mime_type: mime_type.into(),
      filename: filename.map(|f| f.into()),
      file_extension: None,
    })
  }

  pub fn new_binary_with_extension(
    data: impl Into<Vec<u8>>,
    mime_type: impl Into<String>,
    filename: Option<impl Into<String>>,
    file_extension: Option<impl Into<String>>,
  ) -> Self {
    Self::Binary(BinaryData {
      data: data.into(),
      mime_type: mime_type.into(),
      filename: filename.map(|f| f.into()),
      file_extension: file_extension.map(|f| f.into()),
    })
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinaryData {
  pub data: Vec<u8>,
  pub mime_type: String,
  pub filename: Option<String>,
  pub file_extension: Option<String>,
}

impl Value {
  pub fn is_truthy(&self) -> bool {
    match self {
      Value::Null => false,
      Value::Bool(b) => *b,
      Value::Number(n) => *n != 0.0 && !n.is_nan(),
      Value::String(s) => !s.is_empty(),
      Value::Array(arr) => !arr.is_empty(),
      Value::Object(obj) => !obj.is_empty(),
      Value::DateTime(_) => true,
      Value::Binary(_) => true,
    }
  }

  pub fn to_string_repr(&self) -> String {
    match self {
      Value::String(s) => s.clone(),
      Value::Number(n) => n.to_string(),
      Value::Bool(b) => b.to_string(),
      Value::Null => "null".to_string(),
      Value::DateTime(dt) => dt.to_rfc3339(),
      Value::Binary(b) => format!("[Binary: {}]", b.filename.as_ref().unwrap_or(&"unnamed".to_string())),
      Value::Array(arr) => format!("[Array: {} items]", arr.len()),
      Value::Object(obj) => format!("[Object: {} keys]", obj.len()),
    }
  }

  // 转换为 serde_json::Value 以便使用 jsonpath-rust
  pub fn to_json_value(&self) -> serde_json::Value {
    serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
  }

  // 从 serde_json::Value 转换回来
  pub fn from_json_value(json: serde_json::Value) -> Result<Self, serde_json::Error> {
    serde_json::from_value(json)
  }

  // 获取对象引用
  pub fn as_object(&self) -> Option<&HashMap<String, Value>> {
    match self {
      Value::Object(obj) => Some(obj),
      _ => None,
    }
  }
}

// 为 DateTime 实现额外的方法
pub trait DateTimeExt {
  fn plus(&self, duration: Duration) -> OffsetDateTime;
  fn minus(&self, duration: Duration) -> OffsetDateTime;
  fn to_format(&self, format: &str) -> String;
}

impl DateTimeExt for OffsetDateTime {
  fn plus(&self, duration: Duration) -> OffsetDateTime {
    *self + duration
  }

  fn minus(&self, duration: Duration) -> OffsetDateTime {
    *self - duration
  }

  fn to_format(&self, format: &str) -> String {
    self.format(format).to_string()
  }
}
