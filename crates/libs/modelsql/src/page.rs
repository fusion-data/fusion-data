use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Page {
  pub total: i64,
}

impl Page {
  pub fn new(total: i64) -> Self {
    Self { total }
  }
}

#[derive(Debug)]
pub struct PageResult<T> {
  pub page: Page,
  pub result: Vec<T>,
}

impl<T> PageResult<T> {
  pub fn new(total: i64, result: Vec<T>) -> Self {
    Self { page: Page { total }, result }
  }
}
