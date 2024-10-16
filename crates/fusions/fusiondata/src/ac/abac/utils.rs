use serde_json::Value;

use super::policy::{AccessRequest, Policy};

pub fn evaluate_condition(condition: &Value, request: &AccessRequest) -> bool {
  match condition {
    Value::Object(map) => {
      for (key, value) in map {
        let parts: Vec<&str> = key.split('.').collect();
        if parts.len() != 2 {
          return false;
        }
        let (entity_kind, attr_name) = (parts[0], parts[1]);

        let entity_value = match entity_kind {
          "user" => request.subject.get(attr_name),
          // "resource" => request.resource.get(attr_name),
          "environment" => request.environment.get(attr_name),
          _ => return false,
        };

        if let Some(entity_value) = entity_value {
          match value {
            Value::Object(condition_map) => {
              for (_op, _expected_value) in condition_map {
                /*match op.as_str() {
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
                }*/
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

pub fn evaluate_policy(_policy: &Policy, _request: &AccessRequest) -> bool {
  // if policy.resource != request.resource["type"] || policy.action != request.action {
  //   return false;
  // }
  // evaluate_condition(&policy.condition, request)
  false
}
