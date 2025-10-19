use fusion_common::ctx::Ctx;

use crate::model::{CtxExt, DecisionEffect, PolicyCondition, PolicyDocument, PolicyEntity, TenantAccessCondition};

/// 授权决策结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
  Allow,
  Deny,
}

/// 策略评估引擎
pub struct PolicyEngine;

impl PolicyEngine {
  /// 函数级注释：匹配任意策略是否命中指定 effect
  pub fn match_any(policies: &[PolicyEntity], ctx: &Ctx, action: &str, resource: &str, effect: DecisionEffect) -> bool {
    policies.iter().any(|p| Self::match_policy(p, ctx, action, resource, effect))
  }

  /// 函数级注释：匹配单个策略文档（Action/Resource 通配与 Condition 求值）
  pub fn match_policy(
    policy_entity: &PolicyEntity,
    ctx: &Ctx,
    action: &str,
    resource: &str,
    target_effect: DecisionEffect,
  ) -> bool {
    // 解析策略文档
    let policy: Result<PolicyDocument, _> = serde_json::from_value(policy_entity.policy.clone());
    let Ok(policy) = policy else {
      return false;
    };

    // 遍历策略中的所有声明
    for statement in &policy.statement {
      if statement.effect != target_effect {
        continue;
      }

      // 检查动作匹配
      if !Self::match_patterns(&statement.action, action) {
        continue;
      }

      // 检查资源匹配
      if !Self::match_patterns(&statement.resource, resource) {
        continue;
      }

      // 检查条件匹配
      if let Some(condition) = &statement.condition
        && !Self::evaluate_condition(condition, ctx)
      {
        continue;
      }

      // 所有条件都匹配
      return true;
    }

    false
  }

  /// 函数级注释：匹配模式（支持通配符 *）
  pub fn match_patterns(patterns: &[String], target: &str) -> bool {
    patterns.iter().any(|pattern| Self::match_pattern(pattern, target))
  }

  /// 函数级注释：匹配单个模式（支持高级通配符匹配）
  fn match_pattern(pattern: &str, target: &str) -> bool {
    // 完全匹配
    if pattern == target {
      return true;
    }

    // 全通配符
    if pattern == "*" {
      return true;
    }

    // 高级通配符匹配
    Self::wildcard_match(pattern, target)
  }

  /// 通配符匹配算法（支持简单的 * 通配符）
  fn wildcard_match(pattern: &str, target: &str) -> bool {
    // 处理简单的单通配符情况
    if let Some(star_pos) = pattern.find('*') {
      let prefix = &pattern[..star_pos];
      let suffix = &pattern[star_pos + 1..];

      // 检查前缀匹配
      if !prefix.is_empty() && !target.starts_with(prefix) {
        return false;
      }

      // 检查后缀匹配
      if !suffix.is_empty() && !target.ends_with(suffix) {
        return false;
      }

      // 如果有前缀和后缀，检查中间部分长度是否足够
      let middle_start = prefix.len();
      let middle_end = target.len().saturating_sub(suffix.len());
      if middle_end >= middle_start {
        return true;
      }

      false
    } else {
      // 没有通配符，精确匹配
      pattern == target
    }
  }

  /// 函数级注释：求值条件表达式（混合架构）
  fn evaluate_condition(condition: &serde_json::Value, ctx: &Ctx) -> bool {
    // 解析为增强条件类型
    let enhanced_condition: PolicyCondition = serde_json::from_value(condition.clone())
      .map_err(|e| {
        log::warn!("Failed to parse condition as enhanced format: {}", e);
        e
      })
      .unwrap_or_else(|_| {
        // 如果解析失败，尝试创建默认增强条件
        PolicyCondition {
          string_equals: Some(condition.clone()),
          numeric_equals: None,
          bool: None,
          tenant_access: None,
        }
      });

    Self::evaluate_enhanced_condition(&enhanced_condition, ctx)
  }

  /// 函数级注释：求值增强条件表达式（支持租户访问控制）
  fn evaluate_enhanced_condition(condition: &PolicyCondition, ctx: &Ctx) -> bool {
    // 求值字符串条件
    if let Some(string_equals) = &condition.string_equals {
      if let serde_json::Value::Object(operand_map) = string_equals {
        for (key, expected_value) in operand_map {
          let actual_value = Self::resolve_condition_key(key, ctx);
          if !Self::evaluate_condition_operator("string_equals", &actual_value, expected_value) {
            return false;
          }
        }
      }
    }

    // 求值数值条件
    if let Some(numeric_equals) = &condition.numeric_equals {
      if let serde_json::Value::Object(operand_map) = numeric_equals {
        for (key, expected_value) in operand_map {
          let actual_value = Self::resolve_condition_key(key, ctx);
          if !Self::evaluate_condition_operator("numeric_equals", &actual_value, expected_value) {
            return false;
          }
        }
      }
    }

    // 求值布尔条件
    if let Some(bool_condition) = &condition.bool {
      if let serde_json::Value::Object(operand_map) = bool_condition {
        for (key, expected_value) in operand_map {
          let actual_value = Self::resolve_condition_key(key, ctx);
          if !Self::evaluate_condition_operator("bool", &actual_value, expected_value) {
            return false;
          }
        }
      }
    }

    // 求值租户访问条件
    if let Some(tenant_access) = &condition.tenant_access {
      return Self::evaluate_tenant_access_condition(tenant_access, ctx);
    }

    true
  }

  /// 函数级注释：求值租户访问条件
  fn evaluate_tenant_access_condition(condition: &TenantAccessCondition, ctx: &Ctx) -> bool {
    if !ctx.is_platform_admin() {
      return false; // 非平台管理员不能使用租户访问条件
    }

    match condition.mode {
      crate::model::policy::TenantAccessMode::Current => {
        // 当前租户模式：用户只能访问当前租户
        true // 由业务层验证
      }
      crate::model::policy::TenantAccessMode::All => {
        // 全租户访问模式
        true
      }
      crate::model::policy::TenantAccessMode::Specific => {
        // 特定租户列表模式
        if let Some(allowed_tenant_ids) = &condition.tenant_ids {
          let managed_ids = ctx.managed_tenant_ids();
          allowed_tenant_ids.iter().any(|id| managed_ids.contains(id))
        } else {
          false
        }
      }
    }
  }

  /// 函数级注释：解析条件键到实际值（支持混合架构）
  fn resolve_condition_key(key: &str, ctx: &Ctx) -> serde_json::Value {
    match key {
      "iam:tenant_id" => serde_json::Value::Number(ctx.tenant_id().into()),
      "iam:principal_user_id" => serde_json::Value::Number(ctx.user_id().into()),
      "iam:principal_roles" => {
        serde_json::Value::Array(ctx.roles().iter().map(|r| serde_json::Value::String(r.to_string())).collect())
      }
      "iam:is_platform_admin" => serde_json::Value::Bool(ctx.is_platform_admin()),
      "iam:tenant_access_mode" => match ctx.tenant_access_mode() {
        crate::model::policy::TenantAccessMode::Current => serde_json::Value::String("current".to_string()),
        crate::model::policy::TenantAccessMode::All => serde_json::Value::String("all".to_string()),
        crate::model::policy::TenantAccessMode::Specific => serde_json::Value::String("specific".to_string()),
      },
      "iam:managed_tenant_ids" => serde_json::Value::Array(
        ctx.managed_tenant_ids().iter().map(|id| serde_json::Value::String(id.clone())).collect(),
      ),
      "iam:request_ip" => serde_json::Value::String(ctx.client_ip().to_string()),
      "iam:method" => serde_json::Value::String(ctx.request_method().to_string()),
      "iam:path" => serde_json::Value::String(ctx.request_path().to_string()),
      "iam:token_seq" => serde_json::Value::Number(ctx.token_seq().into()),
      // 时间相关的条件键
      "iam:current_time" => serde_json::Value::String(ctx.req_datetime().to_rfc3339()),
      _ => serde_json::Value::Null,
    }
  }

  /// 函数级注释：求值条件操作符
  fn evaluate_condition_operator(operator: &str, actual: &serde_json::Value, expected: &serde_json::Value) -> bool {
    match operator {
      "string_equals" => Self::string_equals(actual, expected),
      "string_not_equals" => Self::string_not_equals(actual, expected),
      "string_like" => Self::string_like(actual, expected),
      "string_not_like" => Self::string_not_like(actual, expected),
      "numeric_equals" => Self::numeric_equals(actual, expected),
      "numeric_not_equals" => Self::numeric_not_equals(actual, expected),
      "numeric_greater_than" => Self::numeric_greater_than(actual, expected),
      "numeric_less_than" => Self::numeric_less_than(actual, expected),
      "numeric_greater_than_equal" => Self::numeric_greater_than_equal(actual, expected),
      "numeric_less_than_equal" => Self::numeric_less_than_equal(actual, expected),
      "bool" => Self::bool_equals(actual, expected),
      "ip_in_network" => Self::ip_in_network(actual, expected),
      "date_less_than" => Self::date_less_than(actual, expected),
      "date_greater_than" => Self::date_greater_than(actual, expected),
      "date_less_than_equal" => Self::date_less_than_equal(actual, expected),
      "date_greater_than_equal" => Self::date_greater_than_equal(actual, expected),
      _ => false,
    }
  }

  fn string_equals(actual: &serde_json::Value, expected: &serde_json::Value) -> bool {
    match (actual, expected) {
      (serde_json::Value::String(a), serde_json::Value::String(e)) => a == e,
      (serde_json::Value::Number(a), serde_json::Value::String(e)) => a.to_string() == *e,
      (serde_json::Value::Array(arr), serde_json::Value::String(e)) => arr.iter().any(|v| match v {
        serde_json::Value::String(a) => a == e,
        serde_json::Value::Number(a) => a.to_string() == *e,
        _ => false,
      }),
      _ => false,
    }
  }

  fn string_like(actual: &serde_json::Value, expected: &serde_json::Value) -> bool {
    match (actual, expected) {
      (serde_json::Value::String(a), serde_json::Value::String(e)) => {
        // 简单的通配符匹配
        Self::match_pattern(e, a)
      }
      _ => false,
    }
  }

  fn numeric_equals(actual: &serde_json::Value, expected: &serde_json::Value) -> bool {
    match (actual, expected) {
      (serde_json::Value::Number(a), serde_json::Value::Number(e)) => a == e,
      _ => false,
    }
  }

  fn bool_equals(actual: &serde_json::Value, expected: &serde_json::Value) -> bool {
    match (actual, expected) {
      (serde_json::Value::Bool(a), serde_json::Value::Bool(e)) => a == e,
      _ => false,
    }
  }

  fn date_less_than(actual: &serde_json::Value, expected: &serde_json::Value) -> bool {
    match (actual, expected) {
      (serde_json::Value::String(a), serde_json::Value::String(e)) => {
        // 简单的字符串比较（实际应该解析为DateTime）
        a < e
      }
      _ => false,
    }
  }

  fn date_greater_than(actual: &serde_json::Value, expected: &serde_json::Value) -> bool {
    match (actual, expected) {
      (serde_json::Value::String(a), serde_json::Value::String(e)) => {
        // 简单的字符串比较（实际应该解析为DateTime）
        a > e
      }
      _ => false,
    }
  }

  fn date_less_than_equal(actual: &serde_json::Value, expected: &serde_json::Value) -> bool {
    match (actual, expected) {
      (serde_json::Value::String(a), serde_json::Value::String(e)) => a <= e,
      _ => false,
    }
  }

  fn date_greater_than_equal(actual: &serde_json::Value, expected: &serde_json::Value) -> bool {
    match (actual, expected) {
      (serde_json::Value::String(a), serde_json::Value::String(e)) => a >= e,
      _ => false,
    }
  }

  fn string_not_equals(actual: &serde_json::Value, expected: &serde_json::Value) -> bool {
    !Self::string_equals(actual, expected)
  }

  fn string_not_like(actual: &serde_json::Value, expected: &serde_json::Value) -> bool {
    !Self::string_like(actual, expected)
  }

  fn numeric_not_equals(actual: &serde_json::Value, expected: &serde_json::Value) -> bool {
    !Self::numeric_equals(actual, expected)
  }

  fn numeric_greater_than(actual: &serde_json::Value, expected: &serde_json::Value) -> bool {
    match (actual, expected) {
      (serde_json::Value::Number(a), serde_json::Value::Number(e)) => {
        a.as_i64().map(|a_val| e.as_i64().map(|e_val| a_val > e_val).unwrap_or(false)).unwrap_or(false)
      }
      _ => false,
    }
  }

  fn numeric_less_than(actual: &serde_json::Value, expected: &serde_json::Value) -> bool {
    match (actual, expected) {
      (serde_json::Value::Number(a), serde_json::Value::Number(e)) => {
        a.as_i64().map(|a_val| e.as_i64().map(|e_val| a_val < e_val).unwrap_or(false)).unwrap_or(false)
      }
      _ => false,
    }
  }

  fn numeric_greater_than_equal(actual: &serde_json::Value, expected: &serde_json::Value) -> bool {
    match (actual, expected) {
      (serde_json::Value::Number(a), serde_json::Value::Number(e)) => {
        a.as_i64().map(|a_val| e.as_i64().map(|e_val| a_val >= e_val).unwrap_or(false)).unwrap_or(false)
      }
      _ => false,
    }
  }

  fn numeric_less_than_equal(actual: &serde_json::Value, expected: &serde_json::Value) -> bool {
    match (actual, expected) {
      (serde_json::Value::Number(a), serde_json::Value::Number(e)) => {
        a.as_i64().map(|a_val| e.as_i64().map(|e_val| a_val <= e_val).unwrap_or(false)).unwrap_or(false)
      }
      _ => false,
    }
  }

  fn ip_in_network(actual: &serde_json::Value, expected: &serde_json::Value) -> bool {
    match (actual, expected) {
      (serde_json::Value::String(ip), serde_json::Value::String(network)) => {
        // 简单的 IP 网络匹配实现（支持基本的 CIDR 表示法）
        // 这里只实现基本的 IPv4 匹配，不依赖外部库
        Self::ip_in_network_simple(ip, network)
      }
      _ => false,
    }
  }

  // 简单的 IP 网络匹配实现
  fn ip_in_network_simple(ip: &str, network: &str) -> bool {
    // 简单实现：只支持精确匹配和 /24, /16, /8 网络匹配
    if ip == network {
      return true;
    }

    if let Some(slash_pos) = network.find('/') {
      let network_ip = &network[..slash_pos];
      let prefix_len: u32 = network[slash_pos + 1..].parse().unwrap_or(32);

      if let (Ok(ip_addr), Ok(network_addr)) = (Self::parse_ipv4(ip), Self::parse_ipv4(network_ip)) {
        if prefix_len == 0 {
          return true; // 0.0.0.0/0 匹配所有
        }
        if prefix_len > 32 {
          return false;
        }

        let mask = if prefix_len == 32 { 0xFFFFFFFF } else { !((1 << (32 - prefix_len)) - 1) };
        (ip_addr & mask) == (network_addr & mask)
      } else {
        false
      }
    } else {
      false
    }
  }

  // 简单的 IPv4 地址解析
  fn parse_ipv4(ip: &str) -> Result<u32, ()> {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
      return Err(());
    }

    let mut result = 0u32;
    for (i, part) in parts.iter().enumerate() {
      let octet: u8 = part.parse().map_err(|_| ())?;
      result |= (octet as u32) << (24 - i * 8);
    }

    Ok(result)
  }
}
