pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; // For early dev.
use fusionsql::filter::{FilterNodes, IntoFilterNodes, OpVal, OpValInt64, OpValString};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug, FilterNodes)]
struct MyFilter {
  id: Option<OpValInt64>,
  name: Option<OpValString>,
}

#[test]
fn test_des_string_simple() -> Result<()> {
  let json = r#"
	{
		"name": {"$eq": "Hello"}
	}
	"#
  .to_string();

  let json: Value = serde_json::from_str(&json)?;
  let my_filter: MyFilter = serde_json::from_value(json)?;

  assert_eq!(my_filter.name.unwrap().eq, Some(String::from("Hello")));

  Ok(())
}

#[test]
fn test_des_string_map() -> Result<()> {
  let json = r#"
{"name": {
	"$contains": "World",
	"$startsWith": "Hello"
}
}"#;

  let my_filter: MyFilter = serde_json::from_str(json)?;

  let mut nodes = my_filter.filter_nodes(None);

  assert_eq!(nodes.len(), 1, "number of filter node should be 1");
  let node = nodes.pop().unwrap();
  let OpVal::String(opvals) = node.opvals else {
    panic!("expect opvals to be string");
  };
  assert_eq!(opvals.contains, Some(String::from("World")));
  assert_eq!(opvals.starts_with, Some(String::from("Hello")));

  Ok(())
}
