use chrono::NaiveTime;
use serde::{Deserialize, Serialize};

use super::OpVal;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub struct OpValTime {
  #[serde(rename = "$eq")]
  #[cfg_attr(target_arch = "wasm32", tsify(type = "string"))]
  pub eq: Option<NaiveTime>,

  #[serde(rename = "$not")]
  #[cfg_attr(target_arch = "wasm32", tsify(type = "string"))]
  pub not: Option<NaiveTime>,

  #[serde(rename = "$in")]
  #[cfg_attr(target_arch = "wasm32", tsify(type = "string[]"))]
  pub in_: Option<Vec<NaiveTime>>,

  #[serde(rename = "$not_in")]
  #[cfg_attr(target_arch = "wasm32", tsify(type = "string[]"))]
  pub not_in: Option<Vec<NaiveTime>>,

  #[serde(rename = "$lt")]
  #[cfg_attr(target_arch = "wasm32", tsify(type = "string"))]
  pub lt: Option<NaiveTime>,

  #[serde(rename = "$lte")]
  #[cfg_attr(target_arch = "wasm32", tsify(type = "string"))]
  pub lte: Option<NaiveTime>,

  #[serde(rename = "$gt")]
  #[cfg_attr(target_arch = "wasm32", tsify(type = "string"))]
  pub gt: Option<NaiveTime>,

  #[serde(rename = "$gte")]
  #[cfg_attr(target_arch = "wasm32", tsify(type = "string"))]
  pub gte: Option<NaiveTime>,

  #[serde(rename = "$null")]
  pub null: Option<bool>,
}

impl OpValTime {
  pub fn eq(v: NaiveTime) -> Self {
    Self { eq: Some(v), ..Default::default() }
  }

  pub fn not(v: NaiveTime) -> Self {
    Self { not: Some(v), ..Default::default() }
  }

  pub fn in_<I>(v: I) -> Self
  where
    I: IntoIterator<Item = NaiveTime>,
  {
    Self { in_: Some(v.into_iter().collect()), ..Default::default() }
  }

  pub fn not_in<I>(v: I) -> Self
  where
    I: IntoIterator<Item = NaiveTime>,
  {
    Self { not_in: Some(v.into_iter().collect()), ..Default::default() }
  }

  pub fn lt(v: NaiveTime) -> Self {
    Self { lt: Some(v), ..Default::default() }
  }

  pub fn lte(v: NaiveTime) -> Self {
    Self { lte: Some(v), ..Default::default() }
  }

  pub fn gt(v: NaiveTime) -> Self {
    Self { gt: Some(v), ..Default::default() }
  }

  pub fn gte(v: NaiveTime) -> Self {
    Self { gte: Some(v), ..Default::default() }
  }

  pub fn null(v: bool) -> Self {
    Self { null: Some(v), ..Default::default() }
  }

  pub fn with_eq(mut self, v: NaiveTime) -> Self {
    self.eq = Some(v);
    self
  }

  pub fn with_not(mut self, v: NaiveTime) -> Self {
    self.not = Some(v);
    self
  }

  pub fn with_in<I>(mut self, v: I) -> Self
  where
    I: IntoIterator<Item = NaiveTime>,
  {
    self.in_ = Some(v.into_iter().collect());
    self
  }

  pub fn with_not_in<I>(mut self, v: I) -> Self
  where
    I: IntoIterator<Item = NaiveTime>,
  {
    self.not_in = Some(v.into_iter().collect());
    self
  }

  pub fn with_lt(mut self, v: NaiveTime) -> Self {
    self.lt = Some(v);
    self
  }

  pub fn with_lte(mut self, v: NaiveTime) -> Self {
    self.lte = Some(v);
    self
  }

  pub fn with_gt(mut self, v: NaiveTime) -> Self {
    self.gt = Some(v);
    self
  }

  pub fn with_gte(mut self, v: NaiveTime) -> Self {
    self.gte = Some(v);
    self
  }

  pub fn with_null(mut self, v: bool) -> Self {
    self.null = Some(v);
    self
  }
}

impl From<NaiveTime> for OpValTime {
  fn from(value: NaiveTime) -> Self {
    Self::eq(value)
  }
}

impl From<OpValTime> for OpVal {
  fn from(value: OpValTime) -> Self {
    OpVal::Time(value)
  }
}

impl From<NaiveTime> for OpVal {
  fn from(value: NaiveTime) -> Self {
    Self::Time(OpValTime::eq(value))
  }
}

#[cfg(feature = "with-sea-query")]
mod with_sea_query {
  use super::*;
  use crate::filter::{FilterNodeOptions, ForSeaCondition, OpValTrait, SeaResult, sea_is_col_value_null};
  use crate::sea_utils::into_node_value_expr;
  use sea_query::{BinOper, ColumnRef, ConditionExpression, SimpleExpr};

  impl OpValTrait for OpValTime {
    fn to_condition_expressions(
      self,
      col: &ColumnRef,
      node_options: &FilterNodeOptions,
      _for_sea_condition: Option<&ForSeaCondition>,
    ) -> SeaResult<Vec<ConditionExpression>> {
      let binary_fn = |op: BinOper, v: NaiveTime| {
        let expr = into_node_value_expr(v, node_options);
        ConditionExpression::SimpleExpr(SimpleExpr::binary(col.clone().into(), op, expr))
      };

      let binaries_fn = |op: BinOper, v: Vec<NaiveTime>| {
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
