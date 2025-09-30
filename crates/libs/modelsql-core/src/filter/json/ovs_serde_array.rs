use serde::{
  Deserialize, Serialize,
  de::{MapAccess, Visitor},
  ser::SerializeMap,
};
use serde_json::Value;

use crate::filter::{
  Error, OpValArrayFloat64, OpValArrayInt32, OpValArrayInt64, OpValArrayString, OpValsArrayFloat64, OpValsArrayInt32,
  OpValsArrayInt64, OpValsArrayString,
};

use super::{as_f64, as_i32, as_i64, into_numbers, into_strings, ovs_json::OpValueToOpValType};
impl Serialize for OpValsArrayString {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let mut map = serializer.serialize_map(Some(self.0.len()))?;
    for opval in &self.0 {
      match opval {
        OpValArrayString::Eq(items) => map.serialize_entry("$eq", items)?,
        OpValArrayString::Not(items) => map.serialize_entry("$not", items)?,
        OpValArrayString::Contains(items) => map.serialize_entry("$contains", items)?,
        OpValArrayString::Contained(items) => map.serialize_entry("$contained", items)?,
      }
    }
    map.end()
  }
}

struct ArrayStringOpValsVisitor;
impl<'de> Visitor<'de> for ArrayStringOpValsVisitor {
  type Value = OpValsArrayString;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    formatter.write_str("OpValsArrayString visitor not implemented for this type.")
  }

  fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
  where
    A: serde::de::SeqAccess<'de>,
  {
    let mut opvals = Vec::new();
    while let Some(opval) = seq.next_element()? {
      opvals.push(opval);
    }
    Ok(OpValsArrayString(vec![OpValArrayString::Eq(opvals)]))
  }

  fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
  where
    A: serde::de::MapAccess<'de>,
  {
    let mut opvals: Vec<OpValArrayString> = Vec::new();
    while let Some(k) = map.next_key::<String>()? {
      let value = map.next_value::<serde_json::Value>()?;
      let opval = OpValArrayString::op_value_to_op_val_type(&k, value).map_err(serde::de::Error::custom)?;
      opvals.push(opval);
    }
    Ok(OpValsArrayString(opvals))
  }
}
impl<'de> Deserialize<'de> for OpValsArrayString {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    deserializer.deserialize_any(ArrayStringOpValsVisitor)
  }
}
impl OpValueToOpValType for OpValArrayString {
  fn op_value_to_op_val_type(op: &str, value: Value) -> crate::filter::Result<Self>
  where
    Self: Sized,
  {
    let items = into_strings(value)?;
    match op {
      "$eq" => Ok(OpValArrayString::Eq(items)),
      "$not" => Ok(OpValArrayString::Not(items)),
      "$contains" => Ok(OpValArrayString::Contains(items)),
      "$contained" => Ok(OpValArrayString::Contained(items)),
      _ => Err(Error::JsonValNotOfType("OpValArrayString")),
    }
  }
}

impl Serialize for OpValsArrayInt32 {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let mut map = serializer.serialize_map(Some(self.0.len()))?;
    for opval in &self.0 {
      match opval {
        OpValArrayInt32::Eq(items) => map.serialize_entry("$eq", items)?,
        OpValArrayInt32::Not(items) => map.serialize_entry("$not", items)?,
        OpValArrayInt32::Contains(items) => map.serialize_entry("$contains", items)?,
        OpValArrayInt32::Contained(items) => map.serialize_entry("$contained", items)?,
      }
    }
    map.end()
  }
}

impl Serialize for OpValsArrayInt64 {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let mut map = serializer.serialize_map(Some(self.0.len()))?;
    for opval in &self.0 {
      match opval {
        OpValArrayInt64::Eq(items) => map.serialize_entry("$eq", items)?,
        OpValArrayInt64::Not(items) => map.serialize_entry("$not", items)?,
        OpValArrayInt64::Contains(items) => map.serialize_entry("$contains", items)?,
        OpValArrayInt64::Contained(items) => map.serialize_entry("$contained", items)?,
      }
    }
    map.end()
  }
}

impl Serialize for OpValsArrayFloat64 {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let mut map = serializer.serialize_map(Some(self.0.len()))?;
    for opval in &self.0 {
      match opval {
        OpValArrayFloat64::Eq(items) => map.serialize_entry("$eq", items)?,
        OpValArrayFloat64::Not(items) => map.serialize_entry("$not", items)?,
        OpValArrayFloat64::Contains(items) => map.serialize_entry("$contains", items)?,
        OpValArrayFloat64::Contained(items) => map.serialize_entry("$contained", items)?,
      }
    }
    map.end()
  }
}

struct ArrayInt32OpValsVisitor;
impl<'de> Visitor<'de> for ArrayInt32OpValsVisitor {
  type Value = OpValsArrayInt32;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    formatter.write_str("OpValsArrayInt32 visitor not implemented for this type.")
  }

  fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
  where
    A: serde::de::SeqAccess<'de>,
  {
    let mut opvals = Vec::new();
    while let Some(opval) = seq.next_element()? {
      opvals.push(opval);
    }
    Ok(OpValsArrayInt32(vec![OpValArrayInt32::Eq(opvals)]))
  }

  fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
  where
    A: MapAccess<'de>,
  {
    let mut opvals: Vec<OpValArrayInt32> = Vec::new();
    while let Some(k) = map.next_key::<String>()? {
      let value = map.next_value::<serde_json::Value>()?;
      let opval = OpValArrayInt32::op_value_to_op_val_type(&k, value).map_err(serde::de::Error::custom)?;
      opvals.push(opval);
    }
    Ok(OpValsArrayInt32(opvals))
  }
}
impl<'de> Deserialize<'de> for OpValsArrayInt32 {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    deserializer.deserialize_any(ArrayInt32OpValsVisitor)
  }
}

struct ArrayInt64OpValsVisitor;
impl<'de> Visitor<'de> for ArrayInt64OpValsVisitor {
  type Value = OpValsArrayInt64;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    formatter.write_str("OpValsArrayInt64 visitor not implemented for this type.")
  }

  fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
  where
    A: serde::de::SeqAccess<'de>,
  {
    let mut opvals = Vec::new();
    while let Some(opval) = seq.next_element()? {
      opvals.push(opval);
    }
    Ok(OpValsArrayInt64(vec![OpValArrayInt64::Eq(opvals)]))
  }

  fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
  where
    A: MapAccess<'de>,
  {
    let mut opvals: Vec<OpValArrayInt64> = Vec::new();
    while let Some(k) = map.next_key::<String>()? {
      let value = map.next_value::<serde_json::Value>()?;
      let opval = OpValArrayInt64::op_value_to_op_val_type(&k, value).map_err(serde::de::Error::custom)?;
      opvals.push(opval);
    }
    Ok(OpValsArrayInt64(opvals))
  }
}
impl<'de> Deserialize<'de> for OpValsArrayInt64 {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    deserializer.deserialize_any(ArrayInt64OpValsVisitor)
  }
}

struct ArrayFloat64OpValsVisitor;
impl<'de> Visitor<'de> for ArrayFloat64OpValsVisitor {
  type Value = OpValsArrayFloat64;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    formatter.write_str("OpValsArrayFloat64 visitor not implemented for this type.")
  }

  fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
  where
    A: serde::de::SeqAccess<'de>,
  {
    let mut opvals = Vec::new();
    while let Some(opval) = seq.next_element()? {
      opvals.push(opval);
    }
    Ok(OpValsArrayFloat64(vec![OpValArrayFloat64::Eq(opvals)]))
  }

  fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
  where
    A: MapAccess<'de>,
  {
    let mut opvals: Vec<OpValArrayFloat64> = Vec::new();
    while let Some(k) = map.next_key::<String>()? {
      let value = map.next_value::<serde_json::Value>()?;
      let opval = OpValArrayFloat64::op_value_to_op_val_type(&k, value).map_err(serde::de::Error::custom)?;
      opvals.push(opval);
    }
    Ok(OpValsArrayFloat64(opvals))
  }
}
impl<'de> Deserialize<'de> for OpValsArrayFloat64 {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    deserializer.deserialize_any(ArrayFloat64OpValsVisitor)
  }
}

macro_rules! from_json_to_opval_array_num {
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
						("$eq", value) => {
							let nums = into_numbers(value)?;
							let nums: Result<Vec<_>, Error> = nums.into_iter().map($asfn).collect();
							let nums = nums?;
							$ov::Eq(nums)
						},
						("$not", value) => {
							let nums = into_numbers(value)?;
							let nums: Result<Vec<_>, Error> = nums.into_iter().map($asfn).collect();
							let nums = nums?;
							$ov::Not(nums)
						},
            ("$contains", value) => {
							let nums = into_numbers(value)?;
							let nums: Result<Vec<_>, Error> = nums.into_iter().map($asfn).collect();
							let nums = nums?;
							$ov::Contains(nums)
						},
						("$contained", value) => {
							let nums = into_numbers(value)?;
							let nums: Result<Vec<_>, Error> = nums.into_iter().map($asfn).collect();
							let nums = nums?;
							$ov::Contained(nums)
						},
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

from_json_to_opval_array_num!((OpValArrayInt32, as_i32), (OpValArrayInt64, as_i64), (OpValArrayFloat64, as_f64));
