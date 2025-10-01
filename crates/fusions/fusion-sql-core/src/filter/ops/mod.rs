use super::*;

// pub mod op_val_array;
pub mod op_val_bool;
pub mod op_val_datetime;
pub mod op_val_nums;
pub mod op_val_string;
#[cfg(feature = "with-uuid")]
pub mod op_val_uuid;
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
  Int64(OpValsInt64),
  // ArrayInt64(Box<OpValsArrayInt64>),
  Int32(OpValsInt32),
  // ArrayInt32(Box<OpValsArrayInt32>),
  Float64(OpValsFloat64),
  // ArrayFloat64(Box<OpValsArrayFloat64>),
  Float32(OpValsFloat32),
  // ArrayFloat32(Box<OpValsArrayFloat32>),
  String(Box<OpValsString>),
  // ArrayString(Box<OpValsArrayString>),
  DateTime(OpValsDateTime),
  #[cfg(feature = "with-uuid")]
  Uuid(OpValsUuid),
  Value(Box<OpValsValue>),
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
        Self::DateTime(op_vals) => op_vals.to_condition_expressions(col, node_options, for_sea_condition),
        Self::String(op_vals) => op_vals.to_condition_expressions(col, node_options, for_sea_condition),
        Self::Int32(op_vals) => op_vals.to_condition_expressions(col, node_options, for_sea_condition),
        Self::Int64(op_vals) => op_vals.to_condition_expressions(col, node_options, for_sea_condition),
        Self::Float64(op_vals) => op_vals.to_condition_expressions(col, node_options, for_sea_condition),
        Self::Float32(op_vals) => op_vals.to_condition_expressions(col, node_options, for_sea_condition),
        #[cfg(feature = "with-uuid")]
        Self::Uuid(op_vals) => op_vals.to_condition_expressions(col, node_options, for_sea_condition),
        Self::Bool(op_vals) => op_vals.to_condition_expressions(col, node_options, for_sea_condition),
        Self::Value(op_vals) => op_vals.to_condition_expressions(col, node_options, for_sea_condition),
      }
    }
  }
}

// endregion: --- OpVal

#[cfg(feature = "with-sea-query")]
pub fn sea_is_col_value_null(col: sea_query::ColumnRef, null: bool) -> ConditionExpression {
  if null { Expr::col(col).is_null().into() } else { Expr::col(col).is_not_null().into() }
}
