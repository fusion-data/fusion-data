use std::sync::Arc;

use fusion_common::ahash::HashMap;
use fusion_common::page::Page;
use fusionsql_core::field::FieldMask;
use fusionsql_core::filter::{OpValBool, OpValInt32, OpValInt64, OpValString, OpValUuid};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use typed_builder::TypedBuilder;

use crate::types::JsonValue;

use super::{
  Connection, ConnectionKind, ExecutionMode, NodeName, NodeRegistry, PinData, ValidationError, WorkflowId, WorkflowNode,
};

/// 工作流状态
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type))]
#[repr(i32)]
pub enum WorkflowStatus {
  /// 草稿
  #[default]
  Draft = 1,
  /// 禁用
  Disabled = 99,
  /// 活跃，已发布
  Active = 100,
}

impl WorkflowStatus {
  pub fn is_active(&self) -> bool {
    *self == WorkflowStatus::Active
  }

  pub fn is_draft(&self) -> bool {
    *self == WorkflowStatus::Draft
  }

  pub fn is_disabled(&self) -> bool {
    *self == WorkflowStatus::Disabled
  }
}

/// 错误处理策略
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum ErrorHandlingStrategy {
  /// 在第一个错误时停止
  #[default]
  StopOnFirstError,

  /// 继续执行其他节点
  ContinueOnError,

  /// 使用错误处理节点，需要在流程中配置错误处理节点
  ErrorNode,
}

#[cfg(feature = "with-db")]
fusionsql::generate_enum_i32_to_sea_query_value!(Enum: WorkflowStatus, Enum: ErrorHandlingStrategy);

/// 工作流设置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkflowSettings {
  /// 执行超时时间（秒）
  #[serde(skip_serializing_if = "Option::is_none")]
  pub execution_timeout: Option<u64>,

  /// 错误处理策略
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error_handling: Option<ErrorHandlingStrategy>,

  /// 执行模式
  #[serde(skip_serializing_if = "Option::is_none")]
  pub execution_mode: Option<ExecutionMode>,

  /// 备注
  #[serde(skip_serializing_if = "Option::is_none")]
  pub remark: Option<String>,
  // /// 保存执行数据天数
  // pub save_execution_data_days: Option<u32>,
  // /// 最大并发执行数
  // pub max_concurrent_executions: Option<u32>,
}

/// 工作流元数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkflowMeta {
  /// 模板凭证设置是否完成
  #[serde(skip_serializing_if = "Option::is_none")]
  pub credentials_setup_completed: Option<bool>,

  /// 模板ID
  #[serde(skip_serializing_if = "Option::is_none")]
  pub template_id: Option<String>,
}

impl WorkflowMeta {
  pub fn credentials_setup_completed(&self) -> bool {
    self.credentials_setup_completed.is_some_and(|b| b)
  }
}

/// 工作流定义
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct Workflow {
  /// 工作流唯一标识符
  pub id: WorkflowId,

  /// 工作流名称
  #[builder(setter(into))]
  pub name: String,

  /// 工作流状态
  #[serde(default)]
  #[builder(default)]
  pub status: WorkflowStatus,

  /// 版本标识符
  #[builder(default, setter(strip_option))]
  pub version: Option<WorkflowId>,

  /// 工作流设置
  #[serde(default)]
  #[builder(default)]
  pub settings: WorkflowSettings,

  /// 元数据
  #[builder(default)]
  #[serde(default)]
  pub meta: WorkflowMeta,

  /// 节点列表
  #[builder(default)]
  pub nodes: Vec<WorkflowNode>,

  /// 节点连接关系。<节点名称, <连接类型, 连接关系>>
  #[serde(default)]
  #[builder(default)]
  pub connections: HashMap<NodeName, HashMap<ConnectionKind, Vec<Connection>>>,

  /// 节点固定数据
  #[serde(default)]
  #[builder(default)]
  pub pin_data: PinData,

  /// 静态数据
  #[builder(default, setter(strip_option))]
  pub static_data: Option<JsonValue>,
}

impl Workflow {
  /// 校验所有必需的输入端口是否都已连接
  ///
  /// # Returns
  /// - `Ok(())` - 如果所有必需的输入端口都已连接
  /// - `Err(Vec<ValidationError>)` - 如果存在未连接的必需输入端口，返回未连接的端口列表 Vec<[ValidationError::UnconnectedRequiredInput]>
  pub fn validate_connectivity(&self, node_registry: Arc<NodeRegistry>) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // <目标节点, Vec<(源节点, 连接类型)>>
    let mut dst_map_src: HashMap<&NodeName, Vec<(&NodeName, ConnectionKind)>> = HashMap::default();

    for (source_name, kind_map) in &self.connections {
      for (kind, connections) in kind_map {
        for connection in connections {
          if let Some(source_names) = dst_map_src.get_mut(connection.node_name()) {
            if source_names.contains(&(source_name, *kind)) {
              errors.push(ValidationError::DuplicateConnection {
                src_name: source_name.clone(),
                src_kind: *kind,
                dst_name: connection.node_name().clone(),
                dst_kind: connection.kind(),
              });
            } else {
              source_names.push((source_name, *kind));
            }
          } else {
            dst_map_src.insert(connection.node_name(), vec![(source_name, *kind)]);
          }
        }
      }
    }

    for node in &self.nodes {
      let node_definition = if let Some(node_definition) = node_registry.get_definition(&node.kind) {
        node_definition
      } else {
        errors.push(ValidationError::NodeDefinitionNotFound { node_kind: node.kind.clone() });
        continue;
      };

      for input_port in &node_definition.inputs {
        let sources = dst_map_src.get(&node.name);

        if input_port.required && sources.is_none() {
          errors.push(ValidationError::UnconnectedInputPort { src_name: node.name.clone(), src_kind: input_port.kind });
        }

        if let Some(sources) = sources {
          for (source_name, kind) in sources {
            if &input_port.kind != kind {
              errors.push(ValidationError::InvalidConnectionKind {
                src_name: (*source_name).clone(),
                src_kind: *kind,
                dst_name: node.name.clone(),
                dst_kind: input_port.kind,
              });
            }
          }
        }
      }
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
  }

  /// 添加节点
  pub fn add_node(&mut self, node: WorkflowNode) {
    self.nodes.push(node);
  }

  /// 根据ID查找节点
  pub fn get_node(&self, id: &NodeName) -> Option<&WorkflowNode> {
    self.nodes.iter().find(|node| &node.name == id)
  }

  /// 添加连接。如相应 **节点/连接类型** 存在，则合并连接；不存在则创建并加入。
  pub fn add_connection(
    &mut self,
    node_name: &NodeName,
    kind: &ConnectionKind,
    connections: Vec<Connection>,
  ) -> &mut Self {
    if !self.connections.contains_key(node_name) {
      self.connections.insert(node_name.clone(), HashMap::default());
    }

    if let Some(entry) = self.connections.get_mut(node_name) {
      if let Some(exists_connections) = entry.get_mut(kind) {
        exists_connections.extend(connections);
      } else {
        entry.insert(*kind, connections);
      }
    }

    self
  }

  // /// 获取所有触发器节点
  // pub fn get_trigger_nodes(&self) -> Vec<&WorkflowNode> {
  //   self.nodes.iter().filter(|node| node.kind.groups().contains(&NodeGroupKind::Trigger)).collect()
  // }
}

#[derive(Clone, Deserialize)]
#[cfg_attr(feature = "fusionsql", derive(fusionsql::field::Fields))]
pub struct WorkflowForCreate {
  pub id: Option<WorkflowId>,
  pub name: String,
  pub status: Option<WorkflowStatus>,
  pub nodes: Option<serde_json::Value>,
  pub connections: Option<serde_json::Value>,
  pub settings: Option<serde_json::Value>,
  pub static_data: Option<serde_json::Value>,
  pub pin_data: Option<serde_json::Value>,
  pub version_id: Option<WorkflowId>,
  pub meta: Option<serde_json::Value>,
}

impl TryFrom<Workflow> for WorkflowForCreate {
  type Error = ValidationError;

  fn try_from(wf: Workflow) -> Result<Self, Self::Error> {
    let v = Self {
      id: Some(wf.id),
      name: wf.name,
      status: Some(wf.status),
      nodes: Some(serde_json::to_value(wf.nodes)?),
      connections: Some(serde_json::to_value(wf.connections)?),
      settings: Some(serde_json::to_value(wf.settings)?),
      static_data: None,
      pin_data: Some(serde_json::to_value(wf.pin_data)?),
      version_id: wf.version,
      meta: Some(serde_json::to_value(wf.meta)?),
    };
    Ok(v)
  }
}

impl TryFrom<WorkflowForCreate> for Workflow {
  type Error = ValidationError;

  fn try_from(wf: WorkflowForCreate) -> Result<Self, Self::Error> {
    let wf = Workflow {
      id: wf.id.ok_or_else(|| ValidationError::required_field_missing("id"))?,
      name: wf.name,
      status: wf.status.unwrap_or_default(),
      version: wf.version_id,
      settings: if let Some(settings) = wf.settings {
        serde_json::from_value(settings)?
      } else {
        WorkflowSettings::default()
      },
      meta: if let Some(meta) = wf.meta { serde_json::from_value(meta)? } else { WorkflowMeta::default() },
      nodes: if let Some(nodes) = wf.nodes { serde_json::from_value(nodes)? } else { Vec::new() },
      connections: if let Some(connections) = wf.connections {
        serde_json::from_value(connections)?
      } else {
        HashMap::default()
      },
      pin_data: if let Some(pin_data) = wf.pin_data { serde_json::from_value(pin_data)? } else { PinData::default() },
      static_data: if let Some(static_data) = wf.static_data {
        Some(serde_json::from_value(static_data)?)
      } else {
        None
      },
    };
    Ok(wf)
  }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "fusionsql", derive(fusionsql::field::Fields))]
pub struct WorkflowForUpdate {
  pub name: Option<String>,
  pub status: Option<WorkflowStatus>,
  pub nodes: Option<serde_json::Value>,
  pub connections: Option<serde_json::Value>,
  pub settings: Option<serde_json::Value>,
  pub static_data: Option<serde_json::Value>,
  pub pin_data: Option<serde_json::Value>,
  pub version_id: Option<WorkflowId>,
  pub trigger_count: Option<i64>,
  pub meta: Option<serde_json::Value>,
  pub parent_folder_id: Option<String>,
  pub is_archived: Option<bool>,
  #[cfg_attr(feature = "fusionsql", field(skip))]
  pub field_mask: Option<FieldMask>,
}

impl TryFrom<WorkflowForUpdate> for Workflow {
  type Error = serde_json::Error;

  fn try_from(wf: WorkflowForUpdate) -> Result<Self, Self::Error> {
    let wf = Self {
      id: WorkflowId::now_v7(), // 生成新的ID，因为这是从更新结构转换
      name: wf.name.unwrap_or_default(),
      status: wf.status.unwrap_or_default(),
      version: wf.version_id,
      settings: if let Some(settings) = wf.settings {
        serde_json::from_value(settings)?
      } else {
        WorkflowSettings::default()
      },
      meta: if let Some(meta) = wf.meta { serde_json::from_value(meta)? } else { WorkflowMeta::default() },
      nodes: if let Some(nodes) = wf.nodes { serde_json::from_value(nodes)? } else { Vec::new() },
      connections: if let Some(connections) = wf.connections {
        serde_json::from_value(connections)?
      } else {
        HashMap::default()
      },
      pin_data: if let Some(pin_data) = wf.pin_data { serde_json::from_value(pin_data)? } else { PinData::default() },
      static_data: if let Some(static_data) = wf.static_data {
        Some(serde_json::from_value(static_data)?)
      } else {
        None
      },
    };
    Ok(wf)
  }
}

#[derive(Serialize, Deserialize)]
pub struct WorkflowForQuery {
  pub options: Page,
  pub filter: WorkflowFilter,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "fusionsql", derive(fusionsql::filter::FilterNodes))]
pub struct WorkflowFilter {
  pub name: Option<OpValString>,
  pub status: Option<OpValInt32>,
  pub version_id: Option<OpValUuid>,
  pub trigger_count: Option<OpValInt64>,
  pub parent_folder_id: Option<OpValUuid>,
  pub is_archived: Option<OpValBool>,
}

/// Request body for validating a workflow.
/// It uses the core workflow model directly.
#[derive(Serialize, Deserialize)]
pub struct ValidateWorkflowRequest {
  pub id: Option<WorkflowId>,
  pub workflow: Option<Workflow>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateWorkflowResponse {
  pub is_valid: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub errors: Option<Vec<ValidationError>>,
}
