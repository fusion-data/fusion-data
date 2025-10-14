use std::collections::HashMap as StdHashMap;

use ahash::{AHashMap, RandomState};

fn main() {
  let m1: StdHashMap<String, String, RandomState> = StdHashMap::with_hasher(RandomState::new());
  let m2 = AHashMap::from_iter(vec![(1, "one"), (2, "two")]);
  println!("m1: {:?}", m1);
  println!("m2: {:?}", m2);
}
