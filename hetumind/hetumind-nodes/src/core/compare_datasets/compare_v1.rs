use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::{
  version::Version,
  workflow::{
    ConnectionKind, ExecutionData, ExecutionDataItems, ExecutionDataMap, InputPortConfig, NodeDefinition,
    FlowNode, NodeExecutionContext, NodeExecutionError, NodeProperty, NodePropertyKind, OutputPortConfig,
    RegistrationError, make_execution_data_map,
  },
};
use serde_json::json;

use super::{CompareDatasetsOperation, utils::compare_datasets};

/// Compare Datasets 数据集比较节点 V1
///
/// 用于比较两个输入数据集，并将结果分类为四种输出：
/// - 仅在数据集A中存在的记录
/// - 仅在数据集B中存在的记录
/// - 两个数据集中完全相同的记录
/// - 在两个数据集中都存在但有差异的记录
///
/// # 功能特性
/// - 多字段匹配支持，支持点表示法
/// - 模糊比较和精确比较
/// - 多种冲突解决策略
/// - 灵活的输出配置
/// - 详细的比较统计信息
#[derive(Debug)]
pub struct CompareDatasetsV1 {
  pub definition: Arc<NodeDefinition>,
}

impl CompareDatasetsV1 {
  /// 解析操作配置
  fn parse_operation_config(
    &self,
    node: &hetumind_core::workflow::NodeElement,
  ) -> Result<CompareDatasetsOperation, NodeExecutionError> {
    // Parse match fields
    let match_fields: Vec<serde_json::Value> = node.get_parameter("match_fields")?;

    let mut field_configs = Vec::new();
    for field_value in match_fields {
      let field_name: String = field_value
        .get("field_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| NodeExecutionError::DataProcessingError {
          message: "Field name is required for match fields".to_string(),
        })?
        .to_string();

      let is_key_field: bool = field_value.get("is_key_field").and_then(|v| v.as_bool()).unwrap_or(false);

      let comparison_mode_str: String =
        field_value.get("comparison_mode").and_then(|v| v.as_str()).unwrap_or("exact").to_string();

      let comparison_mode = match comparison_mode_str.as_str() {
        "exact" => super::ComparisonMode::Exact,
        "fuzzy" => super::ComparisonMode::Fuzzy,
        _ => {
          return Err(NodeExecutionError::DataProcessingError {
            message: format!("Invalid comparison mode: {}", comparison_mode_str),
          });
        }
      };

      let fuzzy_threshold: Option<f64> = field_value.get("fuzzy_threshold").and_then(|v| v.as_f64());

      let case_sensitive: bool = field_value.get("case_sensitive").and_then(|v| v.as_bool()).unwrap_or(true);

      let trim_whitespace: bool = field_value.get("trim_whitespace").and_then(|v| v.as_bool()).unwrap_or(true);

      field_configs.push(super::FieldMatchConfig {
        field_name,
        is_key_field,
        comparison_mode,
        fuzzy_threshold,
        case_sensitive,
        trim_whitespace,
      });
    }

    // Parse conflict resolution
    let conflict_resolution_str: String =
      node.get_parameter("conflict_resolution").unwrap_or_else(|_| "prefer_input_a".to_string());

    let conflict_resolution = match conflict_resolution_str.as_str() {
      "prefer_input_a" => super::ConflictResolution::PreferInputA,
      "prefer_input_b" => super::ConflictResolution::PreferInputB,
      "mix" => super::ConflictResolution::Mix,
      "include_both" => super::ConflictResolution::IncludeBoth,
      _ => {
        return Err(NodeExecutionError::DataProcessingError {
          message: format!("Invalid conflict resolution strategy: {}", conflict_resolution_str),
        });
      }
    };

    // Parse other options
    let include_all_fields: bool = node.get_optional_parameter("include_all_fields").unwrap_or(true);
    let enable_fuzzy_matching: bool = node.get_optional_parameter("enable_fuzzy_matching").unwrap_or(false);
    let fuzzy_threshold: f64 = node.get_optional_parameter("fuzzy_threshold").unwrap_or(0.8);
    let max_differences: Option<usize> = node.get_optional_parameter("max_differences");
    let sort_results: bool = node.get_optional_parameter("sort_results").unwrap_or(false);
    let sort_field: Option<String> = node.get_optional_parameter("sort_field");

    let operation = CompareDatasetsOperation {
      match_fields: field_configs,
      conflict_resolution,
      include_all_fields,
      enable_fuzzy_matching,
      fuzzy_threshold,
      max_differences,
      sort_results,
      sort_field,
    };

    // Validate the operation
    operation.validate().map_err(|e| NodeExecutionError::DataProcessingError {
      message: format!("Invalid operation configuration: {}", e),
    })?;

    Ok(operation)
  }
}

#[async_trait]
impl FlowNode for CompareDatasetsV1 {
  fn definition(&self) -> Arc<NodeDefinition> {
    self.definition.clone()
  }

  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    let node = context.current_node()?;
    log::info!(
      "开始执行 Compare Datasets 节点 workflow_id:{}, node_name:{}, node_kind:{}",
      context.workflow.id,
      node.name,
      node.kind
    );

    // 获取输入数据 - 需要两个输入
    let input_a = if let Some(input_collection) = context.get_input_items(ConnectionKind::Main, 0)
      && let ExecutionDataItems::Items(input_data) = input_collection
    {
      log::info!("Compare Datasets 节点接收到输入A: {} 个数据项", input_data.len());
      input_data
    } else {
      log::error!("Compare Datasets 节点没有接收到输入数据A");
      return Err(NodeExecutionError::InvalidInputData { connection_kind: ConnectionKind::Main, port_index: 0 });
    };

    let input_b = if let Some(input_collection) = context.get_input_items(ConnectionKind::Main, 1)
      && let ExecutionDataItems::Items(input_data) = input_collection
    {
      log::info!("Compare Datasets 节点接收到输入B: {} 个数据项", input_data.len());
      input_data
    } else {
      log::error!("Compare Datasets 节点没有接收到输入数据B");
      return Err(NodeExecutionError::InvalidInputData { connection_kind: ConnectionKind::Main, port_index: 1 });
    };

    // 解析操作配置
    let operation = self.parse_operation_config(node)?;
    log::debug!("Compare Datasets 操作配置解析完成");

    // 提取JSON数据
    let data_a: Vec<serde_json::Value> = input_a.iter().map(|item| item.json().clone()).collect();
    let data_b: Vec<serde_json::Value> = input_b.iter().map(|item| item.json().clone()).collect();

    // 执行比较
    let comparison_results = compare_datasets(&data_a, &data_b, &operation)
      .await
      .map_err(|e| NodeExecutionError::DataProcessingError { message: format!("Dataset comparison failed: {}", e) })?;

    log::info!(
      "Compare Datasets 执行完成 - 总计A:{}, B:{}, 仅A:{}, 仅B:{}, 相同:{}, 不同:{}",
      comparison_results.summary.total_a,
      comparison_results.summary.total_b,
      comparison_results.summary.count_in_a_only,
      comparison_results.summary.count_in_b_only,
      comparison_results.summary.count_same,
      comparison_results.summary.count_different
    );

    // 准备输出数据
    let output_a = comparison_results
      .in_a_only
      .iter()
      .map(|result| result.record_a.clone().unwrap_or_else(|| json!({})))
      .collect::<Vec<_>>();

    let output_b = comparison_results
      .in_b_only
      .iter()
      .map(|result| result.record_b.clone().unwrap_or_else(|| json!({})))
      .collect::<Vec<_>>();

    let output_same = comparison_results
      .same
      .iter()
      .map(|result| result.merged_record.clone().unwrap_or_else(|| json!({})))
      .collect::<Vec<_>>();

    let output_different = comparison_results
      .different
      .iter()
      .map(|result| result.merged_record.clone().unwrap_or_else(|| json!({})))
      .collect::<Vec<_>>();

    // 创建输出数据
    let output_items_a = output_a
      .into_iter()
      .enumerate()
      .map(|(index, data)| {
        ExecutionData::new_json(
          data,
          Some(hetumind_core::workflow::DataSource {
            node_name: context.current_node_name.clone(),
            output_port: ConnectionKind::Main,
            output_index: index,
          }),
        )
      })
      .collect();

    let output_items_b = output_b
      .into_iter()
      .enumerate()
      .map(|(index, data)| {
        ExecutionData::new_json(
          data,
          Some(hetumind_core::workflow::DataSource {
            node_name: context.current_node_name.clone(),
            output_port: ConnectionKind::Main,
            output_index: index,
          }),
        )
      })
      .collect();

    let output_items_same = output_same
      .into_iter()
      .enumerate()
      .map(|(index, data)| {
        ExecutionData::new_json(
          data,
          Some(hetumind_core::workflow::DataSource {
            node_name: context.current_node_name.clone(),
            output_port: ConnectionKind::Main,
            output_index: index,
          }),
        )
      })
      .collect();

    let output_items_different = output_different
      .into_iter()
      .enumerate()
      .map(|(index, data)| {
        ExecutionData::new_json(
          data,
          Some(hetumind_core::workflow::DataSource {
            node_name: context.current_node_name.clone(),
            output_port: ConnectionKind::Main,
            output_index: index,
          }),
        )
      })
      .collect();

    // 返回四个输出
    Ok(make_execution_data_map(vec![(
      ConnectionKind::Main,
      vec![
        ExecutionDataItems::new_items(output_items_a),
        ExecutionDataItems::new_items(output_items_b),
        ExecutionDataItems::new_items(output_items_same),
        ExecutionDataItems::new_items(output_items_different),
      ],
    )]))
  }
}

impl TryFrom<NodeDefinition> for CompareDatasetsV1 {
  type Error = RegistrationError;

  fn try_from(base: NodeDefinition) -> Result<Self, Self::Error> {
    let definition = base
      .with_version(Version::new(1, 0, 0))
      .add_input(InputPortConfig::new(ConnectionKind::Main, "Input Dataset A"))
      .add_input(InputPortConfig::new(ConnectionKind::Main, "Input Dataset B"))
      .add_output(OutputPortConfig::new(ConnectionKind::Main, "In A Only"))
      .add_output(OutputPortConfig::new(ConnectionKind::Main, "In B Only"))
      .add_output(OutputPortConfig::new(ConnectionKind::Main, "Same"))
      .add_output(OutputPortConfig::new(ConnectionKind::Main, "Different"))
      .add_property(
        // Match Fields Configuration
        NodeProperty::new(NodePropertyKind::Collection)
          .with_display_name("Match Fields")
          .with_name("match_fields")
          .with_required(true)
          .with_description("Fields to match datasets on")
          .with_value(json!([])),
      )
      .add_property(
        // Conflict Resolution Strategy
        NodeProperty::new(NodePropertyKind::Options)
          .with_display_name("Conflict Resolution")
          .with_name("conflict_resolution")
          .with_required(false)
          .with_description("How to handle differences between datasets")
          .with_value(json!("prefer_input_a"))
          .with_options(vec![
            Box::new(NodeProperty::new_option(
              "Prefer Input A",
              "prefer_input_a",
              json!("prefer_input_a"),
              NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option(
              "Prefer Input B",
              "prefer_input_b",
              json!("prefer_input_b"),
              NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option("Mix", "mix", json!("mix"), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option(
              "Include Both",
              "include_both",
              json!("include_both"),
              NodePropertyKind::String,
            )),
          ]),
      )
      .add_property(
        // Additional Options
        NodeProperty::new(NodePropertyKind::Boolean)
          .with_display_name("Include All Fields")
          .with_name("include_all_fields")
          .with_required(false)
          .with_description("Include all fields in output results")
          .with_value(json!(true)),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Boolean)
          .with_display_name("Enable Fuzzy Matching")
          .with_name("enable_fuzzy_matching")
          .with_required(false)
          .with_description("Enable fuzzy matching for all fields")
          .with_value(json!(false)),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Number)
          .with_display_name("Global Fuzzy Threshold")
          .with_name("fuzzy_threshold")
          .with_required(false)
          .with_description("Global threshold for fuzzy matching (0.0-1.0)")
          .with_value(json!(0.8)),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Number)
          .with_display_name("Max Differences")
          .with_name("max_differences")
          .with_required(false)
          .with_description("Maximum number of differences to include in output")
          .with_value(json!(null)),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Boolean)
          .with_display_name("Sort Results")
          .with_name("sort_results")
          .with_required(false)
          .with_description("Sort the output results")
          .with_value(json!(false)),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::String)
          .with_display_name("Sort Field")
          .with_name("sort_field")
          .with_required(false)
          .with_description("Field to sort results by")
          .with_value(json!("")),
      );

    Ok(Self { definition: Arc::new(definition) })
  }
}
