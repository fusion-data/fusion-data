use modql::filter::{OpValInt32, OpValString, OpValValue, OpValsInt32, OpValsString, OpValsValue};
use ultimate::DataError;
use ultimate_api::v1::{FilterInt32, FilterInt64, FilterString, OpNumber, OpString};

/// 将 FilterString 转换为 OpValsValue。通常用于可映射为字符串的数据类型，比如：UUID、DateTime(字符串格式化)、……
pub fn try_into_op_vals_value_opt_with_filter_string(
  value: impl IntoIterator<Item = FilterString>,
) -> Result<Option<OpValsValue>, DataError> {
  let mut vals = Vec::new();
  for item in value {
    let op_val = try_into_op_val_value(item)?;
    vals.push(op_val);
  }

  Ok(if vals.is_empty() { None } else { Some(OpValsValue(vals)) })
}
/// 将 FilterInt64 转换为 OpValsValue。通常用于可映射为字符串的数据类型，比如：epoch time 形式的时间戳、……
pub fn try_into_op_vals_value_opt_with_filter_int64(
  value: impl IntoIterator<Item = FilterInt64>,
) -> Result<Option<OpValsValue>, DataError> {
  let mut vals = Vec::new();
  for item in value {
    let op_val = try_into_op_val_value_with_int64(item)?;
    vals.push(op_val);
  }

  Ok(if vals.is_empty() { None } else { Some(OpValsValue(vals)) })
}

fn try_into_op_val_value_with_int64(v: FilterInt64) -> Result<OpValValue, DataError> {
  let op_val = match v.op() {
    OpNumber::Eq => OpValValue::Eq(v.try_into()?),
    OpNumber::Not => OpValValue::Not(v.try_into()?),
    OpNumber::In => OpValValue::In(v.try_into()?),
    OpNumber::NotIn => OpValValue::NotIn(v.try_into()?),
    OpNumber::Gt => OpValValue::Gt(v.try_into()?),
    OpNumber::Gte => OpValValue::Gte(v.try_into()?),
    OpNumber::Lt => OpValValue::Lt(v.try_into()?),
    OpNumber::Lte => OpValValue::Lte(v.try_into()?),
    OpNumber::Null => OpValValue::Null(v.try_into()?),
  };
  Ok(op_val)
}
fn try_into_op_val_value(v: FilterString) -> Result<OpValValue, DataError> {
  let op_val = match v.op() {
    OpString::Eq => OpValValue::Eq(v.try_into()?),
    OpString::Not => OpValValue::Not(v.try_into()?),
    OpString::In => OpValValue::In(v.try_into()?),
    OpString::NotIn => OpValValue::NotIn(v.try_into()?),
    OpString::Gt => OpValValue::Gt(v.try_into()?),
    OpString::Gte => OpValValue::Gte(v.try_into()?),
    OpString::Lt => OpValValue::Lt(v.try_into()?),
    OpString::Lte => OpValValue::Lte(v.try_into()?),
    OpString::Null => OpValValue::Null(v.try_into()?),
  };
  Ok(op_val)
}

pub fn try_into_op_vals_string_opt(
  value: impl IntoIterator<Item = FilterString>,
) -> Result<Option<OpValsString>, DataError> {
  let mut vals = Vec::new();
  for item in value {
    let op_val = try_into_op_val_string(item)?;
    vals.push(op_val);
  }

  Ok(if vals.is_empty() { None } else { Some(OpValsString(vals)) })
}

fn try_into_op_val_string(v: FilterString) -> Result<OpValString, DataError> {
  let op_val = match v.op() {
    OpString::Eq => OpValString::Eq(v.try_into()?),
    OpString::Not => OpValString::Not(v.try_into()?),
    OpString::In => OpValString::In(v.try_into()?),
    OpString::NotIn => OpValString::NotIn(v.try_into()?),
    OpString::Gt => OpValString::Gt(v.try_into()?),
    OpString::Gte => OpValString::Gte(v.try_into()?),
    OpString::Lt => OpValString::Lt(v.try_into()?),
    OpString::Lte => OpValString::Lte(v.try_into()?),
    OpString::Null => OpValString::Null(v.try_into()?),
  };
  Ok(op_val)
}

pub fn try_into_op_vals_int32_opt(
  value: impl IntoIterator<Item = FilterInt32>,
) -> Result<Option<OpValsInt32>, DataError> {
  let mut vals = Vec::new();
  for item in value {
    let op_val = try_into_op_val_int32(item)?;
    vals.push(op_val);
  }

  Ok(if vals.is_empty() { None } else { Some(OpValsInt32(vals)) })
}

pub fn try_into_op_val_int32(v: FilterInt32) -> Result<OpValInt32, DataError> {
  let op_val = match v.op() {
    OpNumber::Eq => OpValInt32::Eq(v.try_into()?),
    OpNumber::Not => OpValInt32::Not(v.try_into()?),
    OpNumber::In => OpValInt32::In(v.try_into()?),
    OpNumber::NotIn => OpValInt32::NotIn(v.try_into()?),
    OpNumber::Lt => OpValInt32::Lt(v.try_into()?),
    OpNumber::Lte => OpValInt32::Lte(v.try_into()?),
    OpNumber::Gt => OpValInt32::Gt(v.try_into()?),
    OpNumber::Gte => OpValInt32::Gte(v.try_into()?),
    OpNumber::Null => OpValInt32::Null(v.try_into()?),
  };
  Ok(op_val)
}
