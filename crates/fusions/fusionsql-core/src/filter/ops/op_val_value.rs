use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::filter::OpVal;

// #[derive(Debug, Clone)]
// pub enum OpValValue {
//   Eq(Value),
//   Not(Value),

//   In(Vec<Value>),
//   NotIn(Vec<Value>),

//   Lt(Value),
//   Lte(Value),

//   Gt(Value),
//   Gte(Value),

//   Null(bool),
// }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "with-wasm", derive(tsify::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub struct OpValValue {
  #[serde(rename = "$eq")]
  #[cfg_attr(feature = "with-wasm", tsify(type = "any"))]
  pub eq: Option<Value>,

  #[serde(rename = "$not")]
  #[cfg_attr(feature = "with-wasm", tsify(type = "any"))]
  pub not: Option<Value>,

  #[serde(rename = "$in")]
  #[cfg_attr(feature = "with-wasm", tsify(type = "any[]"))]
  pub in_: Option<Vec<Value>>,

  #[serde(rename = "$not_in")]
  #[cfg_attr(feature = "with-wasm", tsify(type = "any[]"))]
  pub not_in: Option<Vec<Value>>,

  #[serde(rename = "$lt")]
  #[cfg_attr(feature = "with-wasm", tsify(type = "any"))]
  pub lt: Option<Value>,

  #[serde(rename = "$lte")]
  #[cfg_attr(feature = "with-wasm", tsify(type = "any"))]
  pub lte: Option<Value>,

  #[serde(rename = "$gt")]
  #[cfg_attr(feature = "with-wasm", tsify(type = "any"))]
  pub gt: Option<Value>,

  #[serde(rename = "$gte")]
  #[cfg_attr(feature = "with-wasm", tsify(type = "any"))]
  pub gte: Option<Value>,

  #[serde(rename = "$null")]
  pub null: Option<bool>,
}

impl OpValValue {
  pub fn eq(v: Value) -> Self {
    Self { eq: Some(v), ..Default::default() }
  }

  pub fn not(v: Value) -> Self {
    Self { not: Some(v), ..Default::default() }
  }

  pub fn in_<I>(v: I) -> Self
  where
    I: IntoIterator<Item = Value>,
  {
    Self { in_: Some(v.into_iter().collect()), ..Default::default() }
  }

  pub fn not_in<I>(v: I) -> Self
  where
    I: IntoIterator<Item = Value>,
  {
    Self { not_in: Some(v.into_iter().collect()), ..Default::default() }
  }

  pub fn lt(v: Value) -> Self {
    Self { lt: Some(v), ..Default::default() }
  }

  pub fn lte(v: Value) -> Self {
    Self { lte: Some(v), ..Default::default() }
  }

  pub fn gt(v: Value) -> Self {
    Self { gt: Some(v), ..Default::default() }
  }

  pub fn gte(v: Value) -> Self {
    Self { gte: Some(v), ..Default::default() }
  }

  pub fn null(v: bool) -> Self {
    Self { null: Some(v), ..Default::default() }
  }

  pub fn with_eq(mut self, v: Value) -> Self {
    self.eq = Some(v);
    self
  }

  pub fn with_not(mut self, v: Value) -> Self {
    self.not = Some(v);
    self
  }

  pub fn with_in<I>(mut self, v: I) -> Self
  where
    I: IntoIterator<Item = Value>,
  {
    self.in_ = Some(v.into_iter().collect());
    self
  }

  pub fn with_not_in<I>(mut self, v: I) -> Self
  where
    I: IntoIterator<Item = Value>,
  {
    self.not_in = Some(v.into_iter().collect());
    self
  }

  pub fn with_lt(mut self, v: Value) -> Self {
    self.lt = Some(v);
    self
  }

  pub fn with_lte(mut self, v: Value) -> Self {
    self.lte = Some(v);
    self
  }

  pub fn with_gt(mut self, v: Value) -> Self {
    self.gt = Some(v);
    self
  }

  pub fn with_gte(mut self, v: Value) -> Self {
    self.gte = Some(v);
    self
  }

  pub fn with_null(mut self, v: bool) -> Self {
    self.null = Some(v);
    self
  }
}

// NOTE: We cannot implement the From<Value> for OpValValue, OpValValue, ..
//       because it could fail if the json::Value is not a scalar type

// region:    --- OpValValue to OpVal::Value
impl From<OpValValue> for OpVal {
  fn from(value: OpValValue) -> Self {
    OpVal::Value(Box::new(value))
  }
}
// endregion: --- OpValValue to OpVal::Value

#[cfg(feature = "with-sea-query")]
mod with_sea_query {
  use sea_query::{BinOper, ColumnRef, ConditionExpression, SimpleExpr};

  use crate::filter::{
    FilterNodeOptions, ForSeaCondition, IntoSeaError, OpValTrait, SeaResult, ToSeaValueFnHolder, sea_is_col_value_null,
  };
  use crate::sea_utils::into_node_value_expr;

  use super::*;

  impl OpValTrait for OpValValue {
    fn to_condition_expressions(
      self,
      col: &ColumnRef,
      node_options: &FilterNodeOptions,
      for_sea_condition: Option<&ForSeaCondition>,
    ) -> SeaResult<Vec<ConditionExpression>> {
      let Some(for_sea_cond) = for_sea_condition else {
        return Err(IntoSeaError::Custom(
          "OpValValue must have a #[fusionsql(to_sea_value_fn=\"fn_name\"] or to_sea_condition_fn attribute"
            .to_string(),
        ));
      };

      match for_sea_cond {
        ForSeaCondition::ToSeaCondition(to_sea_condition) => to_sea_condition.call(col, self),
        ForSeaCondition::ToSeaValue(to_sea_value) => self._to_condition_expressions(col, node_options, to_sea_value),
      }
    }
  }

  impl OpValValue {
    fn _to_condition_expressions(
      &self,
      col: &ColumnRef,
      node_options: &FilterNodeOptions,
      to_sea_value: &ToSeaValueFnHolder,
    ) -> SeaResult<Vec<ConditionExpression>> {
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

      let mut cond_exprs = Vec::new();

      // -- $eq
      if let Some(v) = &self.eq {
        cond_exprs.push(binary_fn(BinOper::Equal, v.clone())?);
      }

      // -- $not
      if let Some(v) = &self.not {
        cond_exprs.push(binary_fn(BinOper::NotEqual, v.clone())?);
      }

      // -- $in
      if let Some(v) = &self.in_ {
        cond_exprs.push(binaries_fn(BinOper::In, v.clone())?);
      }

      // -- $not_in
      if let Some(v) = &self.not_in {
        cond_exprs.push(binaries_fn(BinOper::NotIn, v.clone())?);
      }

      // -- $lt
      if let Some(v) = &self.lt {
        cond_exprs.push(binary_fn(BinOper::SmallerThan, v.clone())?);
      }

      // -- $lte
      if let Some(v) = &self.lte {
        cond_exprs.push(binary_fn(BinOper::SmallerThanOrEqual, v.clone())?);
      }

      // -- $gt
      if let Some(v) = &self.gt {
        cond_exprs.push(binary_fn(BinOper::GreaterThan, v.clone())?);
      }

      // -- $gte
      if let Some(v) = &self.gte {
        cond_exprs.push(binary_fn(BinOper::GreaterThanOrEqual, v.clone())?);
      }

      // -- $null
      if let Some(v) = &self.null {
        cond_exprs.push(sea_is_col_value_null(col.clone(), *v));
      }

      Ok(cond_exprs)
    }
  }
}
