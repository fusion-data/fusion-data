//! Tooling: 从 NodeDefinition 生成简化 JSON Schema，并提供运行时校验能力
//!
//! 约束：
//! - 仅收集 NodeProperty.additional_properties.from_ai=true 的参数暴露为工具输入
//! - 使用 jsonschema crate 在运行时进行参数校验
//! - 不引入审计/迁移逻辑

use jsonschema::Validator;
use serde_json::json;

use crate::workflow::{NodeDefinition, NodeExecutionError, NodeProperty, NodePropertyKind, Tool};

/// 根据 NodeDefinition.properties 构建简化 JSON Schema
/// 规则：
/// - 仅包含 additional_properties.from_ai=true 的属性
/// - NodePropertyKind → JSON Schema 类型映射：
///   - String → { type: "string" }
///   - Number → { type: "number" }
///   - Boolean → { type: "boolean" }
///   - Json/Options/FixedCollection 等 → { }
/// - required 根据 NodeProperty.required 设置
pub fn build_json_schema_from_properties(properties: &[NodeProperty]) -> serde_json::Value {
  let mut required: Vec<String> = Vec::new();
  let mut props = serde_json::Map::new();

  for p in properties {
    let from_ai = p.additional_properties.get("from_ai").and_then(|v| v.as_bool()).unwrap_or(false);

    if !from_ai {
      continue;
    }

    let schema_type = match p.kind {
      NodePropertyKind::String => json!({ "type": "string" }),
      NodePropertyKind::Number => json!({ "type": "number" }),
      NodePropertyKind::Boolean => json!({ "type": "boolean" }),
      // 宽松接受任意 JSON（由具体工具处理）
      _ => json!({}),
    };

    props.insert(p.name.clone(), schema_type);
    if p.required {
      required.push(p.name.clone());
    }
  }

  json!({
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "type": "object",
    "properties": props,
    "required": required,
    "additionalProperties": false
  })
}

/// 根据 NodeDefinition 生成简化 JSON Schema，并返回可用于校验的编译器
pub fn compile_tool_schema(def: &NodeDefinition) -> Result<Validator, NodeExecutionError> {
  let schema = build_json_schema_from_properties(&def.properties);
  let validator = jsonschema::validator_for(&schema)
    .map_err(|e| NodeExecutionError::ConfigurationError(format!("JSON Schema compile error: {}", e)))?;
  Ok(validator)
}

/// 将 NodeDefinition 包装为 Agent 可调用的 Tool
/// - 名称使用 NodeKind（kind.as_str()）
/// - 描述优先使用 definition.description，否则回退到 display_name
/// - 参数使用从 properties 构建的简化 JSON Schema
pub fn create_node_as_tool(def: &NodeDefinition) -> Tool {
  let schema = build_json_schema_from_properties(&def.properties);
  Tool {
    name: def.kind.as_str().to_string(),
    description: def.description.clone().unwrap_or_else(|| def.display_name.clone()),
    parameters: schema,
  }
}

#[cfg(test)]
mod tests {
  use semver::Version;

  use crate::workflow::{NodeDefinition, NodeProperty, NodePropertyKind};

  use super::*;

  /// 构建 JSON Schema 并验证必填字段缺失时的错误路径
  #[test]
  fn test_build_and_validate_schema_required() {
    // 构建节点定义，收集 from_ai 参数
    let mut def = NodeDefinition::new("tool::http", "HTTP Tool").with_version(Version::new(1, 0, 0));

    let p_query = NodeProperty::new(NodePropertyKind::String)
      .with_name("query")
      .with_display_name("查询")
      .with_required(true)
      .with_additional_property("from_ai", true);

    let p_limit = NodeProperty::new(NodePropertyKind::Number)
      .with_name("limit")
      .with_display_name("限制")
      .with_required(false)
      .with_additional_property("from_ai", true);

    def = def.add_property(p_query).add_property(p_limit);

    let validator = compile_tool_schema(&def).unwrap();

    // 合法参数
    let ok_instance = serde_json::json!({ "query": "rust" });
    assert!(validator.is_valid(&ok_instance));

    // 缺失必填参数 query
    let bad_instance = serde_json::json!({});
    assert!(!validator.is_valid(&bad_instance));

    let result = validator.validate(&bad_instance);
    assert!(result.is_err());
  }

  /// 基于 NodeDefinition 创建 Tool，并确认参数 schema 中包含 from_ai 字段
  #[test]
  fn test_create_node_as_tool() {
    let def = NodeDefinition::new("tool::kv", "KV Tool")
      .add_property(
        NodeProperty::new(NodePropertyKind::String)
          .with_name("key")
          .with_display_name("键")
          .with_required(true)
          .with_additional_property("from_ai", true),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::String)
          .with_name("namespace")
          .with_display_name("命名空间")
          .with_required(false)
          .with_additional_property("from_ai", true),
      );

    let tool = create_node_as_tool(&def);
    assert_eq!(tool.name, "tool::kv");
    assert!(tool.parameters.get("properties").is_some());
    let props = tool.parameters.get("properties").unwrap().as_object().unwrap();
    assert!(props.contains_key("key"));
    assert!(props.contains_key("namespace"));
  }
}
