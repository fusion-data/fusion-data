use std::ops::Deref;

use fusion_common::page::{OrderBy, OrderBys, Page};
use sea_query::{IntoColumnRef, SelectStatement};

use crate::sea_utils::StringIden;

pub fn apply_to_sea_query(page: &Page, select_query: &mut SelectStatement) {
  if let Some(limit) = page.limit {
    select_query.limit(limit);
  }

  if let Some(offset) = page.get_offset() {
    select_query.offset(offset);
  }

  if let Some(order_bys) = &page.order_bys {
    for (col, order) in into_sea_col_order_iter(order_bys) {
      select_query.order_by(col, order);
    }
  }
}

pub fn into_sea_col_order_iter(bys: &OrderBys) -> impl Iterator<Item = (sea_query::ColumnRef, sea_query::Order)> {
  bys.into_iter().map(into_sea_col_order)
}

pub fn into_sea_col_order(by: &OrderBy) -> (sea_query::ColumnRef, sea_query::Order) {
  let (col, order) = if let Some(stripped) = by.strip_prefix('!') {
    (StringIden(stripped.to_string()), sea_query::Order::Desc)
  } else {
    (StringIden(by.deref().clone()), sea_query::Order::Asc)
  };
  (col.into_column_ref(), order)
}
