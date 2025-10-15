use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::{
  version::Version,
  workflow::{
    ConnectionKind, DataSource, ExecutionData, ExecutionDataItems, ExecutionDataMap, InputPortConfig, NodeDefinition,
    NodeExecutable, NodeExecutionContext, NodeExecutionError, NodeProperty, NodePropertyKind, OutputPortConfig,
    RegistrationError, make_execution_data_map,
  },
};
use serde_json::json;

use super::{
  EditFieldsConfig, FieldOperation, OperationMode, utils::apply_field_operations, utils::apply_json_template,
};

/// Edit Fields 数据编辑节点 V1
///
/// 参考 n8n 的 Edit Fields 节点设计（v3.4），用于编辑、修改或删除数据字段。
/// 支持两种操作模式：Manual Mapping（手动映射）和 JSON（自定义JSON）。
///
/// # 主要功能特性
/// - **双模式操作**: Manual Mapping（手动映射）和 JSON（自定义JSON）两种模式
/// - **灵活字段操作**: 支持添加、修改、删除、复制、增加、追加等多种操作
/// - **多种数据类型**: String、Number、Boolean、Array、Object 及其转换
/// - **输出控制**: 四种包含模式（全部、无、选定、排除除外）
/// - **点表示法支持**: 支持嵌套字段访问和设置
/// - **二进制数据处理**: 可选择包含或剥离二进制数据
/// - **类型转换控制**: 可忽略类型转换错误以提供更灵活的处理
/// - **项目复制功能**: 支持测试和调试时的项目复制
///
/// # 操作类型
/// - `Set`: 设置字段值
/// - `Remove`: 删除字段
/// - `Copy`: 从其他字段复制值
/// - `Increment`: 数值增加
/// - `Append`: 数组追加元素
/// - `Prepend`: 数组前置元素
/// - `Multiply`: 数值乘法
/// - `Replace`: 字符串替换
/// - `Split`: 字符串分割为数组
/// - `Join`: 数组连接为字符串
///
/// # 输入/输出
/// - 输入：任意 JSON 数据
/// - 输出：修改后的 JSON 数据
#[derive(Debug)]
pub struct EditFieldsV1 {
  pub definition: Arc<NodeDefinition>,
}

#[async_trait]
impl NodeExecutable for EditFieldsV1 {
  fn definition(&self) -> Arc<NodeDefinition> {
    self.definition.clone()
  }

  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    let node = context.current_node()?;
    log::info!(
      "开始执行 Edit Fields 数据编辑节点 workflow_id:{}, node_name:{}, node_kind:{}",
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
      log::warn!("Edit Fields 节点没有接收到输入数据");
      return Ok(make_execution_data_map(vec![(
        ConnectionKind::Main,
        vec![ExecutionDataItems::new_items(Default::default())],
      )]));
    };

    // 解析节点配置
    let config = self.parse_node_config(node)?;

    // 验证配置
    config
      .validate()
      .map_err(|e| NodeExecutionError::DataProcessingError { message: format!("Invalid node configuration: {}", e) })?;

    log::debug!("Edit Fields 配置: mode={:?}, options={:?}", config.mode, config.options);

    // 处理每个输入数据项
    let mut processed_items = Vec::new();
    for (index, input_item) in input_items.iter().enumerate() {
      let processed_data = match config.mode {
        OperationMode::Manual => {
          if let Some(manual_config) = &config.manual_config {
            apply_field_operations(
              input_item.json(),
              &manual_config.fields,
              &manual_config.include_mode,
              &manual_config.selected_fields,
              &config.options,
              index,
            )?
          } else {
            input_item.json().clone()
          }
        }
        OperationMode::Json => {
          if let Some(json_config) = &config.json_config {
            apply_json_template(
              input_item.json(),
              &json_config.json_output,
              json_config.use_expressions,
              &config.options,
              index,
            )?
          } else {
            input_item.json().clone()
          }
        }
      };

      // 处理项目复制功能
      let duplicate_count = config.options.duplicate_count.unwrap_or(0);
      if config.options.duplicate_item.unwrap_or(false) && duplicate_count > 0 {
        for _ in 0..=duplicate_count {
          processed_items.push(ExecutionData::new_json(
            processed_data.clone(),
            Some(DataSource {
              node_name: context.current_node_name.clone(),
              output_port: ConnectionKind::Main,
              output_index: index,
            }),
          ));
        }
      } else {
        processed_items.push(ExecutionData::new_json(
          processed_data,
          Some(DataSource {
            node_name: context.current_node_name.clone(),
            output_port: ConnectionKind::Main,
            output_index: index,
          }),
        ));
      }
    }

    log::info!("Edit Fields 节点执行完成: 处理 {} 项数据", processed_items.len());

    Ok(make_execution_data_map(vec![(ConnectionKind::Main, vec![ExecutionDataItems::new_items(processed_items)])]))
  }
}

impl EditFieldsV1 {
  /// 解析节点配置
  fn parse_node_config(
    &self,
    node: &hetumind_core::workflow::WorkflowNode,
  ) -> Result<EditFieldsConfig, NodeExecutionError> {
    // 解析操作模式
    let mode_str: String = node.get_parameter("mode").unwrap_or_else(|_| "manual".to_string());
    let mode = match mode_str.as_str() {
      "manual" => OperationMode::Manual,
      "json" => OperationMode::Json,
      _ => {
        return Err(NodeExecutionError::DataProcessingError {
          message: format!("Invalid operation mode: {}", mode_str),
        });
      }
    };

    // 解析手动配置
    let manual_config = if mode == OperationMode::Manual {
      let fields: Vec<FieldOperation> = node.get_parameter("fields").unwrap_or_default();

      // 解析输出包含模式
      let include_mode_str: String = node.get_parameter("include_mode").unwrap_or_else(|_| "all".to_string());
      let include_mode = match include_mode_str.as_str() {
        "all" => super::IncludeMode::All,
        "none" => super::IncludeMode::None,
        "selected" => super::IncludeMode::Selected,
        "except" => super::IncludeMode::Except,
        _ => {
          return Err(NodeExecutionError::DataProcessingError {
            message: format!("Invalid include mode: {}", include_mode_str),
          });
        }
      };

      // 解析选定字段
      let selected_fields: Option<Vec<String>> = node.get_optional_parameter("selected_fields");

      Some(super::ManualModeConfig { fields, include_mode, selected_fields })
    } else {
      None
    };

    // 解析 JSON 配置
    let json_config = if mode == OperationMode::Json {
      let json_output: String = node.get_parameter("json_output").map_err(|e| {
        NodeExecutionError::DataProcessingError { message: format!("Failed to parse json_output parameter: {}", e) }
      })?;

      let use_expressions: bool = node.get_parameter("use_expressions").unwrap_or(true);
      let validate_json: bool = node.get_parameter("validate_json").unwrap_or(true);

      let error_handling_str: String =
        node.get_parameter("error_handling").unwrap_or_else(|_| "stop_execution".to_string());
      let error_handling = match error_handling_str.as_str() {
        "stop_execution" => super::JsonErrorHandling::StopExecution,
        "use_original_data" => super::JsonErrorHandling::UseOriginalData,
        "return_error" => super::JsonErrorHandling::ReturnError,
        "skip_item" => super::JsonErrorHandling::SkipItem,
        _ => {
          return Err(NodeExecutionError::DataProcessingError {
            message: format!("Invalid error handling strategy: {}", error_handling_str),
          });
        }
      };

      Some(super::JsonModeConfig { json_output, use_expressions, validate_json, error_handling })
    } else {
      None
    };

    // 解析选项
    let options = super::EditFieldsOptions {
      dot_notation: node.get_optional_parameter("dot_notation"),
      ignore_conversion_errors: node.get_optional_parameter("ignore_conversion_errors"),
      binary_data_mode: node.get_optional_parameter("binary_data_mode"),
      keep_original_type: node.get_optional_parameter("keep_original_type"),
      duplicate_item: node.get_optional_parameter("duplicate_item"),
      duplicate_count: node.get_optional_parameter("duplicate_count"),
      debug_mode: node.get_optional_parameter("debug_mode"),
    };

    Ok(EditFieldsConfig { mode, manual_config, json_config, options })
  }
}

impl TryFrom<NodeDefinition> for EditFieldsV1 {
  type Error = RegistrationError;

  fn try_from(base: NodeDefinition) -> Result<Self, Self::Error> {
    let definition = base
      .with_version(Version::new(1, 0, 0))
      .add_input(InputPortConfig::new(ConnectionKind::Main, "Input"))
      .add_output(OutputPortConfig::new(ConnectionKind::Main, "Output"))
      .add_property(
        // 操作模式
        NodeProperty::new(NodePropertyKind::Options)
          .with_display_name("Operation Mode")
          .with_name("mode")
          .with_required(true)
          .with_description("Select the operation mode for field editing")
          .with_value(json!("manual"))
          .with_options(vec![
            Box::new(NodeProperty::new_option("Manual Mapping", "manual", json!("manual"), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option("JSON", "json", json!("json"), NodePropertyKind::String)),
          ]),
      )
      .add_property(
        // 手动映射模式配置
        NodeProperty::new(NodePropertyKind::Collection)
          .with_display_name("Field Operations")
          .with_name("fields")
          .with_required(false)
          .with_description("Field operations to apply (Manual Mapping mode)")
          .with_value(json!([])),
      )
      .add_property(
        // JSON 模式配置
        NodeProperty::new(NodePropertyKind::String)
          .with_display_name("JSON Output Template")
          .with_name("json_output")
          .with_required(false)
          .with_description("JSON template for output (JSON mode)")
          .with_value(json!("{}")),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Boolean)
          .with_display_name("Use Expressions")
          .with_name("use_expressions")
          .with_required(false)
          .with_description("Enable expression support in JSON template")
          .with_value(json!(true)),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Boolean)
          .with_display_name("Validate JSON")
          .with_name("validate_json")
          .with_required(false)
          .with_description("Validate JSON format before processing")
          .with_value(json!(true)),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Options)
          .with_display_name("JSON Error Handling")
          .with_name("error_handling")
          .with_required(false)
          .with_description("How to handle JSON errors")
          .with_value(json!("stop_execution"))
          .with_options(vec![
            Box::new(NodeProperty::new_option(
              "Stop Execution",
              "stop_execution",
              json!("stop_execution"),
              NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option(
              "Use Original Data",
              "use_original_data",
              json!("use_original_data"),
              NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option(
              "Return Error",
              "return_error",
              json!("return_error"),
              NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option("Skip Item", "skip_item", json!("skip_item"), NodePropertyKind::String)),
          ]),
      )
      .add_property(
        // 输出控制
        NodeProperty::new(NodePropertyKind::Options)
          .with_display_name("Include Mode")
          .with_name("include_mode")
          .with_required(false)
          .with_description("Which input fields to include in output")
          .with_value(json!("all"))
          .with_options(vec![
            Box::new(NodeProperty::new_option("All", "all", json!("all"), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option("None", "none", json!("none"), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option("Selected", "selected", json!("selected"), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option("Except", "except", json!("except"), NodePropertyKind::String)),
          ]),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::String)
          .with_display_name("Selected Fields")
          .with_name("selected_fields")
          .with_required(false)
          .with_description("Fields to include or exclude")
          .with_value(json!([])),
      )
      .add_property(
        // 高级选项
        NodeProperty::new(NodePropertyKind::Boolean)
          .with_display_name("Enable Dot Notation")
          .with_name("dot_notation")
          .with_required(false)
          .with_description("Enable dot notation for nested field access")
          .with_value(json!(true)),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Boolean)
          .with_display_name("Ignore Conversion Errors")
          .with_name("ignore_conversion_errors")
          .with_required(false)
          .with_description("Ignore type conversion errors and continue processing")
          .with_value(json!(false)),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Options)
          .with_display_name("Binary Data Mode")
          .with_name("binary_data_mode")
          .with_required(false)
          .with_description("How to handle binary data")
          .with_value(json!("auto"))
          .with_options(vec![
            Box::new(NodeProperty::new_option("Include", "include", json!("include"), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option("Strip", "strip", json!("strip"), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option("Auto", "auto", json!("auto"), NodePropertyKind::String)),
          ]),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Boolean)
          .with_display_name("Keep Original Type")
          .with_name("keep_original_type")
          .with_required(false)
          .with_description("Try to preserve original data types")
          .with_value(json!(false)),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Boolean)
          .with_display_name("Duplicate Item")
          .with_name("duplicate_item")
          .with_required(false)
          .with_description("Duplicate output items for testing")
          .with_value(json!(false)),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Number)
          .with_display_name("Duplicate Count")
          .with_name("duplicate_count")
          .with_required(false)
          .with_description("Number of times to duplicate each item")
          .with_value(json!(0)),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Boolean)
          .with_display_name("Debug Mode")
          .with_name("debug_mode")
          .with_required(false)
          .with_description("Enable debug mode with detailed logging")
          .with_value(json!(false)),
      );

    Ok(Self { definition: Arc::new(definition) })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::constants::EDIT_FIELDS_NODE_KIND;
  use hetumind_core::{
    version::Version,
    workflow::{NodeDefinition, NodeGroupKind, NodeKind, ParameterMap, WorkflowNode},
  };
  use serde_json::json;

  #[test]
  fn test_parse_manual_config() {
    let builder = NodeDefinition::new(EDIT_FIELDS_NODE_KIND, "Edit Fields")
      .with_version(Version::new(1, 0, 0))
      .with_description("Test node")
      .with_icon("edit")
      .add_group(NodeGroupKind::Transform)
      .add_group(NodeGroupKind::Input)
      .add_group(NodeGroupKind::Output);
    let v1 = EditFieldsV1::try_from(builder).unwrap();

    // 创建参数映射
    let mut param_map = serde_json::Map::new();
    param_map.insert("mode".to_string(), json!("manual"));
    param_map.insert("fields".to_string(), json!([]));
    param_map.insert("include_mode".to_string(), json!("all"));

    // 模拟节点参数
    let node = WorkflowNode::new(NodeKind::from(EDIT_FIELDS_NODE_KIND), "test_node")
      .with_display_name("Test Node")
      .with_parameters(ParameterMap::from(param_map));

    let config = v1.parse_node_config(&node).unwrap();
    assert_eq!(config.mode, OperationMode::Manual);
    assert!(config.manual_config.is_some());
    assert!(config.json_config.is_none());
  }

  #[test]
  fn test_parse_json_config() {
    let builder = NodeDefinition::new(EDIT_FIELDS_NODE_KIND, "Edit Fields")
      .with_version(Version::new(1, 0, 0))
      .add_group(NodeGroupKind::Transform)
      .add_group(NodeGroupKind::Input)
      .add_group(NodeGroupKind::Output)
      .with_description("Test node")
      .with_icon("edit");
    let v1 = EditFieldsV1::try_from(builder).unwrap();

    // 创建参数映射
    let mut param_map = serde_json::Map::new();
    param_map.insert("mode".to_string(), json!("json"));
    param_map.insert("json_output".to_string(), json!("{ \"result\": \"{{value}}\" }"));
    param_map.insert("use_expressions".to_string(), json!(true));
    param_map.insert("validate_json".to_string(), json!(true));
    param_map.insert("error_handling".to_string(), json!("stop_execution"));

    // 模拟节点参数
    let node = WorkflowNode::new(NodeKind::from(EDIT_FIELDS_NODE_KIND), "test_node")
      .with_display_name("Test Node")
      .with_parameters(ParameterMap::from(param_map));

    let config = v1.parse_node_config(&node).unwrap();
    assert_eq!(config.mode, OperationMode::Json);
    assert!(config.json_config.is_some());
    assert!(config.manual_config.is_none());
  }
}
