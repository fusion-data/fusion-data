use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};

// RBAC 结构
struct User {
  id: i64,
  roles: HashSet<i64>,
}

struct Role {
  id: i64,
  permissions: HashSet<String>,
}

// ABAC 结构
struct Policy {
  id: i64,
  name: String,
  condition: Value,
  effect: String,
  resource: String,
  action: String,
}

struct AccessRequest {
  user_id: i64,
  resource: String,
  action: String,
  attributes: HashMap<String, Value>,
}

// 模拟数据库
struct Database {
  users: HashMap<i64, User>,
  roles: HashMap<i64, Role>,
  policies: Vec<Policy>,
}

impl Database {
  fn new() -> Self {
    let mut users = HashMap::default();
    let mut roles = HashMap::default();

    // 创建用户和角色
    users.insert(1, User { id: 1, roles: vec![1, 2].into_iter().collect() });
    roles.insert(1, Role { id: 1, permissions: vec!["document:read".to_string()].into_iter().collect() });
    roles.insert(2, Role { id: 2, permissions: vec!["document:write".to_string()].into_iter().collect() });

    // 创建ABAC策略
    let policies = vec![Policy {
      id: 1,
      name: "高级文档访问策略".to_string(),
      condition: json!({
          "user.clearance_level": {"gte": 4},
          "resource.classification": "confidential",
          "environment.time": {"between": ["09:00", "17:00"]}
      }),
      effect: "allow".to_string(),
      resource: "document".to_string(),
      action: "read".to_string(),
    }];

    Database { users, roles, policies }
  }
}

fn check_rbac(db: &Database, user_id: i64, resource: &str, action: &str) -> bool {
  if let Some(user) = db.users.get(&user_id) {
    for role_id in &user.roles {
      if let Some(role) = db.roles.get(role_id) {
        if role.permissions.contains(&format!("{}:{}", resource, action)) {
          return true;
        }
      }
    }
  }
  false
}

fn evaluate_abac_condition(condition: &Value, attributes: &HashMap<String, Value>) -> bool {
  // 简化的ABAC条件评估逻辑
  match condition {
    Value::Object(map) => {
      for (key, value) in map {
        if let Some(attr_value) = attributes.get(key) {
          if attr_value != value {
            return false;
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

fn check_abac(db: &Database, request: &AccessRequest) -> bool {
  for policy in &db.policies {
    if policy.resource == request.resource && policy.action == request.action {
      if evaluate_abac_condition(&policy.condition, &request.attributes) {
        return policy.effect == "allow";
      }
    }
  }
  false
}

fn check_access(db: &Database, request: &AccessRequest) -> bool {
  // 首先检查RBAC
  if !check_rbac(db, request.user_id, &request.resource, &request.action) {
    return false;
  }

  // 然后检查ABAC
  check_abac(db, request)
}

fn main() {
  let db = Database::new();
  let request = AccessRequest {
    user_id: 1,
    resource: "document".to_string(),
    action: "read".to_string(),
    attributes: {
      let mut map = HashMap::default();
      map.insert("user.clearance_level".to_string(), json!(5));
      map.insert("resource.classification".to_string(), json!("confidential"));
      map.insert("environment.time".to_string(), json!("14:00"));
      map
    },
  };

  let access_granted = check_access(&db, &request);
  println!("访问权限: {}", if access_granted { "允许" } else { "拒绝" });
}
