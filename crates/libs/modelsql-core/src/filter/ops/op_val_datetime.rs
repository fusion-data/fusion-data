use ultimate_common::time::OffsetDateTime;

use super::OpVal;

#[derive(Debug, Clone)]
pub struct OpValsDateTime(pub Vec<OpValDateTime>);

impl OpValsDateTime {
  pub fn eq(v: OffsetDateTime) -> Self {
    Self(vec![OpValDateTime::Eq(v)])
  }

  pub fn not(v: OffsetDateTime) -> Self {
    Self(vec![OpValDateTime::Not(v)])
  }

  pub fn in_<I>(v: I) -> Self
  where
    I: IntoIterator<Item = OffsetDateTime>,
  {
    Self(vec![OpValDateTime::In(v.into_iter().collect())])
  }

  pub fn not_in<I>(v: I) -> Self
  where
    I: IntoIterator<Item = OffsetDateTime>,
  {
    Self(vec![OpValDateTime::NotIn(v.into_iter().collect())])
  }

  pub fn lt(v: OffsetDateTime) -> Self {
    Self(vec![OpValDateTime::Lt(v)])
  }

  pub fn lte(v: OffsetDateTime) -> Self {
    Self(vec![OpValDateTime::Lte(v)])
  }

  pub fn gt(v: OffsetDateTime) -> Self {
    Self(vec![OpValDateTime::Gt(v)])
  }

  pub fn gte(v: OffsetDateTime) -> Self {
    Self(vec![OpValDateTime::Gte(v)])
  }

  pub fn null(v: bool) -> Self {
    Self(vec![OpValDateTime::Null(v)])
  }
}

#[derive(Debug, Clone)]
pub enum OpValDateTime {
  Eq(OffsetDateTime),
  Not(OffsetDateTime),

  In(Vec<OffsetDateTime>),
  NotIn(Vec<OffsetDateTime>),

  Lt(OffsetDateTime),
  Lte(OffsetDateTime),

  Gt(OffsetDateTime),
  Gte(OffsetDateTime),

  Null(bool),
}

impl From<OffsetDateTime> for OpValDateTime {
  fn from(value: OffsetDateTime) -> Self {
    OpValDateTime::Eq(value)
  }
}

impl From<OpValDateTime> for OpVal {
  fn from(value: OpValDateTime) -> Self {
    OpVal::Datetime(value)
  }
}

impl From<OffsetDateTime> for OpVal {
  fn from(value: OffsetDateTime) -> Self {
    OpValDateTime::Eq(value).into()
  }
}

#[cfg(feature = "with-sea-query")]
mod with_sea_query {
  use super::*;
  use crate::filter::{FilterNodeOptions, SeaResult, sea_is_col_value_null};
  use crate::sea_utils::into_node_value_expr;
  use sea_query::{BinOper, ColumnRef, ConditionExpression, SimpleExpr};

  impl OpValDateTime {
    pub fn into_sea_cond_expr(
      self,
      col: &ColumnRef,
      node_options: &FilterNodeOptions,
    ) -> SeaResult<ConditionExpression> {
      let binary_fn = |op: BinOper, v: OffsetDateTime| {
        let expr = into_node_value_expr(v, node_options);
        ConditionExpression::SimpleExpr(SimpleExpr::binary(col.clone().into(), op, expr))
      };

      let binaries_fn = |op: BinOper, v: Vec<OffsetDateTime>| {
        let vec_expr: Vec<SimpleExpr> = v.into_iter().map(|v| into_node_value_expr(v, node_options)).collect();
        let expr = SimpleExpr::Tuple(vec_expr);
        ConditionExpression::SimpleExpr(SimpleExpr::binary(col.clone().into(), op, expr))
      };

      let cond = match self {
        OpValDateTime::Eq(v) => binary_fn(BinOper::Equal, v),
        OpValDateTime::Not(v) => binary_fn(BinOper::NotEqual, v),
        OpValDateTime::In(v) => binaries_fn(BinOper::In, v),
        OpValDateTime::NotIn(v) => binaries_fn(BinOper::NotIn, v),
        OpValDateTime::Lt(v) => binary_fn(BinOper::SmallerThan, v),
        OpValDateTime::Lte(v) => binary_fn(BinOper::SmallerThanOrEqual, v),
        OpValDateTime::Gt(v) => binary_fn(BinOper::GreaterThan, v),
        OpValDateTime::Gte(v) => binary_fn(BinOper::GreaterThanOrEqual, v),
        OpValDateTime::Null(null) => sea_is_col_value_null(col.clone(), null),
      };

      Ok(cond)
    }
  }
}
