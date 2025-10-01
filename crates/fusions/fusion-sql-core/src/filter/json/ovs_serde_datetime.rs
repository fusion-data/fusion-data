use std::fmt;

use chrono::{DateTime, FixedOffset};
use serde::{
  Deserialize, Serialize, Serializer,
  de::{MapAccess, Visitor},
  ser::SerializeMap,
};
use serde_json::Value;

use crate::filter::{Error, OpValDateTime, OpValsDateTime};

use super::OpValueToOpValType;

impl Serialize for OpValsDateTime {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut map = serializer.serialize_map(Some(self.0.len()))?;
    for opval in &self.0 {
      match opval {
        OpValDateTime::Eq(date_time) => map.serialize_entry("$eq", date_time)?,
        OpValDateTime::Not(date_time) => map.serialize_entry("$not", date_time)?,
        OpValDateTime::In(date_times) => map.serialize_entry("$in", date_times)?,
        OpValDateTime::NotIn(date_times) => map.serialize_entry("$notIn", date_times)?,
        OpValDateTime::Lt(date_time) => map.serialize_entry("$lt", date_time)?,
        OpValDateTime::Lte(date_time) => map.serialize_entry("$lte", date_time)?,
        OpValDateTime::Gt(date_time) => map.serialize_entry("$gt", date_time)?,
        OpValDateTime::Gte(date_time) => map.serialize_entry("$gte", date_time)?,
        OpValDateTime::Null(b) => map.serialize_entry("$null", b)?,
      }
    }
    map.end()
  }
}

impl<'de> Deserialize<'de> for OpValsDateTime {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    deserializer.deserialize_any(DateTimeOpValsVisitor)
  }
}

struct DateTimeOpValsVisitor;

impl<'de> Visitor<'de> for DateTimeOpValsVisitor {
  type Value = OpValsDateTime;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    write!(formatter, "DateTimeOpValsVisitor visitor not implemented for this type.")
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    let datetime = v.parse().map_err(serde::de::Error::custom)?;
    Ok(OpValDateTime::Eq(datetime).into())
  }

  fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    let datetime = v.parse().map_err(serde::de::Error::custom)?;
    Ok(OpValDateTime::Eq(datetime).into())
  }

  fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
  where
    M: MapAccess<'de>,
  {
    let mut opvals: Vec<OpValDateTime> = Vec::new();

    while let Some(k) = map.next_key::<String>()? {
      let value = map.next_value::<Value>()?;
      let opval = OpValDateTime::op_value_to_op_val_type(&k, value).map_err(serde::de::Error::custom)?;
      opvals.push(opval);
    }

    Ok(OpValsDateTime(opvals))
  }
}

impl OpValueToOpValType for OpValDateTime {
  fn op_value_to_op_val_type(op: &str, value: Value) -> Result<Self, Error>
  where
    Self: Sized,
  {
    let ov = match (op, value) {
      ("$eq", Value::String(s)) => OpValDateTime::Eq(parse_to_datetime(&s)?),
      ("$in", v) => OpValDateTime::In(into_datetimes(v)?),

      ("$not", Value::String(s)) => OpValDateTime::Not(parse_to_datetime(&s)?),
      ("$notIn", v) => OpValDateTime::NotIn(into_datetimes(v)?),

      ("$lt", Value::String(s)) => OpValDateTime::Lt(parse_to_datetime(&s)?),
      ("$lte", Value::String(s)) => OpValDateTime::Lte(parse_to_datetime(&s)?),

      ("$gt", Value::String(s)) => OpValDateTime::Gt(parse_to_datetime(&s)?),
      ("$gte", Value::String(s)) => OpValDateTime::Gte(parse_to_datetime(&s)?),

      ("$null", Value::Bool(b)) => OpValDateTime::Null(b),

      (_, v) => return Err(Error::JsonOpValNotSupported { operator: op.to_string(), value: v }),
    };
    Ok(ov)
  }
}

fn into_datetimes(value: Value) -> Result<Vec<DateTime<FixedOffset>>, Error> {
  let mut values = Vec::new();

  let Value::Array(array) = value else {
    return Err(Error::JsonValArrayWrongType { actual_value: value });
  };

  for item in array.into_iter() {
    if let Value::String(item) = item {
      values.push(parse_to_datetime(&item)?);
    } else {
      return Err(Error::JsonValArrayItemNotOfType { expected_type: "DateTime", actual_value: item });
    }
  }

  Ok(values)
}

fn parse_to_datetime(value: &str) -> Result<DateTime<FixedOffset>, Error> {
  value.parse().map_err(|_| Error::JsonValNotOfType("DateTime"))
}
