pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; // For early dev.
use serde::Deserialize;
use serde_json::Value;
use ultimate_db::modql::filter::{FilterNodes, IntoFilterNodes, OpValsInt64, OpValsString};

#[derive(Deserialize, Debug, FilterNodes)]
struct MyFilter {
  id: Option<OpValsInt64>,
  name: Option<OpValsString>,
}

#[test]
fn test_des_string_simple() -> Result<()> {
  let json = r#"
	{
		"name": "Hello"
	}
	"#
  .to_string();

  let json: Value = serde_json::from_str(&json)?;
  let my_filter: MyFilter = serde_json::from_value(json)?;

  assert!(format!("{my_filter:?}").contains("id: None, name: Some(OpValsString([Eq(\"Hello\")]))"));

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
  assert_eq!(format!("{:?}", node.opvals[0]), "String(Contains(\"World\"))");
  assert_eq!(format!("{:?}", node.opvals[1]), "String(StartsWith(\"Hello\"))");
  // assert_eq!(node.opvals[0])

  Ok(())
}

#[test]
fn test_des_number_simple() -> Result<()> {
  let json = r#"
	{
		"id": 123
	}
	"#;

  let my_filter: MyFilter = serde_json::from_str(json)?;
  let filter_str = format!("{my_filter:?}");
  assert!(filter_str.contains("{ id: Some(OpValsInt64([Eq(123)])), name: None }"), "{filter_str}");

  Ok(())
}

#[test]
fn test_des_number_map() -> Result<()> {
  let json = r#"
	{
		"id": {"$gt": 100}
	}
	"#;

  let my_filter: MyFilter = serde_json::from_str(json)?;
  assert!(format!("{my_filter:?}").contains("{ id: Some(OpValsInt64([Gt(100)])), name: None }"));

  Ok(())
}
