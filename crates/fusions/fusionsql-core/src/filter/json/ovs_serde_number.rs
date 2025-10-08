use std::fmt;

use serde::{
  Deserialize, Deserializer, Serialize, Serializer,
  de::{MapAccess, Visitor},
  ser::SerializeMap,
};
use serde_json::Value;

use crate::filter::{Error, OpValFloat64, OpValInt32, OpValInt64, OpValFloat64, OpValInt32, OpValInt64};

use super::{as_f64, as_i32, as_i64, into_numbers, ovs_json::OpValueToOpValType};

// region:    --- OpValInt64

impl Serialize for OpValInt64 {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut map = serializer.serialize_map(Some(self.0.len()))?;
    for opval in &self.0 {
      match opval {
        OpValInt64::Eq(n) => map.serialize_entry("$eq", n)?,
        OpValInt64::Not(n) => map.serialize_entry("$not", n)?,
        OpValInt64::In(items) => map.serialize_entry("$in", items)?,
        OpValInt64::NotIn(items) => map.serialize_entry("$notIn", items)?,
        OpValInt64::Lt(n) => map.serialize_entry("$lt", n)?,
        OpValInt64::Lte(n) => map.serialize_entry("$lte", n)?,
        OpValInt64::Gt(n) => map.serialize_entry("$gt", n)?,
        OpValInt64::Gte(n) => map.serialize_entry("$gte", n)?,
        OpValInt64::Null(n) => map.serialize_entry("$null", n)?,
      }
    }
    map.end()
  }
}

impl Serialize for OpValInt32 {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut map = serializer.serialize_map(Some(self.0.len()))?;
    for opval in &self.0 {
      match opval {
        OpValInt32::Eq(n) => map.serialize_entry("$eq", n)?,
        OpValInt32::Not(n) => map.serialize_entry("$not", n)?,
        OpValInt32::In(items) => map.serialize_entry("$in", items)?,
        OpValInt32::NotIn(items) => map.serialize_entry("$notIn", items)?,
        OpValInt32::Lt(n) => map.serialize_entry("$lt", n)?,
        OpValInt32::Lte(n) => map.serialize_entry("$lte", n)?,
        OpValInt32::Gt(n) => map.serialize_entry("$gt", n)?,
        OpValInt32::Gte(n) => map.serialize_entry("$gte", n)?,
        OpValInt32::Null(n) => map.serialize_entry("$null", n)?,
      }
    }
    map.end()
  }
}

impl Serialize for OpValFloat64 {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut map = serializer.serialize_map(Some(self.0.len()))?;
    for opval in &self.0 {
      match opval {
        OpValFloat64::Eq(n) => map.serialize_entry("$eq", n)?,
        OpValFloat64::Not(n) => map.serialize_entry("$not", n)?,
        OpValFloat64::In(items) => map.serialize_entry("$in", items)?,
        OpValFloat64::NotIn(items) => map.serialize_entry("$notIn", items)?,
        OpValFloat64::Lt(n) => map.serialize_entry("$lt", n)?,
        OpValFloat64::Lte(n) => map.serialize_entry("$lte", n)?,
        OpValFloat64::Gt(n) => map.serialize_entry("$gt", n)?,
        OpValFloat64::Gte(n) => map.serialize_entry("$gte", n)?,
        OpValFloat64::Null(n) => map.serialize_entry("$null", n)?,
      }
    }
    map.end()
  }
}

impl<'de> Deserialize<'de> for OpValInt64 {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_any(Int64OpValVisitor)
  }
}

struct Int64OpValVisitor;

impl<'de> Visitor<'de> for Int64OpValVisitor {
  type Value = OpValInt64; // for deserialize

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    write!(formatter, "OpValInt64 visitor not implemented for this type.")
  }

  fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(OpValInt64::Eq(v).into())
  }

  fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(OpValInt64::Eq(v as i64).into())
  }

  fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
  where
    M: MapAccess<'de>,
  {
    let mut opvals: Vec<OpValInt64> = Vec::new();
    while let Some(k) = map.next_key::<String>()? {
      let value = map.next_value::<Value>()?;
      let opval = OpValInt64::op_value_to_op_val_type(&k, value).map_err(serde::de::Error::custom)?;
      opvals.push(opval)
    }

    Ok(OpValInt64(opvals))
  }
}
// endregion: --- OpValInt64

// region:    --- OpValInt32
impl<'de> Deserialize<'de> for OpValInt32 {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_any(Int32OpValVisitor)
  }
}

struct Int32OpValVisitor;

impl<'de> Visitor<'de> for Int32OpValVisitor {
  type Value = OpValInt32; // for deserialize

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    write!(formatter, "OpValInt32 visitor not implemented for this type.")
  }

  fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(OpValInt32::Eq(v).into())
  }

  fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
  where
    M: MapAccess<'de>,
  {
    let mut opvals: Vec<OpValInt32> = Vec::new();

    while let Some(k) = map.next_key::<String>()? {
      // Note: Important to always
      let value = map.next_value::<serde_json::Value>()?;
      let opval = OpValInt32::op_value_to_op_val_type(&k, value).map_err(serde::de::Error::custom)?;
      opvals.push(opval)
    }

    Ok(OpValInt32(opvals))
  }
}
// endregion: --- OpValInt64

// region:    --- OpValFloat64
impl<'de> Deserialize<'de> for OpValFloat64 {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_any(FloatOpValVisitor)
  }
}

struct FloatOpValVisitor;

impl<'de> Visitor<'de> for FloatOpValVisitor {
  type Value = OpValFloat64; // for deserialize

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    write!(formatter, "OpValFloat64 visitor not implemented for this type.")
  }

  fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(OpValFloat64::Eq(v).into())
  }

  fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(OpValFloat64::Eq(v as f64).into())
  }

  fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
  where
    M: MapAccess<'de>,
  {
    let mut opvals: Vec<OpValFloat64> = Vec::new();

    while let Some(k) = map.next_key::<String>()? {
      // Note: Important to always
      let value = map.next_value::<Value>()?;
      let opval = OpValFloat64::op_value_to_op_val_type(&k, value).map_err(serde::de::Error::custom)?;
      opvals.push(opval)
    }

    Ok(OpValFloat64(opvals))
  }
}
// endregion: --- OpValFloat64

// - `ov` e.g., `OpValInt64`
// - `asfn` e.g., `as_i64`
macro_rules! from_json_to_opval_num {
	($(($ov:ident, $asfn:expr)),+) => {
		$(
			/// match a the op_value
			impl OpValueToOpValType for $ov {

				fn op_value_to_op_val_type(op: &str, value: Value) -> Result<Self, Error>
				where
					Self: Sized,
				{

					// FIXME: Needs to do the In/Array patterns.
					let ov = match (op, value) {
						("$eq", Value::Number(num)) => $ov::Eq($asfn(num)?),
						("$in", value) => {
							let nums = into_numbers(value)?;
							let nums: Result<Vec<_>, Error> = nums.into_iter().map($asfn).collect();
							let nums = nums?;
							$ov::In(nums)
						},
						("$not", Value::Number(num)) => $ov::Not($asfn(num)?),
						("$notIn", value) => {
							let nums = into_numbers(value)?;
							let nums: Result<Vec<_>, Error> = nums.into_iter().map($asfn).collect();
							let nums = nums?;
							$ov::NotIn(nums)
						},

						("$lt", Value::Number(num)) => $ov::Lt($asfn(num)?),
						("$lte", Value::Number(num)) => $ov::Lte($asfn(num)?),

						("$gt", Value::Number(num)) => $ov::Gt($asfn(num)?),
						("$gte", Value::Number(num)) => $ov::Gte($asfn(num)?),

						("$null", Value::Number(num)) => $ov::Gte($asfn(num)?),

						(_, value) => return Err(Error::JsonOpValNotSupported{
									operator: op.to_string(),
									value,
								}),
					};

					Ok(ov)
				}
			}
		)+
	};
}

from_json_to_opval_num!((OpValInt64, as_i64), (OpValInt32, as_i32), (OpValFloat64, as_f64));
