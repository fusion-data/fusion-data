use std::ops::Deref;

use serde::{Deserialize, Serialize};

// region:    --- OrderBy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
#[serde(transparent)]
pub struct OrderBy(String);

impl OrderBy {
  pub fn to_sql(&self) -> String {
    if let Some(stripped) = self.0.strip_prefix('!') { format!("{} desc", stripped) } else { format!("{} asc", self.0) }
  }
}

impl From<&str> for OrderBy {
  fn from(value: &str) -> Self {
    Self(value.to_string())
  }
}

impl From<&String> for OrderBy {
  fn from(value: &String) -> Self {
    Self(value.to_string())
  }
}

impl From<String> for OrderBy {
  fn from(value: String) -> Self {
    Self(value)
  }
}

impl Deref for OrderBy {
  type Target = String;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
// endregion: --- OrderBy

// region:    --- OrderBys
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
#[serde(transparent)]
pub struct StaticOrderBys(pub &'static [&'static str]);

impl From<&'static [&'static str]> for StaticOrderBys {
  fn from(value: &'static [&'static str]) -> Self {
    Self(value)
  }
}

impl From<StaticOrderBys> for OrderBys {
  fn from(value: StaticOrderBys) -> Self {
    Self(value.0.iter().map(|s| OrderBy::from(*s)).collect())
  }
}

impl From<&StaticOrderBys> for OrderBys {
  fn from(value: &StaticOrderBys) -> Self {
    Self(value.0.iter().map(|s| OrderBy::from(*s)).collect())
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
#[serde(transparent)]
pub struct OrderBys(Vec<OrderBy>);

impl Default for OrderBys {
  fn default() -> Self {
    OrderBys::new(vec![])
  }
}

impl OrderBys {
  pub fn new(v: Vec<OrderBy>) -> Self {
    OrderBys(v)
  }

  pub fn into_inner(self) -> Vec<OrderBy> {
    self.0
  }

  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }
}

// This will allow us to iterate over &OrderBys
impl<'a> IntoIterator for &'a OrderBys {
  type Item = &'a OrderBy;
  type IntoIter = std::slice::Iter<'a, OrderBy>;

  fn into_iter(self) -> Self::IntoIter {
    self.0.iter()
  }
}

// This will allow us to iterate over OrderBys directly (consuming it)
impl IntoIterator for OrderBys {
  type Item = OrderBy;
  type IntoIter = std::vec::IntoIter<OrderBy>;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}

// NOTE: If we want the Vec<T> and T, we have to make the individual from
//       specific to the type. Otherwise, conflict.

impl From<&str> for OrderBys {
  fn from(val: &str) -> Self {
    OrderBys(vec![val.into()])
  }
}
impl From<&String> for OrderBys {
  fn from(val: &String) -> Self {
    OrderBys(vec![val.into()])
  }
}
impl From<String> for OrderBys {
  fn from(val: String) -> Self {
    OrderBys(vec![val.into()])
  }
}

impl From<OrderBy> for OrderBys {
  fn from(val: OrderBy) -> Self {
    OrderBys(vec![val])
  }
}

impl From<Vec<&str>> for OrderBys {
  fn from(val: Vec<&str>) -> Self {
    let d = val.into_iter().map(OrderBy::from).collect::<Vec<_>>();
    OrderBys(d)
  }
}

impl From<Vec<&String>> for OrderBys {
  fn from(val: Vec<&String>) -> Self {
    let d = val.into_iter().map(OrderBy::from).collect::<Vec<_>>();
    OrderBys(d)
  }
}

impl From<Vec<String>> for OrderBys {
  fn from(val: Vec<String>) -> Self {
    let d = val.into_iter().map(OrderBy::from).collect::<Vec<_>>();
    OrderBys(d)
  }
}

// endregion: --- OrderBys
