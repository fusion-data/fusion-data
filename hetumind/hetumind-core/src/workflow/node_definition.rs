use derive_builder::Builder;
use fusion_common::helper::default_bool_false;
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::types::IconColor;

use super::{InputPortConfig, NodeGroupKind, NodeKind, NodeProperties, OutputPortConfig};

/// Node 定义
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct NodeDefinition {
  /// 唯一标识一种类型的节点，可以存在多个不同的版本。PK
  #[builder(setter(into))]
  pub kind: NodeKind,

  /// 版本号
  #[builder(setter(into))]
  pub version: Version,

  /// 节点分组
  #[builder(setter(into))]
  pub groups: Vec<NodeGroupKind>,

  /// 显示名称
  #[builder(setter(into))]
  pub display_name: String,

  /// 节点描述
  #[builder(default, setter(into, strip_option))]
  pub description: Option<String>,

  /// 输入端口定义。与 node 的 outputs 对应。
  #[builder(default, setter(into))]
  pub inputs: Vec<InputPortConfig>,

  /// 输出端口定义。
  #[builder(default, setter(into))]
  pub outputs: Vec<OutputPortConfig>,

  /// 属性定义
  #[builder(default, setter(into))]
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
