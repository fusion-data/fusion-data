use crate::filter::OpVal;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct OpValsString(pub Vec<OpValString>);

impl OpValsString {
  pub fn eq(val: impl Into<String>) -> Self {
    Self(vec![OpValString::Eq(val.into())])
  }

  pub fn not(val: impl Into<String>) -> Self {
    Self(vec![OpValString::Not(val.into())])
  }

  pub fn in_<I, S>(vals: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    Self(vec![OpValString::In(vals.into_iter().map(|v| v.into()).collect())])
  }

  pub fn not_in<I, S>(vals: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    Self(vec![OpValString::NotIn(vals.into_iter().map(|v| v.into()).collect())])
  }

  pub fn lt(val: impl Into<String>) -> Self {
    Self(vec![OpValString::Lt(val.into())])
  }

  pub fn lte(val: impl Into<String>) -> Self {
    Self(vec![OpValString::Lte(val.into())])
  }

  pub fn gt(val: impl Into<String>) -> Self {
    Self(vec![OpValString::Gt(val.into())])
  }

  pub fn gte(val: impl Into<String>) -> Self {
    Self(vec![OpValString::Gte(val.into())])
  }

  pub fn contains(val: impl Into<String>) -> Self {
    Self(vec![OpValString::Contains(val.into())])
  }

  pub fn not_contains(val: impl Into<String>) -> Self {
    Self(vec![OpValString::NotContains(val.into())])
  }

  pub fn contains_any<I, S>(vals: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    Self(vec![OpValString::ContainsAny(vals.into_iter().map(|v| v.into()).collect())])
  }

  pub fn not_contains_any<I, S>(vals: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    Self(vec![OpValString::NotContainsAny(vals.into_iter().map(|v| v.into()).collect())])
  }

  pub fn contains_all<I, S>(vals: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    Self(vec![OpValString::ContainsAll(vals.into_iter().map(|v| v.into()).collect())])
  }

  pub fn starts_with(val: impl Into<String>) -> Self {
    Self(vec![OpValString::StartsWith(val.into())])
  }

  pub fn not_starts_with(val: impl Into<String>) -> Self {
    Self(vec![OpValString::NotStartsWith(val.into())])
  }

  pub fn starts_with_any<I, S>(vals: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    Self(vec![OpValString::StartsWithAny(vals.into_iter().map(|v| v.into()).collect())])
  }

  pub fn not_starts_with_any<I, S>(vals: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    Self(vec![OpValString::NotStartsWithAny(vals.into_iter().map(|v| v.into()).collect())])
  }

  pub fn ends_with(val: impl Into<String>) -> Self {
    Self(vec![OpValString::EndsWith(val.into())])
  }

  pub fn not_ends_with(val: impl Into<String>) -> Self {
    Self(vec![OpValString::NotEndsWith(val.into())])
  }

  pub fn ends_with_any<I, S>(vals: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    Self(vec![OpValString::EndsWithAny(vals.into_iter().map(|v| v.into()).collect())])
  }

  pub fn not_ends_with_any<I, S>(vals: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    Self(vec![OpValString::NotEndsWithAny(vals.into_iter().map(|v| v.into()).collect())])
  }

  pub fn empty(val: bool) -> Self {
    Self(vec![OpValString::Empty(val)])
  }

  pub fn null(val: bool) -> Self {
    Self(vec![OpValString::Null(val)])
  }

  pub fn contains_ci(val: impl Into<String>) -> Self {
    Self(vec![OpValString::ContainsCi(val.into())])
  }

  pub fn not_contains_ci(val: impl Into<String>) -> Self {
    Self(vec![OpValString::NotContainsCi(val.into())])
  }

  pub fn starts_with_ci(val: impl Into<String>) -> Self {
    Self(vec![OpValString::StartsWithCi(val.into())])
  }

  pub fn not_starts_with_ci(val: impl Into<String>) -> Self {
    Self(vec![OpValString::NotStartsWithCi(val.into())])
  }

  pub fn ends_with_ci(val: impl Into<String>) -> Self {
    Self(vec![OpValString::EndsWithCi(val.into())])
  }

  pub fn not_ends_with_ci(val: impl Into<String>) -> Self {
    Self(vec![OpValString::NotEndsWithCi(val.into())])
  }

  pub fn ilike(val: impl Into<String>) -> Self {
    Self(vec![OpValString::Ilike(val.into())])
  }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub enum OpValString {
  Eq(String),
  Not(String),

  In(Vec<String>),
  NotIn(Vec<String>),

  Lt(String),
  Lte(String),

  Gt(String),
  Gte(String),

  Contains(String),
  NotContains(String),

  ContainsAny(Vec<String>),
  NotContainsAny(Vec<String>),

  ContainsAll(Vec<String>),
  NotContainsAll(Vec<String>),

  StartsWith(String),
  NotStartsWith(String),

  StartsWithAny(Vec<String>),
  NotStartsWithAny(Vec<String>),

  EndsWith(String),
  NotEndsWith(String),

  EndsWithAny(Vec<String>),
  NotEndsWithAny(Vec<String>),

  Empty(bool),
  Null(bool),

  ContainsCi(String),
  NotContainsCi(String),

  StartsWithCi(String),
  NotStartsWithCi(String),

  EndsWithCi(String),
  NotEndsWithCi(String),

  Ilike(String),
}

// region:    --- Simple value to Eq OpValString
impl From<String> for OpValString {
  fn from(val: String) -> Self {
    OpValString::Eq(val)
  }
}

impl From<&str> for OpValString {
  fn from(val: &str) -> Self {
    OpValString::Eq(val.to_string())
  }
}
// endregion: --- Simple value to Eq OpValString

// region:    --- Simple value to Eq OpValStrings
impl From<String> for OpValsString {
  fn from(val: String) -> Self {
    OpValString::from(val).into()
  }
}

impl From<&str> for OpValsString {
  fn from(val: &str) -> Self {
    OpValString::from(val).into()
  }
}
// endregion: --- Simple value to Eq OpValStrings

// region:    --- StringOpVal to OpVal
impl From<OpValString> for OpVal {
  fn from(val: OpValString) -> Self {
    OpVal::String(val)
  }
}
// endregion: --- StringOpVal to OpVal

// region:    --- Primitive to OpVal::String(StringOpVal::Eq)
impl From<String> for OpVal {
  fn from(val: String) -> Self {
    OpValString::Eq(val).into()
  }
}

impl From<&str> for OpVal {
  fn from(val: &str) -> Self {
    OpValString::Eq(val.to_string()).into()
  }
}
// endregion: --- Primitive to OpVal::String(StringOpVal::Eq)

#[cfg(feature = "with-sea-query")]
mod with_sea_query {
  use super::*;
  use crate::filter::{FilterNodeOptions, SeaResult, sea_is_col_value_null};
  use crate::sea_utils::into_node_value_expr;
  use sea_query::{BinOper, ColumnRef, Condition, ConditionExpression, Expr, Func, SimpleExpr};

  #[cfg(feature = "with-ilike")]
  use sea_query::extension::postgres::PgBinOper;

  impl OpValString {
    pub fn into_sea_cond_expr(
      self,
      col: &ColumnRef,
      node_options: &FilterNodeOptions,
    ) -> SeaResult<ConditionExpression> {
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

      let cond = match self {
        OpValString::Eq(s) => binary_fn(BinOper::Equal, s),
        OpValString::Not(s) => binary_fn(BinOper::NotEqual, s),
        OpValString::In(s) => binaries_fn(BinOper::In, s),
        OpValString::NotIn(s) => binaries_fn(BinOper::NotIn, s),
        OpValString::Lt(s) => binary_fn(BinOper::SmallerThan, s),
        OpValString::Lte(s) => binary_fn(BinOper::SmallerThanOrEqual, s),
        OpValString::Gt(s) => binary_fn(BinOper::GreaterThan, s),
        OpValString::Gte(s) => binary_fn(BinOper::GreaterThanOrEqual, s),

        OpValString::Contains(s) => binary_fn(BinOper::Like, format!("%{s}%")),

        OpValString::NotContains(s) => binary_fn(BinOper::NotLike, format!("%{s}%")),

        OpValString::ContainsAll(values) => {
          let mut cond = Condition::all();
          for value in values {
            let expr = binary_fn(BinOper::Like, format!("%{value}%"));
            cond = cond.add(expr);
          }
          ConditionExpression::Condition(cond)
        }
        OpValString::NotContainsAll(values) => {
          let mut cond = Condition::any();
          for value in values {
            let expr = binary_fn(BinOper::Like, format!("%{value}%"));
            cond = cond.add(expr);
          }
          ConditionExpression::Condition(cond.not())
        }

        OpValString::ContainsAny(values) => cond_any_of_fn(BinOper::Like, values, "%", "%"),
        OpValString::NotContainsAny(values) => cond_any_of_fn(BinOper::NotLike, values, "%", "%"),

        OpValString::StartsWith(s) => binary_fn(BinOper::Like, format!("{s}%")),
        OpValString::StartsWithAny(values) => cond_any_of_fn(BinOper::Like, values, "", "%"),

        OpValString::NotStartsWith(s) => binary_fn(BinOper::NotLike, format!("{s}%")),
        OpValString::NotStartsWithAny(values) => cond_any_of_fn(BinOper::NotLike, values, "", "%"),

        OpValString::EndsWith(s) => binary_fn(BinOper::Like, format!("%{s}")),
        OpValString::EndsWithAny(values) => cond_any_of_fn(BinOper::Like, values, "%", ""),

        OpValString::NotEndsWith(s) => binary_fn(BinOper::Like, format!("%{s}")),
        OpValString::NotEndsWithAny(values) => cond_any_of_fn(BinOper::NotLike, values, "%", ""),

        OpValString::Null(null) => sea_is_col_value_null(col.clone(), null),
        OpValString::Empty(empty) => {
          let op = if empty { BinOper::Equal } else { BinOper::NotEqual };
          Condition::any()
            .add(sea_is_col_value_null(col.clone(), empty))
            .add(binary_fn(op, "".to_string()))
            .into()
        }

        OpValString::ContainsCi(s) => case_insensitive_fn(BinOper::Like, format!("%{s}%")),
        OpValString::NotContainsCi(s) => case_insensitive_fn(BinOper::NotLike, format!("%{s}%")),

        OpValString::StartsWithCi(s) => case_insensitive_fn(BinOper::Like, format!("{s}%")),
        OpValString::NotStartsWithCi(s) => case_insensitive_fn(BinOper::NotLike, format!("{s}%")),

        OpValString::EndsWithCi(s) => case_insensitive_fn(BinOper::Like, format!("%{s}")),
        OpValString::NotEndsWithCi(s) => case_insensitive_fn(BinOper::NotLike, format!("%{s}")),

        OpValString::Ilike(s) => {
          #[cfg(feature = "with-ilike")]
          {
            pg_binary_fn(PgBinOper::ILike, format!("%{s}%"))
          }
          #[cfg(not(feature = "with-ilike"))]
          {
            case_insensitive_fn(BinOper::Like, format!("%{s}%"))
          }
        }
      };

      Ok(cond)
    }
  }
}
