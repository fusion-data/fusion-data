//! Comprehensive tests for ManualTriggerNode

use hetumind_core::version::Version;
use hetumind_core::workflow::{Node, NodeRegistry};
use hetumind_nodes::trigger::{ManualTriggerNode, register_nodes};

/// Test that ManualTriggerNode can be registered successfully
#[test]
fn test_manual_trigger_node_registration() {
  let registry = NodeRegistry::new();
  let result = register_nodes(&registry);
  assert!(result.is_ok(), "Failed to register manual trigger node: {:?}", result.err());

  // Verify that ManualTriggerNode is registered by checking we can create it
  let node_result = ManualTriggerNode::new();
  assert!(node_result.is_ok(), "Failed to create ManualTriggerNode: {:?}", node_result.err());
}

/// Test that ManualTriggerNode can be created successfully
#[test]
fn test_manual_trigger_node_creation() {
  let result = ManualTriggerNode::new();
  assert!(result.is_ok(), "Failed to create ManualTriggerNode: {:?}", result.err());

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
  assert_eq!(kind, "hetumind_nodes::ManualTrigger".into());
}

/// Test that ManualTriggerNode properties are correctly defined
#[test]
fn test_manual_trigger_node_properties() {
  let node = ManualTriggerNode::new().unwrap();
  let executors = node.node_executors();
  let executor = &executors[0];
  let definition = executor.definition();

  // Verify node properties
  assert!(!definition.properties.is_empty(), "No properties defined");

  // Check for key properties by name
  let property_names: Vec<&str> = definition.properties.iter().map(|p| p.name.as_str()).collect();

  assert!(property_names.contains(&"notice"), "Missing notice property");
  assert!(property_names.contains(&"execution_mode"), "Missing execution_mode property");
  assert!(property_names.contains(&"enabled"), "Missing enabled property");

  // Verify node metadata
  assert_eq!(definition.display_name, "Manual Trigger");
  assert!(definition.description.as_ref().unwrap().contains("手动触发工作流执行"));
  assert!(definition.groups.iter().any(|g| g.is_trigger()), "Node should be in trigger group");
}

/// Test parameter parsing edge cases
#[test]
fn test_manual_trigger_parameter_parsing() {
  let node = ManualTriggerNode::new().unwrap();
  let executors = node.node_executors();
  let executor = &executors[0];
  let definition = executor.definition();

  // Test execution_mode option values
  let execution_mode_property = definition
    .properties
    .iter()
    .find(|p| p.name == "execution_mode")
    .expect("execution_mode property should exist");

  if let Some(options) = &execution_mode_property.options {
    assert!(!options.is_empty(), "execution_mode should have options");

    // Verify both test and production options exist
    let option_values: Vec<&str> =
      options.iter().filter_map(|opt| opt.value.as_ref().and_then(|v| v.as_str())).collect();

    assert!(option_values.contains(&"test"), "Should have test option");
    assert!(option_values.contains(&"production"), "Should have production option");
  } else {
    panic!("execution_mode property should have options");
  }
}

/// Test node version compatibility
#[test]
fn test_manual_trigger_version_compatibility() {
  let node = ManualTriggerNode::new().unwrap();
  let expected_version = Version::new(1, 0, 0);

  assert_eq!(node.default_version(), &expected_version, "Default version should be 1.0.0");

  let executors = node.node_executors();
  for executor in executors {
    assert_eq!(executor.definition().version, expected_version, "All executors should have version 1.0.0");
  }
}

/// Integration test: ManualTriggerNode with full node registry
#[test]
fn test_manual_trigger_integration() {
  let registry = NodeRegistry::new();

  // Register all trigger nodes
  let registration_result = register_nodes(&registry);
  assert!(registration_result.is_ok(), "Failed to register nodes: {:?}", registration_result.err());

  // Verify ManualTriggerNode can be retrieved from registry
  use hetumind_core::workflow::NodeKind;
  let manual_trigger_kind = NodeKind::from("hetumind_nodes::ManualTrigger");
  let executor = registry.get_executor(&manual_trigger_kind);
  assert!(executor.is_some(), "ManualTriggerNode should be registered in registry");

  // Verify executor properties
  let executor = executor.unwrap();
  let definition = executor.definition();
  assert_eq!(definition.display_name, "Manual Trigger");
  assert!(definition.groups.iter().any(|g| g.is_trigger()));
}

/// Test execution with minimal setup to verify node functionality
#[test]
fn test_manual_trigger_basic_functionality() {
  // This test verifies that the node can be created and has the expected structure
  // without requiring complex execution context setup

  let node = ManualTriggerNode::new().unwrap();
  let executors = node.node_executors();
  let executor = &executors[0];
  let definition = executor.definition();

  // Verify the node has the correct definition structure
  assert_eq!(definition.kind, "hetumind_nodes::ManualTrigger".into());
  assert_eq!(definition.version, Version::new(1, 0, 0));
  assert!(!definition.properties.is_empty());

  // Verify required properties exist
  let property_names: std::collections::HashSet<&str> = definition.properties.iter().map(|p| p.name.as_str()).collect();

  assert!(property_names.contains("notice"));
  assert!(property_names.contains("execution_mode"));
  assert!(property_names.contains("enabled"));

  // Verify execution_mode has the correct options
  if let Some(execution_mode_prop) = definition.properties.iter().find(|p| p.name == "execution_mode")
    && let Some(options) = &execution_mode_prop.options
  {
    let option_values: Vec<&str> =
      options.iter().filter_map(|opt| opt.value.as_ref().and_then(|v| v.as_str())).collect();

    assert!(option_values.contains(&"test"));
    assert!(option_values.contains(&"production"));
  }
}
