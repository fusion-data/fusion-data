use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::types::JsonValue;

use super::{
  CredentialKind, DataPathRequirement, DisplayOptions, FieldType, NodePropertyKind, NodePropertyKindOptions,
  NodePropertyMode, NodePropertyRouting, NodePropertyValueExtractor,
};

/// 节点属性定义（元数据）
#[derive(Debug, Clone, Default, Serialize, Deserialize, TypedBuilder)]
pub struct NodeProperties {
  /// 显示名称
  #[builder(setter(into))]
  pub display_name: String,

  /// 在节点定义中唯一标识一个属性。实际工作流使用时，由工作流节点的 parameters 中的 key 表示。
  #[builder(setter(into))]
  pub name: String,

  /// 属性类型。UI 渲染方式依赖于此
  #[builder(default)]
  pub kind: NodePropertyKind,

  /// 针对特定类型的附加配置选项
  #[builder(default, setter(strip_option))]
  pub kind_options: Option<NodePropertyKindOptions>,

  /// 是否必填
  #[builder(default = true)]
  pub required: bool,

  /// 参数的默认值
  #[builder(default, setter(strip_option))]
  pub value: Option<JsonValue>,

  /// 参数的描述信息
  #[builder(default, setter(into, strip_option))]
  pub description: Option<String>,

  /// 参数的提示信息
  #[builder(default, setter(into, strip_option))]
  pub hint: Option<String>,

  /// 禁用选项
  #[builder(default, setter(strip_option))]
  pub disable_options: Option<DisplayOptions>,

  /// 显示选项
  #[builder(default, setter(strip_option))]
  pub display_options: Option<DisplayOptions>,

  /// 选项
  #[builder(default, setter(strip_option))]
  pub options: Option<Vec<Box<NodeProperties>>>,

  /// 输入框占位符文本
  #[builder(default, setter(into, strip_option))]
  pub placeholder: Option<String>,

  /// 是否为节点级别设置
  #[builder(default, setter(strip_option))]
  pub is_node_setting: Option<bool>,

  /// 是否禁止使用数据表达式
  #[builder(default, setter(strip_option))]
  pub no_data_expression: Option<bool>,

  /// API 路由配置
  #[builder(default, setter(strip_option))]
  pub routing: Option<NodePropertyRouting>,

  /// 支持的凭据类型列表
  #[builder(default, setter(strip_option))]
  pub credential_kinds: Option<Vec<CredentialKind>>,

  /// 值提取配置
  #[builder(default, setter(strip_option))]
  pub extract_value: Option<NodePropertyValueExtractor>,

  /// 参数的不同模式配置
  #[builder(default, setter(strip_option))]
  pub modes: Option<Vec<NodePropertyMode>>,

  /// 需要的数据路径类型
  #[builder(default, setter(strip_option))]
  pub requires_data_path: Option<DataPathRequirement>,

  /// 是否不从父级继承此参数
  #[builder(default, setter(strip_option))]
  pub do_not_inherit: Option<bool>,

  /// 用于验证和类型转换的预期类型
  #[builder(default, setter(strip_option))]
  pub validate_type: Option<FieldType>,

  /// 执行期间是否跳过验证
  #[builder(default, setter(strip_option))]
  pub ignore_validation_during_execution: Option<bool>,

  /// 扩展属性
  #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
  #[builder(default)]
  additional_properties: serde_json::Map<String, JsonValue>,
}

impl NodeProperties {
  pub fn new_option(
    display_name: impl Into<String>,
    name: impl Into<String>,
    value: JsonValue,
    kind: NodePropertyKind,
  ) -> Self {
    Self::builder().display_name(display_name).name(name).value(value).kind(kind).build()
  }
}
