use std::fmt::Pointer;

use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub struct Point {
  x: i32,
  y: i32,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub struct OpValString {
  #[serde(rename = "$eq")]
  pub eq: Option<String>,
  #[serde(rename = "$not")]
  pub not: Option<String>,
  #[serde(rename = "$in")]
  pub in_: Option<Vec<String>>,
  #[serde(rename = "$notIn")]
  pub not_in: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
#[serde(transparent)]
pub struct OpValsString(Vec<OpValString>);

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub enum OpValInt32 {
  #[serde(rename = "$eq")]
  Eq(i32),
  #[serde(rename = "$not")]
  Not(i32),
  #[serde(rename = "$in")]
  In(Vec<i32>),
  #[serde(rename = "$notIn")]
  NotIn(Vec<i32>),
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub struct OpValsInt32(Vec<OpValInt32>);

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub struct Filter {
  name: Option<OpValsString>,
  age: Option<OpValsInt32>,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub struct Query {
  filters: Vec<Filter>,
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct HetuSdk {
  pub counter: u32,
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl HetuSdk {
  #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
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
