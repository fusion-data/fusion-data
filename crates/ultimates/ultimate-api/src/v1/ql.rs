use serde_json::json;

use super::{
  ArrayBool, ArrayDouble, ArrayInt32, ArrayInt64, ArrayString, Null, OpBool, OpNumber, OpString, ValBool, ValDouble,
  ValInt32, ValInt64, ValString, val_bool, val_double, val_int32, val_int64, val_string,
};
use crate::Error;

macro_rules! impl_filter_helpers {
  ($S:ty, $Op:ty, $op_v:expr, $t:ty, $arr_t:ty, $v_package:ident) => {
    impl $S {
      pub fn new_is_null() -> Self {
        Self { o: $op_v.into(), value: Some($v_package::Value::N(Null::IsNull as i32)) }
      }

      pub fn new_not_null() -> Self {
        Self { o: $op_v.into(), value: Some($v_package::Value::N(Null::NotNull as i32)) }
      }

      pub fn new_value(op: $Op, value: impl Into<$t>) -> Self {
        Self { o: op.into(), value: Some($v_package::Value::V(value.into())) }
      }

      pub fn new_values(op: $Op, values: impl Into<$arr_t>) -> Self {
        Self { o: op.into(), value: Some($v_package::Value::Vs(values.into())) }
      }
    }

    impl TryFrom<$S> for Null {
      type Error = Error;

      fn try_from(value: $S) -> Result<Self, Self::Error> {
        let v = value
          .value
          .ok_or_else(|| Error::bad_request(format!("Invalid From<{}> for Null, missing field 'v'", stringify!($S))))?;
        match v {
          $v_package::Value::N(v) => Ok(v.try_into()?),
          _ => Err(Error::bad_request(format!("Invalid From<{}> for Null, missing field 'is_null'", stringify!($S)))),
        }
      }
    }

    impl TryFrom<$S> for $t {
      type Error = Error;

      fn try_from(value: $S) -> Result<Self, Self::Error> {
        let v = value.value.ok_or_else(|| {
          Error::bad_request(format!("Invalid From<{}> for {}, missing field 'v'", stringify!($S), stringify!($t)))
        })?;
        match v {
          $v_package::Value::V(v) => Ok(v),
          _ => Err(Error::bad_request(format!(
            "Invalid From<{}> for {}, missing field 'value'",
            stringify!($S),
            stringify!($t)
          ))),
        }
      }
    }

    impl TryFrom<$S> for $arr_t {
      type Error = Error;

      fn try_from(value: $S) -> Result<Self, Self::Error> {
        let v = value.value.ok_or_else(|| {
          Error::bad_request(format!("Invalid From<{}> for {}, missing field 'v'", stringify!($S), stringify!($t)))
        })?;
        match v {
          $v_package::Value::Vs(v) => Ok(v),
          _ => Err(Error::bad_request(
            (format!("Invalid From<{}> for {}, missing field 'values'", stringify!($S), stringify!($arr_t))),
          )),
        }
      }
    }

    impl TryFrom<$S> for Vec<$t> {
      type Error = Error;

      fn try_from(value: $S) -> Result<Self, Self::Error> {
        let arr: $arr_t = value.try_into()?;
        Ok(arr.value)
      }
    }

    impl From<$S> for Vec<$S> {
      fn from(value: $S) -> Self {
        vec![value]
      }
    }
  };
}

macro_rules! impl_filter_serde_helpers {
  ($S:ty, $v_package:ident) => {
    impl TryFrom<$S> for serde_json::Value {
      type Error = Error;

      fn try_from(value: $S) -> Result<Self, Self::Error> {
        let v = value.value.ok_or_else(|| {
          Error::bad_request(format!("Invalid From<{}> for serde_json::Value, missing field 'v'", stringify!($S)))
        })?;
        match v {
          $v_package::Value::V(v) => Ok(json!(v)),
          _ => Err(Error::bad_request("Invalid filter string, need field 'value")),
        }
      }
    }

    impl TryFrom<$S> for Vec<serde_json::Value> {
      type Error = Error;

      fn try_from(value: $S) -> Result<Self, Self::Error> {
        let v = value.value.ok_or_else(|| Error::bad_request("Missing field 'v'"))?;
        match v {
          $v_package::Value::Vs(v) => Ok(v.value.into_iter().map(|v| json!(v)).collect()),
          _ => Err(Error::bad_request(
            (format!("Invalid From<{}> for Vec<serde_json::Value>, missing field 'values'", stringify!($S))),
          )),
        }
      }
    }
  };
}

// -- OpBool begin
impl_filter_helpers!(ValBool, OpBool, OpBool::Null, bool, ArrayBool, val_bool);
// -- OpBool end

// -- OpString begin
impl_filter_helpers!(ValString, OpString, OpString::Null, String, ArrayString, val_string);
impl_filter_serde_helpers!(ValString, val_string);
// -- OpString end

// -- ValInt32 begin
impl_filter_helpers!(ValInt32, OpNumber, OpNumber::Null, i32, ArrayInt32, val_int32);
// -- ValInt32 end

// -- ValInt64 begin
impl_filter_helpers!(ValInt64, OpNumber, OpNumber::Null, i64, ArrayInt64, val_int64);
impl_filter_serde_helpers!(ValInt64, val_int64);
// -- ValInt64 end

// -- ValDouble begin
impl_filter_helpers!(ValDouble, OpNumber, OpNumber::Null, f64, ArrayDouble, val_double);
// -- ValDouble end

// -- Uuid begin
impl TryFrom<ValString> for uuid::Uuid {
  type Error = Error;

  fn try_from(value: ValString) -> Result<Self, Self::Error> {
    let v = value.value.ok_or_else(|| {
      Error::bad_request(format!("Invalid From<{}> for serde_json::Value, missing field 'v'", stringify!($S)))
    })?;
    match v {
      val_string::Value::V(v) => Ok(v.parse()?),
      _ => Err(Error::bad_request("Invalid filter string, need field 'value")),
    }
  }
}

impl TryFrom<ValString> for Vec<uuid::Uuid> {
  type Error = Error;

  fn try_from(value: ValString) -> Result<Self, Self::Error> {
    let v = value.value.ok_or_else(|| Error::bad_request("Missing field 'v'"))?;
    match v {
      val_string::Value::Vs(v) => {
        let mut vs = Vec::with_capacity(v.value.len());
        for item in v.value {
          vs.push(item.parse()?);
        }
        Ok(vs)
      }
      _ => {
        Err(Error::bad_request(format!("Invalid From<{}> for Vec<uuid::Uuid>, missing field 'values'", stringify!($S))))
      }
    }
  }
}
// -- Uuid end

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_int32() {
    let fi32 = ValInt32::new_is_null();
    let n: Null = fi32.try_into().unwrap();
    assert_eq!(Null::IsNull, n);
  }
}
