use crate::filter::OpVal;

#[derive(Debug, Clone)]
pub struct OpValsBool(pub Vec<OpValBool>);

impl OpValsBool {
  pub fn eq(val: bool) -> Self {
    Self(vec![OpValBool::Eq(val)])
  }

  pub fn not(val: bool) -> Self {
    Self(vec![OpValBool::Not(val)])
  }

  pub fn null(val: bool) -> Self {
    Self(vec![OpValBool::Null(val)])
  }
}

#[derive(Debug, Clone)]
pub enum OpValBool {
  Eq(bool),
  Not(bool),
  Null(bool),
}

// region:    --- Simple Value to Eq BoolOpVal
impl From<bool> for OpValBool {
  fn from(val: bool) -> Self {
    OpValBool::Eq(val)
  }
}

impl From<&bool> for OpValBool {
  fn from(val: &bool) -> Self {
    OpValBool::Eq(*val)
  }
}
// endregion: --- Simple Value to Eq BoolOpVal

// region:    --- Simple Value to Eq BoolOpVals
impl From<bool> for OpValsBool {
  fn from(val: bool) -> Self {
    OpValBool::from(val).into()
  }
}

impl From<&bool> for OpValsBool {
  fn from(val: &bool) -> Self {
    OpValBool::from(*val).into()
  }
}
// endregion: --- Simple Value to Eq BoolOpVals

// region:    --- BoolOpVal to OpVal
impl From<OpValBool> for OpVal {
  fn from(val: OpValBool) -> Self {
    OpVal::Bool(val)
  }
}
// endregion: --- BoolOpVal to OpVal

// region:    --- Simple Value to Eq OpVal::Bool(BoolOpVal::Eq)
impl From<bool> for OpVal {
  fn from(val: bool) -> Self {
    OpValBool::Eq(val).into()
  }
}

impl From<&bool> for OpVal {
  fn from(val: &bool) -> Self {
    OpValBool::Eq(*val).into()
  }
}
// endregion: --- Simple Value to Eq OpVal::Bool(BoolOpVal::Eq)

#[cfg(feature = "with-sea-query")]
mod with_sea_query {
  use super::*;
  use crate::filter::{FilterNodeOptions, SeaResult, sea_is_col_value_null};
  use crate::sea_utils::into_node_value_expr;
  use sea_query::{BinOper, ColumnRef, ConditionExpression, SimpleExpr};

  impl OpValBool {
    pub fn into_sea_cond_expr(
      self,
      col: &ColumnRef,
      node_options: &FilterNodeOptions,
    ) -> SeaResult<ConditionExpression> {
      let binary_fn = |op: BinOper, val: bool| {
        let expr = into_node_value_expr(val, node_options);
        ConditionExpression::SimpleExpr(SimpleExpr::binary(col.clone().into(), op, expr))
      };

      let cond = match self {
        OpValBool::Eq(b) => binary_fn(BinOper::Equal, b),
        OpValBool::Not(b) => binary_fn(BinOper::NotEqual, b),
        OpValBool::Null(null) => sea_is_col_value_null(col.clone(), null),
      };

      Ok(cond)
    }
  }
}
