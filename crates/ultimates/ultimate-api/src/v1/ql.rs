use serde_json::json;
use ultimate::DataError;

use super::{
  filter_bool, filter_double, filter_int32, filter_int64, filter_string, ArrayBool, ArrayDouble, ArrayInt32,
  ArrayInt64, ArrayString, FilterBool, FilterDouble, FilterInt32, FilterInt64, FilterString, Null, OpNumber, OpString,
};

macro_rules! impl_filter_helpers {
  ($S:ty, $Op:ty, $op_v:expr, $t:ty, $arr_t:ty, $v_package:ident) => {
    impl $S {
      pub fn new_is_null() -> Self {
        Self { op: $op_v.into(), v: Some($v_package::V::IsNull(Null::IsNull as i32)) }
      }

      pub fn new_not_null() -> Self {
        Self { op: $op_v.into(), v: Some($v_package::V::IsNull(Null::NotNull as i32)) }
      }

      pub fn new_value(op: $Op, value: impl Into<$t>) -> Self {
        Self { op: op.into(), v: Some($v_package::V::Value(value.into())) }
      }

      pub fn new_values(op: $Op, values: impl Into<$arr_t>) -> Self {
        Self { op: op.into(), v: Some($v_package::V::Values(values.into())) }
      }
    }

    impl TryFrom<$S> for Null {
      type Error = DataError;

      fn try_from(value: $S) -> Result<Self, Self::Error> {
        let v = value.v.ok_or_else(|| {
          DataError::bad_request(format!("Invalid From<{}> for Null, missing field 'v'", stringify!($S)))
        })?;
        match v {
          $v_package::V::IsNull(v) => Ok(v.try_into()?),
          _ => {
            Err(DataError::bad_request(format!("Invalid From<{}> for Null, missing field 'is_null'", stringify!($S))))
          }
        }
      }
    }

    impl TryFrom<$S> for $t {
      type Error = DataError;

      fn try_from(value: $S) -> Result<Self, Self::Error> {
        let v = value.v.ok_or_else(|| {
          DataError::bad_request(format!("Invalid From<{}> for {}, missing field 'v'", stringify!($S), stringify!($t)))
        })?;
        match v {
          $v_package::V::Value(v) => Ok(v),
          _ => Err(DataError::bad_request(format!(
            "Invalid From<{}> for {}, missing field 'value'",
            stringify!($S),
            stringify!($t)
          ))),
        }
      }
    }

    impl TryFrom<$S> for $arr_t {
      type Error = DataError;

      fn try_from(value: $S) -> Result<Self, Self::Error> {
        let v = value.v.ok_or_else(|| {
          DataError::bad_request(format!("Invalid From<{}> for {}, missing field 'v'", stringify!($S), stringify!($t)))
        })?;
        match v {
          $v_package::V::Values(v) => Ok(v),
          _ => Err(DataError::bad_request(
            (format!("Invalid From<{}> for {}, missing field 'values'", stringify!($S), stringify!($arr_t))),
          )),
        }
      }
    }

    impl TryFrom<$S> for Vec<$t> {
      type Error = DataError;

      fn try_from(value: $S) -> Result<Self, Self::Error> {
        let arr: $arr_t = value.try_into()?;
        Ok(arr.value)
      }
    }
  };
}

macro_rules! impl_filter_serde_helpers {
  ($S:ty, $v_package:ident) => {
    impl TryFrom<$S> for serde_json::Value {
      type Error = DataError;

      fn try_from(value: $S) -> Result<Self, Self::Error> {
        let v = value.v.ok_or_else(|| {
          DataError::bad_request(format!("Invalid From<{}> for serde_json::Value, missing field 'v'", stringify!($S)))
        })?;
        match v {
          $v_package::V::Value(v) => Ok(json!(v)),
          _ => Err(DataError::bad_request("Invalid filter string, need field 'value")),
        }
      }
    }

    impl TryFrom<$S> for Vec<serde_json::Value> {
      type Error = DataError;

      fn try_from(value: $S) -> Result<Self, Self::Error> {
        let v = value.v.ok_or_else(|| DataError::bad_request("Missing field 'v'"))?;
        match v {
          $v_package::V::Values(v) => Ok(v.value.into_iter().map(|v| json!(v)).collect()),
          _ => Err(DataError::bad_request(
            (format!("Invalid From<{}> for Vec<serde_json::Value>, missing field 'values'", stringify!($S))),
          )),
        }
      }
    }
  };
}

// -- FilterBool begin
impl_filter_helpers!(FilterBool, OpNumber, OpNumber::Null, bool, ArrayBool, filter_bool);
// -- FilterBool end

// -- FilterString begin
impl_filter_helpers!(FilterString, OpString, OpString::Null, String, ArrayString, filter_string);
impl_filter_serde_helpers!(FilterString, filter_string);
// -- FilterString end

// -- FilterInt32 begin
impl_filter_helpers!(FilterInt32, OpNumber, OpNumber::Null, i32, ArrayInt32, filter_int32);
// -- FilterInt32 end

// -- FilterInt64 begin
impl_filter_helpers!(FilterInt64, OpNumber, OpNumber::Null, i64, ArrayInt64, filter_int64);
impl_filter_serde_helpers!(FilterInt64, filter_int64);
// -- FilterInt64 end

// -- FilterDouble begin
impl_filter_helpers!(FilterDouble, OpNumber, OpNumber::Null, f64, ArrayDouble, filter_double);
// -- FilterDouble end

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_int32() {
    let fi32 = FilterInt32::new_is_null();
    let n: Null = fi32.try_into().unwrap();
    assert_eq!(Null::IsNull, n);
  }
}
