use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Paged {
  pub total: u64,
}

impl Paged {
  pub fn new(total: u64) -> Self {
    Self { total }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageResult<T> {
  pub page: Paged,
  pub result: Vec<T>,
}

impl<T> PageResult<T> {
  pub fn new(total: u64, result: Vec<T>) -> Self {
    Self { page: Paged { total }, result }
  }
}
