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
  use std::collections::HashMap;

  use crate::configuration::{FusionConfig, model::KeyConf, util::load_config_with_env};

  #[test]
  fn test_config_load() {
    // 使用自定义环境变量源，确保测试隔离
    let mut test_env = HashMap::new();
    test_env.insert("FUSION__WEB__SERVER_ADDR".to_string(), "0.0.0.0:8000".to_string());
    test_env.insert(
      "FUSION__SECURITY__TOKEN__SECRET_KEY".to_string(),
      "8462b1ec9af827ebed13926f8f1e5409774fa1a21a1c8f726a4a34cf7dcabaf2".to_string(),
    );
    test_env.insert("FUSION__SECURITY__PWD__PWD_KEY".to_string(), "80c9a35c0f231219ca14c44fe10c728d".to_string());
    test_env.insert("FUSION__APP__NAME".to_string(), "fusion-test".to_string());

    let c = load_config_with_env(Some(test_env)).unwrap();
    println!("Config cache: {}", c.cache);
    let qc: FusionConfig = c.get("fusion").unwrap();

    assert_eq!(qc.security().pwd().pwd_key(), b"80c9a35c0f231219ca14c44fe10c728d");
    assert_eq!(qc.security().token().secret_key(), b"8462b1ec9af827ebed13926f8f1e5409774fa1a21a1c8f726a4a34cf7dcabaf2");

    // 由环境变量 FUSION__APP__NAME 提供
    assert_eq!(qc.app().name(), "fusion-test");

    // 不需要清理环境变量，因为使用了独立的环境变量源
  }
}
