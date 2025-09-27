use serde::de::DeserializeOwned;

use crate::{types::JsonValue, workflow::ValidationError};

pub fn take_value_from_map<T>(map: &mut serde_json::Map<String, JsonValue>, key: &str) -> Result<T, ValidationError>
where
  T: DeserializeOwned,
{
  let value = map.remove(key).ok_or_else(|| ValidationError::required_field_missing(key))?;
  serde_json::from_value(value).map_err(ValidationError::from)
}
