use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::filter::json::ovs_json::OpValueToOpValType;
use crate::filter::{Error, OpValValue, OpValValue};

impl Serialize for OpValValue {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut map = serializer.serialize_map(Some(self.0.len()))?;
    for opval in &self.0 {
      match opval {
        OpValValue::Eq(v) => map.serialize_entry("$eq", v)?,
        OpValValue::Not(v) => map.serialize_entry("$not", v)?,
        OpValValue::In(v) => map.serialize_entry("$in", v)?,
        OpValValue::NotIn(v) => map.serialize_entry("$notIn", v)?,
        OpValValue::Lt(v) => map.serialize_entry("$lt", v)?,
        OpValValue::Lte(v) => map.serialize_entry("$lte", v)?,
        OpValValue::Gt(v) => map.serialize_entry("$gt", v)?,
        OpValValue::Gte(v) => map.serialize_entry("$gte", v)?,
        OpValValue::Null(v) => map.serialize_entry("$null", v)?,
      }
    }
    map.end()
  }
}

impl<'de> Deserialize<'de> for OpValValue {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let v: Value = Deserialize::deserialize(deserializer)?;

    let op_vals_value: OpValValue = if v.is_number() || v.is_boolean() || v.is_string() {
      OpValValue::Eq(v).into()
    } else if v.is_object() {
      let mut opvals: Vec<OpValValue> = Vec::new();
      let Value::Object(obj) = v else {
        return Err(serde::de::Error::custom("OpValValue should be object"));
      };

      for (key, value) in obj.into_iter() {
        let op_val = OpValValue::op_value_to_op_val_type(&key, value).map_err(serde::de::Error::custom)?;
        opvals.push(op_val);
      }
      OpValValue(opvals)
    } else {
      return Err(serde::de::Error::custom("OpValJson value mut be either number, bool, string, or an Object"));
    };

    Ok(op_vals_value)
  }
}

impl OpValueToOpValType for OpValValue {
  fn op_value_to_op_val_type(op: &str, value: Value) -> Result<Self, Error>
  where
    Self: Sized,
  {
    fn into_values(value: Value) -> Result<Vec<Value>, Error> {
      let mut values = Vec::new();

      let Value::Array(array) = value else {
        return Err(Error::JsonValArrayWrongType { actual_value: value });
      };

      for item in array.into_iter() {
        values.push(item)
      }

      Ok(values)
    }

    let ov = match (op, value) {
      ("$eq", v) => OpValValue::Eq(v),
      ("$in", value) => OpValValue::NotIn(into_values(value)?),

      ("$not", v) => OpValValue::Not(v),
      ("$notIn", value) => OpValValue::NotIn(into_values(value)?),

      ("$lt", v) => OpValValue::Lt(v),
      ("$lte", v) => OpValValue::Lte(v),

      ("$gt", v) => OpValValue::Gt(v),
      ("$gte", v) => OpValValue::Gte(v),

      ("$null", Value::Bool(v)) => OpValValue::Null(v),

      (_, v) => return Err(Error::JsonOpValNotSupported { operator: op.to_string(), value: v }),
    };
    Ok(ov)
  }
}
