use crate::filter::OpVal;

/// - `ovs` OpValsType, e.g., `OpValsUint64`
/// - `ov` OpValType, e.g., `OpValUint64`
/// - `nt` Number type, e.g., `u64`
/// - `vr` Opval Variant e.g., `OpVal::Uint64`
macro_rules! impl_op_val {
	($(($ovs:ident, $ov:ident, $nt:ty, $vr:expr)),+) => {
		$(

#[derive(Debug, Clone)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct $ovs(pub Vec<$ov>);

impl $ovs {
	pub fn eq(v: $nt) -> Self {
		Self(vec![$ov::Eq(v)])
	}

	pub fn not(v: $nt) -> Self {
		Self(vec![$ov::Not(v)])
	}

	pub fn in_<I>(v: I) -> Self
	where
		I: IntoIterator<Item = $nt>,
	{
		Self(vec![$ov::In(v.into_iter().collect())])
	}

	pub fn not_in<I>(v: I) -> Self
	where
		I: IntoIterator<Item = $nt>,
	{
		Self(vec![$ov::NotIn(v.into_iter().collect())])
	}

	pub fn lt(v: $nt) -> Self {
		Self(vec![$ov::Lt(v)])
	}

	pub fn lte(v: $nt) -> Self {
		Self(vec![$ov::Lte(v)])
	}

	pub fn gt(v: $nt) -> Self {
		Self(vec![$ov::Gt(v)])
	}

	pub fn gte(v: $nt) -> Self {
		Self(vec![$ov::Gte(v)])
	}

	pub fn null(null: bool) -> Self {
		Self(vec![$ov::Null(null)])
	}
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub enum $ov {
	Eq($nt),
	Not($nt),
	In(Vec<$nt>),
	NotIn(Vec<$nt>),
	Lt($nt),
	Lte($nt),
	Gt($nt),
	Gte($nt),
	Null(bool),
}

// region:    --- Simple value to Eq e.g., OpValUint64
impl From<$nt> for $ov {
	fn from(val: $nt) -> Self {
		$ov::Eq(val)
	}
}

impl From<&$nt> for $ov {
	fn from(val: &$nt) -> Self {
		$ov::Eq(*val)
	}
}
// endregion: --- Simple value to Eq e.g., OpValUint64

// region:    --- Simple value to Eq e.g., OpValsUint64
impl From<$nt> for $ovs {
	fn from(val: $nt) -> Self {
		$ov::from(val).into()
	}
}

impl From<&$nt> for $ovs {
	fn from(val: &$nt) -> Self {
		$ov::from(*val).into()
	}
}
// endregion: --- Simple value to Eq e.g., OpValsUint64

// region:    --- e.g., OpValUint64 to OpVal
impl From<$ov> for OpVal {
	fn from(val: $ov) -> Self {
		$vr(val)
	}
}
// endregion: --- e.g., OpValUint64 to OpVal

// region:    --- Primitive to OpVal::Int(IntOpVal::Eq)
impl From<$nt> for OpVal {
	fn from(val: $nt) -> Self {
		$ov::Eq(val).into()
	}
}

impl From<&$nt> for OpVal {
	fn from(val: &$nt) -> Self {
		$ov::Eq(*val).into()
	}
}
// endregion: --- Primitive to OpVal::Int(IntOpVal::Eq)
		)+
	};
}

impl_op_val!(
  (OpValsInt64, OpValInt64, i64, OpVal::Int64),
  (OpValsInt32, OpValInt32, i32, OpVal::Int32),
  (OpValsFloat64, OpValFloat64, f64, OpVal::Float64)
);

#[cfg(feature = "with-sea-query")]
mod with_sea_query {
  use super::*;
  use crate::filter::{FilterNodeOptions, SeaResult, sea_is_col_value_null};
  use crate::sea_utils::into_node_value_expr;
  use sea_query::{BinOper, ColumnRef, ConditionExpression, SimpleExpr};

  macro_rules! impl_into_sea_op_val {
		($($ov:ident),+) => {
			$(
	impl $ov {
		pub fn into_sea_cond_expr(self, col: &ColumnRef, node_options: &FilterNodeOptions) -> SeaResult<ConditionExpression>  {
			let binary_fn = |op: BinOper, expr: SimpleExpr| {
				ConditionExpression::SimpleExpr(SimpleExpr::binary(col.clone().into(), op, expr))
			};
			let cond = match self {
				$ov::Eq(s) => binary_fn(BinOper::Equal, into_node_value_expr(s, node_options)),
				$ov::Not(s) => binary_fn(BinOper::NotEqual, into_node_value_expr(s, node_options)),
				$ov::In(s) => binary_fn(
					BinOper::In,
					SimpleExpr::Tuple(s.into_iter().map(|v| into_node_value_expr(v, node_options)).collect()),
				),
				$ov::NotIn(s) => binary_fn(
					BinOper::NotIn,
					SimpleExpr::Tuple(s.into_iter().map(|v| into_node_value_expr(v, node_options)).collect()),
				),
				$ov::Lt(s) => binary_fn(BinOper::SmallerThan, into_node_value_expr(s, node_options)),
				$ov::Lte(s) => binary_fn(BinOper::SmallerThanOrEqual, into_node_value_expr(s, node_options)),
				$ov::Gt(s) => binary_fn(BinOper::GreaterThan, into_node_value_expr(s, node_options)),
				$ov::Gte(s) => binary_fn(BinOper::GreaterThanOrEqual, into_node_value_expr(s, node_options)),

				$ov::Null(null) => sea_is_col_value_null(col.clone(), null),
			};

			Ok(cond)
		}
	}
			)+
		};
	}

  impl_into_sea_op_val!(OpValInt64, OpValInt32, OpValFloat64);
}
