use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub struct Paged {
  pub total: u64,
  pub has_more: bool,
}

impl Paged {
  pub fn new(total: u64) -> Self {
    Self { total, has_more: false }
  }

  pub fn with_has_more(mut self, has_more: bool) -> Self {
    self.has_more = has_more;
    self
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub struct PageResult<T> {
  pub page: Paged,
  pub result: Vec<T>,
}

impl<T> PageResult<T> {
  pub fn new(total: u64, result: Vec<T>) -> Self {
    Self { page: Paged::new(total), result }
  }

  pub fn with_has_more(mut self, has_more: bool) -> Self {
    self.page = self.page.with_has_more(has_more);
    self
  }
}
