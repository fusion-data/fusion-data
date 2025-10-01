use crate::filter::OpVal;

macro_rules! impl_array_op_val {
  ($(($ovs:ident, $ov:ident, $nt:ty, $vr:expr)),+) => {
		$(

 #[derive(Debug, Clone)]
 #[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
 pub struct $ovs(pub Vec<$ov>);

 impl $ovs {
	pub fn eq(vs: $nt) -> Self
 {
		Self(vec![$ov::Eq(vs)])
	}

	pub fn not(vs: $nt) -> Self
 {
		Self(vec![$ov::Not(vs)])
	}

	pub fn contains(vs: $nt) -> Self
	{
		Self(vec![$ov::Contains(vs)])
	}

	pub fn contained(vs: $nt) -> Self
	{
		Self(vec![$ov::Contained(vs)])
	}

 }

 #[derive(Debug, Clone)]
 #[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
 pub enum $ov {
	Eq($nt),
	Not($nt),
	Contains($nt),
	Contained($nt),
 }

 // region:    --- Simple value to Eq e.g., OpValUint64
 impl From<$nt> for $ov {
	fn from(vs: $nt) -> Self {
		$ov::Eq(vs)
	}
 }

 impl From<&$nt> for $ov {
	fn from(vs: &$nt) -> Self {
		$ov::Eq(vs.clone())
	}
 }
 // endregion: --- Simple value to Eq e.g., OpValUint64

 // region:    --- Simple value to Eq e.g., OpValUint64
 impl From<$nt> for $ovs {
	fn from(val: $nt) -> Self {
		$ovs(vec![$ov::from(val)])
	}
 }

 impl From<&$nt> for $ovs {
	fn from(vs: &$nt) -> Self {
		$ovs(vec![$ov::from(vs.clone())])
	}
 }
 // endregion: --- Simple value to Eq e.g., OpValUint64

 // region:    --- e.g., OpValUint64 to OpVal
 impl From<$ov> for OpVal {
	fn from(val: $ov) -> Self {
		$vr(val)
	}
 }
 // endregion: --- e.g., OpValUint64 to OpVal

 // region:    --- Primitive to OpVal::Int(IntOpVal::Eq)
 impl From<$nt> for OpVal {
	fn from(vs: $nt) -> Self {
		$ov::Eq(vs).into()
	}
 }

 impl From<&$nt> for OpVal {
	fn from(vs: &$nt) -> Self {
		$ov::Eq(vs.clone()).into()
	}
 }
 // endregion: --- Primitive to OpVal::Int(IntOpVal::Eq)
		)+
	};
}

impl_array_op_val!(
  (OpValArrayInt64, OpValArrayInt64, Vec<i64>, OpVal::ArrayInt64),
  (OpValArrayInt32, OpValArrayInt32, Vec<i32>, OpVal::ArrayInt32),
  (OpValArrayFloat64, OpValArrayFloat64, Vec<f64>, OpVal::ArrayFloat64),
  (OpValArrayString, OpValArrayString, Vec<String>, OpVal::ArrayString)
);

#[cfg(feature = "with-sea-query")]
mod with_sea_query {
  use super::*;
  use crate::filter::{FilterNodeOptions, SeaResult};
  use sea_query::extension::postgres::PgBinOper;
  use sea_query::{BinOper, ColumnRef, ConditionExpression, SimpleExpr};

  fn binary_fn<T>(col: &ColumnRef, op: BinOper, expr: T) -> ConditionExpression
  where
    T: Into<SimpleExpr>,
  {
    ConditionExpression::SimpleExpr(SimpleExpr::binary(col.clone().into(), op, expr.into()))
  }

  macro_rules! impl_into_sea_op_val {
		($($ov:ident),+) => {
			$(
	impl $ov {
		pub fn into_sea_cond_expr(self, col: &ColumnRef, _node_options: &FilterNodeOptions) -> SeaResult<ConditionExpression>  {
			let cond = match self {
				$ov::Eq(arr) => binary_fn(col, BinOper::Equal, arr),
				$ov::Not(arr) => binary_fn(col, BinOper::NotEqual, arr),
				$ov::Contains(arr) => binary_fn(col, BinOper::PgOperator(PgBinOper::Contains), arr),
				$ov::Contained(arr) => binary_fn(col, BinOper::PgOperator(PgBinOper::Contained), arr),
			};

			Ok(cond)
		}
	}
			)+
		};
	}

  impl_into_sea_op_val!(OpValArrayInt64, OpValArrayInt32, OpValArrayFloat64, OpValArrayString);
}
