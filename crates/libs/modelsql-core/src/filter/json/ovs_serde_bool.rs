use std::fmt;

use serde::{
  Deserialize, Deserializer, Serialize, Serializer,
  de::{MapAccess, Visitor},
  ser::SerializeMap,
};
use serde_json::Value;

use crate::filter::{Error, OpValBool, OpValsBool};

use super::ovs_json::OpValueToOpValType;

impl OpValueToOpValType for OpValBool {
  fn op_value_to_op_val_type(op: &str, value: Value) -> Result<Self, Error>
  where
    Self: Sized,
  {
    let ov = match (op, value) {
      ("$eq", Value::Bool(v)) => OpValBool::Eq(v),
      ("$not", Value::Bool(v)) => OpValBool::Not(v),
      ("$null", Value::Bool(v)) => OpValBool::Not(v),
      (_, value) => return Err(Error::JsonOpValNotSupported { operator: op.to_string(), value }),
    };

    Ok(ov)
  }
}

impl OpValBool {
  pub fn op_val_type_to_entry(&self) -> (&str, &bool) {
    match self {
      OpValBool::Eq(v) => ("$eq", v),
      OpValBool::Not(v) => ("$not", v),
      OpValBool::Null(v) => ("$null", v),
    }
  }
}

impl Serialize for OpValsBool {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut seq = serializer.serialize_map(Some(self.0.len()))?;
    for opval in &self.0 {
      let (key, value) = opval.op_val_type_to_entry();
      seq.serialize_entry(key, value)?;
    }
    seq.end()
  }
}

impl<'de> Deserialize<'de> for OpValsBool {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_any(BoolOpValsVisitor)
  }
}

struct BoolOpValsVisitor;

impl<'de> Visitor<'de> for BoolOpValsVisitor {
  type Value = OpValsBool; // for deserialize

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    write!(formatter, "BoolOpValsVisitor visitor not implemented for this type.")
  }

  fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(OpValBool::Eq(v).into())
  }

  fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
  where
    M: MapAccess<'de>,
  {
    let mut opvals: Vec<OpValBool> = Vec::new();

    while let Some(k) = map.next_key::<String>()? {
      // Note: Important to always call next_value
      let value = map.next_value::<Value>()?;
      let opval = OpValBool::op_value_to_op_val_type(&k, value).map_err(serde::de::Error::custom)?;
      opvals.push(opval)
    }

    Ok(OpValsBool(opvals))
  }
}
