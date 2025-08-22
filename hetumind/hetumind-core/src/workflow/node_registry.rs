use std::{
  ops::{Deref, DerefMut},
  sync::Arc,
};

use dashmap::DashMap;

use super::{NodeDefinition, NodeExecutor, NodeGroupKind, NodeKind, RegistrationError};

/// 节点注册表，用于管理节点执行器和相关元数据
///
/// # 使用示例
///
/// ```rust,no_run
/// use hetumind_context::node::{NodeRegistry, RegistrationError};
/// use hetumind_core::workflow::{NodeKind, NodeGroupKind, NodeDefinition};
///
/// // 创建节点注册表
/// let mut registry = NodeRegistry::new();
///
/// // 注册节点执行器（假设我们有一个 HttpRequestNode）
/// // let http_node = HttpRequestNode::new();
/// // let metadata = NodeDefinition {
/// //     name: "HttpRequest".to_string(),
/// //     display_name: "HTTP Request".to_string(),
/// //     description: "发送HTTP请求".to_string(),
/// //     group: NodeGroup::Core,
/// //     version: 1,
/// //     icon: Some("globe".to_string()),
/// //     inputs: vec![],
/// //     outputs: vec![],
/// //     parameters: vec![],
/// // };
/// //
/// // registry.register_node_with_metadata(http_node, metadata)?;
///
/// // 查找和使用节点执行器
/// // if let Some(executor) = registry.get_executor(&NodeKind::Transform) {
/// //     // 使用执行器执行节点
/// // }
///
/// // 获取节点元数据
/// // if let Some(metadata) = registry.get_definition(&NodeKind::Transform) {
/// //     println!("节点名称: {}", metadata.display_name);
/// // }
/// ```
#[derive(Clone, Default)]
pub struct NodeRegistry(Arc<InnerNodeRegistry>);

impl Deref for NodeRegistry {
  type Target = Arc<InnerNodeRegistry>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for NodeRegistry {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl NodeRegistry {
  /// 创建新的节点注册表
  pub fn new() -> Self {
    Self::default()
  }
}

#[derive(Default)]
pub struct InnerNodeRegistry {
  /// 节点执行器映射
  executors: DashMap<NodeKind, Arc<dyn NodeExecutor + Send + Sync>>,
  /// 节点定义
  definitions: DashMap<NodeKind, Arc<NodeDefinition>>,
}

impl InnerNodeRegistry {
  /// 注册节点执行器
  ///
  /// Args:
  /// - `executor` - 实现了 NodeExecutor trait 的节点执行器
  pub fn register_node<T>(&self, executor: T) -> Result<(), RegistrationError>
  where
    T: NodeExecutor + Send + Sync + 'static,
  {
    let node_kind = executor.definition().kind.clone();

    // 检查是否已经注册了相同类型的节点
    if self.executors.contains_key(&node_kind) {
      return Err(RegistrationError::NodeKindAlreadyExists { node_kind });
    }

    let definition = executor.definition();
    self.definitions.insert(node_kind.clone(), definition);
    self.executors.insert(node_kind, Arc::new(executor));

    Ok(())
  }

  /// 获取节点执行器
  pub fn get_executor(&self, node_kind: &NodeKind) -> Option<Arc<dyn NodeExecutor + Send + Sync>> {
    self.executors.get(node_kind).map(|x| x.value().clone())
  }

  /// 获取节点定义
  pub fn get_definition(&self, node_kind: &NodeKind) -> Option<Arc<NodeDefinition>> {
    self.definitions.get(node_kind).map(|x| x.value().clone())
  }

  /// 检查节点类型是否已注册
  pub fn contains(&self, node_kind: &NodeKind) -> bool {
    self.executors.contains_key(node_kind)
  }

  /// 获取所有已注册的节点类型
  pub fn registered_node_kinds(&self) -> Vec<NodeKind> {
    self.executors.iter().map(|x| x.key().clone()).collect()
  }

  /// 获取所有已注册的节点定义
  pub fn all_definitions(&self) -> Vec<Arc<NodeDefinition>> {
    self.definitions.iter().map(|x| x.value().clone()).collect()
  }

  /// 按分组获取节点定义
  pub fn definitions_by_group(&self, group: &NodeGroupKind) -> Vec<Arc<NodeDefinition>> {
    self
      .definitions
      .iter()
      .filter(|x| x.value().groups.contains(group))
      .map(|x| x.value().clone())
      .collect()
  }

  /// 注销节点执行器
  pub fn unregister_node(&self, node_kind: &NodeKind) -> bool {
    let executor_removed = self.executors.remove(node_kind).is_some();
    let metadata_removed = self.definitions.remove(node_kind).is_some();

    executor_removed || metadata_removed
  }

  /// 清空所有已注册的节点
  pub fn clear(&self) {
    self.executors.clear();
    self.definitions.clear();
  }

  /// 获取已注册节点数量
  pub fn len(&self) -> usize {
    self.executors.len()
  }

  /// 检查注册表是否为空
  pub fn is_empty(&self) -> bool {
    self.executors.is_empty()
  }
}
