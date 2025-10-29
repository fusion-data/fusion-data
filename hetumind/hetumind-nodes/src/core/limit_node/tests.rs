use hetumind_core::workflow::{ConnectionKind, Node, NodeGroupKind};

use crate::core::LimitNode;
use crate::core::limit_node::{KeepStrategy, LimitConfig};

#[test]
fn test_node_metadata() {
  let node = LimitNode::new().unwrap();
  let definition = node.default_node_executor().unwrap().definition();

  assert_eq!(definition.kind.as_ref(), "hetumind_nodes::Limit");
  assert_eq!(&definition.groups, &[NodeGroupKind::Transform]);
  assert_eq!(&definition.display_name, "Limit");
  assert_eq!(definition.inputs.len(), 1);
  assert_eq!(definition.outputs.len(), 1);
}

#[test]
fn test_node_ports() {
  let node = LimitNode::new().unwrap();
  let definition = node.default_node_executor().unwrap().definition();

  let input_ports = &definition.inputs[..];
  assert_eq!(input_ports.len(), 1);
  assert_eq!(input_ports[0].kind, ConnectionKind::Main);

  let output_ports = &definition.outputs[..];
  assert_eq!(output_ports.len(), 1);
  assert_eq!(output_ports[0].kind, ConnectionKind::Main);
}

#[test]
fn test_limit_config_validation() {
  // 有效配置
  let valid_config = LimitConfig { max_items: 10, keep_strategy: KeepStrategy::FirstItems, warn_on_limit: true };
  assert!(valid_config.validate().is_ok());

  // 无效配置 - max_items 为 0
  let invalid_config = LimitConfig { max_items: 0, keep_strategy: KeepStrategy::FirstItems, warn_on_limit: true };
  assert!(invalid_config.validate().is_err());
}

#[test]
fn test_keep_strategy_default() {
  let default_strategy = KeepStrategy::default();
  assert!(matches!(default_strategy, KeepStrategy::FirstItems));
}

#[test]
fn test_limit_config_description() {
  let config = LimitConfig { max_items: 5, keep_strategy: KeepStrategy::FirstItems, warn_on_limit: false };
  assert_eq!(config.get_description(), "Limit to 5 first items");

  let config = LimitConfig { max_items: 3, keep_strategy: KeepStrategy::LastItems, warn_on_limit: false };
  assert_eq!(config.get_description(), "Limit to 3 last items");
}
