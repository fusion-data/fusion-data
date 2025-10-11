use fusion_common::types::JsonValue;
use hetumind_core::workflow::NodeExecutionError;

/// 验证 JSON 结构
pub fn validate_json_structure(json: &JsonValue, required_fields: &[&str]) -> Result<(), NodeExecutionError> {
    if let JsonValue::Object(map) = json {
        for field in required_fields {
            if !map.contains_key(*field) {
                return Err(NodeExecutionError::InvalidInput(format!(
                    "Missing required field: {}", field
                )));
            }
        }
        Ok(())
    } else {
        Err(NodeExecutionError::InvalidInput("Expected JSON object".to_string()))
    }
}

/// 从 JSON 中提取字符串值
pub fn extract_string(json: &JsonValue, key: &str) -> Option<String> {
    json.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// 从 JSON 中提取数字值
pub fn extract_number<T: serde_json::Number>(json: &JsonValue, key: &str) -> Option<T> {
    json.get(key)
        .and_then(|v| v.as_number())
        .and_then(|n| serde_json::from_value(JsonValue::Number(n.clone())).ok())
}

/// 从 JSON 中提取布尔值
pub fn extract_bool(json: &JsonValue, key: &str) -> Option<bool> {
    json.get(key).and_then(|v| v.as_bool())
}

/// 构建错误消息
pub fn build_error_message(error: &str, context: &str) -> String {
    format!("{}: {}", context, error)
}

/// 解析工具调用参数
pub fn parse_tool_call_params(json: &JsonValue) -> Result<Vec<String>, NodeExecutionError> {
    match json {
        JsonValue::Array(arr) => {
            let params: Result<Vec<String>, _> = arr.iter()
                .map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            params.map_err(|_| NodeExecutionError::InvalidInput("Invalid array format".to_string()))
        },
        JsonValue::String(s) => Ok(vec![s.clone()]),
        _ => Err(NodeExecutionError::InvalidInput("Expected string or array".to_string()))
    }
}

/// 格式化时间戳
pub fn format_timestamp(timestamp: i64) -> String {
    chrono::DateTime::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| chrono::Utc::now())
        .format("%Y-%m-%d %H:%M:%S UTC")
        .to_string()
}

/// 创建响应数据结构
pub fn create_response_data(
    content: &str,
    metadata: Option<JsonValue>,
) -> JsonValue {
    let mut response = json!({
        "content": content,
        "timestamp": chrono::Utc::now().timestamp(),
    });

    if let Some(meta) = metadata {
        response["metadata"] = meta;
    }

    response
}

/// 验证工具调用格式
pub fn validate_tool_call_format(tool_call: &JsonValue) -> Result<(), NodeExecutionError> {
    validate_json_structure(tool_call, &["id", "tool_name", "parameters"])?;

    // 验证 ID 格式
    if let Some(id) = tool_call.get("id") {
        if !id.is_string() {
            return Err(NodeExecutionError::InvalidInput("Tool call ID must be a string".to_string()));
        }
    }

    // 验证工具名称格式
    if let Some(name) = tool_call.get("tool_name") {
        if !name.is_string() {
            return Err(NodeExecutionError::InvalidInput("Tool name must be a string".to_string()));
        }
    }

    Ok(())
}