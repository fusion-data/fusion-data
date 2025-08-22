use serde::de::{Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum ApiValidEffect {
  Allow,
  Deny,
}
impl ApiValidEffect {
  pub fn is_deny(&self) -> bool {
    match self {
      ApiValidEffect::Allow => false,
      ApiValidEffect::Deny => true,
    }
  }

  pub fn is_allow(&self) -> bool {
    match self {
      ApiValidEffect::Allow => true,
      ApiValidEffect::Deny => false,
    }
  }
}

impl<'de> Deserialize<'de> for ApiValidEffect {
  fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_str(StrToApiValidEffect)
  }
}

struct StrToApiValidEffect;
impl Visitor<'_> for StrToApiValidEffect {
  type Value = ApiValidEffect;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    formatter.write_str("expect 'allow' or 'deny'.")
  }

  fn visit_str<E>(self, v: &str) -> core::result::Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    if v.eq_ignore_ascii_case("allow") {
      Ok(ApiValidEffect::Allow)
    } else if v.eq_ignore_ascii_case("deny") {
      Ok(ApiValidEffect::Deny)
    } else {
      Err(serde::de::Error::invalid_value(Unexpected::Str(v), &"expect 'allow' or 'deny'."))
    }
  }
}

#[cfg(test)]
mod tests {
  use ultimate_common::env::set_env;

  use crate::configuration::{UltimateConfig, model::KeyConf, util::load_config};

  #[test]
  fn test_config_load() {
    // 两个下划线作为层级分隔符
    set_env("ULTIMATE__WEB__SERVER_ADDR", "0.0.0.0:8000").unwrap();

    set_env(
      "ULTIMATE__SECURITY__TOKEN__SECRET_KEY",
      "8462b1ec9af827ebed13926f8f1e5409774fa1a21a1c8f726a4a34cf7dcabaf2",
    )
    .unwrap();
    set_env("ULTIMATE__SECURITY__PWD__PWD_KEY", "80c9a35c0f231219ca14c44fe10c728d").unwrap();
    set_env("ULTIMATE__APP__NAME", "ultimate").unwrap();

    let c = load_config().unwrap();
    let qc: UltimateConfig = c.get("ultimate").unwrap();

    assert_eq!(qc.security().pwd().pwd_key(), b"80c9a35c0f231219ca14c44fe10c728d");
    assert_eq!(qc.security().token().secret_key(), b"8462b1ec9af827ebed13926f8f1e5409774fa1a21a1c8f726a4a34cf7dcabaf2");

    // 由默认配置文件提供
    assert_eq!(qc.app().name(), "ultimate");
  }
}
