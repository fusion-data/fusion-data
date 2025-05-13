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
  pub fn get_page(&self) -> Option<i64> {
    if self.page > 0 { Some(self.page) } else { None }
  }

  pub fn get_page_size(&self) -> Option<i64> {
    if self.page_size > 0 { Some(self.page_size) } else { None }
  }

  pub fn get_offset(&self) -> Option<i64> {
    self.offset
  }

  pub fn sort_bys(&self) -> Vec<&SortBy> {
    self.sort_bys.iter().collect()
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
  pub fn new(total_size: i64) -> Self {
    Self { total_size }
  }
}

pub fn default_page() -> i64 {
  1
}

pub fn default_page_size() -> i64 {
  20
}
