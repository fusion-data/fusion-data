use std::collections::{HashMap, HashSet};

use petgraph::algo::toposort;
use petgraph::prelude::*;

use crate::workflow::{ExecutionGraph, NodeName, WorkflowExecutionError};

/// 任务唯一标识符
#[derive(
  Debug,
  Clone,
  PartialEq,
  Eq,
  Hash,
  serde::Serialize,
  serde::Deserialize,
  derive_more::Constructor,
  derive_more::Display,
  derive_more::From,
  derive_more::Into,
  derive_more::AsRef,
)]
#[serde(transparent)]
pub struct TaskId(uuid::Uuid);

/// 执行计划结构体
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
  /// 优化的执行顺序
  pub execution_order: Vec<Vec<NodeName>>,
  /// 并行执行组
  pub parallel_groups: Vec<Vec<NodeName>>,
  /// 依赖关系图
  pub dependency_graph: DiGraph<NodeName, ()>,
  /// 关键路径
  pub critical_path: Vec<NodeName>,
}

/// 执行计划优化器
#[derive(Debug, Clone)]
pub struct ExecutionPlanner {
  max_parallel_tasks: usize,
  enable_dependency_optimization: bool,
}

impl ExecutionPlanner {
  /// 创建新的执行计划器
  pub fn new() -> Self {
    Self { max_parallel_tasks: 10, enable_dependency_optimization: true }
  }

  /// 设置最大并行任务数
  pub fn with_max_parallel_tasks(mut self, max_tasks: usize) -> Self {
    self.max_parallel_tasks = max_tasks;
    self
  }

  /// 启用/禁用依赖优化
  pub fn with_dependency_optimization(mut self, enable: bool) -> Self {
    self.enable_dependency_optimization = enable;
    self
  }

  /// 基于执行图生成优化执行计划
  pub fn plan_execution(&self, graph: &ExecutionGraph) -> Result<ExecutionPlan, WorkflowExecutionError> {
    // 1. 构建依赖图
    let dependency_graph = self.build_dependency_graph(graph)?;

    // 2. 识别并行执行组
    let parallel_groups = self.identify_parallel_groups(&dependency_graph)?;

    // 3. 计算关键路径
    let critical_path = self.calculate_critical_path(&dependency_graph)?;

    // 4. 生成执行顺序
    let execution_order = self.generate_execution_order(&parallel_groups);

    Ok(ExecutionPlan { execution_order, parallel_groups, dependency_graph, critical_path })
  }

  /// 构建依赖关系图
  fn build_dependency_graph(&self, graph: &ExecutionGraph) -> Result<DiGraph<NodeName, ()>, WorkflowExecutionError> {
    let mut g = DiGraph::new();
    let mut node_indices = HashMap::new();

    // 添加所有节点
    for node_name in &graph.nodes {
      let index = g.add_node(node_name.clone());
      node_indices.insert(node_name, index);
    }

    // 添加依赖关系（基于父节点关系）
    for node_name in &graph.nodes {
      if let Some(parents) = graph.get_parents(node_name) {
        for parent in parents {
          if let (Some(from_idx), Some(to_idx)) = (node_indices.get(parent), node_indices.get(node_name)) {
            g.add_edge(*from_idx, *to_idx, ());
          }
        }
      }
    }

    Ok(g)
  }

  /// 识别可并行执行的节点组
  fn identify_parallel_groups(
    &self,
    graph: &DiGraph<NodeName, ()>,
  ) -> Result<Vec<Vec<NodeName>>, WorkflowExecutionError> {
    // 1. 拓扑排序
    let mut sorted_nodes = toposort(graph, None).map_err(|_| WorkflowExecutionError::CircularDependency)?;

    // 2. 按层级分组
    let mut groups = Vec::new();
    let mut processed = HashSet::new();

    while !sorted_nodes.is_empty() {
      let mut current_group = Vec::new();
      let mut to_remove = Vec::new();

      for node_idx in &sorted_nodes {
        let node_name = &graph[*node_idx];

        // 检查所有依赖是否已处理
        let dependencies: Vec<NodeIndex> = graph.neighbors_directed(*node_idx, Incoming).collect();
        let all_dependencies_processed = dependencies.iter().all(|dep_idx| processed.contains(&graph[*dep_idx]));

        if all_dependencies_processed {
          current_group.push(node_name.clone());
          to_remove.push(*node_idx);
        }
      }

      if current_group.is_empty() {
        return Err(WorkflowExecutionError::CircularDependency);
      }

      // 限制并行任务数量
      if current_group.len() > self.max_parallel_tasks {
        current_group.truncate(self.max_parallel_tasks);
      }

      groups.push(current_group);
      processed.extend(to_remove.iter().map(|idx| graph[*idx].clone()));

      // 移除已处理的节点
      for node_idx in to_remove {
        sorted_nodes.retain(|idx| *idx != node_idx);
      }
    }

    Ok(groups)
  }

  /// 计算关键路径
  fn calculate_critical_path(&self, graph: &DiGraph<NodeName, ()>) -> Result<Vec<NodeName>, WorkflowExecutionError> {
    // 使用拓扑排序作为关键路径的简化版本
    let sorted_nodes = toposort(graph, None).map_err(|_| WorkflowExecutionError::CircularDependency)?;

    let critical_path = sorted_nodes.iter().map(|idx| graph[*idx].clone()).collect();

    Ok(critical_path)
  }

  /// 生成执行顺序
  fn generate_execution_order(&self, groups: &[Vec<NodeName>]) -> Vec<Vec<NodeName>> {
    groups.to_vec()
  }

  /// 优化执行计划
  pub fn optimize_execution_plan(&self, plan: &mut ExecutionPlan) -> Result<(), WorkflowExecutionError> {
    if !self.enable_dependency_optimization {
      return Ok(());
    }

    // 基于关键路径优化并行组
    self.optimize_parallel_groups(&mut plan.parallel_groups, &plan.critical_path)?;

    // 重新生成执行顺序
    plan.execution_order = self.generate_execution_order(&plan.parallel_groups);

    Ok(())
  }

  /// 优化并行组
  fn optimize_parallel_groups(
    &self,
    parallel_groups: &mut [Vec<NodeName>],
    critical_path: &[NodeName],
  ) -> Result<(), WorkflowExecutionError> {
    let critical_set: HashSet<&NodeName> = critical_path.iter().collect();

    for group in parallel_groups.iter_mut() {
      // 将关键路径上的节点优先执行
      group.sort_by(|a, b| {
        let a_critical = critical_set.contains(a);
        let b_critical = critical_set.contains(b);

        match (a_critical, b_critical) {
          (true, false) => std::cmp::Ordering::Less,
          (false, true) => std::cmp::Ordering::Greater,
          _ => std::cmp::Ordering::Equal,
        }
      });
    }

    Ok(())
  }

  /// 验证执行计划的有效性
  pub fn validate_execution_plan(&self, plan: &ExecutionPlan) -> Result<(), WorkflowExecutionError> {
    // 检查执行顺序是否为空
    if plan.execution_order.is_empty() {
      return Err(WorkflowExecutionError::InvalidWorkflowStructure("执行顺序不能为空".to_string()));
    }

    // 检查是否存在循环依赖
    if petgraph::algo::is_cyclic_directed(&plan.dependency_graph) {
      return Err(WorkflowExecutionError::CircularDependency);
    }

    // 检查并行组中的节点是否真的可以并行执行
    for group in &plan.parallel_groups {
      for (i, node_a) in group.iter().enumerate() {
        for (j, node_b) in group.iter().enumerate() {
          if i != j {
            // 检查是否存在依赖关系
            if self.has_dependency(&plan.dependency_graph, node_a, node_b)
              || self.has_dependency(&plan.dependency_graph, node_b, node_a)
            {
              return Err(WorkflowExecutionError::InvalidWorkflowStructure(format!(
                "节点 {} 和 {} 存在依赖关系，不能并行执行",
                node_a, node_b
              )));
            }
          }
        }
      }
    }

    Ok(())
  }

  /// 检查两个节点是否存在依赖关系
  fn has_dependency(&self, graph: &DiGraph<NodeName, ()>, from: &NodeName, to: &NodeName) -> bool {
    // 查找节点索引
    let from_idx = graph.node_indices().find(|idx| graph[*idx] == *from);
    let to_idx = graph.node_indices().find(|idx| graph[*idx] == *to);

    match (from_idx, to_idx) {
      (Some(from), Some(to)) => {
        // 检查是否存在从from到to的路径
        petgraph::algo::has_path_connecting(graph, from, to, None)
      }
      _ => false,
    }
  }
}

impl Default for ExecutionPlanner {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_execution_planner_basic() {
    let planner = ExecutionPlanner::new();

    // 创建一个简单的测试工作流图
    // 这里需要实际的ExecutionGraph创建逻辑
    // let graph = create_test_execution_graph().await;

    // let plan = planner.plan_execution(&graph).unwrap();

    // assert!(!plan.execution_order.is_empty());
    // assert!(!plan.parallel_groups.is_empty());
  }

  #[test]
  fn test_max_parallel_tasks() {
    let planner = ExecutionPlanner::new().with_max_parallel_tasks(5);

    assert_eq!(planner.max_parallel_tasks, 5);
  }

  #[test]
  fn test_dependency_optimization() {
    let planner = ExecutionPlanner::new().with_dependency_optimization(false);

    assert!(!planner.enable_dependency_optimization);
  }
}
