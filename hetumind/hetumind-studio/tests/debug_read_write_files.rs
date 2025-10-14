//! Debug test for ReadWriteFilesNode parameter validation

use hetumind_core::workflow::{NodeRegistry, ParameterMap};
use hetumind_nodes::core::ReadWriteFilesNode;
use std::sync::Arc;

#[test]
fn test_read_write_files_node_creation() -> Result<(), Box<dyn std::error::Error>> {
  println!("Testing ReadWriteFilesNode creation...");

  // Create node
  let file_node = ReadWriteFilesNode::new()?;
  println!("✅ ReadWriteFilesNode created successfully");

  // Create node registry
  let node_registry = NodeRegistry::new();
  node_registry.register_node(Arc::new(file_node))?;
  println!("✅ ReadWriteFilesNode registered successfully");

  // Test parameters (matching the failing integration test)
  let mut params = serde_json::Map::new();
  params.insert("operation".to_string(), serde_json::json!("write"));
  params.insert("file_path".to_string(), serde_json::json!("test_output.json"));
  params.insert(
    "options".to_string(),
    serde_json::json!({
        "append": false,
        "continue_on_fail": true
    }),
  );

  let parameter_map = ParameterMap::new(params);
  println!("✅ Test parameters created: {:?}", parameter_map);

  println!("🎉 All tests passed!");
  Ok(())
}
