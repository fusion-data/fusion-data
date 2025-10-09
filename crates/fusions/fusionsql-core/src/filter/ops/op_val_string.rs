use serde::{Deserialize, Serialize};

use crate::filter::OpVal;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub struct OpValString {
  #[serde(rename = "$eq")]
  pub eq: Option<String>,
  #[serde(rename = "$not")]
  pub not: Option<String>,
  #[serde(rename = "$in")]
  pub in_: Option<Vec<String>>,
  #[serde(rename = "$notIn")]
  pub not_in: Option<Vec<String>>,
  #[serde(rename = "$lt")]
  pub lt: Option<String>,
  #[serde(rename = "$lte")]
  pub lte: Option<String>,
  #[serde(rename = "$gt")]
  pub gt: Option<String>,
  #[serde(rename = "$gte")]
  pub gte: Option<String>,
  #[serde(rename = "$contains")]
  pub contains: Option<String>,
  #[serde(rename = "$notContains")]
  pub not_contains: Option<String>,
  #[serde(rename = "$containsAny")]
  pub contains_any: Option<Vec<String>>,
  #[serde(rename = "$notContainsAny")]
  pub not_contains_any: Option<Vec<String>>,
  #[serde(rename = "$containsAll")]
  pub contains_all: Option<Vec<String>>,
  #[serde(rename = "$notContainsAll")]
  pub not_contains_all: Option<Vec<String>>,
  #[serde(rename = "$startsWith")]
  pub starts_with: Option<String>,
  #[serde(rename = "$notStartsWith")]
  pub not_starts_with: Option<String>,
  #[serde(rename = "$startsWithAny")]
  pub starts_with_any: Option<Vec<String>>,
  #[serde(rename = "$notStartsWithAny")]
  pub not_starts_with_any: Option<Vec<String>>,
  #[serde(rename = "$endsWith")]
  pub ends_with: Option<String>,
  #[serde(rename = "$notEndsWith")]
  pub not_ends_with: Option<String>,
  #[serde(rename = "$endsWithAny")]
  pub ends_with_any: Option<Vec<String>>,
  #[serde(rename = "$notEndsWithAny")]
  pub not_ends_with_any: Option<Vec<String>>,
  #[serde(rename = "$empty")]
  pub empty: Option<bool>,
  #[serde(rename = "$null")]
  pub null: Option<bool>,
  #[serde(rename = "$containsCi")]
  pub contains_ci: Option<String>,
  #[serde(rename = "$notContainsCi")]
  pub not_contains_ci: Option<String>,
  #[serde(rename = "$startsWithCi")]
  pub starts_with_ci: Option<String>,
  #[serde(rename = "$notStartsWithCi")]
  pub not_starts_with_ci: Option<String>,
  #[serde(rename = "$endsWithCi")]
  pub ends_with_ci: Option<String>,
  #[serde(rename = "$notEndsWithCi")]
  pub not_ends_with_ci: Option<String>,
  #[serde(rename = "$ilike")]
  pub ilike: Option<String>,
}

impl OpValString {
  pub fn eq(val: impl Into<String>) -> Self {
    Self { eq: Some(val.into()), ..Default::default() }
  }

  pub fn not(val: impl Into<String>) -> Self {
    Self { not: Some(val.into()), ..Default::default() }
  }

  pub fn in_<I, S>(vals: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    Self { in_: Some(vals.into_iter().map(|v| v.into()).collect()), ..Default::default() }
  }

  pub fn not_in<I, S>(vals: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    Self { not_in: Some(vals.into_iter().map(|v| v.into()).collect()), ..Default::default() }
  }

  pub fn lt(val: impl Into<String>) -> Self {
    Self { lt: Some(val.into()), ..Default::default() }
  }

  pub fn lte(val: impl Into<String>) -> Self {
    Self { lte: Some(val.into()), ..Default::default() }
  }

  pub fn gt(val: impl Into<String>) -> Self {
    Self { gt: Some(val.into()), ..Default::default() }
  }

  pub fn gte(val: impl Into<String>) -> Self {
    Self { gte: Some(val.into()), ..Default::default() }
  }

  pub fn contains(val: impl Into<String>) -> Self {
    Self { contains: Some(val.into()), ..Default::default() }
  }

  pub fn not_contains(val: impl Into<String>) -> Self {
    Self { not_contains: Some(val.into()), ..Default::default() }
  }

  pub fn contains_any<I, S>(vals: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    Self { contains_any: Some(vals.into_iter().map(|v| v.into()).collect()), ..Default::default() }
  }

  pub fn not_contains_any<I, S>(vals: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    Self { not_contains_any: Some(vals.into_iter().map(|v| v.into()).collect()), ..Default::default() }
  }

  pub fn contains_all<I, S>(vals: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    Self { contains_all: Some(vals.into_iter().map(|v| v.into()).collect()), ..Default::default() }
  }

  pub fn starts_with(val: impl Into<String>) -> Self {
    Self { starts_with: Some(val.into()), ..Default::default() }
  }

  pub fn not_starts_with(val: impl Into<String>) -> Self {
    Self { not_starts_with: Some(val.into()), ..Default::default() }
  }

  pub fn starts_with_any<I, S>(vals: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    Self { starts_with_any: Some(vals.into_iter().map(|v| v.into()).collect()), ..Default::default() }
  }

  pub fn not_starts_with_any<I, S>(vals: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    Self { not_starts_with_any: Some(vals.into_iter().map(|v| v.into()).collect()), ..Default::default() }
  }

  pub fn ends_with(val: impl Into<String>) -> Self {
    Self { ends_with: Some(val.into()), ..Default::default() }
  }

  pub fn not_ends_with(val: impl Into<String>) -> Self {
    Self { not_ends_with: Some(val.into()), ..Default::default() }
  }

  pub fn ends_with_any<I, S>(vals: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    Self { ends_with_any: Some(vals.into_iter().map(|v| v.into()).collect()), ..Default::default() }
  }

  pub fn not_ends_with_any<I, S>(vals: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    Self { not_ends_with_any: Some(vals.into_iter().map(|v| v.into()).collect()), ..Default::default() }
  }

  pub fn empty(val: bool) -> Self {
    Self { empty: Some(val), ..Default::default() }
  }

  pub fn null(val: bool) -> Self {
    Self { null: Some(val), ..Default::default() }
  }

  pub fn contains_ci(val: impl Into<String>) -> Self {
    Self { contains_ci: Some(val.into()), ..Default::default() }
  }

  pub fn not_contains_ci(val: impl Into<String>) -> Self {
    Self { not_contains_ci: Some(val.into()), ..Default::default() }
  }

  pub fn starts_with_ci(val: impl Into<String>) -> Self {
    Self { starts_with_ci: Some(val.into()), ..Default::default() }
  }

  pub fn not_starts_with_ci(val: impl Into<String>) -> Self {
    Self { not_starts_with_ci: Some(val.into()), ..Default::default() }
  }

  pub fn ends_with_ci(val: impl Into<String>) -> Self {
    Self { ends_with_ci: Some(val.into()), ..Default::default() }
  }

  pub fn not_ends_with_ci(val: impl Into<String>) -> Self {
    Self { not_ends_with_ci: Some(val.into()), ..Default::default() }
  }

  pub fn ilike(val: impl Into<String>) -> Self {
    Self { ilike: Some(val.into()), ..Default::default() }
  }
}

// region:    --- Simple value to Eq OpValString
impl From<String> for OpValString {
  fn from(val: String) -> Self {
    OpValString::eq(val)
  }
}

impl From<&str> for OpValString {
  fn from(val: &str) -> Self {
    OpValString::eq(val.to_string())
  }
}
// endregion: --- Simple value to Eq OpValString

// region:    --- StringOpVal to OpVal
impl From<OpValString> for OpVal {
  fn from(val: OpValString) -> Self {
    OpVal::String(Box::new(val))
  }
}
// endregion: --- StringOpVal to OpVal

// region:    --- Primitive to OpVal::String(StringOpVal::Eq)
impl From<String> for OpVal {
  fn from(val: String) -> Self {
    OpVal::String(Box::new(OpValString::eq(val)))
  }
}

impl From<&str> for OpVal {
  fn from(val: &str) -> Self {
    OpVal::String(Box::new(OpValString::eq(val.to_string())))
  }
}
// endregion: --- Primitive to OpVal::String(StringOpVal::Eq)

#[cfg(feature = "with-sea-query")]
mod with_sea_query {
  use super::*;
  use crate::filter::{FilterNodeOptions, ForSeaCondition, OpValTrait, SeaResult, sea_is_col_value_null};
  use crate::sea_utils::into_node_value_expr;
  use sea_query::{BinOper, ColumnRef, Condition, ConditionExpression, Expr, Func, SimpleExpr};

  #[cfg(feature = "with-ilike")]
  use sea_query::extension::postgres::PgBinOper;

  impl OpValTrait for OpValString {
    fn to_condition_expressions(
      self,
      col: &ColumnRef,
      node_options: &FilterNodeOptions,
      _for_sea_condition: Option<&ForSeaCondition>,
    ) -> SeaResult<Vec<ConditionExpression>> {
      let binary_fn = |op: BinOper, v: String| {
        let expr = into_node_value_expr(v, node_options);
        ConditionExpression::SimpleExpr(SimpleExpr::binary(col.clone().into(), op, expr))
      };

      #[cfg(feature = "with-ilike")]
      let pg_binary_fn = |op: PgBinOper, v: String| {
        let expr = into_node_value_expr(v, node_options);
        ConditionExpression::SimpleExpr(SimpleExpr::binary(col.clone().into(), BinOper::PgOperator(op), expr))
      };

      let binaries_fn = |op: BinOper, v: Vec<String>| {
        let vec_expr: Vec<SimpleExpr> = v.into_iter().map(|v| into_node_value_expr(v, node_options)).collect();
        let expr = SimpleExpr::Tuple(vec_expr);
        ConditionExpression::SimpleExpr(SimpleExpr::binary(col.clone().into(), op, expr))
      };

      let cond_any_of_fn = |op: BinOper, values: Vec<String>, val_prefix: &str, val_suffix: &str| {
        let mut cond = Condition::any();

        for value in values {
          let expr = binary_fn(op, format!("{val_prefix}{value}{val_suffix}"));
          cond = cond.add(expr);
        }

        ConditionExpression::Condition(cond)
      };

      let case_insensitive_fn = |op: BinOper, v: String| {
        let expr = SimpleExpr::Value(v.into());
        let col_expr = SimpleExpr::FunctionCall(Func::lower(Expr::col(col.clone())));
        let value_expr = SimpleExpr::FunctionCall(Func::lower(expr));
        ConditionExpression::SimpleExpr(SimpleExpr::binary(col_expr, op, value_expr))
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
      if let Some(v) = self.contains {
        cond_exprs.push(binary_fn(BinOper::Like, format!("%{}%", v)));
      }
      if let Some(v) = self.not_contains {
        cond_exprs.push(binary_fn(BinOper::NotLike, format!("%{}%", v)));
      }
      if let Some(v) = self.not_contains_any {
        cond_exprs.push(cond_any_of_fn(BinOper::NotLike, v, "%", "%"));
      }
      if let Some(v) = self.contains_any {
        cond_exprs.push(cond_any_of_fn(BinOper::Like, v, "%", "%"));
      }
      if let Some(values) = self.contains_all {
        let mut cond = Condition::all();
        for value in values {
          let expr = binary_fn(BinOper::Like, format!("%{value}%"));
          cond = cond.add(expr);
        }
        cond_exprs.push(ConditionExpression::Condition(cond));
      }
      if let Some(values) = self.not_contains_all {
        let mut cond = Condition::any();
        for value in values {
          let expr = binary_fn(BinOper::Like, format!("%{value}%"));
          cond = cond.add(expr);
        }
        cond_exprs.push(ConditionExpression::Condition(cond.not()));
      }
      if let Some(s) = self.starts_with {
        cond_exprs.push(binary_fn(BinOper::Like, format!("{s}%")));
      }
      if let Some(values) = self.starts_with_any {
        cond_exprs.push(cond_any_of_fn(BinOper::Like, values, "", "%"));
      }
      if let Some(s) = self.not_starts_with {
        cond_exprs.push(binary_fn(BinOper::NotLike, format!("{s}%")));
      }
      if let Some(values) = self.not_starts_with_any {
        cond_exprs.push(cond_any_of_fn(BinOper::NotLike, values, "", "%"));
      }
      if let Some(s) = self.ends_with {
        cond_exprs.push(binary_fn(BinOper::Like, format!("%{s}")));
      }
      if let Some(values) = self.ends_with_any {
        cond_exprs.push(cond_any_of_fn(BinOper::Like, values, "%", ""));
      }
      if let Some(s) = self.not_ends_with {
        cond_exprs.push(binary_fn(BinOper::NotLike, format!("%{s}")));
      }
      if let Some(values) = self.not_ends_with_any {
        cond_exprs.push(cond_any_of_fn(BinOper::NotLike, values, "%", ""));
      }
      if let Some(null) = self.null {
        cond_exprs.push(sea_is_col_value_null(col.clone(), null));
      }
      if let Some(empty) = self.empty {
        let op = if empty { BinOper::Equal } else { BinOper::NotEqual };
        let expression = Condition::any()
          .add(sea_is_col_value_null(col.clone(), empty))
          .add(binary_fn(op, "".to_string()))
          .into();
        cond_exprs.push(expression);
      }
      if let Some(s) = self.contains_ci {
        cond_exprs.push(case_insensitive_fn(BinOper::Like, format!("%{s}%")));
      }
      if let Some(s) = self.not_contains_ci {
        cond_exprs.push(case_insensitive_fn(BinOper::NotLike, format!("%{s}%")));
      }
      if let Some(s) = self.starts_with_ci {
        cond_exprs.push(case_insensitive_fn(BinOper::Like, format!("{s}%")));
      }
      if let Some(s) = self.not_starts_with_ci {
        cond_exprs.push(case_insensitive_fn(BinOper::NotLike, format!("{s}%")));
      }
      if let Some(s) = self.ends_with_ci {
        cond_exprs.push(case_insensitive_fn(BinOper::Like, format!("%{s}")));
      }
      if let Some(s) = self.not_ends_with_ci {
        cond_exprs.push(case_insensitive_fn(BinOper::NotLike, format!("%{s}")));
      }
      if let Some(s) = self.ilike {
        #[cfg(feature = "with-ilike")]
        let expression = pg_binary_fn(PgBinOper::ILike, format!("%{s}%"));

        #[cfg(not(feature = "with-ilike"))]
        let expression = case_insensitive_fn(BinOper::Like, format!("%{s}%"));

        cond_exprs.push(expression);
      }

      Ok(cond_exprs)
    }
  }
}
