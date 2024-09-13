use derive_more::derive::Display;
use modql::{field::HasSeaFields, filter::FilterNode};
use sea_query::SimpleExpr;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, FromRow};
use uuid::Uuid;

pub trait DbRowType: HasSeaFields + for<'r> FromRow<'r, PgRow> + Unpin + Send {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display)]
#[serde(untagged)]
pub enum Id {
  I32(i32),
  I64(i64),
  String(String),
  Uuid(Uuid),
}

impl Id {
  pub fn to_filter_node(&self, col: &str) -> FilterNode {
    match self {
      Id::I32(id) => (col, *id).into(),
      Id::I64(id) => (col, *id).into(),
      Id::String(id) => (col, id).into(),
      Id::Uuid(id) => (col, id.to_string()).into(),
    }
  }
}

impl From<Id> for FilterNode {
  fn from(id: Id) -> Self {
    id.to_filter_node("id")
  }
}

impl From<Id> for SimpleExpr {
  fn from(value: Id) -> Self {
    match value {
      Id::I32(id) => SimpleExpr::Value(id.into()),
      Id::I64(id) => SimpleExpr::Value(id.into()),
      Id::String(id) => SimpleExpr::Value(id.into()),
      Id::Uuid(id) => SimpleExpr::Value(id.into()),
    }
  }
}

impl From<i32> for Id {
  fn from(value: i32) -> Self {
    Id::I32(value)
  }
}

impl From<i64> for Id {
  fn from(value: i64) -> Self {
    Id::I64(value)
  }
}

impl From<String> for Id {
  fn from(value: String) -> Self {
    Id::String(value)
  }
}

impl From<&str> for Id {
  fn from(value: &str) -> Self {
    Id::String(value.to_string())
  }
}

pub fn to_vec_id<V, I>(ids: I) -> Vec<Id>
where
  V: Into<Id>,
  I: IntoIterator<Item = V>,
{
  ids.into_iter().map(|v| v.into()).collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(PartialEq, Serialize, Deserialize)]
  struct TestModel {
    pub role_id: i32,
    pub user_id: i64,
    pub order_id: Uuid,
    pub dict_id: String,
  }

  #[test]
  fn test_id() -> anyhow::Result<()> {
    let id = Id::I32(32);
    println!("id is {id}");
    let order_id = Id::Uuid(Uuid::now_v7());
    println!("order id is {order_id}");
    assert_eq!("32", serde_json::to_string(&id)?);
    assert_eq!(serde_json::to_string(&Id::String("abcdefg".into()))?, r#""abcdefg""#);

    let tm = TestModel {
      role_id: 53,
      user_id: 2309457238947,
      order_id: Uuid::now_v7(),
      dict_id: "system.run.mode".to_string(),
    };

    let v = serde_json::to_value(tm)?;
    let role_id: i32 = serde_json::from_value(v.get("role_id").unwrap().clone())?;
    assert_eq!(role_id, 53);

    Ok(())
  }
}
