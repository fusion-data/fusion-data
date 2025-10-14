use serde::{Deserialize, Serialize};

use crate::filter::OpVal;

macro_rules! impl_array_op_val {
  ($(($ovs:ident, $nt:ty, $vr:expr)),+) => {
		$(
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct $ovs {
  pub eq: Option<$nt>,
  pub not: Option<$nt>,
  pub contains: Option<$nt>,
  pub contained: Option<$nt>,
}

impl $ovs {
	pub fn eq(vs: $nt) -> Self {
		Self { eq: Some(vs), ..Default::default() }
	}

	pub fn not(vs: $nt) -> Self {
		Self { not: Some(vs), ..Default::default() }
	}

	pub fn contains(vs: $nt) -> Self {
		Self { contains: Some(vs), ..Default::default() }
	}

	pub fn contained(vs: $nt) -> Self {
		Self { contained: Some(vs), ..Default::default() }
	}

	pub fn with_eq(mut self, vs: $nt) -> Self {
		self.eq = Some(vs);
		self
	}

	pub fn with_not(mut self, vs: $nt) -> Self {
		self.not = Some(vs);
		self
	}

	pub fn with_contains(mut self, vs: $nt) -> Self {
		self.contains = Some(vs);
		self
	}

	pub fn with_contained(mut self, vs: $nt) -> Self {
		self.contained = Some(vs);
		self
	}
}

// region:    --- Simple value to Eq e.g., OpValUint64
impl From<$nt> for $ovs {
  fn from(val: $nt) -> Self {
  	$ovs::eq(val)
  }
}

impl From<&$nt> for $ovs {
  fn from(vs: &$nt) -> Self {
  	$ovs::eq(vs.clone())
  }
}
// endregion: --- Simple value to Eq e.g., OpValUint64

// region:    --- e.g., OpValUint64 to OpVal
impl From<$ovs> for OpVal {
  fn from(val: $ovs) -> Self {
  	$vr(Box::new(val))
  }
}
// endregion: --- e.g., OpValUint64 to OpVal

// region:    --- Primitive to OpVal::Int(IntOpVal::Eq)
impl From<$nt> for OpVal {
fn from(vs: $nt) -> Self {
	$ovs::eq(vs).into()
}
}

impl From<&$nt> for OpVal {
fn from(vs: &$nt) -> Self {
	$ovs::eq(vs.clone()).into()
}
}
 // endregion: --- Primitive to OpVal::Int(IntOpVal::Eq)
		)+
	};
}

impl_array_op_val!(
  (OpValArrayInt64, Vec<i64>, OpVal::ArrayInt64),
  (OpValArrayInt32, Vec<i32>, OpVal::ArrayInt32),
  (OpValArrayFloat64, Vec<f64>, OpVal::ArrayFloat64),
  (OpValArrayFloat32, Vec<f32>, OpVal::ArrayFloat32),
  (OpValArrayString, Vec<String>, OpVal::ArrayString)
);

#[cfg(feature = "with-sea-query")]
mod with_sea_query {
  use sea_query::extension::postgres::PgBinOper;
  use sea_query::{BinOper, ColumnRef, ConditionExpression, SimpleExpr};

  use crate::filter::{FilterNodeOptions, ForSeaCondition, SeaResult};

  use super::*;

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
  		pub fn to_condition_expressions(self, col: &ColumnRef, _node_options: &FilterNodeOptions, _for_sea_condition: Option<&ForSeaCondition>) -> SeaResult<Vec<ConditionExpression>>  {
  		  let mut cond_exprs = Vec::new();

  			if let Some(arr) = self.eq {
  				cond_exprs.push(binary_fn(col, BinOper::Equal, arr));
  			}
  			if let Some(arr) = self.not {
  				cond_exprs.push(binary_fn(col, BinOper::NotEqual, arr));
  			}
  			if let Some(arr) = self.contains {
  				cond_exprs.push(binary_fn(col, BinOper::PgOperator(PgBinOper::Contains), arr));
  			}
  			if let Some(arr) = self.contained {
  				cond_exprs.push(binary_fn(col, BinOper::PgOperator(PgBinOper::Contained), arr));
  			}

  			Ok(cond_exprs)
  		}
  	}
			)+
		};
	}

  impl_into_sea_op_val!(OpValArrayInt32, OpValArrayInt64, OpValArrayFloat32, OpValArrayFloat64, OpValArrayString);
}
