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

impl SortBy {
  pub fn new(field: impl Into<String>, direction: SortDirection) -> Self {
    Self { field: field.into(), direction: direction.into() }
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

pub fn default_page() -> i64 {
  1
}

pub fn default_page_size() -> i64 {
  20
}
