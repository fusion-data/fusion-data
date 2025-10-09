use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::{
  version::Version,
  workflow::{
    ConnectionKind, DataSource, ExecutionData, ExecutionDataItems, ExecutionDataMap, InputPortConfig, NodeDefinition,
    NodeDefinitionBuilder, NodeExecutable, NodeExecutionContext, NodeExecutionError, NodeProperty, NodePropertyKind,
    OutputPortConfig, RegistrationError, make_execution_data_map,
  },
};
use serde_json::json;

use super::{SetOperation, utils::apply_operations};

/// Set 数据设置节点 V1
///
/// 用于对输入数据进行字段设置、修改或删除操作。
/// 支持多种数据来源和操作类型，可以构建复杂的数据转换逻辑。
///
/// # 操作类型
/// - `Set`: 设置字段值
/// - `Remove`: 删除字段
/// - `Copy`: 从其他字段复制值
/// - `Increment`: 数值增加
/// - `Append`: 数组追加元素
///
/// # 数据来源
/// - `Static`: 静态值
/// - `Expression`: 表达式（如 $.field.subfield）
/// - `CurrentTimestamp`: 当前时间戳
/// - `Random`: 随机值
///
/// # 输入/输出
/// - 输入：任意 JSON 数据
/// - 输出：修改后的 JSON 数据
#[derive(Debug)]
pub struct SetV1 {
  pub definition: Arc<NodeDefinition>,
}

#[async_trait]
impl NodeExecutable for SetV1 {
  fn definition(&self) -> Arc<NodeDefinition> {
    self.definition.clone()
  }

  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    let node = context.current_node()?;
    log::info!(
      "开始执行 Set 数据设置节点 workflow_id:{}, node_name:{}, node_kind:{}",
      context.workflow.id,
      node.name,
      node.kind
    );

    // 获取输入数据
    let input_items = if let Some(input_collection) = context.get_input_items(ConnectionKind::Main, 0)
      && let ExecutionDataItems::Items(input_data) = input_collection
    {
      input_data
    } else {
      log::warn!("Set 节点没有接收到输入数据");
      return Ok(make_execution_data_map(vec![(
        ConnectionKind::Main,
        vec![ExecutionDataItems::new_items(Default::default())],
      )]));
    };

    // 获取操作配置
    let operations: Vec<SetOperation> = node.get_parameter("operations")?;

    // 验证操作配置
    for operation in &operations {
      if let Err(e) = operation.validate() {
        return Err(NodeExecutionError::DataProcessingError {
          message: format!("Invalid operation configuration: {}", e),
        });
      }
    }

    log::debug!("Set 操作配置: {} 个操作", operations.len());

    // 处理每个输入数据项
    let mut processed_items = Vec::new();
    for (index, input_item) in input_items.iter().enumerate() {
      let modified_data = apply_operations(input_item.json(), &operations)?;

      processed_items.push(ExecutionData::new_json(
        modified_data,
        Some(DataSource {
          node_name: context.current_node_name.clone(),
          output_port: ConnectionKind::Main,
          output_index: index,
        }),
      ));
    }

    log::info!("Set 节点执行完成: 处理 {} 项数据", processed_items.len());

    Ok(make_execution_data_map(vec![(ConnectionKind::Main, vec![ExecutionDataItems::new_items(processed_items)])]))
  }
}

impl TryFrom<NodeDefinitionBuilder> for SetV1 {
  type Error = RegistrationError;

  fn try_from(mut base: NodeDefinitionBuilder) -> Result<Self, Self::Error> {
    base
      .version(Version::new(1, 0, 0))
      .inputs([InputPortConfig::builder().kind(ConnectionKind::Main).display_name("Input").build()])
      .outputs([OutputPortConfig::builder().kind(ConnectionKind::Main).display_name("Output").build()])
      .properties([
        NodeProperty::builder()
          .display_name("操作列表")
          .name("operations")
          .kind(NodePropertyKind::Collection)
          .required(true)
          .description("要执行的设置操作列表")
          .placeholder("添加操作...")
          .value(json!([]))
          .build(),
        NodeProperty::builder()
          .display_name("保持原始类型")
          .name("keep_original_type")
          .kind(NodePropertyKind::Boolean)
          .required(false)
          .description("是否尝试保持字段的原始数据类型")
          .value(json!(false))
          .build(),
      ]);

    let definition = base.build()?;

    Ok(Self { definition: Arc::new(definition) })
  }
}
