use super::*;

// pub mod op_val_array;
pub mod op_val_bool;
pub mod op_val_datetime;
// pub mod op_val_nums;
pub mod op_val_string;
// #[cfg(feature = "with-uuid")]
// pub mod op_val_uuid;
pub mod op_val_value;

#[cfg(feature = "with-sea-query")]
use sea_query::{ConditionExpression, Expr};

pub trait OpValTrait: Clone {
  #[cfg(feature = "with-sea-query")]
  fn to_condition_expressions(
    self,
    col: &sea_query::ColumnRef,
    node_options: &FilterNodeOptions,
    for_sea_condition: Option<&ForSeaCondition>,
  ) -> SeaResult<Vec<ConditionExpression>>;
}

// region:    --- OpVal
#[derive(Debug, Clone)]
pub enum OpVal {
  Bool(OpValsBool),
  // Int64(OpValInt64),
  // ArrayInt64(OpValArrayInt64),

  // Int32(OpValInt32),
  // ArrayInt32(OpValArrayInt32),

  // Float64(OpValFloat64),
  // ArrayFloat64(OpValArrayFloat64),
  String(OpValsString),
  // ArrayString(OpValArrayString),
  Datetime(OpValsDateTime),

  // #[cfg(feature = "with-uuid")]
  // Uuid(OpValUuid),
  Value(OpValsValue),
}

#[cfg(feature = "with-sea-query")]
mod with_sea_query {

  use sea_query::ConditionExpression;

  use crate::filter::{FilterNodeOptions, SeaResult};

  use super::*;

  impl OpVal {
    pub fn to_condition_expressions(
      self,
      col: &sea_query::ColumnRef,
      node_options: &FilterNodeOptions,
      for_sea_condition: Option<&ForSeaCondition>,
    ) -> SeaResult<Vec<ConditionExpression>> {
      match self {
        Self::Bool(op_vals) => op_vals.to_condition_expressions(col, node_options, for_sea_condition),
        Self::Datetime(op_vals) => op_vals.to_condition_expressions(col, node_options, for_sea_condition),
        Self::String(op_vals) => op_vals.to_condition_expressions(col, node_options, for_sea_condition),
        Self::Value(op_vals) => op_vals.to_condition_expressions(col, node_options, for_sea_condition),
      }
    }
  }
}

// endregion: --- OpVal

// region:    --- From [Type]OpVal & Vec<[Type]OpVal> to [Type]OpVals

// Convenient implementation when single constraints.
// Common implementation
// macro_rules! impl_from_for_opvals {
// 	($($ov:ident, $ovs:ident),*) => {
// 		$(
// 			impl From<$ov> for $ovs {
// 				fn from(val: $ov) -> Self {
// 					$ovs(vec![val])
// 				}
// 			}

// 			impl From<Vec<$ov>> for $ovs {
// 				fn from(val: Vec<$ov>) -> Self {
// 					$ovs(val)
// 				}
// 			}
// 		)*
// 	};
// }

// For all opvals (must specified the pair as macro rules are hygienic)
// impl_from_for_opvals!(
// String
// OpValString,
// OpValsString,
// Datetime
// OpValDateTime,
// OpValsDateTime,
// Ints
// OpValInt64,
// OpValsInt64,
// OpValInt32,
// OpValsInt32,
// Floats
// OpValFloat64,
// OpValsFloat64,
// Bool
// OpValBool,
// OpValJson
// OpValValue,
// OpValsValue,
// OpValsBool
// );

// Uuid
#[cfg(feature = "with-uuid")]
impl_from_for_opvals!(OpValUuid, OpValsUuid);

// endregion: --- From [Type]OpVal & Vec<[Type]OpVal> to [Type]OpVals

#[cfg(feature = "with-sea-query")]
pub fn sea_is_col_value_null(col: sea_query::ColumnRef, null: bool) -> ConditionExpression {
  if null { Expr::col(col).is_null().into() } else { Expr::col(col).is_not_null().into() }
}
