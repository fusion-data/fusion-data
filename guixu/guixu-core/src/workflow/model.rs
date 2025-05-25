use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

/// 工作流定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
  /// 工作流唯一标识符
  pub id: String,
  /// 工作流名称
  pub name: String,
  /// 工作流是否激活
  pub active: bool,
  /// 版本标识符
  pub version_id: String,
  /// 工作流设置
  pub settings: WorkflowSettings,
  /// 元数据
  pub meta: WorkflowMeta,
  /// 节点列表
  pub nodes: Vec<WorkflowNode>,
  /// 节点连接关系
  pub connections: HashMap<String, NodeConnections>,
  /// 节点固定数据
  pub pin_data: PinData,
}

/// 工作流设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSettings {
  /// 执行顺序版本
  pub execution_order: String,
}

/// 工作流元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMeta {
  /// 模板凭证设置是否完成
  pub template_creds_setup_completed: bool,
}

/// 工作流节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
  /// 节点唯一标识符
  pub id: String,
  /// 节点名称
  pub name: String,
  /// 节点参数, Value::Object(Map<String, Value>)
  pub parameters: serde_json::Value,
  /// 节点类型
  pub kind: String,
  /// 类型版本
  pub kind_version: i32,
  /// 节点在画布上的位置 [x, y]
  pub position: Vec<i32>,
  /// Webhook ID (可选)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub webhook_id: Option<String>,
  /// 凭证信息 (可选)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub credentials: Option<HashMap<String, CredentialInfo>>,
  /// 始终输出数据 (可选)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub always_output_data: Option<bool>,
  /// 只执行一次 (可选)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub execute_once: Option<bool>,
  /// 失败时重试 (可选)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub retry_on_fail: Option<bool>,
  /// 在流程中显示备注 (可选)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub notes_in_flow: Option<bool>,
  /// 备注信息 (可选)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub notes: Option<String>,
}

/// 凭证信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialInfo {
  /// 凭证ID
  pub id: String,
  /// 凭证名称
  pub name: String,
}

/// 节点连接关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConnections {
  /// 主要连接
  #[serde(skip_serializing_if = "Option::is_none")]
  pub main: Option<Vec<Vec<ConnectionTarget>>>,
  /// AI语言模型连接
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ai_language_model: Option<Vec<Vec<ConnectionTarget>>>,
}

/// 连接目标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTarget {
  /// 目标节点名称
  pub node: String,
  /// 连接类型
  pub kind: String,
  /// 连接索引
  pub index: u32,
}

/// 节点固定数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinData {
  pub data: serde_json::Value,
}

impl Workflow {
  /// 创建新的工作流
  pub fn new(id: String, name: String, version_id: String) -> Self {
    Self {
      id,
      name,
      active: true,
      version_id,
      settings: WorkflowSettings { execution_order: "v1".to_string() },
      meta: WorkflowMeta { template_creds_setup_completed: false },
      nodes: Vec::new(),
      connections: HashMap::new(),
      pin_data: PinData { data: json!({}) },
    }
  }

  /// 添加节点
  pub fn add_node(&mut self, node: WorkflowNode) {
    self.nodes.push(node);
  }

  /// 根据ID查找节点
  pub fn find_node_by_id(&self, id: &str) -> Option<&WorkflowNode> {
    self.nodes.iter().find(|node| node.id == id)
  }

  /// 根据名称查找节点
  pub fn find_node_by_name(&self, name: &str) -> Option<&WorkflowNode> {
    self.nodes.iter().find(|node| node.name == name)
  }

  /// 添加连接
  pub fn add_connection(&mut self, from_node: String, connections: NodeConnections) {
    self.connections.insert(from_node, connections);
  }

  /// 获取所有触发器节点
  pub fn get_trigger_nodes(&self) -> Vec<&WorkflowNode> {
    self
      .nodes
      .iter()
      .filter(|node| node.kind.contains("Trigger") || node.kind.contains("trigger"))
      .collect()
  }
}

impl WorkflowNode {
  /// 创建新节点
  pub fn new(
    id: String,
    name: String,
    kind: String,
    kind_version: i32,
    position: Vec<i32>,
    parameters: serde_json::Value,
  ) -> Self {
    Self {
      parameters,
      kind,
      kind_version,
      position,
      id,
      name,
      webhook_id: None,
      credentials: None,
      always_output_data: None,
      execute_once: None,
      retry_on_fail: None,
      notes_in_flow: None,
      notes: None,
    }
  }

  /// 设置凭证
  pub fn with_credentials(mut self, credentials: HashMap<String, CredentialInfo>) -> Self {
    self.credentials = Some(credentials);
    self
  }

  /// 设置备注
  pub fn with_notes(mut self, notes: String) -> Self {
    self.notes = Some(notes);
    self
  }

  /// 设置Webhook ID
  pub fn with_webhook_id(mut self, webhook_id: String) -> Self {
    self.webhook_id = Some(webhook_id);
    self
  }

  /// 检查是否为触发器节点
  pub fn is_trigger(&self) -> bool {
    self.kind.contains("Trigger") || self.kind.contains("trigger")
  }
}

impl ConnectionTarget {
  /// 创建新的连接目标
  pub fn new(node: String, kind: String, index: u32) -> Self {
    Self { node, kind, index }
  }
}
