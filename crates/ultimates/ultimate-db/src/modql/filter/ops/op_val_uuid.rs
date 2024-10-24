use uuid::Uuid;

use crate::modql::filter::OpVal;

#[derive(Debug, Clone)]
pub struct OpValsUuid(pub Vec<OpValUuid>);

#[derive(Debug, Clone)]
pub enum OpValUuid {
  Eq(Uuid),
  Not(Uuid),

  In(Vec<Uuid>),
  NotIn(Vec<Uuid>),

  Null(bool),
}

impl From<Uuid> for OpValUuid {
  fn from(value: Uuid) -> Self {
    OpValUuid::Eq(value)
  }
}

impl From<OpValUuid> for OpVal {
  fn from(value: OpValUuid) -> Self {
    OpVal::Uuid(value)
  }
}

impl From<Uuid> for OpVal {
  fn from(value: Uuid) -> Self {
    OpValUuid::Eq(value).into()
  }
}

mod json {
  use std::str::FromStr;

  use crate::modql::filter::json::OpValueToOpValType;
  use crate::modql::filter::OpValUuid;
  use crate::modql::{Error, Result};
  use serde_json::Value;
  use uuid::Uuid;

  impl OpValueToOpValType for OpValUuid {
    fn op_value_to_op_val_type(op: &str, value: serde_json::Value) -> crate::modql::Result<Self>
    where
      Self: Sized,
    {
      fn into_uuids(value: Value) -> Result<Vec<Uuid>> {
        let mut values = Vec::new();

        let Value::Array(array) = value else {
          return Err(Error::JsonValArrayWrongType { actual_value: value });
        };

        for item in array.into_iter() {
          if let Value::String(item) = item {
            values.push(parse_to_uuid(&item)?);
          } else {
            return Err(Error::JsonValArrayItemNotOfType { expected_type: "Uuid", actual_value: item });
          }
        }

        Ok(values)
      }

      let ov = match (op, value) {
        ("$eq", Value::String(string_v)) => OpValUuid::Eq(parse_to_uuid(&string_v)?),
        ("$in", value) => OpValUuid::In(into_uuids(value)?),

        ("$not", Value::String(string_v)) => OpValUuid::Not(parse_to_uuid(&string_v)?),
        ("$notIn", value) => OpValUuid::NotIn(into_uuids(value)?),

        ("$null", Value::Bool(v)) => OpValUuid::Null(v),

        (_, v) => return Err(Error::JsonOpValNotSupported { operator: op.to_string(), value: v }),
      };
      Ok(ov)
    }
  }

  fn parse_to_uuid(value: &str) -> Result<Uuid> {
    Uuid::from_str(value).map_err(|_| Error::JsonValNotOfType("Uuid"))
  }
}

mod with_sea_query {
  use super::*;
  use crate::modql::filter::{sea_is_col_value_null, FilterNodeOptions, SeaResult};
  use crate::modql::into_node_value_expr;
  use sea_query::{BinOper, ColumnRef, Condition, ConditionExpression, Expr, Func, SimpleExpr};

  impl OpValUuid {
    pub fn into_sea_cond_expr(
      self,
      col: &ColumnRef,
      node_options: &FilterNodeOptions,
    ) -> SeaResult<ConditionExpression> {
      let binary_fn = |op: BinOper, v: Uuid| {
        let vxpr = into_node_value_expr(v, node_options);
        ConditionExpression::SimpleExpr(SimpleExpr::binary(col.clone().into(), op, vxpr))
      };

      let binaries_fn = |op: BinOper, v: Vec<Uuid>| {
        let vxpr_list: Vec<SimpleExpr> = v.into_iter().map(|v| into_node_value_expr(v, node_options)).collect();
        let vxpr = SimpleExpr::Tuple(vxpr_list);
        ConditionExpression::SimpleExpr(SimpleExpr::binary(col.clone().into(), op, vxpr))
      };

      let cond = match self {
        OpValUuid::Eq(s) => binary_fn(BinOper::Equal, s),
        OpValUuid::Not(s) => binary_fn(BinOper::NotEqual, s),
        OpValUuid::In(s) => binaries_fn(BinOper::In, s),
        OpValUuid::NotIn(s) => binaries_fn(BinOper::NotIn, s),

        OpValUuid::Null(null) => sea_is_col_value_null(col.clone(), null),
      };

      Ok(cond)
    }
  }
}
