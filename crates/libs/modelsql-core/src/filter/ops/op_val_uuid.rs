use uuid::Uuid;

use crate::filter::OpVal;

#[derive(Debug, Clone)]
pub struct OpValsUuid(pub Vec<OpValUuid>);
impl OpValsUuid {
  pub fn eq(uuid: impl Into<Uuid>) -> Self {
    Self(vec![OpValUuid::Eq(uuid.into())])
  }

  pub fn not(uuid: impl Into<Uuid>) -> Self {
    Self(vec![OpValUuid::Not(uuid.into())])
  }

  pub fn in_<I, U>(uuids: I) -> Self
  where
    I: IntoIterator<Item = U>,
    U: Into<Uuid>,
  {
    Self(vec![OpValUuid::In(uuids.into_iter().map(|u| u.into()).collect())])
  }

  pub fn not_in<I, U>(uuids: I) -> Self
  where
    I: IntoIterator<Item = U>,
    U: Into<Uuid>,
  {
    Self(vec![OpValUuid::NotIn(uuids.into_iter().map(|u| u.into()).collect())])
  }

  pub fn lt(uuid: Uuid) -> Self {
    Self(vec![OpValUuid::Lt(uuid)])
  }

  pub fn lte(uuid: Uuid) -> Self {
    Self(vec![OpValUuid::Lte(uuid)])
  }

  pub fn gt(uuid: Uuid) -> Self {
    Self(vec![OpValUuid::Gt(uuid)])
  }

  pub fn gte(uuid: Uuid) -> Self {
    Self(vec![OpValUuid::Gte(uuid)])
  }

  pub fn null(null: bool) -> Self {
    Self(vec![OpValUuid::Null(null)])
  }

  pub fn array_contains(uuid: Vec<Uuid>) -> Self {
    Self(vec![OpValUuid::ArrayContains(uuid)])
  }
}

#[derive(Debug, Clone)]
pub enum OpValUuid {
  Eq(Uuid),
  Not(Uuid),

  In(Vec<Uuid>),
  NotIn(Vec<Uuid>),

  Lt(Uuid),
  Lte(Uuid),

  Gt(Uuid),
  Gte(Uuid),

  Null(bool),

  // TODO: 为左操作数是 ARRAY 类型单独定义 OpVal 操作符？
  ArrayContains(Vec<Uuid>),
}

impl From<Uuid> for OpValUuid {
  fn from(value: Uuid) -> Self {
    OpValUuid::Eq(value)
  }
}

impl From<OpValUuid> for OpVal {
  fn from(value: OpValUuid) -> Self {
    OpVal::Uuid(value)
  }
}

impl From<Uuid> for OpVal {
  fn from(value: Uuid) -> Self {
    Self::Uuid(OpValUuid::Eq(value))
  }
}

#[cfg(feature = "with-sea-query")]
mod with_sea_query {
  use sea_query::extension::postgres::PgBinOper;
  use sea_query::{BinOper, ColumnRef, ConditionExpression, SimpleExpr};

  use crate::filter::{FilterNodeOptions, SeaResult, sea_is_col_value_null};
  use crate::sea_utils::into_node_value_expr;

  use super::*;

  impl OpValUuid {
    pub fn into_sea_cond_expr(
      self,
      col: &ColumnRef,
      node_options: &FilterNodeOptions,
    ) -> SeaResult<ConditionExpression> {
      let binary_fn = |op: BinOper, v: Uuid| {
        let expr = into_node_value_expr(v, node_options);
        ConditionExpression::SimpleExpr(SimpleExpr::binary(col.clone().into(), op, expr))
      };

      let binaries_fn = |op: BinOper, v: Vec<Uuid>| {
        let vec_expr: Vec<SimpleExpr> = v.into_iter().map(|v| into_node_value_expr(v, node_options)).collect();
        let expr = SimpleExpr::Tuple(vec_expr);
        ConditionExpression::SimpleExpr(SimpleExpr::binary(col.clone().into(), op, expr))
      };

      let cond = match self {
        OpValUuid::Eq(v) => binary_fn(BinOper::Equal, v),
        OpValUuid::Not(v) => binary_fn(BinOper::NotEqual, v),
        OpValUuid::In(v) => binaries_fn(BinOper::In, v),
        OpValUuid::NotIn(v) => binaries_fn(BinOper::NotIn, v),
        OpValUuid::Lt(v) => binary_fn(BinOper::SmallerThan, v),
        OpValUuid::Lte(v) => binary_fn(BinOper::SmallerThanOrEqual, v),
        OpValUuid::Gt(v) => binary_fn(BinOper::GreaterThan, v),
        OpValUuid::Gte(v) => binary_fn(BinOper::GreaterThanOrEqual, v),
        OpValUuid::Null(null) => sea_is_col_value_null(col.clone(), null),
        OpValUuid::ArrayContains(uuids) => ConditionExpression::SimpleExpr(SimpleExpr::binary(
          col.clone().into(),
          BinOper::PgOperator(PgBinOper::Contains),
          uuids,
        )),
      };

      Ok(cond)
    }
  }
}
