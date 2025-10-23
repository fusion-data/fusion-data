//! Read/Write Files Node 文件读写节点实现
//!
//! 参考 n8n 的 Read/Write Files from Disk Node 设计，用于在运行 hetumind 的计算机上读取和写入文件。
//! 支持多种文件格式和操作模式，基于 hetumind 的二进制数据引用系统提高内存效率。

use std::sync::Arc;

use hetumind_core::{
  version::Version,
  workflow::{Node, NodeDefinition, NodeExecutor, NodeGroupKind, NodeKind, RegistrationError},
};
use serde::{Deserialize, Serialize};

mod read_write_files_v1;
mod utils;

use read_write_files_v1::ReadWriteFilesV1;

use crate::constants::READ_WRITE_FILES_NODE_KIND;

/// 操作类型
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
  /// 读取文件
  Read,
  /// 写入文件
  Write,
}

/// 文件错误上下文
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FileErrorContext {
  pub operation: String, // "read" or "write"
  pub file_path: String,
}

pub struct ReadWriteFilesNode {
  default_version: Version,
  executors: Vec<NodeExecutor>,
}

impl ReadWriteFilesNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let base = Self::base();
    let executors: Vec<NodeExecutor> = vec![Arc::new(ReadWriteFilesV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }

  fn base() -> NodeDefinition {
    NodeDefinition::new(READ_WRITE_FILES_NODE_KIND, "Read/Write Files")
      .add_group(NodeGroupKind::Input)
      .add_group(NodeGroupKind::Output)
      .with_description("从磁盘读取文件或将文件写入磁盘。支持多种文件格式和操作模式。")
      .with_icon("file")
  }
}

impl Node for ReadWriteFilesNode {
  fn default_version(&self) -> &Version {
    &self.default_version
  }

  fn node_executors(&self) -> &[NodeExecutor] {
    &self.executors
  }

  fn kind(&self) -> NodeKind {
    self.executors[0].definition().kind.clone()
  }
}

#[cfg(test)]
mod tests {
  use hetumind_core::workflow::{ConnectionKind, NodeGroupKind};

  use super::*;

  #[test]
  fn test_node_metadata() {
    let node = ReadWriteFilesNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    assert_eq!(definition.kind.as_ref(), "ReadWriteFiles");
    assert_eq!(&definition.groups, &[NodeGroupKind::Input, NodeGroupKind::Output]);
    assert_eq!(&definition.display_name, "Read/Write Files");
    assert_eq!(definition.inputs.len(), 1);
    assert_eq!(definition.outputs.len(), 1);
  }

  #[test]
  fn test_node_ports() {
    let node = ReadWriteFilesNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    let input_ports = &definition.inputs[..];
    assert_eq!(input_ports.len(), 1);
    assert_eq!(input_ports[0].kind, ConnectionKind::Main);

    let output_ports = &definition.outputs[..];
    assert_eq!(output_ports.len(), 1);
    assert_eq!(output_ports[0].kind, ConnectionKind::Main);
  }

  #[test]
  fn test_operation_kind_serialization() {
    let read_op = OperationKind::Read;
    let serialized = serde_json::to_string(&read_op).unwrap();
    assert_eq!(serialized, "\"read\"");

    let write_op = OperationKind::Write;
    let serialized = serde_json::to_string(&write_op).unwrap();
    assert_eq!(serialized, "\"write\"");

    let deserialized: OperationKind = serde_json::from_str("\"read\"").unwrap();
    assert_eq!(deserialized, OperationKind::Read);
  }
}
