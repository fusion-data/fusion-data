use super::*;

pub mod op_val_array;
pub mod op_val_bool;
pub mod op_val_datetime;
pub mod op_val_nums;
pub mod op_val_string;
#[cfg(feature = "with-uuid")]
pub mod op_val_uuid;
pub mod op_val_value;

// region:    --- OpVal
#[derive(Debug, Clone)]
pub enum OpVal {
  Bool(OpValBool),

  Int64(OpValInt64),
  ArrayInt64(OpValArrayInt64),

  Int32(OpValInt32),
  ArrayInt32(OpValArrayInt32),

  Float64(OpValFloat64),
  ArrayFloat64(OpValArrayFloat64),

  String(OpValString),
  ArrayString(OpValArrayString),

  Datetime(OpValDateTime),

  #[cfg(feature = "with-uuid")]
  Uuid(OpValUuid),

  Value(OpValValue),
}

// endregion: --- OpVal

// region:    --- From [Type]OpVal & Vec<[Type]OpVal> to [Type]OpVals

// Convenient implementation when single constraints.
// Common implementation
macro_rules! impl_from_for_opvals {
	($($ov:ident, $ovs:ident),*) => {
		$(
			impl From<$ov> for $ovs {
				fn from(val: $ov) -> Self {
					$ovs(vec![val])
				}
			}

			impl From<Vec<$ov>> for $ovs {
				fn from(val: Vec<$ov>) -> Self {
					$ovs(val)
				}
			}
		)*
	};
}

// For all opvals (must specified the pair as macro rules are hygienic)
impl_from_for_opvals!(
  // String
  OpValString,
  OpValsString,
  // Datetime
  OpValDateTime,
  OpValsDateTime,
  // Ints
  OpValInt64,
  OpValsInt64,
  OpValInt32,
  OpValsInt32,
  // Floats
  OpValFloat64,
  OpValsFloat64,
  // Bool
  OpValBool,
  OpValsBool,
  // OpValJson
  OpValValue,
  OpValsValue
);

// Uuid
#[cfg(feature = "with-uuid")]
impl_from_for_opvals!(OpValUuid, OpValsUuid);

// endregion: --- From [Type]OpVal & Vec<[Type]OpVal> to [Type]OpVals

#[cfg(feature = "with-sea-query")]
use sea_query::{ColumnRef, ConditionExpression, Expr};

#[cfg(feature = "with-sea-query")]
pub fn sea_is_col_value_null(col: ColumnRef, null: bool) -> ConditionExpression {
  if null { Expr::col(col).is_null().into() } else { Expr::col(col).is_not_null().into() }
}
