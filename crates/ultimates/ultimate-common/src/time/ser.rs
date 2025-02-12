use std::time::Duration;

use chrono::FixedOffset;
use duration_str::HumanFormat;
use serde::Serializer;

pub fn serialize_fixed_offset<S>(offset: &FixedOffset, serializer: S) -> core::result::Result<S::Ok, S::Error>
where
  S: Serializer,
{
  let text = offset.to_string();
  serializer.serialize_str(&text)
}

pub fn serialize_duration<S>(duration: &Duration, serializer: S) -> core::result::Result<S::Ok, S::Error>
where
  S: Serializer,
{
  let text = duration.human_format();
  serializer.serialize_str(&text)
}
