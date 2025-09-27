use fusion_common::helper::default_bool_false;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::types::IconColor;

use super::{InputPortConfig, NodeGroupKind, NodeKind, NodeProperties, OutputPortConfig};

/// Node 定义
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct NodeDefinition {
  /// 节点类型, PK
  pub kind: NodeKind,

  /// 默认版本号。不设置则使用最新版本（versions 中的最大值）。
  #[builder(default, setter(strip_option))]
  pub default_version: Option<u16>,

  /// 节点支持的版本号列表
  #[builder(setter(into))]
  pub versions: Vec<u16>,

  /// 节点分组
  pub groups: Vec<NodeGroupKind>,

  /// 显示名称
  #[builder(setter(into))]
  pub display_name: String,

  /// 节点描述
  #[builder(default, setter(into, strip_option))]
  pub description: Option<String>,

  /// 输入端口定义。与 node 的 outputs 对应。
  #[builder(default)]
  pub inputs: Vec<InputPortConfig>,

  /// 输出端口定义。
  #[builder(default)]
  pub outputs: Vec<OutputPortConfig>,

  /// 属性定义
  #[builder(default)]
  pub properties: Vec<NodeProperties>,

  /// 官方文档URL
  #[builder(default, setter(into, strip_option))]
  pub document_url: Option<String>,

  /// 子标题
  #[builder(default, setter(into, strip_option))]
  pub sub_title: Option<String>,

  /// 是否隐藏
  #[builder(default = default_bool_false())]
  pub hidden: bool,

  /// 节点图标。（支持 FontAwesome 图标或文件图标）或自定义图标URL
  #[builder(default, setter(into, strip_option))]
  pub icon: Option<String>,

  /// 图标颜色
  #[builder(default, setter(into, strip_option))]
  pub icon_color: Option<IconColor>,

  /// 自定义图标URL
  #[builder(default, setter(into, strip_option))]
  pub icon_url: Option<String>,

  /// 徽章图标URL
  #[builder(default, setter(into, strip_option))]
  pub badge_icon_url: Option<String>,
}
