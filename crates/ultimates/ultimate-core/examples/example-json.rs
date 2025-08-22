use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum TestEnum {
  None,
  Array(Vec<String>),
}

fn main() {
  let enum1 = TestEnum::Array(vec!["1".to_string(), "2".to_string()]);
  let json = serde_json::to_string(&enum1).unwrap();
  println!("{}", json);

  let enum2 = TestEnum::None;
  let json2 = serde_json::to_string(&enum2).unwrap();
  println!("{}", json2);
}
