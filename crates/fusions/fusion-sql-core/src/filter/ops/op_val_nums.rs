use serde::{Deserialize, Serialize};

use crate::filter::OpVal;

/// - `ovs` OpValsType, e.g., `OpValsUint64`
/// - `ov` OpValType, e.g., `OpValUint64`
/// - `nt` Number type, e.g., `u64`
/// - `vr` Opval Variant e.g., `OpVal::Uint64`
macro_rules! impl_op_val {
	($(($ovs:ident, $nt:ty, $vr:expr)),+) => {
		$(

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "with-wasm", derive(tsify::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub struct $ovs {
	#[serde(rename = "$eq")]
	pub eq: Option<$nt>,
	#[serde(rename = "$not")]
	pub not: Option<$nt>,
	#[serde(rename = "$in")]
	pub in_: Option<Vec<$nt>>,
	#[serde(rename = "$not_in")]
	pub not_in: Option<Vec<$nt>>,
	#[serde(rename = "$lt")]
	pub lt: Option<$nt>,
	#[serde(rename = "$lte")]
	pub lte: Option<$nt>,
	#[serde(rename = "$gt")]
	pub gt: Option<$nt>,
	#[serde(rename = "$gte")]
	pub gte: Option<$nt>,
	#[serde(rename = "$null")]
	pub null: Option<bool>,
}

impl $ovs {
	pub fn eq(v: $nt) -> Self {
		Self { eq: Some(v), ..Default::default() }
	}

	pub fn not(v: $nt) -> Self {
		Self { not: Some(v), ..Default::default() }
	}

	pub fn in_<I>(v: I) -> Self
	where
		I: IntoIterator<Item = $nt>,
	{
		let vs = v.into_iter().collect::<Vec<_>>();
		Self { in_: if vs.is_empty() { None } else { Some(vs) }, ..Default::default() }
	}

	pub fn not_in<I>(v: I) -> Self
	where
		I: IntoIterator<Item = $nt>,
	{
		let vs = v.into_iter().collect::<Vec<_>>();
		Self { not_in: if vs.is_empty() { None } else { Some(vs) }, ..Default::default() }
	}

	pub fn lt(v: $nt) -> Self {
		Self { lt: Some(v), ..Default::default() }
	}

	pub fn lte(v: $nt) -> Self {
		Self { lte: Some(v), ..Default::default() }
	}

	pub fn gt(v: $nt) -> Self {
		Self { gt: Some(v), ..Default::default() }
	}

	pub fn gte(v: $nt) -> Self {
		Self { gte: Some(v), ..Default::default() }
	}

	pub fn null(null: bool) -> Self {
		Self { null: Some(null), ..Default::default() }
	}
}

// // region:    --- Simple value to Eq e.g., OpValUint64
// impl From<$nt> for $ov {
// 	fn from(val: $nt) -> Self {
// 		$ov::Eq(val)
// 	}
// }

// impl From<&$nt> for $ov {
// 	fn from(val: &$nt) -> Self {
// 		$ovs::eq(*val)
// 	}
// }
// // endregion: --- Simple value to Eq e.g., OpValUint64

// region:    --- Simple value to Eq e.g., OpValsUint64
impl From<$nt> for $ovs {
	fn from(val: $nt) -> Self {
		$ovs::eq(val)
	}
}

impl From<&$nt> for $ovs {
	fn from(val: &$nt) -> Self {
		$ovs::eq(*val)
	}
}
// endregion: --- Simple value to Eq e.g., OpValsUint64

impl From<$ovs> for OpVal {
	fn from(val: $ovs) -> Self {
		$vr(val)
	}
}

// region:    --- Primitive to OpVal::Int(IntOpVal::Eq)
impl From<$nt> for OpVal {
	fn from(val: $nt) -> Self {
		$vr($ovs::eq(val))
	}
}

impl From<&$nt> for OpVal {
	fn from(val: &$nt) -> Self {
		$vr($ovs::eq(*val))
	}
}
// endregion: --- Primitive to OpVal::Int(IntOpVal::Eq)
		)+
	};
}

impl_op_val!(
  // (OpValsUInt64, u64, OpVal::UInt64),
  // (OpValsUInt32, u32, OpVal::UInt32),
  (OpValsInt64, i64, OpVal::Int64),
  (OpValsInt32, i32, OpVal::Int32),
  (OpValsFloat64, f64, OpVal::Float64),
  (OpValsFloat32, f32, OpVal::Float32)
);

#[cfg(feature = "with-sea-query")]
mod with_sea_query {
  use sea_query::{BinOper, ColumnRef, ConditionExpression, SimpleExpr};

  use crate::filter::{FilterNodeOptions, ForSeaCondition, OpValTrait, SeaResult, sea_is_col_value_null};
  use crate::sea_utils::into_node_value_expr;

  use super::*;

  macro_rules! impl_into_sea_op_val {
		($($ov:ident),+) => {
			$(
	impl OpValTrait for $ov {
    fn to_condition_expressions(
      self,
      col: &ColumnRef,
      node_options: &FilterNodeOptions,
      _for_sea_condition: Option<&ForSeaCondition>,
    ) -> SeaResult<Vec<ConditionExpression>> {
			let binary_fn = |op: BinOper, expr: SimpleExpr| {
				ConditionExpression::SimpleExpr(SimpleExpr::binary(col.clone().into(), op, expr))
			};

      let mut expressions = Vec::new();
      if let Some(val) = self.eq {
        expressions.push(binary_fn(BinOper::Equal, into_node_value_expr(val, node_options)));
      }
      if let Some(val) = self.not {
        expressions.push(binary_fn(BinOper::NotEqual, into_node_value_expr(val, node_options)));
      }
      if let Some(val) = self.in_ {
				if !val.is_empty() {
					expressions.push(binary_fn(
						BinOper::In,
						SimpleExpr::Tuple(val.into_iter().map(|v| into_node_value_expr(v, node_options)).collect()),
					));
				}
      }
      if let Some(val) = self.not_in {
				if !val.is_empty() {
					expressions.push(binary_fn(
						BinOper::NotIn,
						SimpleExpr::Tuple(val.into_iter().map(|v| into_node_value_expr(v, node_options)).collect()),
					));
				}
      }
			if let Some(val) = self.lt {
				expressions.push(binary_fn(
					BinOper::SmallerThan,
					into_node_value_expr(val, node_options),
				));
			}
			if let Some(val) = self.lte {
				expressions.push(binary_fn(
					BinOper::SmallerThanOrEqual,
					into_node_value_expr(val, node_options),
				));
			}
			if let Some(val) = self.gt {
				expressions.push(binary_fn(
					BinOper::GreaterThan,
					into_node_value_expr(val, node_options),
				));
			}
			if let Some(val) = self.gte {
				expressions.push(binary_fn(
					BinOper::GreaterThanOrEqual,
					into_node_value_expr(val, node_options),
				));
			}
			if let Some(val) = self.null {
				expressions.push(sea_is_col_value_null(col.clone(), val));
			}

      Ok(expressions)
    }
	}
			)+
		};
	}

  impl_into_sea_op_val!(
    // OpValsUInt64,
    // OpValsUInt32,
    OpValsInt64,
    OpValsInt32,
    OpValsFloat64,
    OpValsFloat32
  );
}
