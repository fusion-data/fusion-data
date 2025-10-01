use chrono::{DateTime, FixedOffset};

use crate::filter::{
  OpValsBool, OpValsDateTime, OpValsFloat32, OpValsFloat64, OpValsInt32, OpValsInt64, OpValsString, OpValsValue,
  ops::OpVal,
};

pub trait IntoFilterNodes {
  fn filter_nodes(self, rel: Option<String>) -> Vec<FilterNode>;
}

#[derive(Debug, Clone, Default)]
pub struct FilterNodeOptions {
  pub cast_as: Option<String>, // for db casting. e.g., Will be applied to sea-query value.
}

#[derive(Clone)]
pub struct FilterNode {
  pub rel: Option<String>, // would be for the project.title (project in this case)
  pub name: String,
  pub opvals: OpVal,
  pub options: FilterNodeOptions,

  #[cfg(feature = "with-sea-query")]
  pub for_sea_condition: Option<crate::filter::ForSeaCondition>,
  #[cfg(not(feature = "with-sea-query"))]
  pub for_sea_condition: Option<()>,
}

impl FilterNode {
  pub fn new(name: impl Into<String>, opvals: impl Into<OpVal>) -> FilterNode {
    FilterNode {
      rel: None,
      name: name.into(),
      opvals: opvals.into(),
      options: FilterNodeOptions::default(),

      for_sea_condition: None,
    }
  }

  pub fn new_with_rel(rel: Option<String>, name: impl Into<String>, opvals: impl Into<OpVal>) -> FilterNode {
    FilterNode {
      rel,
      name: name.into(),
      opvals: opvals.into(),
      options: FilterNodeOptions::default(),

      for_sea_condition: None,
    }
  }
}

// region:    --- From Tuples (OpValType)
// Implements the From trait from tuples to FilterNode
macro_rules! from_tuples_opval {
	($($OV:ident),+) => {
		$(
			/// From trait from (prop_name, OpVal) for FilterNode
			/// (e.g., `let node: FilterNode = ("id", IntOpVal::Gt(1)).into()`)
			impl From<(&str, $OV)> for FilterNode {
				fn from((name, ov): (&str, $OV)) -> Self {
					FilterNode::new(name, ov)
				}
			}

      impl From<(String, $OV)> for FilterNode {
				fn from((name, ov): (String, $OV)) -> Self {
					FilterNode::new(name, ov)
				}
			}
		)+
	};
}
from_tuples_opval!(
  OpValsString,
  OpValsDateTime,
  // OpValsUint64,
  // OpValsUint32,
  OpValsInt64,
  OpValsInt32,
  OpValsFloat64,
  OpValsFloat32,
  OpValsValue,
  OpValsBool
);

#[cfg(feature = "with-uuid")]
use crate::filter::OpValsUuid;

#[cfg(feature = "with-uuid")]
from_tuples_opval!(OpValsUuid);
// endregion: --- From Tuples (OpValType)

// region:    --- From Tuples (Uuid val)
#[cfg(feature = "with-uuid")]
impl From<(&str, &uuid::Uuid)> for FilterNode {
  fn from((name, ov): (&str, &uuid::Uuid)) -> Self {
    let opvals = OpValsUuid::eq(*ov);
    FilterNode::new(name.to_string(), opvals)
  }
}

#[cfg(feature = "with-uuid")]
impl From<(&str, uuid::Uuid)> for FilterNode {
  fn from((name, ov): (&str, uuid::Uuid)) -> Self {
    let opvals = OpValsUuid::eq(ov);
    FilterNode::new(name.to_string(), opvals)
  }
}
// endregion: --- From Tuples (Uuid val)

// region:    --- From Tuples (String val)
impl From<(&str, &str)> for FilterNode {
  fn from((name, ov): (&str, &str)) -> Self {
    let opvals = OpValsString::eq(ov.to_string());
    FilterNode::new(name, opvals)
  }
}

impl From<(&str, String)> for FilterNode {
  fn from((name, ov): (&str, String)) -> Self {
    let opvals = OpValsString::eq(ov);
    FilterNode::new(name.to_string(), opvals)
  }
}
// endregion: --- From Tuples (String val)

// region:    --- From Tuples (DateTime val)
impl From<(&str, &DateTime<FixedOffset>)> for FilterNode {
  fn from((name, ov): (&str, &DateTime<FixedOffset>)) -> Self {
    let opvals = OpValsDateTime::eq(*ov);
    FilterNode::new(name.to_string(), opvals)
  }
}

impl From<(&str, DateTime<FixedOffset>)> for FilterNode {
  fn from((name, ov): (&str, DateTime<FixedOffset>)) -> Self {
    let opvals = OpValsDateTime::eq(ov);
    FilterNode::new(name.to_string(), opvals)
  }
}
// endregion: --- From Tuples (DateTime val)

// region:    --- From Tuples (num val)
// - `nt` e.g., `u64`
// - `ov` e.g., `OpValUint64`
macro_rules! from_tuples_num{
	($(($nt:ty, $ov:ident)),+) => {
		$(

impl From<(&str, $nt)> for FilterNode {
	fn from((name, ov): (&str, $nt)) -> Self {
		FilterNode::new(name, ov)
	}
}
		)+
	};
}

from_tuples_num!(
  // (u64, OpValsUint64),
  // (u32, OpValsUint32),
  (i64, OpValsInt64),
  (i32, OpValsInt32),
  (f32, OpValsFloat32),
  (f64, OpValsFloat64)
);
// endregion: --- From Tuples (num val)

// region:    --- From Tuples (bool val)
impl From<(&str, bool)> for FilterNode {
  fn from((name, ov): (&str, bool)) -> Self {
    let opvals = OpValsBool::eq(ov);
    FilterNode::new(name.to_string(), opvals)
  }
}
// endregion: --- From Tuples (bool val)

#[cfg(feature = "with-sea-query")]
mod with_sea_query {
  use sea_query::{ColumnRef, ConditionExpression, IntoColumnRef, IntoIden};

  use crate::filter::SeaResult;
  use crate::sea_utils::StringIden;

  use super::*;

  impl FilterNode {
    pub fn into_sea_cond_expr_list(self) -> SeaResult<Vec<ConditionExpression>> {
      let col: ColumnRef = match self.rel {
        Some(rel) => ColumnRef::TableColumn(StringIden(rel).into_iden(), StringIden(self.name).into_iden()),
        None => StringIden(self.name).into_column_ref(),
      };
      let for_sea_condition = self.for_sea_condition.as_ref();
      let node_options = &self.options;
      let node_sea_exprs = self.opvals.to_condition_expressions(&col, node_options, for_sea_condition)?;
      Ok(node_sea_exprs)
    }
  }
}
