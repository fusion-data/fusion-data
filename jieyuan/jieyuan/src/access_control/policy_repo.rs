use fusion_core::Result;
use fusionsql::{ModelManager, SqlError};
use jieyuan_core::model::{PolicyEngine, PolicyEntity, TABLE_POLICY, TABLE_POLICY_ATTACHMENT};

/// 策略仓库（基于 BMC 实现）
#[derive(Clone)]
pub struct PolicyRepo {
  mm: ModelManager,
}

impl PolicyRepo {
  pub fn new(mm: ModelManager) -> Self {
    Self { mm }
  }

  /// 查询直接附加给用户的策略
  pub async fn list_attached_policies_for_user(&self, tenant_id: i64, user_id: i64) -> Result<Vec<PolicyEntity>> {
    let db = self.mm.dbx().db_postgres()?;

    let query = format!(
      r#"
      SELECT p.*
      FROM {} p
      INNER JOIN {} pa ON p.id = pa.policy_id
      WHERE pa.tenant_id = $1
        AND pa.principal_type = 1
        AND pa.principal_id = $2
      ORDER BY p.created_at DESC
      "#,
      TABLE_POLICY, TABLE_POLICY_ATTACHMENT
    );

    let policies = db
      .fetch_all(sqlx::query_as::<_, PolicyEntity>(&query).bind(tenant_id).bind(user_id))
      .await
      .map_err(SqlError::from)?;

    Ok(policies)
  }

  /// 通过角色查询策略
  pub async fn list_policies_for_roles(&self, tenant_id: i64, role_codes: &[String]) -> Result<Vec<PolicyEntity>> {
    let db = self.mm.dbx().db_postgres()?;

    if role_codes.is_empty() {
      return Ok(vec![]);
    }

    // 构建角色名称的占位符
    let role_placeholders: Vec<String> = role_codes.iter().enumerate().map(|(i, _)| format!("${}", i + 3)).collect();
    let role_list = role_placeholders.join(",");

    let query = format!(
      r#"
      SELECT DISTINCT p.*
      FROM {} p
      INNER JOIN {} pa ON p.id = pa.policy_id
      INNER JOIN iam_role r ON pa.principal_id = r.id
      WHERE pa.tenant_id = $1
        AND pa.principal_type = 2
        AND r.tenant_id = $2
        AND r.name IN ({})
        AND r.logical_deletion IS NULL
        AND p.logical_deletion IS NULL
      ORDER BY p.created_at DESC
      "#,
      TABLE_POLICY, TABLE_POLICY_ATTACHMENT, role_list
    );

    let mut query_builder = sqlx::query_as::<_, PolicyEntity>(&query).bind(tenant_id).bind(tenant_id);

    // 绑定角色代码参数
    for role_code in role_codes {
      query_builder = query_builder.bind(role_code);
    }

    let policies = db.fetch_all(query_builder).await.map_err(SqlError::from)?;

    Ok(policies)
  }

  /// 查询资源策略
  pub async fn list_resource_policies(&self, tenant_id: i64, resource: &str) -> Result<Vec<PolicyEntity>> {
    let db = self.mm.dbx().db_postgres()?;

    // 查询租户的所有有效策略
    let query = format!(
      r#"
      SELECT p.*
      FROM {} p
      WHERE p.tenant_id = $1
        AND p.logical_deletion IS NULL
      ORDER BY p.created_at DESC
      "#,
      TABLE_POLICY
    );

    let all_policies = db
      .fetch_all(sqlx::query_as::<_, PolicyEntity>(&query).bind(tenant_id))
      .await
      .map_err(SqlError::from)?;

    // 在应用层进行资源匹配
    let mut matched_policies = Vec::new();

    for policy in all_policies {
      if let Ok(policy_doc) = serde_json::from_value::<jieyuan_core::model::PolicyDocument>(policy.policy.clone()) {
        for statement in &policy_doc.statement {
          // 检查资源的通配符匹配
          if PolicyEngine::match_patterns(&statement.resource, resource) {
            matched_policies.push(policy);
            break; // 只要有一个 statement 匹配就足够了
          }
        }
      }
    }

    Ok(matched_policies)
  }

  /// 查询权限边界
  pub async fn find_permission_boundary(&self, tenant_id: i64, user_id: i64) -> Result<Option<PolicyEntity>> {
    let db = self.mm.dbx().db_postgres()?;

    let query = format!(
      r#"
      SELECT p.*
      FROM {} p
      INNER JOIN iam_user u ON u.permission_boundary_policy_id = p.id
      INNER JOIN iam_tenant_user tu ON tu.user_id = u.id
      WHERE tu.tenant_id = $1
        AND u.id = $2
        AND u.permission_boundary_policy_id IS NOT NULL
        AND p.logical_deletion IS NULL
        AND u.logical_deletion IS NULL
      LIMIT 1
      "#,
      TABLE_POLICY
    );

    let policy = db
      .fetch_optional(sqlx::query_as::<_, PolicyEntity>(&query).bind(tenant_id).bind(user_id))
      .await
      .map_err(SqlError::from)?;

    Ok(policy)
  }

  /// 查询会话策略
  pub async fn find_session_policy(&self, token_id: &str) -> Result<Option<PolicyEntity>> {
    let db = self.mm.dbx().db_postgres()?;

    let query = format!(
      r#"
      SELECT p.*
      FROM {} p
      INNER JOIN iam_session_policy sp ON sp.policy_id = p.id
      WHERE sp.token_id = $1
        AND sp.expires_at > NOW()
        AND p.logical_deletion IS NULL
      ORDER BY sp.created_at DESC
      LIMIT 1
      "#,
      TABLE_POLICY
    );

    let policy = db
      .fetch_optional(sqlx::query_as::<_, PolicyEntity>(&query).bind(token_id))
      .await
      .map_err(SqlError::from)?;

    Ok(policy)
  }
}
