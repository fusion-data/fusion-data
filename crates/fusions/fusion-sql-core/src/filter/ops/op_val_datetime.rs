use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use super::OpVal;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "with-wasm", derive(tsify::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub struct OpValsDateTime {
  #[serde(rename = "$eq")]
  pub eq: Option<DateTime<FixedOffset>>,
  #[serde(rename = "$not")]
  pub not: Option<DateTime<FixedOffset>>,
  #[serde(rename = "$in")]
  pub in_: Option<Vec<DateTime<FixedOffset>>>,
  #[serde(rename = "$not_in")]
  pub not_in: Option<Vec<DateTime<FixedOffset>>>,
  #[serde(rename = "$lt")]
  pub lt: Option<DateTime<FixedOffset>>,
  #[serde(rename = "$lte")]
  pub lte: Option<DateTime<FixedOffset>>,
  #[serde(rename = "$gt")]
  pub gt: Option<DateTime<FixedOffset>>,
  #[serde(rename = "$gte")]
  pub gte: Option<DateTime<FixedOffset>>,
  #[serde(rename = "$null")]
  pub null: Option<bool>,
}

impl OpValsDateTime {
  pub fn eq(v: DateTime<FixedOffset>) -> Self {
    Self { eq: Some(v), ..Default::default() }
  }

  pub fn not(v: DateTime<FixedOffset>) -> Self {
    Self { not: Some(v), ..Default::default() }
  }

  pub fn in_<I>(v: I) -> Self
  where
    I: IntoIterator<Item = DateTime<FixedOffset>>,
  {
    Self { in_: Some(v.into_iter().collect()), ..Default::default() }
  }

  pub fn not_in<I>(v: I) -> Self
  where
    I: IntoIterator<Item = DateTime<FixedOffset>>,
  {
    Self { not_in: Some(v.into_iter().collect()), ..Default::default() }
  }

  pub fn lt(v: DateTime<FixedOffset>) -> Self {
    Self { lt: Some(v), ..Default::default() }
  }

  pub fn lte(v: DateTime<FixedOffset>) -> Self {
    Self { lte: Some(v), ..Default::default() }
  }

  pub fn gt(v: DateTime<FixedOffset>) -> Self {
    Self { gt: Some(v), ..Default::default() }
  }

  pub fn gte(v: DateTime<FixedOffset>) -> Self {
    Self { gte: Some(v), ..Default::default() }
  }

  pub fn null(v: bool) -> Self {
    Self { null: Some(v), ..Default::default() }
  }

  pub fn with_eq(mut self, v: DateTime<FixedOffset>) -> Self {
    self.eq = Some(v);
    self
  }

  pub fn with_not(mut self, v: DateTime<FixedOffset>) -> Self {
    self.not = Some(v);
    self
  }

  pub fn with_in<I>(mut self, v: I) -> Self
  where
    I: IntoIterator<Item = DateTime<FixedOffset>>,
  {
    self.in_ = Some(v.into_iter().collect());
    self
  }

  pub fn with_not_in<I>(mut self, v: I) -> Self
  where
    I: IntoIterator<Item = DateTime<FixedOffset>>,
  {
    self.not_in = Some(v.into_iter().collect());
    self
  }

  pub fn with_lt(mut self, v: DateTime<FixedOffset>) -> Self {
    self.lt = Some(v);
    self
  }

  pub fn with_lte(mut self, v: DateTime<FixedOffset>) -> Self {
    self.lte = Some(v);
    self
  }

  pub fn with_gt(mut self, v: DateTime<FixedOffset>) -> Self {
    self.gt = Some(v);
    self
  }

  pub fn with_gte(mut self, v: DateTime<FixedOffset>) -> Self {
    self.gte = Some(v);
    self
  }

  pub fn with_null(mut self, v: bool) -> Self {
    self.null = Some(v);
    self
  }
}

impl From<DateTime<FixedOffset>> for OpValsDateTime {
  fn from(value: DateTime<FixedOffset>) -> Self {
    Self::eq(value)
  }
}

impl From<OpValsDateTime> for OpVal {
  fn from(value: OpValsDateTime) -> Self {
    OpVal::DateTime(value)
  }
}

impl From<DateTime<FixedOffset>> for OpVal {
  fn from(value: DateTime<FixedOffset>) -> Self {
    Self::DateTime(OpValsDateTime::eq(value))
  }
}

#[cfg(feature = "with-sea-query")]
mod with_sea_query {
  use super::*;
  use crate::filter::{FilterNodeOptions, ForSeaCondition, OpValTrait, SeaResult, sea_is_col_value_null};
  use crate::sea_utils::into_node_value_expr;
  use sea_query::{BinOper, ColumnRef, ConditionExpression, SimpleExpr};

  impl OpValTrait for OpValsDateTime {
    fn to_condition_expressions(
      self,
      col: &ColumnRef,
      node_options: &FilterNodeOptions,
      _for_sea_condition: Option<&ForSeaCondition>,
    ) -> SeaResult<Vec<ConditionExpression>> {
      let binary_fn = |op: BinOper, v: DateTime<FixedOffset>| {
        let expr = into_node_value_expr(v, node_options);
        ConditionExpression::SimpleExpr(SimpleExpr::binary(col.clone().into(), op, expr))
      };

      let binaries_fn = |op: BinOper, v: Vec<DateTime<FixedOffset>>| {
        let vec_expr: Vec<SimpleExpr> = v.into_iter().map(|v| into_node_value_expr(v, node_options)).collect();
        let expr = SimpleExpr::Tuple(vec_expr);
        ConditionExpression::SimpleExpr(SimpleExpr::binary(col.clone().into(), op, expr))
      };

      let mut cond_exprs = Vec::new();
      if let Some(v) = self.eq {
        cond_exprs.push(binary_fn(BinOper::Equal, v));
      }
      if let Some(v) = self.not {
        cond_exprs.push(binary_fn(BinOper::NotEqual, v));
      }
      if let Some(v) = self.in_ {
        cond_exprs.push(binaries_fn(BinOper::In, v));
      }
      if let Some(v) = self.not_in {
        cond_exprs.push(binaries_fn(BinOper::NotIn, v));
      }
      if let Some(v) = self.lt {
        cond_exprs.push(binary_fn(BinOper::SmallerThan, v));
      }
      if let Some(v) = self.lte {
        cond_exprs.push(binary_fn(BinOper::SmallerThanOrEqual, v));
      }
      if let Some(v) = self.gt {
        cond_exprs.push(binary_fn(BinOper::GreaterThan, v));
      }
      if let Some(v) = self.gte {
        cond_exprs.push(binary_fn(BinOper::GreaterThanOrEqual, v));
      }
      if let Some(null) = self.null {
        cond_exprs.push(sea_is_col_value_null(col.clone(), null));
      }

      Ok(cond_exprs)
    }
  }
}
