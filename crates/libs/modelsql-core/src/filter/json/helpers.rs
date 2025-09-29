use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value, json};
use uuid::Uuid;

use crate::filter::Error;

pub fn as_i64(num: Number) -> Result<i64, Error> {
  num.as_i64().ok_or(Error::JsonValNotOfType("i64"))
}

pub fn as_i32(num: Number) -> Result<i32, Error> {
  num.as_i64().map(|n| n as i32).ok_or(Error::JsonValNotOfType("i32"))
}

pub fn as_f64(num: Number) -> Result<f64, Error> {
  num.as_f64().ok_or(Error::JsonValNotOfType("f64"))
}

pub fn as_string(value: Value) -> Result<String, Error> {
  if let Value::String(item) = value { Ok(item) } else { Err(Error::JsonValNotOfType("String")) }
}

pub fn as_uuid(value: Value) -> Result<Uuid, Error> {
  if let Value::String(item) = value {
    Ok(Uuid::parse_str(item.as_str()).map_err(|_e| Error::JsonValNotOfType("Uuid"))?)
  } else {
    Err(Error::JsonValNotOfType("Uuid"))
  }
}

pub fn as_datetime(value: Value) -> Result<DateTime<FixedOffset>, Error> {
  serde_json::from_value(value).map_err(|_e| Error::JsonValNotOfType("DateTime<FixedOffset>"))
}

pub fn into_numbers(value: Value) -> Result<Vec<Number>, Error> {
  let mut values = Vec::new();

  let Value::Array(array) = value else {
    return Err(Error::JsonValArrayWrongType { actual_value: value });
  };

  for item in array.into_iter() {
    if let Value::Number(item) = item {
      values.push(item);
    } else {
      return Err(Error::JsonValArrayItemNotOfType { expected_type: "Number", actual_value: item });
    }
  }

  Ok(values)
}

pub fn into_strings(value: Value) -> Result<Vec<String>, Error> {
  let mut values = Vec::new();

  let Value::Array(array) = value else {
    return Err(Error::JsonValArrayWrongType { actual_value: value });
  };

  for item in array.into_iter() {
    if let Value::String(item) = item {
      values.push(item);
    } else {
      return Err(Error::JsonValArrayItemNotOfType { expected_type: "String", actual_value: item });
    }
  }

  Ok(values)
}

pub fn into_uuids(value: Value) -> Result<Vec<Uuid>, Error> {
  let mut values = Vec::new();

  let Value::Array(array) = value else {
    return Err(Error::JsonValArrayWrongType { actual_value: value });
  };

  for value in array.into_iter() {
    if let Value::String(item) = value {
      values.push(
        Uuid::parse_str(item.as_str())
          .map_err(|_e| Error::JsonValArrayItemNotOfType { expected_type: "Uuid", actual_value: json!(item) })?,
      );
    } else {
      return Err(Error::JsonValArrayItemNotOfType { expected_type: "Uuid", actual_value: value });
    }
  }

  Ok(values)
}

pub fn into_datetimes(original: Value) -> Result<Vec<DateTime<FixedOffset>>, Error> {
  let mut values = Vec::new();

  let Value::Array(array) = original else {
    return Err(Error::JsonValArrayWrongType { actual_value: original });
  };

  for value in array.into_iter() {
    let item = serde_json::from_value(value.clone())
      .map_err(|_e| Error::JsonValArrayItemNotOfType { expected_type: "DateTime<FixedOffset>", actual_value: value })?;
    values.push(item);
  }

  Ok(values)
}
