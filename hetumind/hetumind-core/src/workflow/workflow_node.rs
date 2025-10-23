use fusion_common::ahash::HashMap;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::credential::CredentialInfo;

use super::{NodeKind, NodeName, ParameterMap, ValidationError, VecExecutionData};

/// 节点执行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum NodeExecutionStatus {
  /// 等待执行
  Waiting = 1,

  /// 正在执行
  Running = 10,

  /// 跳过执行
  Skipped = 11,

  /// 重试中
  Retrying = 21,

  /// 执行失败
  Failed = 99,

  /// 执行成功
  Success = 100,
}

/// 节点分组。由平台预定义，用户自定义的节点必需属于某一个分组。
///
/// 使用场景：
/// 1. 在界面上查找节点时，可根据节点分组进行筛选。
/// 2. 某些业务要求只能在特定分组的节点之间进行连接。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum NodeGroupKind {
  /// 输入节点
  /// - 表示节点主要从外部系统读取或获取数据到 hetumind 中
  /// - 应用场景: 与第三方 API、数据库、文件系统等进行数据交换
  Input = 1,
  /// 输出节点
  /// - 表示节点主要将 hetumind 的数据发送或写入到外部系统
  /// - 与第三方 API、数据库、文件系统等进行数据交换
  Output = 2,
  /// 触发器节点
  /// - 含义: 触发器。这类节点是工作流的起点，用于启动整个流程。它们通常没有输入端。
  /// - 应用场景: 监听外部事件或按计划启动工作流。
  /// - 典型节点: ManualTrigger (手动触发), Webhook (通过 HTTP 请求触发), Event (事件触发)
  Trigger = 3,
  /// 转换节点
  /// - 含义: 数据转换。这类节点的核心功能是接收数据，对数据进行修改、重组、过滤或计算，然后将处理后的数据向下游传递。
  /// - 应用场景: 数据清洗、格式转换、数据聚合、逻辑计算等。
  /// - 典型节点: Set (设置或修改字段值), Merge (合并数据), LoopOverItems (拆分数据批次), Code (执行自定义代码), RemoveDuplicates (去重)。
  Transform = 4,
  /// 调度节点
  /// - 含义: 调度。这类节点用于管理工作流的执行计划和调度规则。
  /// - 应用场景: 定时执行、周期性任务、事件驱动等。
  /// - 典型节点: Interval (间隔触发), Cron (定时触发)。
  Schedule = 5,
  /// 组织节点
  /// - 含义: 组织/辅助。用于那些不直接处理数据流，而是帮助组织和美化工作流画布的节点。
  /// - 应用场景: 提高工作流的可读性和可维护性。
  /// - 典型节点: StickyNote (便签)，用于在画布上添加注释。
  Organization = 6,
}

pub static NODE_GROUP_KINDS: [NodeGroupKind; 5] = [
  NodeGroupKind::Input,
  NodeGroupKind::Output,
  NodeGroupKind::Trigger,
  NodeGroupKind::Transform,
  NodeGroupKind::Organization,
];

impl NodeGroupKind {
  pub fn is_input(&self) -> bool {
    self == &NodeGroupKind::Input
  }
  pub fn is_output(&self) -> bool {
    self == &NodeGroupKind::Output
  }
  pub fn is_trigger(&self) -> bool {
    self == &NodeGroupKind::Trigger
  }
  pub fn is_transform(&self) -> bool {
    self == &NodeGroupKind::Transform
  }
  pub fn is_organization(&self) -> bool {
    self == &NodeGroupKind::Organization
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NodeExecutionMode {
  /// Execute one item at a time
  #[default]
  EachItem,

  /// Execute all items as a whole list
  All,
}

/// The handling strategy when workflow execution encounters an error
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnErrorBehavior {
  /// Stop workflow execution
  #[default]
  Stop,

  /// Continue execution and skip the current node
  Continue,

  /// Output error to specified port
  ErrorOutput,
}

/// Node fixed data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PinData {
  /// Node fixed data, used to store data that needs to be saved during node execution
  /// - key: node name
  /// - value: node execution data (results)
  pub data: HashMap<NodeName, VecExecutionData>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Position {
  pub x: i32,
  pub y: i32,
}

impl From<(i32, i32)> for Position {
  fn from((x, y): (i32, i32)) -> Self {
    Self { x, y }
  }
}

/// Node element of workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeElement {
  pub kind: NodeKind,

  /// The unique identifier name of the node within the workflow. Other nodes can be accessed via this name,
  /// for example, accessing other node's data through `$node(name)` in expressions.
  pub name: NodeName,

  /// The display name of the node, shown in the UI. Uses the value of `name` if not set.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub display_name: Option<String>,

  /// Node parameters. Parameters configured when defining the workflow. For configurable parameters,
  /// see the `properties` property of NodeDefinition.
  pub parameters: ParameterMap,

  /// 节点在画布上的位置 [x, y]
  pub position: Option<Position>,

  /// Webhook ID (可选)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub webhook_id: Option<String>,

  /// 凭证信息 (可选)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub credentials: Option<HashMap<String, CredentialInfo>>,

  /// 始终输出数据 (可选)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub always_output_data: Option<bool>,

  #[serde(default)]
  pub execute_mode: NodeExecutionMode,

  /// 在流程中显示备注 (可选)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub notes_in_flow: Option<bool>,

  /// 备注信息 (可选)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub notes: Option<String>,

  /// 错误处理策略，默认为 StopWorkflow
  #[serde(default)]
  pub on_error: OnErrorBehavior,

  /// 失败时最大重试次数，默认为 0，表示不重试
  #[serde(default)]
  pub max_tries: u32,

  /// 重试间隔（毫秒）
  pub wait_between_tries: Option<u64>,

  /// 超时时间（秒）
  pub timeout: Option<u64>,
}

impl NodeElement {
  pub fn new(kind: impl Into<NodeKind>, name: impl Into<NodeName>) -> Self {
    Self {
      kind: kind.into(),
      name: name.into(),
      display_name: None,
      parameters: ParameterMap::default(),
      position: None,
      webhook_id: None,
      credentials: None,
      always_output_data: None,
      execute_mode: NodeExecutionMode::default(),
      notes_in_flow: None,
      notes: None,
      on_error: OnErrorBehavior::default(),
      max_tries: 0,
      wait_between_tries: None,
      timeout: None,
    }
  }

  pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
    self.display_name = Some(display_name.into());
    self
  }

  pub fn with_parameters(mut self, parameters: ParameterMap) -> Self {
    self.parameters = parameters;
    self
  }

  pub fn with_position(mut self, position: Option<Position>) -> Self {
    self.position = position;
    self
  }

  pub fn with_webhook_id(mut self, webhook_id: impl Into<String>) -> Self {
    self.webhook_id = Some(webhook_id.into());
    self
  }

  pub fn with_credentials(mut self, credentials: HashMap<String, CredentialInfo>) -> Self {
    self.credentials = Some(credentials);
    self
  }

  pub fn with_always_output_data(mut self, always_output_data: bool) -> Self {
    self.always_output_data = Some(always_output_data);
    self
  }

  pub fn with_execute_mode(mut self, execute_mode: NodeExecutionMode) -> Self {
    self.execute_mode = execute_mode;
    self
  }

  pub fn with_notes_in_flow(mut self, notes_in_flow: bool) -> Self {
    self.notes_in_flow = Some(notes_in_flow);
    self
  }

  pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
    self.notes = Some(notes.into());
    self
  }

  pub fn with_on_error(mut self, on_error: OnErrorBehavior) -> Self {
    self.on_error = on_error;
    self
  }

  pub fn with_max_tries(mut self, max_tries: u32) -> Self {
    self.max_tries = max_tries;
    self
  }

  pub fn with_wait_between_tries(mut self, wait_between_tries: Option<u64>) -> Self {
    self.wait_between_tries = wait_between_tries;
    self
  }

  pub fn with_timeout(mut self, timeout: Option<u64>) -> Self {
    self.timeout = timeout;
    self
  }

  pub fn display_name(&self) -> &str {
    self.display_name.as_deref().unwrap_or(&self.name)
  }

  /// 设置凭证 (legacy method for backwards compatibility)
  pub fn set_credentials(&mut self, credentials: HashMap<String, CredentialInfo>) -> &mut Self {
    self.credentials = Some(credentials);
    self
  }

  /// 设置备注 (legacy method for backwards compatibility)
  pub fn set_notes(&mut self, notes: String) -> &mut Self {
    self.notes = Some(notes);
    self
  }

  /// 设置Webhook ID (legacy method for backwards compatibility)
  pub fn set_webhook_id(&mut self, webhook_id: String) -> &mut Self {
    self.webhook_id = Some(webhook_id);
    self
  }

  /// 获取指定字段的参数值
  pub fn get_parameter<T>(&self, field: &str) -> Result<T, ValidationError>
  where
    T: DeserializeOwned,
  {
    self.parameters.get_parameter(field)
  }

  /// 获取指定字段的参数值（可选）
  pub fn get_optional_parameter<T>(&self, field: &str) -> Option<T>
  where
    T: DeserializeOwned,
  {
    self.parameters.get_optional_parameter(field)
  }
}
