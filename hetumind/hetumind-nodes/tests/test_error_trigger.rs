//! Basic test for ErrorTriggerNode

use hetumind_core::workflow::{Node, NodeRegistry};
use hetumind_nodes::trigger::{ErrorTriggerNode, register_nodes};

/// Test that ErrorTriggerNode can be registered successfully
#[test]
fn test_error_trigger_node_registration() {
  let registry = NodeRegistry::new();
  let result = register_nodes(&registry);
  assert!(result.is_ok(), "Failed to register error trigger node: {:?}", result.err());

  // Verify that ErrorTriggerNode is registered by checking we can create it
  let node_result = ErrorTriggerNode::new();
  assert!(node_result.is_ok(), "Failed to create ErrorTriggerNode: {:?}", node_result.err());
}

/// Test that ErrorTriggerNode can be created successfully
#[test]
fn test_error_trigger_node_creation() {
  let result = ErrorTriggerNode::new();
  assert!(result.is_ok(), "Failed to create ErrorTriggerNode: {:?}", result.err());

  let node = result.unwrap();

  // Verify basic node properties
  assert_eq!(node.default_version().major, 1);
  assert_eq!(node.default_version().minor, 0);
  assert_eq!(node.default_version().patch, 0);

  // Verify we have executors
  let executors = node.node_executors();
  assert!(!executors.is_empty(), "No node executors found");

  // Verify node kind
  let kind = node.kind();
  assert_eq!(kind, "hetumind_nodes::ErrorTrigger".into());
}

/// Test that ErrorTriggerNode properties are correctly defined
#[test]
fn test_error_trigger_node_properties() {
  let node = ErrorTriggerNode::new().unwrap();
  let executors = node.node_executors();
  let executor = &executors[0];
  let definition = executor.definition();

  // Verify node properties
  assert!(!definition.properties.is_empty(), "No properties defined");

  // Check for key properties by name
  let property_names: Vec<&str> = definition.properties.iter().map(|p| p.name.as_str()).collect();

  assert!(property_names.contains(&"trigger_mode"), "Missing trigger_mode property");
  assert!(property_names.contains(&"error_types"), "Missing error_types property");
  assert!(property_names.contains(&"enable_retry"), "Missing enable_retry property");

  // Verify node metadata
  assert_eq!(definition.display_name, "Error Trigger");
  assert_eq!(definition.description, Some("Triggers workflow when other workflows encounter errors".to_string()));
  assert!(definition.groups.iter().any(|g| g.is_trigger()), "Node should be in trigger group");
}
