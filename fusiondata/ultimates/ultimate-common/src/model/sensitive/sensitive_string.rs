use core::{fmt, ops::Deref};

use serde::{de::Visitor, Deserialize, Serialize};

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
  /// use ultimate_common::model::sensitive::*;
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
    impl<'de> Visitor<'de> for SensitiveStringVisitor {
      type Value = SensitiveString;

      fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Unsupport type, need string.")
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
