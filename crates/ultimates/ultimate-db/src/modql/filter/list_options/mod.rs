mod order_by;

pub use order_by::*;
use serde::Deserialize;
use ultimate_api::v1::{default_page_size, Pagination};

#[derive(Default, Debug, Clone, Deserialize)]
pub struct ListOptions {
  pub limit: Option<i64>,
  pub offset: Option<i64>,
  pub order_bys: Option<OrderBys>,
}

// region:    --- Constructors

impl ListOptions {
  pub fn from_limit(limit: i64) -> Self {
    Self { limit: Some(limit), ..Default::default() }
  }

  pub fn from_offset_limit(offset: i64, limit: i64) -> Self {
    Self { limit: Some(limit), offset: Some(offset), ..Default::default() }
  }

  pub fn from_order_bys(order_bys: impl Into<OrderBys>) -> Self {
    Self { order_bys: Some(order_bys.into()), ..Default::default() }
  }
}

// endregion: --- Constructors

// region:    --- Froms

impl From<OrderBys> for ListOptions {
  fn from(val: OrderBys) -> Self {
    Self { order_bys: Some(val), ..Default::default() }
  }
}

impl From<OrderBys> for Option<ListOptions> {
  fn from(val: OrderBys) -> Self {
    Some(ListOptions { order_bys: Some(val), ..Default::default() })
  }
}

impl From<OrderBy> for ListOptions {
  fn from(val: OrderBy) -> Self {
    Self { order_bys: Some(OrderBys::from(val)), ..Default::default() }
  }
}

impl From<OrderBy> for Option<ListOptions> {
  fn from(val: OrderBy) -> Self {
    Some(ListOptions { order_bys: Some(OrderBys::from(val)), ..Default::default() })
  }
}

// endregion: --- Froms

// region:    --- with-sea-query
mod with_sea_query {
  use super::*;
  use sea_query::SelectStatement;

  impl ListOptions {
    pub fn apply_to_sea_query(self, select_query: &mut SelectStatement) {
      fn as_positive_u64(num: i64) -> u64 {
        if num < 0 {
          0
        } else {
          num as u64
        }
      }
      if let Some(limit) = self.limit {
        select_query.limit(as_positive_u64(limit)); // Note: Negative == 0
      }

      if let Some(offset) = self.offset {
        select_query.offset(as_positive_u64(offset)); // Note: Negative == 0
      }

      if let Some(order_bys) = self.order_bys {
        for (col, order) in order_bys.into_sea_col_order_iter() {
          select_query.order_by(col, order);
        }
      }
    }
  }
}
// endregion: --- with-sea-query

// region: --- ultimate-api
impl From<Pagination> for ListOptions {
  fn from(value: Pagination) -> Self {
    let offset = Some(value.offset_value());
    let limit = Some(if value.page_size > 0 { value.page_size } else { default_page_size() });
    let order_bys = Some(OrderBys::new(value.sort_bys.into_iter().map(Into::into).collect()));
    ListOptions { limit, offset, order_bys }
  }
}

impl From<&Pagination> for ListOptions {
  fn from(value: &Pagination) -> Self {
    let offset = Some(value.offset_value());
    let limit = Some(if value.page_size > 0 { value.page_size } else { default_page_size() });
    let order_bys = Some(OrderBys::new(value.sort_bys.iter().map(|v| v.into()).collect()));
    ListOptions { limit, offset, order_bys }
  }
}
// endregion: --- ultimate-api
