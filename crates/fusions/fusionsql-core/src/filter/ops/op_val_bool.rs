use serde::{Deserialize, Serialize};

#[cfg(feature = "with-sea-query")]
use crate::filter::FilterNodeOptions;
use crate::filter::OpVal;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "with-wasm", derive(tsify::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub struct OpValBool {
  #[serde(rename = "$eq")]
  pub eq: Option<bool>,

  #[serde(rename = "$not")]
  pub not: Option<bool>,

  #[serde(rename = "$null")]
  pub null: Option<bool>,
}

impl OpValBool {
  pub fn eq(val: bool) -> Self {
    Self { eq: Some(val), not: None, null: None }
  }

  pub fn not(val: bool) -> Self {
    Self { eq: None, not: Some(val), null: None }
  }

  pub fn null(val: bool) -> Self {
    Self { eq: None, not: None, null: Some(val) }
  }

  pub fn with_eq(mut self, val: bool) -> Self {
    self.eq = Some(val);
    self
  }

  pub fn with_not(mut self, val: bool) -> Self {
    self.not = Some(val);
    self
  }

  pub fn with_null(mut self, val: bool) -> Self {
    self.null = Some(val);
    self
  }
}

impl From<OpValBool> for OpVal {
  fn from(value: OpValBool) -> Self {
    OpVal::Bool(value)
  }
}

#[cfg(feature = "with-sea-query")]
mod with_sea_query {
  use sea_query::{BinOper, ColumnRef, ConditionExpression, SimpleExpr};

  use crate::filter::{ForSeaCondition, OpValTrait, SeaResult, sea_is_col_value_null};
  use crate::sea_utils::into_node_value_expr;

  use super::*;

  impl OpValTrait for OpValBool {
    fn to_condition_expressions(
      self,
      col: &ColumnRef,
      node_options: &FilterNodeOptions,
      _for_sea_condition: Option<&ForSeaCondition>,
    ) -> SeaResult<Vec<ConditionExpression>> {
      let binary_fn = |op: BinOper, val: bool| {
        let expr = into_node_value_expr(val, node_options);
        ConditionExpression::SimpleExpr(SimpleExpr::binary(col.clone().into(), op, expr))
      };

      let mut expressions = Vec::new();
      if let Some(val) = self.eq {
        expressions.push(binary_fn(BinOper::Equal, val));
      }
      if let Some(val) = self.not {
        expressions.push(binary_fn(BinOper::NotEqual, val));
      }
      if let Some(val) = self.null {
        expressions.push(sea_is_col_value_null(col.clone(), val));
      }

      Ok(expressions)
    }
  }
}
