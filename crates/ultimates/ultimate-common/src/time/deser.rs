use std::str::FromStr;

use chrono::FixedOffset;
use serde::{Deserialize, Deserializer};

use super::local_offset;

pub fn deserialize_fixed_offset<'de, D>(deserializer: D) -> core::result::Result<FixedOffset, D::Error>
where
  D: Deserializer<'de>,
{
  let s: String = Deserialize::deserialize(deserializer)?;
  if s.is_empty() {
    Ok(*local_offset())
  } else {
    FixedOffset::from_str(&s)
      .map_err(|e| serde::de::Error::custom(format!("Invalid time offset error: {}, value is '{}'", e, s)))
  }
}
