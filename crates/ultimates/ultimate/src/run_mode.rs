use serde::{
  de::{Unexpected, Visitor},
  Deserialize, Deserializer, Serialize,
};
use strum::AsRefStr;

#[derive(Debug, Clone, PartialEq, Serialize, AsRefStr)]
pub enum RunMode {
  DEV,
  TEST,
  DEMO,
  PROD,
}

impl RunMode {
  pub fn is_dev(&self) -> bool {
    self == &RunMode::DEV
  }

  pub fn is_test(&self) -> bool {
    self == &RunMode::TEST
  }

  pub fn is_demo(&self) -> bool {
    self == &RunMode::DEMO
  }

  pub fn is_prod(&self) -> bool {
    self == &RunMode::PROD
  }
}

impl<'de> Deserialize<'de> for RunMode {
  fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    static MSG: &str = "expect in ('DEV', 'TEST', 'DEMO', 'PROD').";

    struct StrToRunMode;

    impl<'d> Visitor<'d> for StrToRunMode {
      type Value = RunMode;

      fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(MSG)
      }

      fn visit_str<E>(self, v: &str) -> core::result::Result<Self::Value, E>
      where
        E: serde::de::Error,
      {
        if v.eq_ignore_ascii_case(RunMode::DEV.as_ref()) {
          Ok(RunMode::DEV)
        } else if v.eq_ignore_ascii_case(RunMode::TEST.as_ref()) {
          Ok(RunMode::TEST)
        } else if v.eq_ignore_ascii_case(RunMode::DEMO.as_ref()) {
          Ok(RunMode::DEMO)
        } else if v.eq_ignore_ascii_case(RunMode::PROD.as_ref()) {
          Ok(RunMode::PROD)
        } else {
          Err(serde::de::Error::invalid_value(Unexpected::Str(v), &MSG))
        }
      }
    }
    deserializer.deserialize_str(StrToRunMode)
  }
}
