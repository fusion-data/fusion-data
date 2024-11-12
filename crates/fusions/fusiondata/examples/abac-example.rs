use serde_json::json;
use std::collections::HashMap;

use fusiondata::ac::abac::{
  evaluate_policy,
  policy::{AccessRequest, Policy},
};
use fusiondata::ac::Effect;

fn main() {
  let policy = Policy {
    description: Some("高级文档访问策略".to_string()),
    resource: vec!["document".to_string()],
    action: vec!["read".to_string()],
    condition: Some(json!({
      "user.clearance_level": {"gte": 4},
      "resource.document_classification": "confidential",
      "environment.time_of_day": {"between": ["09:00", "17:00"]}
    })),
    effect: Effect::Allow,
  };

  let request = AccessRequest {
    subject: {
      let mut map = HashMap::new();
      map.insert("clearance_level".to_string(), json!(5));
      map
    },
    resource: vec!["type:document".to_string(), "document_classification:confidential".to_string()],
    action: vec!["read".to_string()],
    environment: {
      let mut map = HashMap::new();
      map.insert("time_of_day".to_string(), json!("14:30"));
      map
    },
  };

  let result = evaluate_policy(&policy, &request);
  println!("访问权限: {}", if result { "允许" } else { "拒绝" });
}
