use modelsql::{
  filter::{ListOptions, OrderBy, OrderBys},
  page::PageResult,
};

use crate::v1::{Page, PagePayload, Pagination, SortBy, SortDirection};

impl<T> From<PageResult<T>> for PagePayload<T> {
  fn from(page_result: PageResult<T>) -> Self {
    Self::new(Page::new(page_result.page.total), page_result.result)
  }
}

impl From<Pagination> for ListOptions {
  fn from(value: Pagination) -> Self {
    ListOptions::from(&value)
  }
}

impl From<&Pagination> for ListOptions {
  fn from(value: &Pagination) -> Self {
    ListOptions {
      limit: value.get_page_size(),
      page: value.get_page(),
      offset: value.get_offset(),
      order_bys: Some(OrderBys::new(value.sort_bys.iter().map(|v| v.into()).collect())),
    }
  }
}

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
