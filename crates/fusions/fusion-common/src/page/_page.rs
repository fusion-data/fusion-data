use serde::{Deserialize, Serialize};

pub use super::*;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "with-wasm", derive(tsify::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub struct Page {
  /// 指定返回的页码
  pub page: Option<u64>,
  /// 指定返回的条数
  pub limit: Option<u64>,
  /// 指定返回的偏移量
  pub offset: Option<u64>,
  /// 指定返回的排序
  pub order_bys: Option<OrderBys>,
}

impl Page {
  pub fn new_with_limit(limit: u64) -> Self {
    Self { limit: Some(limit), ..Default::default() }
  }

  pub fn new_with_offset_limit(offset: u64, limit: u64) -> Self {
    Self { limit: Some(limit), offset: Some(offset), ..Default::default() }
  }

  pub fn new_with_order_bys(order_bys: impl Into<OrderBys>) -> Self {
    Self { order_bys: Some(order_bys.into()), ..Default::default() }
  }

  pub fn get_offset(&self) -> Option<u64> {
    self.offset.or_else(|| self.page.map(|page| (page - 1) * self.limit.unwrap_or(0)))
  }
}

impl From<OrderBys> for Page {
  fn from(val: OrderBys) -> Self {
    Self { order_bys: Some(val), ..Default::default() }
  }
}

impl From<OrderBys> for Option<Page> {
  fn from(val: OrderBys) -> Self {
    Some(Page { order_bys: Some(val), ..Default::default() })
  }
}

impl From<OrderBy> for Page {
  fn from(val: OrderBy) -> Self {
    Self { order_bys: Some(OrderBys::from(val)), ..Default::default() }
  }
}

impl From<OrderBy> for Option<Page> {
  fn from(val: OrderBy) -> Self {
    Some(Page { order_bys: Some(OrderBys::from(val)), ..Default::default() })
  }
}
