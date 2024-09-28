use serde::Serialize;

use super::{Page, Pagination, SortBy, SortDirection};

#[derive(Debug, Clone, Serialize)]
pub struct PagePayload<T> {
  pub page: Page,
  pub items: Vec<T>,
}

impl<T> PagePayload<T> {
  pub fn new(page: Page, items: Vec<T>) -> Self {
    Self { page, items }
  }
}

impl Pagination {
  pub fn page(&self) -> i64 {
    self.page
  }

  pub fn page_size(&self) -> i64 {
    self.page_size
  }

  pub fn sort_bys(&self) -> Vec<&SortBy> {
    self.sort_bys.iter().collect()
  }

  pub fn offset_value(&self) -> i64 {
    if let Some(offset) = self.offset {
      return offset;
    }
    let page = self.page();
    let page_size = self.page_size();
    if page < 2 {
      return 0;
    }
    page_size * (page - 1)
  }

  pub fn new_default() -> Self {
    Self {
      page: default_page(),
      page_size: default_page_size(),
      sort_bys: Default::default(),
      offset: Default::default(),
    }
  }
}

impl Page {
  pub fn new(pagination: &Pagination, total_size: i64) -> Self {
    let page = pagination.page;
    let page_size = pagination.page_size;
    let total_page = if total_size == 0 { 0 } else { (total_size + page_size - 1) / page_size };
    Self { page, page_size, total_size, total_page }
  }
}

#[cfg(feature = "modql")]
impl From<SortBy> for modql::filter::OrderBy {
  fn from(value: SortBy) -> Self {
    match value.direction() {
      SortDirection::Asc | SortDirection::Unspecified => modql::filter::OrderBy::Asc(value.field),
      SortDirection::Desc => modql::filter::OrderBy::Desc(value.field),
    }
  }
}

#[cfg(feature = "modql")]
impl From<&SortBy> for modql::filter::OrderBy {
  fn from(value: &SortBy) -> Self {
    match value.direction() {
      SortDirection::Asc | SortDirection::Unspecified => modql::filter::OrderBy::Asc(value.field.clone()),
      SortDirection::Desc => modql::filter::OrderBy::Desc(value.field.clone()),
    }
  }
}
#[cfg(feature = "modql")]
impl From<Pagination> for modql::filter::ListOptions {
  fn from(value: Pagination) -> Self {
    let offset = Some(value.offset_value());
    let limit = Some(if value.page_size > 0 { value.page_size } else { default_page_size() });
    let order_bys = Some(modql::filter::OrderBys::new(value.sort_bys.into_iter().map(Into::into).collect()));
    modql::filter::ListOptions { limit, offset, order_bys }
  }
}

#[cfg(feature = "modql")]
impl From<&Pagination> for modql::filter::ListOptions {
  fn from(value: &Pagination) -> Self {
    let offset = Some(value.offset_value());
    let limit = Some(if value.page_size > 0 { value.page_size } else { default_page_size() });
    let order_bys = Some(modql::filter::OrderBys::new(value.sort_bys.iter().map(|v| v.into()).collect()));
    modql::filter::ListOptions { limit, offset, order_bys }
  }
}

fn default_page() -> i64 {
  1
}

fn default_page_size() -> i64 {
  20
}
