use fusionsql_core::filter::{OpValInt32, OpValString};
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "with-wasm", derive(tsify::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub struct Point {
  x: i32,
  y: i32,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "with-wasm", derive(tsify::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub struct Filter {
  name: Option<OpValString>,
  age: Option<OpValInt32>,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "with-wasm", derive(tsify::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub struct Query {
  filters: Vec<Filter>,
}

#[cfg_attr(feature = "with-wasm", wasm_bindgen)]
pub struct HetuSdk {
  pub counter: u32,
}

#[cfg_attr(feature = "with-wasm", wasm_bindgen)]
impl HetuSdk {
  #[cfg_attr(feature = "with-wasm", wasm_bindgen(constructor))]
  pub fn new() -> HetuSdk {
    HetuSdk { counter: 3 }
  }

  pub async fn into_js(&self) -> Point {
    Point { x: 0, y: 0 }
  }

  pub async fn from_js(&self, point: Point) -> Point {
    point
  }

  pub async fn query(&self, query: Query) -> Point {
    Point { x: 0, y: 0 }
  }
}
