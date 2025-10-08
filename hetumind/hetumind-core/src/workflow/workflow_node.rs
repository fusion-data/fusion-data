use fusion_common::ahash::HashMap;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_repr::{Deserialize_repr, Serialize_repr};
use typed_builder::TypedBuilder;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr, Default)]
#[repr(i32)]
pub enum NodeExecutionMode {
  /// 每次执行一个 item
  #[default]
  EachItem = 1,
  /// 将所有
  All = 2,
}

/// 错误处理策略
#[derive(Debug, Clone, Default, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum OnErrorBehavior {
  /// 停止工作流
  #[default]
  StopWorkflow = 1,
  /// 继续执行，并跳过当前节点
  Continue = 2,
  /// 输出错误到指定端口
  ContinueErrorOutput = 3,
}

/// 节点固定数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PinData {
  /// 节点固定数据, 用于存储节点执行过程中需要保存的数据
  /// - key: 节点 ID
  /// - value: 节点执行数据（结果）
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

/// 工作流节点
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct WorkflowNode {
  /// 流程内节点唯一标识名。可通过此名字访问其它 node，比如：访问其它 node 的数据。
  pub name: NodeName,

  /// 节点类型
  pub kind: NodeKind,

  /// 节点显示名称，界面上节点显示的名称。未设置时使用 name 的值。
  #[builder(default, setter(into, strip_option))]
  pub display_name: Option<String>,

  /// 节点参数。
  /// NodeDefinition.properties 限制可使用的参数。
  #[builder(default)]
  pub parameters: ParameterMap,

  /// 节点在画布上的位置 [x, y]
  #[builder(default, setter(into, strip_option))]
  pub position: Option<Position>,

  /// Webhook ID (可选)
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default, setter(into, strip_option))]
  pub webhook_id: Option<String>,

  /// 凭证信息 (可选)
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default, setter(strip_option))]
  pub credentials: Option<HashMap<String, CredentialInfo>>,

  /// 始终输出数据 (可选)
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default, setter(strip_option))]
  pub always_output_data: Option<bool>,

  /// 只执行一次 (可选)。为 true 时将 items 作为整体
  #[serde(default)]
  #[builder(default)]
  pub execute_mode: NodeExecutionMode,

  /// 在流程中显示备注 (可选)
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default, setter(strip_option))]
  pub notes_in_flow: Option<bool>,

  /// 备注信息 (可选)
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default, setter(strip_option))]
  pub notes: Option<String>,

  /// 错误处理策略，默认为 StopWorkflow
  #[serde(default)]
  #[builder(default)]
  pub on_error: OnErrorBehavior,

  /// 失败时最大重试次数，默认为 0，表示不重试
  #[serde(default)]
  #[builder(default)]
  pub max_tries: u32,

  /// 重试间隔（毫秒）
  #[builder(default, setter(strip_option))]
  pub wait_between_tries: Option<u64>,

  /// 超时时间（秒）
  #[builder(default, setter(strip_option))]
  pub timeout: Option<u64>,
}

impl WorkflowNode {
  pub fn display_name(&self) -> &str {
    self.display_name.as_deref().unwrap_or(&self.name)
  }

  /// 设置凭证
  pub fn with_credentials(&mut self, credentials: HashMap<String, CredentialInfo>) -> &Self {
    self.credentials = Some(credentials);
    self
  }

  /// 设置备注
  pub fn with_notes(&mut self, notes: String) -> &Self {
    self.notes = Some(notes);
    self
  }

  /// 设置Webhook ID
  pub fn with_webhook_id(&mut self, webhook_id: String) -> &Self {
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
