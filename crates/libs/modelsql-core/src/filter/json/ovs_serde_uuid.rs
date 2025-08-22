use std::fmt;
use std::str::FromStr;

use serde::ser::SerializeMap;
use serde::{
  Deserialize,
  de::{MapAccess, Visitor},
};
use serde::{Serialize, Serializer};
use serde_json::Value;
use uuid::Uuid;

use crate::filter::json::OpValueToOpValType;
use crate::filter::{Error, OpValUuid, OpValsUuid};

impl Serialize for OpValsUuid {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut map = serializer.serialize_map(Some(self.0.len()))?;
    for opval in &self.0 {
      match opval {
        OpValUuid::Eq(uuid) => map.serialize_entry("$eq", uuid)?,
        OpValUuid::Not(uuid) => map.serialize_entry("$not", uuid)?,
        OpValUuid::In(uuids) => map.serialize_entry("$in", uuids)?,
        OpValUuid::NotIn(uuids) => map.serialize_entry("$notIn", uuids)?,
        OpValUuid::Lt(uuid) => map.serialize_entry("$lt", uuid)?,
        OpValUuid::Lte(uuid) => map.serialize_entry("$lte", uuid)?,
        OpValUuid::Gt(uuid) => map.serialize_entry("$gt", uuid)?,
        OpValUuid::Gte(uuid) => map.serialize_entry("$gte", uuid)?,
        OpValUuid::Null(b) => map.serialize_entry("$null", b)?,
        OpValUuid::ArrayContains(uuids) => map.serialize_entry("$@>", uuids)?,
      }
    }
    map.end()
  }
}

impl<'de> Deserialize<'de> for OpValsUuid {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    deserializer.deserialize_any(UuidOpValsVisitor)
  }
}

struct UuidOpValsVisitor;

impl<'de> Visitor<'de> for UuidOpValsVisitor {
  type Value = OpValsUuid;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    write!(formatter, "UuidOpValsVisitor visitor not implemented for this type.")
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    let id = Uuid::parse_str(v).map_err(serde::de::Error::custom)?;
    Ok(OpValUuid::Eq(id).into())
  }

  fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    let id = Uuid::parse_str(&v).map_err(serde::de::Error::custom)?;
    Ok(OpValUuid::Eq(id).into())
  }

  fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
  where
    M: MapAccess<'de>,
  {
    let mut opvals: Vec<OpValUuid> = Vec::new();

    while let Some(k) = map.next_key::<String>()? {
      let value = map.next_value::<Value>()?;
      let opval = OpValUuid::op_value_to_op_val_type(&k, value).map_err(serde::de::Error::custom)?;
      opvals.push(opval);
    }

    Ok(OpValsUuid(opvals))
  }
}

impl OpValueToOpValType for OpValUuid {
  fn op_value_to_op_val_type(op: &str, value: serde_json::Value) -> Result<Self, Error>
  where
    Self: Sized,
  {
    let ov = match (op, value) {
      ("$eq", Value::String(s)) => OpValUuid::Eq(parse_to_uuid(&s)?),
      ("$in", v) => OpValUuid::In(into_uuids(v)?),

      ("$not", Value::String(s)) => OpValUuid::Not(parse_to_uuid(&s)?),
      ("$notIn", v) => OpValUuid::NotIn(into_uuids(v)?),

      ("$lt", Value::String(s)) => OpValUuid::Lt(parse_to_uuid(&s)?),
      ("$lte", Value::String(s)) => OpValUuid::Lte(parse_to_uuid(&s)?),

      ("$gt", Value::String(s)) => OpValUuid::Gt(parse_to_uuid(&s)?),
      ("$gte", Value::String(s)) => OpValUuid::Gte(parse_to_uuid(&s)?),

      ("$null", Value::Bool(b)) => OpValUuid::Null(b),

      (_, v) => return Err(Error::JsonOpValNotSupported { operator: op.to_string(), value: v }),
    };
    Ok(ov)
  }
}

fn into_uuids(value: Value) -> Result<Vec<Uuid>, Error> {
  let mut values = Vec::new();

  let Value::Array(array) = value else {
    return Err(Error::JsonValArrayWrongType { actual_value: value });
  };

  for item in array.into_iter() {
    if let Value::String(item) = item {
      values.push(parse_to_uuid(&item)?);
    } else {
      return Err(Error::JsonValArrayItemNotOfType { expected_type: "Uuid", actual_value: item });
    }
  }

  Ok(values)
}

fn parse_to_uuid(value: &str) -> Result<Uuid, Error> {
  Uuid::from_str(value).map_err(|_| Error::JsonValNotOfType("Uuid"))
}
