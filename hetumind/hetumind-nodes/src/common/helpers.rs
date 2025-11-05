//! 通用 Helper：在节点执行上下文中发现 LLM/Memory/Tool 供应器
//!
//! 注意：当前返回 SubNodeRef（未进行 typed downcast），后续将引入 typed 引用。

use crate::store::simple_memory_node::SIMPLE_MEMORY_NODE_KIND;
use hetumind_core::workflow::{
  AgentSubNodeProviderRef, ConnectionKind, LLMSubNodeProviderRef, MemorySubNodeProviderRef, NodeExecutionContext,
  NodeExecutionError, NodeKind, NodeRegistry, SubNodeRef, SubNodeType, ToolSubNodeProviderRef,
};

/// 从当前节点的上游连接中检索 LLM 供应器（按连接逆序）
pub async fn get_llm_providers(
  ctx: &NodeExecutionContext,
  index: usize,
) -> Result<Vec<SubNodeRef>, NodeExecutionError> {
  let parents = ctx.get_all_connections(&ConnectionKind::AiLM);
  let mut providers = Vec::new();

  for (i, conn) in parents.iter().rev().enumerate() {
    if i < index {
      continue;
    }
    let node =
      ctx
        .workflow
        .nodes
        .iter()
        .find(|n| n.name == conn.node_name)
        .ok_or_else(|| NodeExecutionError::NodeNotFound {
          workflow_id: ctx.workflow.id.clone(),
          node_name: conn.node_name.clone(),
        })?;
    if let Some(p) = ctx.node_registry.get_subnode_provider(&node.kind)
      && p.provider_type() == SubNodeType::LLM
    {
      providers.push(p.clone());
    }
  }
  Ok(providers)
}

/// 获取 Memory 供应器（上游第 index 个连接）
pub async fn get_memory_provider(
  ctx: &NodeExecutionContext,
  index: usize,
) -> Result<Option<SubNodeRef>, NodeExecutionError> {
  let parents = ctx.get_all_connections(&ConnectionKind::AiMemory);
  let target = parents.iter().rev().nth(index);
  if let Some(conn) = target {
    let node =
      ctx
        .workflow
        .nodes
        .iter()
        .find(|n| n.name == conn.node_name)
        .ok_or_else(|| NodeExecutionError::NodeNotFound {
          workflow_id: ctx.workflow.id.clone(),
          node_name: conn.node_name.clone(),
        })?;
    if let Some(p) = ctx.node_registry.get_subnode_provider(&node.kind)
      && p.provider_type() == SubNodeType::Memory
    {
      return Ok(Some(p.clone()));
    }
  }
  Ok(None)
}

/// 获取所有连接的工具供应器（扁平化）
pub async fn get_connected_tools(ctx: &NodeExecutionContext) -> Result<Vec<SubNodeRef>, NodeExecutionError> {
  let parents = ctx.get_all_connections(&ConnectionKind::AiTool);
  let mut providers = Vec::new();
  for conn in parents.iter().rev() {
    let node =
      ctx
        .workflow
        .nodes
        .iter()
        .find(|n| n.name == conn.node_name)
        .ok_or_else(|| NodeExecutionError::NodeNotFound {
          workflow_id: ctx.workflow.id.clone(),
          node_name: conn.node_name.clone(),
        })?;
    if let Some(p) = ctx.node_registry.get_subnode_provider(&node.kind)
      && p.provider_type() == SubNodeType::Tool
    {
      providers.push(p.clone());
    }
  }
  Ok(providers)
}

/// 获取 SimpleMemory 的 typed 供应器（直接按 NodeKind）
pub fn get_simple_memory_supplier_typed(registry: &NodeRegistry) -> Option<MemorySubNodeProviderRef> {
  let kind = NodeKind::new(SIMPLE_MEMORY_NODE_KIND);
  registry.get_memory_supplier_typed(&kind)
}

/// 获取指定 NodeKind 的 typed LLM 供应器
pub fn get_llm_supplier_typed(registry: &NodeRegistry, kind: &NodeKind) -> Option<LLMSubNodeProviderRef> {
  registry.get_llm_supplier_typed(kind)
}

/// 获取指定 NodeKind 的 typed Tool 供应器
pub fn get_tool_supplier_typed(registry: &NodeRegistry, kind: &NodeKind) -> Option<ToolSubNodeProviderRef> {
  registry.get_tool_supplier_typed(kind)
}

/// 获取指定 NodeKind 的 typed Agent 供应器
pub fn get_agent_supplier_typed(registry: &NodeRegistry, kind: &NodeKind) -> Option<AgentSubNodeProviderRef> {
  registry.get_agent_supplier_typed(kind)
}
