use std::fmt;

use serde::{
  Deserialize, Deserializer, Serialize, Serializer,
  de::{MapAccess, Visitor},
  ser::SerializeMap,
};
use serde_json::Value;

use crate::filter::{Error, OpValString, OpValString};

use super::ovs_json::OpValueToOpValType;

impl Serialize for OpValString {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut map = serializer.serialize_map(Some(self.0.len()))?;
    for opval in &self.0 {
      match opval {
        OpValString::Eq(s) => map.serialize_entry("$eq", s)?,
        OpValString::Not(s) => map.serialize_entry("$not", s)?,
        OpValString::In(s) => map.serialize_entry("$in", s)?,
        OpValString::NotIn(s) => map.serialize_entry("$notIn", s)?,
        OpValString::Lt(s) => map.serialize_entry("$lt", s)?,
        OpValString::Lte(s) => map.serialize_entry("$lte", s)?,
        OpValString::Gt(s) => map.serialize_entry("$gt", s)?,
        OpValString::Gte(s) => map.serialize_entry("$gte", s)?,
        OpValString::Contains(s) => map.serialize_entry("$contains", s)?,
        OpValString::NotContains(s) => map.serialize_entry("$notContains", s)?,
        OpValString::ContainsAny(s) => map.serialize_entry("$containsAny", s)?,
        OpValString::NotContainsAny(s) => map.serialize_entry("$notContainsAny", s)?,
        OpValString::ContainsAll(s) => map.serialize_entry("$containsAll", s)?,
        OpValString::NotContainsAll(s) => map.serialize_entry("$notContainsAll", s)?,
        OpValString::StartsWith(s) => map.serialize_entry("$startsWith", s)?,
        OpValString::StartsWithAny(s) => map.serialize_entry("$startsWithAny", s)?,
        OpValString::NotStartsWith(s) => map.serialize_entry("$notStartsWith", s)?,
        OpValString::NotStartsWithAny(s) => map.serialize_entry("$notStartsWithAny", s)?,
        OpValString::EndsWith(s) => map.serialize_entry("$endsWith", s)?,
        OpValString::EndsWithAny(s) => map.serialize_entry("$endsWithAny", s)?,
        OpValString::NotEndsWith(s) => map.serialize_entry("$notEndsWith", s)?,
        OpValString::NotEndsWithAny(s) => map.serialize_entry("$notEndsWithAny", s)?,
        OpValString::Empty(s) => map.serialize_entry("$empty", s)?,
        OpValString::Null(s) => map.serialize_entry("$null", s)?,
        OpValString::ContainsCi(s) => map.serialize_entry("$containsCi", s)?,
        OpValString::NotContainsCi(s) => map.serialize_entry("$notContainsCi", s)?,
        OpValString::StartsWithCi(s) => map.serialize_entry("$startsWithCi", s)?,
        OpValString::NotStartsWithCi(s) => map.serialize_entry("$notStartsWithCi", s)?,
        OpValString::EndsWithCi(s) => map.serialize_entry("$endsWithCi", s)?,
        OpValString::NotEndsWithCi(s) => map.serialize_entry("$notEndsWithCi", s)?,
        OpValString::Ilike(s) => map.serialize_entry("$ilike", s)?,
      }
    }
    map.end()
  }
}

impl<'de> Deserialize<'de> for OpValString {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_any(StringOpValVisitor)
  }
}

struct StringOpValVisitor;

impl<'de> Visitor<'de> for StringOpValVisitor {
  type Value = OpValString; // for deserialize

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    write!(formatter, "StringOpValVisitor visitor not implemented for this type.")
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(OpValString::Eq(v.to_string()).into())
  }

  fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(OpValString::Eq(v).into())
  }

  fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
  where
    M: MapAccess<'de>,
  {
    let mut opvals: Vec<OpValString> = Vec::new();

    // Note: If use next_key::<&str>, error "invalid type: string \"$contains\", expected a borrowed string"
    //       so using String for now.
    while let Some(k) = map.next_key::<String>()? {
      // Note: Important to always call next_value
      let value = map.next_value::<Value>()?;
      let opval = OpValString::op_value_to_op_val_type(&k, value).map_err(serde::de::Error::custom)?;
      opvals.push(opval)
    }

    Ok(OpValString(opvals))
  }
}

impl OpValueToOpValType for OpValString {
  fn op_value_to_op_val_type(op: &str, value: Value) -> Result<Self, Error>
  where
    Self: Sized,
  {
    fn into_strings(value: Value) -> Result<Vec<String>, Error> {
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

    let ov = match (op, value) {
      ("$eq", Value::String(string_v)) => OpValString::Eq(string_v),
      ("$in", value) => OpValString::In(into_strings(value)?),

      ("$not", Value::String(string_v)) => OpValString::Not(string_v),
      ("$notIn", value) => OpValString::NotIn(into_strings(value)?),

      ("$lt", Value::String(string_v)) => OpValString::Lt(string_v),
      ("$lte", Value::String(string_v)) => OpValString::Lte(string_v),

      ("$gt", Value::String(string_v)) => OpValString::Gt(string_v),
      ("$gte", Value::String(string_v)) => OpValString::Gte(string_v),

      ("$contains", Value::String(string_v)) => OpValString::Contains(string_v),
      ("$containsAny", value) => OpValString::ContainsAny(into_strings(value)?),

      ("$containsAll", value) => OpValString::ContainsAll(into_strings(value)?),

      ("$notContains", Value::String(string_v)) => OpValString::NotContains(string_v),
      ("$notContainsAny", value) => OpValString::NotContainsAny(into_strings(value)?),

      ("$startsWith", Value::String(string_v)) => OpValString::StartsWith(string_v),
      ("$startsWithAny", value) => OpValString::StartsWithAny(into_strings(value)?),

      ("$notStartsWith", Value::String(string_v)) => OpValString::NotStartsWith(string_v),
      ("$notStartsWithAny", value) => OpValString::NotStartsWithAny(into_strings(value)?),

      ("$endsWith", Value::String(string_v)) => OpValString::EndsWith(string_v),
      ("$endsWithAny", value) => OpValString::EndsWithAny(into_strings(value)?),

      ("$notEndsWith", Value::String(string_v)) => OpValString::NotEndsWith(string_v),
      ("$notEndsWithAny", value) => OpValString::NotEndsWithAny(into_strings(value)?),

      ("$empty", Value::Bool(v)) => OpValString::Empty(v),
      ("$null", Value::Bool(v)) => OpValString::Null(v),

      ("$containsCi", Value::String(string_v)) => OpValString::ContainsCi(string_v),
      ("$notContainsCi", Value::String(string_v)) => OpValString::NotContainsCi(string_v),

      ("$startsWithCi", Value::String(string_v)) => OpValString::StartsWithCi(string_v),
      ("$notStartsWithCi", Value::String(string_v)) => OpValString::NotStartsWithCi(string_v),

      ("$endsWithCi", Value::String(string_v)) => OpValString::EndsWithCi(string_v),
      ("$notEndsWithCi", Value::String(string_v)) => OpValString::NotEndsWithCi(string_v),

      // Postgres optimized case insensitive like
      ("$ilike", Value::String(string_v)) => OpValString::Ilike(string_v),

      (_, v) => return Err(Error::JsonOpValNotSupported { operator: op.to_string(), value: v }),
    };
    Ok(ov)
  }
}
