use crate::modql::filter::{
  OpValInt32, OpValInt64, OpValString, OpValUuid, OpValValue, OpValsInt32, OpValsInt64, OpValsString, OpValsUuid,
  OpValsValue,
};
use ultimate::DataError;
use ultimate_api::v1::{Null, OpNumber, OpString, ValInt32, ValInt64, ValString};

/// 将 ValString 转换为 OpValsUuid。通常用于可映射为字符串的数据类型，比如：UUID、DateTime(字符串格式化)、……
pub fn try_into_op_vals_uuid_with_filter_string(
  value: impl IntoIterator<Item = ValString>,
) -> Result<Option<OpValsUuid>, DataError> {
  let mut vals = Vec::new();
  for item in value {
    let op_val = try_into_op_val_uuid(item)?;
    vals.push(op_val);
  }

  Ok(if vals.is_empty() { None } else { Some(OpValsUuid(vals)) })
}

/// 将 ValString 转换为 OpValsValue。通常用于可映射为字符串的数据类型，比如：UUID、DateTime(字符串格式化)、……
pub fn try_into_op_vals_value_opt_with_filter_string(
  value: impl IntoIterator<Item = ValString>,
) -> Result<Option<OpValsValue>, DataError> {
  let mut vals = Vec::new();
  for item in value {
    let op_val = try_into_op_val_value(item)?;
    vals.push(op_val);
  }

  Ok(if vals.is_empty() { None } else { Some(OpValsValue(vals)) })
}

/// 将 ValInt64 转换为 OpValsValue。通常用于可映射为字符串的数据类型，比如：epoch time 形式的时间戳、……
pub fn try_into_op_vals_value_opt_with_filter_int64(
  value: impl IntoIterator<Item = ValInt64>,
) -> Result<Option<OpValsValue>, DataError> {
  let mut vals = Vec::new();
  for item in value {
    let op_val = try_into_op_val_value_with_int64(item)?;
    vals.push(op_val);
  }

  Ok(if vals.is_empty() { None } else { Some(OpValsValue(vals)) })
}

fn try_into_op_val_value_with_int64(v: ValInt64) -> Result<OpValValue, DataError> {
  let op_val = match v.o() {
    OpNumber::Eq => OpValValue::Eq(v.try_into()?),
    OpNumber::Not => OpValValue::Not(v.try_into()?),
    OpNumber::In => OpValValue::In(v.try_into()?),
    OpNumber::NotIn => OpValValue::NotIn(v.try_into()?),
    OpNumber::Gt => OpValValue::Gt(v.try_into()?),
    OpNumber::Gte => OpValValue::Gte(v.try_into()?),
    OpNumber::Lt => OpValValue::Lt(v.try_into()?),
    OpNumber::Lte => OpValValue::Lte(v.try_into()?),
    OpNumber::Null => OpValValue::Null(Null::try_from(v)?.is_null()),
  };
  Ok(op_val)
}

fn try_into_op_val_uuid(v: ValString) -> Result<OpValUuid, DataError> {
  let op_val = match v.o() {
    OpString::Eq => OpValUuid::Eq(v.try_into()?),
    OpString::Not => OpValUuid::Not(v.try_into()?),
    OpString::In => OpValUuid::In(v.try_into()?),
    OpString::NotIn => OpValUuid::NotIn(v.try_into()?),
    OpString::Null => OpValUuid::Null(Null::try_from(v)?.is_null()),
    _ => return Err(DataError::bad_request(format!("Invalid Operator: {:?}", v.o()))),
  };
  Ok(op_val)
}

pub fn try_into_op_values_with_string_opt(
  value: impl IntoIterator<Item = ValString>,
) -> Result<Option<OpValsValue>, DataError> {
  let mut vals = Vec::new();
  for item in value {
    let op_val = try_into_op_val_value(item)?;
    vals.push(op_val);
  }

  Ok(if vals.is_empty() { None } else { Some(OpValsValue(vals)) })
}

fn try_into_op_val_value(v: ValString) -> Result<OpValValue, DataError> {
  let op_val = match v.o() {
    OpString::Eq => OpValValue::Eq(v.try_into()?),
    OpString::Not => OpValValue::Not(v.try_into()?),
    OpString::In => OpValValue::In(v.try_into()?),
    OpString::NotIn => OpValValue::NotIn(v.try_into()?),
    OpString::Gt => OpValValue::Gt(v.try_into()?),
    OpString::Gte => OpValValue::Gte(v.try_into()?),
    OpString::Lt => OpValValue::Lt(v.try_into()?),
    OpString::Lte => OpValValue::Lte(v.try_into()?),
    OpString::Null => OpValValue::Null(Null::try_from(v)?.is_null()),
    _ => return Err(DataError::bad_request(format!("Invalid Operator: {:?}", v.o()))),
  };
  Ok(op_val)
}

pub fn try_into_op_vals_string_opt(
  value: impl IntoIterator<Item = ValString>,
) -> Result<Option<OpValsString>, DataError> {
  let mut vals = Vec::new();
  for item in value {
    let op_val = try_into_op_val_string(item)?;
    vals.push(op_val);
  }

  Ok(if vals.is_empty() { None } else { Some(OpValsString(vals)) })
}

fn try_into_op_val_string(v: ValString) -> Result<OpValString, DataError> {
  let op_val = match v.o() {
    OpString::Eq => OpValString::Eq(v.try_into()?),
    OpString::Not => OpValString::Not(v.try_into()?),
    OpString::In => OpValString::In(v.try_into()?),
    OpString::NotIn => OpValString::NotIn(v.try_into()?),
    OpString::Gt => OpValString::Gt(v.try_into()?),
    OpString::Gte => OpValString::Gte(v.try_into()?),
    OpString::Lt => OpValString::Lt(v.try_into()?),
    OpString::Lte => OpValString::Lte(v.try_into()?),
    OpString::Contains => OpValString::Contains(v.try_into()?),
    OpString::NotContains => OpValString::NotContains(v.try_into()?),
    OpString::ContainsAny => OpValString::ContainsAny(v.try_into()?),
    OpString::NotContainsAny => OpValString::NotContainsAny(v.try_into()?),
    OpString::ContainsAll => OpValString::ContainsAll(v.try_into()?),
    OpString::StartsWith => OpValString::StartsWith(v.try_into()?),
    OpString::NotStartsWith => OpValString::NotStartsWith(v.try_into()?),
    OpString::StartsWithAny => OpValString::StartsWithAny(v.try_into()?),
    OpString::NotStartsWithAny => OpValString::NotStartsWithAny(v.try_into()?),
    OpString::EndsWith => OpValString::EndsWith(v.try_into()?),
    OpString::NotEndsWith => OpValString::NotEndsWith(v.try_into()?),
    OpString::EndsWithAny => OpValString::EndsWithAny(v.try_into()?),
    OpString::NotEndsWithAny => OpValString::NotEndsWithAny(v.try_into()?),
    OpString::Empty => OpValString::Empty(String::try_from(v)?.is_empty()),
    OpString::Null => OpValString::Null(Null::try_from(v)?.is_null()),
    OpString::ContainsCi => OpValString::ContainsCi(v.try_into()?),
    OpString::NotContainsCi => OpValString::NotContainsCi(v.try_into()?),
    OpString::StartsWithCi => OpValString::StartsWithCi(v.try_into()?),
    OpString::NotStartsWithCi => OpValString::NotStartsWithCi(v.try_into()?),
    OpString::EndsWithCi => OpValString::EndsWithCi(v.try_into()?),
    OpString::NotEndsWithCi => OpValString::NotEndsWithCi(v.try_into()?),
    OpString::ILike => OpValString::Ilike(v.try_into()?),
  };
  Ok(op_val)
}

pub fn try_into_op_vals_int32_opt(value: impl IntoIterator<Item = ValInt32>) -> Result<Option<OpValsInt32>, DataError> {
  let mut vals = Vec::new();
  for item in value {
    let op_val = try_into_op_val_int32(item)?;
    vals.push(op_val);
  }

  Ok(if vals.is_empty() { None } else { Some(OpValsInt32(vals)) })
}

pub fn try_into_op_val_int32(v: ValInt32) -> Result<OpValInt32, DataError> {
  let op_val = match v.o() {
    OpNumber::Eq => OpValInt32::Eq(v.try_into()?),
    OpNumber::Not => OpValInt32::Not(v.try_into()?),
    OpNumber::In => OpValInt32::In(v.try_into()?),
    OpNumber::NotIn => OpValInt32::NotIn(v.try_into()?),
    OpNumber::Lt => OpValInt32::Lt(v.try_into()?),
    OpNumber::Lte => OpValInt32::Lte(v.try_into()?),
    OpNumber::Gt => OpValInt32::Gt(v.try_into()?),
    OpNumber::Gte => OpValInt32::Gte(v.try_into()?),
    OpNumber::Null => OpValInt32::Null(Null::try_from(v)?.is_null()),
  };
  Ok(op_val)
}

pub fn try_into_op_vals_int64_opt(value: impl IntoIterator<Item = ValInt64>) -> Result<Option<OpValsInt64>, DataError> {
  let mut vals = Vec::new();
  for item in value {
    let op_val = try_into_op_val_int64(item)?;
    vals.push(op_val);
  }

  Ok(if vals.is_empty() { None } else { Some(OpValsInt64(vals)) })
}

pub fn try_into_op_val_int64(v: ValInt64) -> Result<OpValInt64, DataError> {
  let op_val = match v.o() {
    OpNumber::Eq => OpValInt64::Eq(v.try_into()?),
    OpNumber::Not => OpValInt64::Not(v.try_into()?),
    OpNumber::In => OpValInt64::In(v.try_into()?),
    OpNumber::NotIn => OpValInt64::NotIn(v.try_into()?),
    OpNumber::Lt => OpValInt64::Lt(v.try_into()?),
    OpNumber::Lte => OpValInt64::Lte(v.try_into()?),
    OpNumber::Gt => OpValInt64::Gt(v.try_into()?),
    OpNumber::Gte => OpValInt64::Gte(v.try_into()?),
    OpNumber::Null => OpValInt64::Null(Null::try_from(v)?.is_null()),
  };
  Ok(op_val)
}
