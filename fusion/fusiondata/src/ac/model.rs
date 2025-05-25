use serde::{
  Deserialize, Deserializer, Serialize,
  de::{Unexpected, Visitor},
};
use strum::AsRefStr;

/// 定义Action中的操作权限是否允许执行。
#[derive(Debug, Default, Serialize, AsRefStr)]
pub enum Effect {
  /// 允许执行
  #[default]
  Allow,

  /// 禁止执行
  Deny,
}
impl<'de> Deserialize<'de> for Effect {
  fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    static MSG: &str = "The 'effect' field expect in ('allow', 'deny').";

    struct StrToEffect;

    impl Visitor<'_> for StrToEffect {
      type Value = Effect;

      fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(MSG)
      }

      fn visit_str<E>(self, v: &str) -> core::result::Result<Self::Value, E>
      where
        E: serde::de::Error,
      {
        if v.eq_ignore_ascii_case(Effect::Allow.as_ref()) {
          Ok(Effect::Allow)
        } else if v.eq_ignore_ascii_case(Effect::Deny.as_ref()) {
          Ok(Effect::Deny)
        } else {
          Err(serde::de::Error::invalid_value(Unexpected::Str(v), &MSG))
        }
      }
    }
    deserializer.deserialize_str(StrToEffect)
  }
}
