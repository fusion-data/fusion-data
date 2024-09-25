use core::{fmt, ops::Deref};
use std::borrow::Cow;

use regex::Regex;
use serde::{de::Visitor, Deserialize, Serialize};

use super::{AsUnderlying, ToSensitive};

#[derive(Clone)]
pub struct UriString(String);

impl UriString {
  pub fn new(s: impl Into<String>) -> Self {
    Self(s.into())
  }
}

impl fmt::Display for UriString {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(&self.to_sensitive())
  }
}

impl fmt::Debug for UriString {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("UriString").field(&self.to_string()).finish()
  }
}

impl Deref for UriString {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl AsRef<str> for UriString {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

impl ToSensitive for UriString {
  fn to_sensitive(&self) -> String {
    replace_uri_username_password(self.deref()).into_owned()
  }
}

impl AsUnderlying for UriString {
  fn as_underlying(&self) -> &str {
    &self.0
  }
}

impl Serialize for UriString {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let s = replace_uri_username_password(self.deref());
    serializer.serialize_str(s.as_ref())
  }
}

/// 替换 <username>:<password> 部分
fn replace_uri_username_password(v: &str) -> Cow<'_, str> {
  let r: Regex = Regex::new(r#"://(.+):(.+)@"#).unwrap();
  r.replace(v, "://<username>:<password>@")
}

impl<'de> Deserialize<'de> for UriString {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    struct UriStringVisitor;
    impl<'de> Visitor<'de> for UriStringVisitor {
      type Value = UriString;

      fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Unsupport type, need string.")
      }

      fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
      where
        E: serde::de::Error,
      {
        Ok(UriString(v))
      }

      fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
      where
        E: serde::de::Error,
      {
        Ok(UriString(v.to_string()))
      }
    }
    deserializer.deserialize_string(UriStringVisitor)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_url_string() {
    let url_string = UriString::new("postgres://abcdefg:2023.ultimate@localhost:5432/ultimate");
    let text = serde_json::to_string(&url_string).unwrap();
    assert_eq!(text, "\"postgres://<username>:<password>@localhost:5432/ultimate\"");

    let url_string = UriString::new("postgres://localhost:5432/ultimate");
    let text = serde_json::to_string(&url_string).unwrap();
    assert_eq!(text, "\"postgres://localhost:5432/ultimate\"");
  }
}
