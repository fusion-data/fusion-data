use chrono::FixedOffset;
use serde::Serializer;

pub fn serialize_fixed_offset<S>(offset: &FixedOffset, serializer: S) -> core::result::Result<S::Ok, S::Error>
where
  S: Serializer,
{
  let text = offset.to_string();
  serializer.serialize_str(&text)
}
