use crate::core::if_node::utils::{resolve_value, to_number};
use hetumind_core::types::JsonValue;
use serde_json::json;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_json_path_basic() {
    let input_data = json!({
        "user": {
            "name": "John",
            "age": 30,
            "address": {
                "city": "New York"
            }
        },
        "items": ["apple", "banana", "orange"]
    });

    // Test simple property access
    let result = resolve_value(&json!("$.user.name"), &input_data).unwrap();
    assert_eq!(result, json!("John"));

    // Test nested property access
    let result = resolve_value(&json!("$.user.address.city"), &input_data).unwrap();
    assert_eq!(result, json!("New York"));

    // Test array access
    let result = resolve_value(&json!("$.items[0]"), &input_data).unwrap();
    assert_eq!(result, json!("apple"));

    // Test array slice
    let result = resolve_value(&json!("$.items[0:2]"), &input_data).unwrap();
    assert_eq!(result, json!(["apple", "banana"]));
  }

  #[test]
  fn test_json_path_fallback_to_simple() {
    let input_data = json!({
        "user": {
            "name": "Alice"
        }
    });

    // Test backward compatibility with simple path
    let result = resolve_value(&json!("$.user.name"), &input_data).unwrap();
    assert_eq!(result, json!("Alice"));
  }

  #[test]
  fn test_json_path_non_existent() {
    let input_data = json!({
        "user": {
            "name": "Bob"
        }
    });

    // Test non-existent path returns null
    let result = resolve_value(&json!("$.user.email"), &input_data).unwrap();
    assert_eq!(result, json!(null));
  }

  #[test]
  fn test_enhanced_number_conversion() {
    // Test string number with comma
    let result = to_number(&json!("1,234.56")).unwrap();
    assert_eq!(result, 1234.56);

    // Test boolean to number conversion
    let result = to_number(&json!(true)).unwrap();
    assert_eq!(result, 1.0);

    let result = to_number(&json!(false)).unwrap();
    assert_eq!(result, 0.0);

    // Test null to number conversion
    let result = to_number(&json!(null)).unwrap();
    assert_eq!(result, 0.0);

    // Test string boolean to number
    let result = to_number(&json!("true")).unwrap();
    assert_eq!(result, 1.0);

    let result = to_number(&json!("false")).unwrap();
    assert_eq!(result, 0.0);
  }

  #[test]
  fn test_backward_compatibility_simple_path() {
    let input_data = json!({
        "level1": {
            "level2": {
                "value": "deep_value"
            }
        }
    });

    // Test the old simple dot notation still works
    let result = resolve_value(&json!("$.level1.level2.value"), &input_data).unwrap();
    assert_eq!(result, json!("deep_value"));
  }
}
