use core::{fmt, ops::Deref};

use serde::{Deserialize, Serialize, de::Visitor};

use crate::string;

use super::{AsUnderlying, ToSensitive};

/// 当使用 serde 序列化时可进行脱敏
#[derive(Clone)]
pub struct SensitiveString {
  underlying: String,
  sensitive_len: usize,
  c: char,
}

impl SensitiveString {
  /// 构造一个 SensitiveString
  ///
  /// # Arguments
  ///
  /// * `underlying` - 原始字符串
  /// * `sensitive_len` - 要脱敏的字符长度
  /// * `c` - 用于脱敏替换的字符
  ///
  /// # Examples
  ///
  /// ```rust
  /// use fusion_common::model::sensitive::*;
  /// let ss = SensitiveString::new("13883712048", 4, '*');
  /// let text = serde_json::to_string(&ss).unwrap();
  /// assert_eq!(text, "\"138****2048\"");
  ///
  /// let ss = SensitiveString::new("abc", 4, '*');
  /// let text = serde_json::to_string(&ss).unwrap();
  /// assert_eq!(text, "\"***\"");
  ///
  /// let ss = SensitiveString::new("abc", 3, '*');
  /// let text = serde_json::to_string(&ss).unwrap();
  /// assert_eq!(text, "\"***\"");
  ///
  /// let ss = SensitiveString::new("abcdefg", 3, '*');
  /// let text = serde_json::to_string(&ss).unwrap();
  /// assert_eq!(text, "\"ab***fg\"");
  ///
  /// let ss = SensitiveString::new("abcdefg", 4, '*');
  /// let text = serde_json::to_string(&ss).unwrap();
  /// assert_eq!(text, "\"a****fg\"");
  /// ```
  pub fn new(underlying: impl Into<String>, sensitive_len: usize, c: char) -> Self {
    Self { underlying: underlying.into(), sensitive_len, c }
  }

  pub fn sensitive_len(&self) -> usize {
    self.sensitive_len
  }

  pub fn c(&self) -> char {
    self.c
  }

  pub fn as_str(&self) -> &str {
    &self.underlying
  }
}

impl fmt::Debug for SensitiveString {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("SensitiveString").field(&self.to_sensitive()).finish()
  }
}

impl fmt::Display for SensitiveString {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(&self.to_sensitive())
  }
}

impl From<String> for SensitiveString {
  fn from(underlying: String) -> Self {
    let sensitive_len = underlying.len() / 2;
    Self::new(underlying, sensitive_len, '*')
  }
}

impl From<&str> for SensitiveString {
  fn from(underlying: &str) -> Self {
    Self::from(underlying.to_string())
  }
}

impl Deref for SensitiveString {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    &self.underlying
  }
}

impl AsRef<str> for SensitiveString {
  fn as_ref(&self) -> &str {
    &self.underlying
  }
}

impl ToSensitive for SensitiveString {
  fn to_sensitive(&self) -> String {
    let v = self.deref();
    if v.len() < self.sensitive_len() {
      return string::repeat_char(self.c(), v.len());
    }

    let sensitive_start = v.len() / 2 - self.sensitive_len() / 2;
    let mut s = String::with_capacity(v.len());
    s.push_str(&v[0..sensitive_start]);
    for _ in 0..self.sensitive_len() {
      s.push(self.c);
    }
    s.push_str(&v[(self.sensitive_len() + sensitive_start)..v.len()]);
    s
  }
}

impl AsUnderlying for SensitiveString {
  fn as_underlying(&self) -> &str {
    &self.underlying
  }
}

impl Serialize for SensitiveString {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(&self.to_sensitive())
  }
}

impl<'de> Deserialize<'de> for SensitiveString {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    struct SensitiveStringVisitor;
    impl Visitor<'_> for SensitiveStringVisitor {
      type Value = SensitiveString;

      fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Unsupported type, need string.")
      }

      fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
      where
        E: serde::de::Error,
      {
        let sensitive_len = v.len() / 2;
        Ok(SensitiveString::new(v, sensitive_len, '*'))
      }

      fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
      where
        E: serde::de::Error,
      {
        let sensitive_len = v.len() / 2;
        Ok(SensitiveString::new(v, sensitive_len, '*'))
      }
    }
    deserializer.deserialize_string(SensitiveStringVisitor)
  }
}

#[cfg(feature = "with-db")]
mod _db {
  use sea_query::Value;
  use sqlx::{Database, Decode, Type};

  use super::SensitiveString;

  impl<DB: Database> Type<DB> for SensitiveString
  where
    String: Type<DB>,
  {
    fn type_info() -> <DB as Database>::TypeInfo {
      <String as Type<DB>>::type_info()
    }
  }

  // `'r` is the lifetime of the `Row` being decoded
  impl<'r, DB: Database> Decode<'r, DB> for SensitiveString
  where
    // we want to delegate some of the work to string decoding so let's make sure strings
    // are supported by the database
    &'r str: Decode<'r, DB>,
  {
    fn decode(
      value: <DB as Database>::ValueRef<'r>,
    ) -> Result<SensitiveString, Box<dyn core::error::Error + 'static + Send + Sync>> {
      // the interface of ValueRef is largely unstable at the moment
      // so this is not directly implementable

      // however, you can delegate to a type that matches the format of the type you want
      // to decode (such as a UTF-8 string)

      let value = <&str as Decode<DB>>::decode(value)?;

      Ok(SensitiveString::new(value.to_string(), value.len() / 2, '*'))
    }
  }

  impl From<SensitiveString> for sea_query::Value {
    fn from(s: SensitiveString) -> Self {
      Value::String(Some(Box::new(s.underlying)))
    }
  }

  impl sea_query::Nullable for SensitiveString {
    fn null() -> sea_query::Value {
      sea_query::Value::String(None)
    }
  }
}
