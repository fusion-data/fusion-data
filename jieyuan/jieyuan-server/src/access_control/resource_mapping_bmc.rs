use std::sync::OnceLock;

use fusions::common::ahash::HashMap;
use fusionsql::page::PageResult;
use fusionsql::{
  ModelManager, SqlError,
  base::{BmcConfig, DbBmc, pg_page},
  generate_pg_bmc_common, generate_pg_bmc_filter,
};
use jieyuan_core::model::{
  IamResourceMappingEntity, IamResourceMappingFilter, IamResourceMappingForCreate, IamResourceMappingForQuery,
  IamResourceMappingForUpdate, TABLE_IAM_RESOURCE_MAPPING,
};

pub struct ResourceMappingBmc;

impl DbBmc for ResourceMappingBmc {
  fn _static_config() -> &'static BmcConfig {
    static CONFIG: OnceLock<BmcConfig> = OnceLock::new();
    CONFIG.get_or_init(|| BmcConfig::new_table(TABLE_IAM_RESOURCE_MAPPING))
  }
}

generate_pg_bmc_common!(
  Bmc: ResourceMappingBmc,
  Entity: IamResourceMappingEntity,
  ForCreate: IamResourceMappingForCreate,
  ForUpdate: IamResourceMappingForUpdate,
);

generate_pg_bmc_filter!(
  Bmc: ResourceMappingBmc,
  Entity: IamResourceMappingEntity,
  Filter: IamResourceMappingFilter,
);

impl ResourceMappingBmc {
  /// 查找资源映射（支持模式匹配）
  pub async fn find_by_path_pattern(
    mm: &ModelManager,
    service: &str,
    method: &str,
    path: &str,
  ) -> Result<Option<IamResourceMappingEntity>, SqlError> {
    let db = mm
      .dbx()
      .db_postgres()
      .map_err(|e| SqlError::InvalidArgument { message: format!("Database connection error: {}", e) })?;

    let sql = r#"
      SELECT * FROM iam_resource_mapping
      WHERE service = $1 AND enabled = true
      ORDER BY LENGTH(path_pattern) DESC
    "#;

    let rows = db
      .fetch_all(sqlx::query_as::<_, IamResourceMappingEntity>(sql).bind(service))
      .await
      .map_err(|e| SqlError::InvalidArgument { message: format!("Query error: {}", e) })?;

    for row in rows {
      if row.method != method && row.method != "*" {
        continue;
      }

      if let Ok(Some(_params)) = Self::match_path_pattern(&row.path_pattern, path) {
        return Ok(Some(row));
      }
    }

    Ok(None)
  }

  /// 路径模式匹配
  pub fn match_path_pattern(pattern: &str, actual: &str) -> Result<Option<HashMap<String, String>>, SqlError> {
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    let actual_parts: Vec<&str> = actual.split('/').collect();

    if pattern_parts.len() != actual_parts.len() {
      return Ok(None);
    }

    let mut params = HashMap::default();

    for (pattern_part, actual_part) in pattern_parts.iter().zip(actual_parts.iter()) {
      if pattern_part.starts_with('{') && pattern_part.ends_with('}') {
        let param_name = &pattern_part[1..pattern_part.len() - 1];
        params.insert(param_name.to_string(), actual_part.to_string());
      } else if pattern_part != actual_part {
        return Ok(None);
      }
    }

    Ok(Some(params))
  }

  /// 列出资源映射（带分页和过滤）
  pub async fn list_with_query(
    mm: &ModelManager,
    query: IamResourceMappingForQuery,
  ) -> Result<PageResult<IamResourceMappingEntity>, SqlError> {
    pg_page::<Self, _, _>(mm, query.filters, query.page).await
  }

  /// 根据映射代码查找资源映射
  pub async fn find_by_code(
    mm: &ModelManager,
    mapping_code: &str,
  ) -> Result<Option<IamResourceMappingEntity>, SqlError> {
    let db = mm
      .dbx()
      .db_postgres()
      .map_err(|e| SqlError::InvalidArgument { message: format!("Database connection error: {}", e) })?;

    let sql = r#"
      SELECT * FROM iam_resource_mapping
      WHERE mapping_code = $1 AND enabled = true
      LIMIT 1
    "#;

    let entity = db
      .fetch_optional(sqlx::query_as::<_, IamResourceMappingEntity>(sql).bind(mapping_code))
      .await
      .map_err(|e| SqlError::InvalidArgument { message: format!("Query error: {}", e) })?;

    Ok(entity)
  }

  /// 根据路径查找资源映射（支持租户隔离）
  pub async fn find_by_path(
    mm: &ModelManager,
    service: &str,
    path: &str,
    method: &str,
  ) -> Result<Option<IamResourceMappingEntity>, SqlError> {
    let db = mm
      .dbx()
      .db_postgres()
      .map_err(|e| SqlError::InvalidArgument { message: format!("Database connection error: {}", e) })?;

    let sql = r#"
      SELECT * FROM iam_resource_mapping
      WHERE service = $1 AND enabled = true AND (
        tenant_id IS NULL OR
        tenant_id = (SELECT tenant_id FROM current_setting('app.current_tenant_id')::bigint)
      )
      ORDER BY
        CASE WHEN tenant_id IS NULL THEN 1 ELSE 2 END,
        LENGTH(path_pattern) DESC
    "#;

    let rows = db
      .fetch_all(sqlx::query_as::<_, IamResourceMappingEntity>(sql).bind(service))
      .await
      .map_err(|e| SqlError::InvalidArgument { message: format!("Query error: {}", e) })?;

    for row in rows {
      if row.method != method && row.method != "*" {
        continue;
      }

      if let Ok(Some(_params)) = Self::match_path_pattern(&row.path_pattern, path) {
        return Ok(Some(row));
      }
    }

    Ok(None)
  }

  /// 根据服务、路径和方法查找资源映射（支持缓存）
  pub async fn find_by_service_path_method(
    mm: &ModelManager,
    service: &str,
    path: &str,
    method: &str,
  ) -> Result<Option<IamResourceMappingEntity>, SqlError> {
    let db = mm
      .dbx()
      .db_postgres()
      .map_err(|e| SqlError::InvalidArgument { message: format!("Database connection error: {}", e) })?;

    let sql = r#"
      SELECT * FROM iam_resource_mapping
      WHERE service = $1 AND method = $2 AND enabled = true AND (
        tenant_id IS NULL OR
        tenant_id = (SELECT tenant_id FROM current_setting('app.current_tenant_id')::bigint)
      )
      ORDER BY
        CASE WHEN tenant_id IS NULL THEN 1 ELSE 2 END,
        LENGTH(path_pattern) DESC
      LIMIT 1
    "#;

    let entity = db
      .fetch_optional(sqlx::query_as::<_, IamResourceMappingEntity>(sql).bind(service).bind(method))
      .await
      .map_err(|e| SqlError::InvalidArgument { message: format!("Query error: {}", e) })?;

    // 如果找到记录，验证路径模式匹配
    if let Some(row) = entity
      && Self::match_path_pattern(&row.path_pattern, path)?.is_some()
    {
      return Ok(Some(row));
    }

    Ok(None)
  }
}
