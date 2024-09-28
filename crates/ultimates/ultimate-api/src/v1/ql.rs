use serde_json::json;
use ultimate::DataError;

use super::{
  filter_bool, filter_double, filter_int32, filter_int64, filter_string, ArrayDouble, ArrayInt32, ArrayInt64,
  ArrayString, FilterDouble, FilterInt32, FilterInt64, FilterString, OpNumber, OpString,
};

// -- FilterString begin

impl FilterString {
  pub fn op_null(op: OpString, is_null: bool) -> Self {
    Self { op: op.into(), v: Some(filter_string::V::IsNull(is_null)) }
  }

  pub fn op_value(op: OpString, value: impl Into<String>) -> Self {
    Self { op: op.into(), v: Some(filter_string::V::Value(value.into())) }
  }

  pub fn op_values(op: OpString, values: impl Into<ArrayString>) -> Self {
    Self { op: op.into(), v: Some(filter_string::V::Values(values.into())) }
  }
}

impl TryFrom<FilterString> for String {
  type Error = DataError;

  fn try_from(value: FilterString) -> Result<Self, Self::Error> {
    let v = value.v.ok_or_else(|| DataError::bad_request("Missing field 'v'"))?;
    match v {
      filter_string::V::Value(v) => Ok(v),
      _ => Err(DataError::bad_request("Invalid filter string, need field 'value'")),
    }
  }
}

impl TryFrom<FilterString> for bool {
  type Error = DataError;

  fn try_from(value: FilterString) -> Result<Self, Self::Error> {
    let v = value.v.ok_or_else(|| DataError::bad_request("Missing field 'v'"))?;
    match v {
      filter_string::V::IsNull(v) => Ok(v),
      _ => Err(DataError::bad_request("Invalid filter string, need field 'is_null'")),
    }
  }
}

impl TryFrom<FilterString> for ArrayString {
  type Error = DataError;

  fn try_from(value: FilterString) -> Result<Self, Self::Error> {
    let v = value.v.ok_or_else(|| DataError::bad_request("Missing field 'v'"))?;
    match v {
      filter_string::V::Values(v) => Ok(v),
      _ => Err(DataError::bad_request("Invalid filter string, need field 'values'")),
    }
  }
}

impl TryFrom<FilterString> for Vec<String> {
  type Error = DataError;

  fn try_from(value: FilterString) -> Result<Self, Self::Error> {
    let arr: ArrayString = value.try_into()?;
    Ok(arr.value)
  }
}

impl TryFrom<FilterString> for serde_json::Value {
  type Error = DataError;

  fn try_from(value: FilterString) -> Result<Self, Self::Error> {
    let v = value.v.ok_or_else(|| DataError::bad_request("Missing field 'v'"))?;
    match v {
      filter_string::V::Value(v) => Ok(serde_json::Value::String(v)),
      filter_string::V::IsNull(v) => Ok(serde_json::Value::Bool(v)),
      _ => Err(DataError::bad_request("Invalid filter string, need field 'value")),
    }
  }
}

impl TryFrom<FilterString> for Vec<serde_json::Value> {
  type Error = DataError;

  fn try_from(value: FilterString) -> Result<Self, Self::Error> {
    let v = value.v.ok_or_else(|| DataError::bad_request("Missing field 'v'"))?;
    match v {
      filter_string::V::Values(v) => Ok(v.value.into_iter().map(serde_json::Value::String).collect()),
      _ => Err(DataError::bad_request("Invalid filter string, need field 'values")),
    }
  }
}

// -- FilterString end

// -- FilterInt32 begin

impl FilterInt32 {}

impl TryFrom<FilterInt32> for bool {
  type Error = DataError;

  fn try_from(value: FilterInt32) -> Result<Self, Self::Error> {
    let v = value.v.ok_or_else(|| DataError::bad_request("Missing field 'v'"))?;
    match v {
      filter_int32::V::IsNull(v) => Ok(v),
      _ => Err(DataError::bad_request("Invalid filter int32, need field 'is_null'")),
    }
  }
}

impl TryFrom<FilterInt32> for i32 {
  type Error = DataError;

  fn try_from(value: FilterInt32) -> Result<Self, Self::Error> {
    let v = value.v.ok_or_else(|| DataError::bad_request("Missing field 'v'"))?;
    match v {
      filter_int32::V::Value(v) => Ok(v),
      _ => Err(DataError::bad_request("Invalid filter int32, need field 'value")),
    }
  }
}

impl TryFrom<FilterInt32> for ArrayInt32 {
  type Error = DataError;

  fn try_from(value: FilterInt32) -> Result<Self, Self::Error> {
    let v = value.v.ok_or_else(|| DataError::bad_request("Missing field 'v'"))?;
    match v {
      filter_int32::V::Values(v) => Ok(v),
      _ => Err(DataError::bad_request("Invalid filter int32, need field 'values")),
    }
  }
}

impl TryFrom<FilterInt32> for Vec<i32> {
  type Error = DataError;

  fn try_from(value: FilterInt32) -> Result<Self, Self::Error> {
    let arr: ArrayInt32 = value.try_into()?;
    Ok(arr.value)
  }
}

// -- FilterInt32 end

// -- FilterInt64 begin

impl FilterInt64 {
  pub fn op_value(op: OpNumber, value: impl Into<i64>) -> Self {
    Self { op: op.into(), v: Some(filter_int64::V::Value(value.into())) }
  }

  pub fn op_null(op: OpNumber, is_null: bool) -> Self {
    Self { op: op.into(), v: Some(filter_int64::V::IsNull(is_null)) }
  }

  pub fn op_values(op: OpNumber, values: impl Into<ArrayInt64>) -> Self {
    Self { op: op.into(), v: Some(filter_int64::V::Values(values.into())) }
  }
}

impl TryFrom<FilterInt64> for bool {
  type Error = DataError;

  fn try_from(value: FilterInt64) -> Result<Self, Self::Error> {
    let v = value.v.ok_or_else(|| DataError::bad_request("Missing field 'v'"))?;
    match v {
      filter_int64::V::IsNull(v) => Ok(v),
      _ => Err(DataError::bad_request("Invalid filter int64, need field 'is_null'")),
    }
  }
}

impl TryFrom<FilterInt64> for i64 {
  type Error = DataError;

  fn try_from(value: FilterInt64) -> Result<Self, Self::Error> {
    let v = value.v.ok_or_else(|| DataError::bad_request("Missing field 'v'"))?;
    match v {
      filter_int64::V::Value(v) => Ok(v),
      _ => Err(DataError::bad_request("Invalid filter int64, need field 'value")),
    }
  }
}

impl TryFrom<FilterInt64> for ArrayInt64 {
  type Error = DataError;

  fn try_from(value: FilterInt64) -> Result<Self, Self::Error> {
    let v = value.v.ok_or_else(|| DataError::bad_request("Missing field 'v'"))?;
    match v {
      filter_int64::V::Values(v) => Ok(v),
      _ => Err(DataError::bad_request("Invalid filter int64, need field 'values")),
    }
  }
}

impl TryFrom<FilterInt64> for Vec<i64> {
  type Error = DataError;

  fn try_from(value: FilterInt64) -> Result<Self, Self::Error> {
    let arr: ArrayInt64 = value.try_into()?;
    Ok(arr.value)
  }
}

impl TryFrom<FilterInt64> for serde_json::Value {
  type Error = DataError;

  fn try_from(value: FilterInt64) -> Result<Self, Self::Error> {
    let v = value.v.ok_or_else(|| DataError::bad_request("Missing field 'v'"))?;
    match v {
      filter_int64::V::Value(v) => Ok(json!(v)),
      _ => Err(DataError::bad_request("Invalid filter int64, need field 'value")),
    }
  }
}

impl TryFrom<FilterInt64> for Vec<serde_json::Value> {
  type Error = DataError;

  fn try_from(value: FilterInt64) -> Result<Self, Self::Error> {
    let v = value.v.ok_or_else(|| DataError::bad_request("Missing field 'v'"))?;
    match v {
      filter_int64::V::Values(v) => Ok(v.value.into_iter().map(|item| json!(item)).collect()),
      _ => Err(DataError::bad_request("Invalid filter int64, need field 'values")),
    }
  }
}

// -- FilterInt64 end

// -- FilterDouble begin

impl FilterDouble {
  pub fn op_value(op: OpNumber, value: impl Into<f64>) -> Self {
    Self { op: op.into(), v: Some(filter_double::V::Value(value.into())) }
  }

  pub fn op_values(op: OpNumber, values: impl Into<ArrayDouble>) -> Self {
    Self { op: op.into(), v: Some(filter_double::V::Values(values.into())) }
  }

  pub fn op_null(op: OpNumber, is_null: bool) -> Self {
    Self { op: op.into(), v: Some(filter_double::V::IsNull(is_null)) }
  }
}

impl TryFrom<FilterDouble> for bool {
  type Error = DataError;

  fn try_from(value: FilterDouble) -> Result<Self, Self::Error> {
    let v = value.v.ok_or_else(|| DataError::bad_request("Missing field 'v'"))?;
    match v {
      filter_double::V::IsNull(v) => Ok(v),
      _ => Err(DataError::bad_request("Invalid filter double, need field 'is_null")),
    }
  }
}

impl TryFrom<FilterDouble> for f64 {
  type Error = DataError;

  fn try_from(value: FilterDouble) -> Result<Self, Self::Error> {
    let v = value.v.ok_or_else(|| DataError::bad_request("Missing field 'v'"))?;
    match v {
      filter_double::V::Value(v) => Ok(v),
      _ => Err(DataError::bad_request("Invalid filter double, need field 'value")),
    }
  }
}

impl TryFrom<FilterDouble> for ArrayDouble {
  type Error = DataError;

  fn try_from(value: FilterDouble) -> Result<Self, Self::Error> {
    let v = value.v.ok_or_else(|| DataError::bad_request("Missing field 'v'"))?;
    match v {
      filter_double::V::Values(v) => Ok(v),
      _ => Err(DataError::bad_request("Invalid filter double, need field 'values")),
    }
  }
}

impl TryFrom<FilterDouble> for Vec<f64> {
  type Error = DataError;

  fn try_from(value: FilterDouble) -> Result<Self, Self::Error> {
    let arr: ArrayDouble = value.try_into()?;
    Ok(arr.value)
  }
}

// -- FilterDouble end
