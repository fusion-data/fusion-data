use chrono::{NaiveTime, Timelike};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug)]
struct Policy {
  name: String,
  condition: Value,
  effect: String,
  resource: String,
  action: String,
}

#[derive(Debug)]
struct AccessRequest {
  subject: HashMap<String, Value>,
  resource: HashMap<String, Value>,
  action: String,
  environment: HashMap<String, Value>,
}

fn evaluate_condition(condition: &Value, request: &AccessRequest) -> bool {
  match condition {
    Value::Object(map) => {
      for (key, value) in map {
        let parts: Vec<&str> = key.split('.').collect();
        if parts.len() != 2 {
          return false;
        }
        let (entity_type, attr_name) = (parts[0], parts[1]);

        let entity_value = match entity_type {
          "user" => request.subject.get(attr_name),
          "resource" => request.resource.get(attr_name),
          "environment" => request.environment.get(attr_name),
          _ => return false,
        };

        if let Some(entity_value) = entity_value {
          match value {
            Value::Object(condition_map) => {
              for (op, expected_value) in condition_map {
                match op.as_str() {
                  "eq" => {
                    if entity_value != expected_value {
                      return false;
                    }
                  }
                  "gt" => {
                    if entity_value <= expected_value {
                      return false;
                    }
                  }
                  "gte" => {
                    if entity_value < expected_value {
                      return false;
                    }
                  }
                  "lt" => {
                    if entity_value >= expected_value {
                      return false;
                    }
                  }
                  "lte" => {
                    if entity_value > expected_value {
                      return false;
                    }
                  }
                  "between" => {
                    if let Value::Array(range) = expected_value {
                      if range.len() != 2 {
                        return false;
                      }
                      if entity_value < &range[0] || entity_value > &range[1] {
                        return false;
                      }
                    } else {
                      return false;
                    }
                  }
                  _ => return false,
                }
              }
            }
            _ => {
              if entity_value != value {
                return false;
              }
            }
          }
        } else {
          return false;
        }
      }
      true
    }
    _ => false,
  }
}

fn evaluate_policy(policy: &Policy, request: &AccessRequest) -> bool {
  if policy.resource != request.resource["type"] || policy.action != request.action {
    return false;
  }
  evaluate_condition(&policy.condition, request)
}

fn main() {
  let policy = Policy {
    name: "高级文档访问策略".to_string(),
    condition: json!({
        "user.clearance_level": {"gte": 4},
        "resource.document_classification": "confidential",
        "environment.time_of_day": {"between": ["09:00", "17:00"]}
    }),
    effect: "allow".to_string(),
    resource: "document".to_string(),
    action: "read".to_string(),
  };

  let request = AccessRequest {
    subject: {
      let mut map = HashMap::new();
      map.insert("clearance_level".to_string(), json!(5));
      map
    },
    resource: {
      let mut map = HashMap::new();
      map.insert("type".to_string(), json!("document"));
      map.insert("document_classification".to_string(), json!("confidential"));
      map
    },
    action: "read".to_string(),
    environment: {
      let mut map = HashMap::new();
      map.insert("time_of_day".to_string(), json!("14:30"));
      map
    },
  };

  let result = evaluate_policy(&policy, &request);
  println!("访问权限: {}", if result { "允许" } else { "拒绝" });
}
