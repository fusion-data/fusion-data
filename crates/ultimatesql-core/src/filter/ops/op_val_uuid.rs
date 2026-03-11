use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::filter::OpVal;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub struct OpValUuid {
  #[serde(rename = "$eq")]
  #[cfg_attr(target_arch = "wasm32", tsify(type = "string"))]
  pub eq: Option<Uuid>,

  #[serde(rename = "$not")]
  #[cfg_attr(target_arch = "wasm32", tsify(type = "string"))]
  pub not: Option<Uuid>,

  #[serde(rename = "$in")]
  #[cfg_attr(target_arch = "wasm32", tsify(type = "string[]"))]
  pub in_: Option<Vec<Uuid>>,

  #[serde(rename = "$not_in")]
  #[cfg_attr(target_arch = "wasm32", tsify(type = "string[]"))]
  pub not_in: Option<Vec<Uuid>>,

  #[serde(rename = "$lt")]
  #[cfg_attr(target_arch = "wasm32", tsify(type = "string"))]
  pub lt: Option<Uuid>,

  #[serde(rename = "$lte")]
  #[cfg_attr(target_arch = "wasm32", tsify(type = "string"))]
  pub lte: Option<Uuid>,

  #[serde(rename = "$gt")]
  #[cfg_attr(target_arch = "wasm32", tsify(type = "string"))]
  pub gt: Option<Uuid>,

  #[serde(rename = "$gte")]
  #[cfg_attr(target_arch = "wasm32", tsify(type = "string"))]
  pub gte: Option<Uuid>,

  #[serde(rename = "$null")]
  pub null: Option<bool>,
}

impl OpValUuid {
  pub fn eq(uuid: impl Into<Uuid>) -> Self {
    Self { eq: Some(uuid.into()), ..Default::default() }
  }

  pub fn not(uuid: impl Into<Uuid>) -> Self {
    Self { not: Some(uuid.into()), ..Default::default() }
  }

  pub fn in_<I, U>(uuids: I) -> Self
  where
    I: IntoIterator<Item = U>,
    U: Into<Uuid>,
  {
    Self { in_: Some(uuids.into_iter().map(|u| u.into()).collect()), ..Default::default() }
  }

  pub fn not_in<I, U>(uuids: I) -> Self
  where
    I: IntoIterator<Item = U>,
    U: Into<Uuid>,
  {
    Self { not_in: Some(uuids.into_iter().map(|u| u.into()).collect()), ..Default::default() }
  }

  pub fn lt(uuid: Uuid) -> Self {
    Self { lt: Some(uuid), ..Default::default() }
  }

  pub fn lte(uuid: Uuid) -> Self {
    Self { lte: Some(uuid), ..Default::default() }
  }

  pub fn gt(uuid: Uuid) -> Self {
    Self { gt: Some(uuid), ..Default::default() }
  }

  pub fn gte(uuid: Uuid) -> Self {
    Self { gte: Some(uuid), ..Default::default() }
  }

  pub fn null(null: bool) -> Self {
    Self { null: Some(null), ..Default::default() }
  }
}

impl From<Uuid> for OpVal {
  fn from(value: Uuid) -> Self {
    Self::Uuid(OpValUuid::eq(value))
  }
}

impl From<&Uuid> for OpVal {
  fn from(value: &Uuid) -> Self {
    Self::Uuid(OpValUuid::eq(*value))
  }
}

impl From<OpValUuid> for OpVal {
  fn from(value: OpValUuid) -> Self {
    Self::Uuid(value)
  }
}

#[cfg(feature = "with-sea-query")]
mod with_sea_query {
  use sea_query::{BinOper, ColumnRef, ConditionExpression, SimpleExpr};

  use crate::filter::{FilterNodeOptions, ForSeaCondition, OpValTrait, SeaResult, sea_is_col_value_null};
  use crate::sea_utils::into_node_value_expr;

  use super::*;

  impl OpValTrait for OpValUuid {
    fn to_condition_expressions(
      self,
      col: &ColumnRef,
      node_options: &FilterNodeOptions,
      _for_sea_condition: Option<&ForSeaCondition>,
    ) -> SeaResult<Vec<ConditionExpression>> {
      let binary_fn = |op: BinOper, v: Uuid| {
        let expr = into_node_value_expr(v, node_options);
        ConditionExpression::SimpleExpr(SimpleExpr::binary(col.clone().into(), op, expr))
      };

      let binaries_fn = |op: BinOper, v: Vec<Uuid>| {
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
