mod order_by;

pub use order_by::*;
use serde::Deserialize;

use crate::utils::as_positive_u64;

#[derive(Default, Debug, Clone, Deserialize)]
pub struct ListOptions {
  /// 指定返回的条数
  pub limit: Option<i64>,
  /// 指定返回的偏移量
  pub offset: Option<i64>,
  /// 指定返回的页码
  pub page: Option<i64>,
  /// 指定返回的排序
  pub order_bys: Option<OrderBys>,
}

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

  pub fn get_offset(&self) -> Option<u64> {
    self
      .offset
      .map(as_positive_u64)
      .or_else(|| self.page.map(|page| as_positive_u64((page - 1) * self.limit.unwrap_or(0))))
  }

  pub fn get_limit(&self) -> Option<u64> {
    self.limit.map(as_positive_u64)
  }
}

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

mod with_sea_query {
  use sea_query::SelectStatement;

  use super::*;

  impl ListOptions {
    pub fn apply_to_sea_query(self, select_query: &mut SelectStatement) {
      if let Some(limit) = self.get_limit() {
        select_query.limit(limit);
      }

      if let Some(offset) = self.get_offset() {
        select_query.offset(offset);
      }

      if let Some(order_bys) = self.order_bys {
        for (col, order) in order_bys.into_sea_col_order_iter() {
          select_query.order_by(col, order);
        }
      }
    }
  }
}
