use ultimate_api::v1::{SortBy, SortDirection};

// region:    --- OrderBy
#[derive(Debug, Clone)]
pub enum OrderBy {
  Asc(String),
  Desc(String),
}

impl core::fmt::Display for OrderBy {
  fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
    match self {
      OrderBy::Asc(val) => {
        fmt.write_str(val)?;
        fmt.write_str(" ")?;
        fmt.write_str("ASC")?;
      }
      OrderBy::Desc(val) => {
        fmt.write_str(val)?;
        fmt.write_str(" ")?;
        fmt.write_str("DESC")?;
      }
    };

    Ok(())
  }
}

impl From<&str> for OrderBy {
  fn from(value: &str) -> Self {
    if let Some(stripped) = value.strip_prefix('!') {
      OrderBy::Desc(stripped.to_string())
    } else {
      OrderBy::Asc(value.to_string())
    }
  }
}

impl From<&String> for OrderBy {
  fn from(value: &String) -> Self {
    if let Some(stripped) = value.strip_prefix('!') {
      OrderBy::Desc(stripped.to_string())
    } else {
      OrderBy::Asc(value.to_string())
    }
  }
}

impl From<String> for OrderBy {
  fn from(value: String) -> Self {
    if let Some(stripped) = value.strip_prefix('!') {
      OrderBy::Desc(stripped.to_string())
    } else {
      OrderBy::Asc(value)
    }
  }
}

// endregion: --- OrderBy

// region:    --- OrderBys
#[derive(Debug, Clone)]
pub struct OrderBys(Vec<OrderBy>);

impl OrderBys {
  pub fn new(v: Vec<OrderBy>) -> Self {
    OrderBys(v)
  }
  pub fn order_bys(self) -> Vec<OrderBy> {
    self.0
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

// region:    --- with-sea-query
mod with_sea_query {
  use super::*;
  use crate::modql::sea_utils::StringIden;
  use sea_query::IntoColumnRef;

  impl OrderBys {
    pub fn into_sea_col_order_iter(self) -> impl Iterator<Item = (sea_query::ColumnRef, sea_query::Order)> {
      self.0.into_iter().map(OrderBy::into_sea_col_order)
    }
  }

  impl OrderBy {
    pub fn into_sea_col_order(self) -> (sea_query::ColumnRef, sea_query::Order) {
      let (col, order) = match self {
        OrderBy::Asc(col) => (StringIden(col), sea_query::Order::Asc),
        OrderBy::Desc(col) => (StringIden(col), sea_query::Order::Desc),
      };

      (col.into_column_ref(), order)
    }
  }
}
// endregion: --- with-sea-query

// region: --- ultimate-api
impl From<SortBy> for OrderBy {
  fn from(value: SortBy) -> Self {
    match value.direction() {
      SortDirection::Asc | SortDirection::Unspecified => OrderBy::Asc(value.field),
      SortDirection::Desc => OrderBy::Desc(value.field),
    }
  }
}

impl From<&SortBy> for OrderBy {
  fn from(value: &SortBy) -> Self {
    match value.direction() {
      SortDirection::Asc | SortDirection::Unspecified => OrderBy::Asc(value.field.clone()),
      SortDirection::Desc => OrderBy::Desc(value.field.clone()),
    }
  }
}
// endregion: --- ultimate-api
