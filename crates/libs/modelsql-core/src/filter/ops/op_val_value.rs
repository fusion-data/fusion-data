use crate::filter::OpVal;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct OpValsValue(pub Vec<OpValValue>);

impl OpValsValue {
  pub fn eq(v: Value) -> Self {
    Self(vec![OpValValue::Eq(v)])
  }

  pub fn not(v: Value) -> Self {
    Self(vec![OpValValue::Not(v)])
  }

  pub fn in_<I>(v: I) -> Self
  where
    I: IntoIterator<Item = Value>,
  {
    Self(vec![OpValValue::In(v.into_iter().collect())])
  }

  pub fn not_in<I>(v: I) -> Self
  where
    I: IntoIterator<Item = Value>,
  {
    Self(vec![OpValValue::NotIn(v.into_iter().collect())])
  }

  pub fn lt(v: Value) -> Self {
    Self(vec![OpValValue::Lt(v)])
  }

  pub fn lte(v: Value) -> Self {
    Self(vec![OpValValue::Lte(v)])
  }

  pub fn gt(v: Value) -> Self {
    Self(vec![OpValValue::Gt(v)])
  }

  pub fn gte(v: Value) -> Self {
    Self(vec![OpValValue::Gte(v)])
  }

  pub fn null(v: bool) -> Self {
    Self(vec![OpValValue::Null(v)])
  }
}

#[derive(Debug, Clone)]
pub enum OpValValue {
  Eq(Value),
  Not(Value),

  In(Vec<Value>),
  NotIn(Vec<Value>),

  Lt(Value),
  Lte(Value),

  Gt(Value),
  Gte(Value),

  Null(bool),
}

// NOTE: We cannot implement the From<Value> for OpValValue, OpValsValue, ..
//       because it could fail if the json::Value is not a scalar type

// region:    --- OpValValue to OpVal::Value
impl From<OpValValue> for OpVal {
  fn from(val: OpValValue) -> Self {
    OpVal::Value(val)
  }
}
// endregion: --- OpValValue to OpVal::Value

#[cfg(feature = "with-sea-query")]
mod with_sea_query {
  use super::*;
  use crate::filter::{FilterNodeOptions, SeaResult, ToSeaValueFnHolder, sea_is_col_value_null};
  use crate::sea_utils::into_node_value_expr;
  use sea_query::{BinOper, ColumnRef, ConditionExpression, SimpleExpr};

  impl OpValValue {
    pub fn into_sea_cond_expr_with_json_to_sea(
      self,
      col: &ColumnRef,
      node_options: &FilterNodeOptions,
      to_sea_value: &ToSeaValueFnHolder,
    ) -> SeaResult<ConditionExpression> {
      // -- CondExpr builder for single value
      let binary_fn = |op: BinOper, json_value: serde_json::Value| -> SeaResult<ConditionExpression> {
        let sea_value = to_sea_value.call(json_value)?;

        let expr = into_node_value_expr(sea_value, node_options);
        Ok(ConditionExpression::SimpleExpr(SimpleExpr::binary(col.clone().into(), op, expr)))
      };

      // -- CondExpr builder for single value
      let binaries_fn = |op: BinOper, json_values: Vec<serde_json::Value>| -> SeaResult<ConditionExpression> {
        // -- Build the list of sea_query::Value
        let sea_values: Vec<sea_query::Value> =
          json_values.into_iter().map(|v| to_sea_value.call(v)).collect::<SeaResult<_>>()?;

        // -- Transform to the list of SimpleExpr
        let vec_expr: Vec<SimpleExpr> = sea_values.into_iter().map(|v| into_node_value_expr(v, node_options)).collect();
        let expr = SimpleExpr::Tuple(vec_expr);

        // -- Return the condition expression
        Ok(ConditionExpression::SimpleExpr(SimpleExpr::binary(col.clone().into(), op, expr)))
      };

      let cond = match self {
        OpValValue::Eq(json_value) => binary_fn(BinOper::Equal, json_value)?,
        OpValValue::In(json_values) => binaries_fn(BinOper::In, json_values)?,

        OpValValue::Not(json_value) => binary_fn(BinOper::NotEqual, json_value)?,
        OpValValue::NotIn(json_value) => binaries_fn(BinOper::NotIn, json_value)?,

        OpValValue::Lt(json_value) => binary_fn(BinOper::SmallerThan, json_value)?,
        OpValValue::Lte(json_value) => binary_fn(BinOper::SmallerThanOrEqual, json_value)?,

        OpValValue::Gt(json_value) => binary_fn(BinOper::GreaterThan, json_value)?,
        OpValValue::Gte(json_value) => binary_fn(BinOper::GreaterThanOrEqual, json_value)?,

        OpValValue::Null(null) => sea_is_col_value_null(col.clone(), null),
      };

      Ok(cond)
    }
  }
}
