use std::collections::HashMap;

use jieyuan_core::model::AuthContext;

/// 扩展的资源模板渲染函数
/// 支持内置占位符和自定义占位符
pub fn render_resource_ext(tpl: &str, ac: &AuthContext, extras: &HashMap<String, String>) -> String {
  let mut s = tpl.to_string();

  // 内置占位符
  s = s
    .replace("{tenant_id}", &ac.principal_tenant_id.to_string())
    .replace("{user_id}", &ac.principal_user_id.to_string())
    .replace("{method}", &ac.method)
    .replace("{path}", &ac.path)
    .replace("{token_seq}", &ac.token_seq.to_string());

  // 角色（拼接为逗号分隔）
  if s.contains("{principal_roles}") {
    let joined = ac.principal_roles.join(",");
    s = s.replace("{principal_roles}", &joined);
  }

  // 其它自定义占位符（如 role_id/policy_id/resource_id 等）
  for (k, v) in extras.iter() {
    let ph = format!("{{{}}}", k);
    if s.contains(&ph) {
      s = s.replace(&ph, v);
    }
  }

  s
}

/// 简单的资源模板渲染函数（仅内置占位符）
pub fn render_resource(tpl: &str, ac: &AuthContext) -> String {
  let extras = HashMap::new();
  render_resource_ext(tpl, ac, &extras)
}
