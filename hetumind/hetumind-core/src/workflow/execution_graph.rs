use fusion_common::ahash::HashMap;

use super::{NodeName, Workflow};

/// 用于表示工作流执行图的内部结构
#[derive(Debug)]
pub struct ExecutionGraph {
  /// 邻接表，表示依赖关系。 key: parent_node_id, value: Vec<child_node_id>
  pub adjacency: HashMap<NodeName, Vec<NodeName>>,
  /// 存储每个节点的入度
  pub in_degrees: HashMap<NodeName, usize>,
  /// 存储每个节点的父节点列表
  pub parents: HashMap<NodeName, Vec<NodeName>>,
  /// 所有节点的ID列表
  pub nodes: Vec<NodeName>,
}

impl ExecutionGraph {
  /// 从工作流构建执行图
  pub fn new(workflow: &Workflow) -> Self {
    let mut adjacency: HashMap<NodeName, Vec<NodeName>> = HashMap::default();
    let mut in_degrees: HashMap<NodeName, usize> = HashMap::default();
    let mut parents: HashMap<NodeName, Vec<NodeName>> = HashMap::default();
    let nodes: Vec<NodeName> = workflow.nodes.iter().map(|n| n.name.clone()).collect();

    // 初始化所有节点的入度和邻接表
    for node_name in &nodes {
      in_degrees.insert(node_name.clone(), 0);
      adjacency.insert(node_name.clone(), Vec::new());
      parents.insert(node_name.clone(), Vec::new());
    }

    // 根据连接关系构建图
    for (source_id, kind_connections) in &workflow.connections {
      for (_kind, connections) in kind_connections {
        for connection in connections {
          let target_id = connection.node_name();

          // 添加边到邻接表
          if let Some(children) = adjacency.get_mut(source_id)
            && !children.iter().any(|v| v == target_id)
          {
            children.push(target_id.clone());
          }

          // 增加目标节点的入度
          if let Some(degree) = in_degrees.get_mut(target_id) {
            *degree += 1;
          }

          // 记录父节点
          if let Some(parent_nodes) = parents.get_mut(target_id)
            && !parent_nodes.iter().any(|v| v == source_id)
          {
            parent_nodes.push(source_id.clone());
          }
        }
      }
    }

    Self { adjacency, in_degrees, parents, nodes }
  }

  /// 获取起始节点（入度为0的节点）
  pub fn get_start_nodes(&self) -> Vec<NodeName> {
    self
      .nodes
      .iter()
      .filter(|node_name| self.in_degrees.get(node_name).copied().unwrap_or(0) == 0)
      .cloned()
      .collect()
  }

  /// 获取结束节点（没有子节点的节点）
  pub fn get_end_nodes(&self) -> Vec<NodeName> {
    self
      .nodes
      .iter()
      .filter(|node_name| self.adjacency.get(node_name).map(|children| children.is_empty()).unwrap_or(true))
      .cloned()
      .collect()
  }

  /// 获取节点的子节点
  pub fn get_children(&self, node_name: &NodeName) -> Option<&Vec<NodeName>> {
    self.adjacency.get(node_name)
  }

  /// 获取节点的父节点
  pub fn get_parents(&self, node_name: &NodeName) -> Option<&Vec<NodeName>> {
    self.parents.get(node_name)
  }

  /// 检查是否存在循环依赖
  pub fn has_cycles(&self) -> bool {
    // 使用标准的 DFS 循环检测算法，如果在递归栈中遇到已访问的节点，说明存在循环：
    // - visited: 记录已访问的节点
    // - rec_stack: 记录当前递归栈中的节点
    let mut visited = HashMap::default();
    let mut rec_stack = HashMap::default();

    for node_name in &self.nodes {
      if !visited.get(node_name).is_some_and(|b| *b) && self.has_cycle_util(node_name, &mut visited, &mut rec_stack) {
        return true;
      }
    }
    false
  }

  fn has_cycle_util(
    &self,
    node_name: &NodeName,
    visited: &mut HashMap<NodeName, bool>,
    rec_stack: &mut HashMap<NodeName, bool>,
  ) -> bool {
    visited.insert(node_name.clone(), true);
    rec_stack.insert(node_name.clone(), true);

    if let Some(children) = self.adjacency.get(node_name) {
      for child_id in children {
        if !visited.get(child_id).is_some_and(|b| *b) {
          if self.has_cycle_util(child_id, visited, rec_stack) {
            return true;
          }
        } else if rec_stack.get(child_id).is_some_and(|b| *b) {
          return true;
        }
      }
    }

    rec_stack.insert(node_name.clone(), false);
    false
  }
}
