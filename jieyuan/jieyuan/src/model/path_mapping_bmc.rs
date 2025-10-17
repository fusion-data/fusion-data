use fusionsql::{
  ModelManager, SqlError,
  base::{DbBmc, pg_page},
  generate_pg_bmc_common, generate_pg_bmc_filter,
};
use jieyuan_core::model::{
  PathMappingEntity, PathMappingFilter, PathMappingForCreate, PathMappingForQuery, PathMappingForUpdate,
  TABLE_PATH_MAPPING,
};
use std::collections::HashMap;

pub struct PathMappingBmc;

impl DbBmc for PathMappingBmc {
  const TABLE: &'static str = TABLE_PATH_MAPPING;
}

generate_pg_bmc_common!(
  Bmc: PathMappingBmc,
  Entity: PathMappingEntity,
  ForCreate: PathMappingForCreate,
  ForUpdate: PathMappingForUpdate,
);

generate_pg_bmc_filter!(
  Bmc: PathMappingBmc,
  Entity: PathMappingEntity,
  Filter: PathMappingFilter,
);

impl PathMappingBmc {
  /// 查找路径映射（支持模式匹配）
  pub async fn find_by_path_pattern(
    mm: &ModelManager,
    service: &str,
    method: &str,
    path: &str,
  ) -> Result<Option<PathMappingEntity>, SqlError> {
    let db = mm
      .dbx()
      .db_postgres()
      .map_err(|e| SqlError::InvalidArgument { message: format!("Database connection error: {}", e) })?;

    let sql = r#"
      SELECT * FROM service_path_mappings
      WHERE service = $1 AND enabled = true
      ORDER BY LENGTH(path_pattern) DESC
    "#;

    let rows = db
      .fetch_all(sqlx::query_as::<_, PathMappingEntity>(sql).bind(service))
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

    let mut params = HashMap::new();

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

  /// 列出路径映射（带分页和过滤）
  pub async fn list_with_query(
    mm: &ModelManager,
    query: PathMappingForQuery,
  ) -> Result<fusion_common::page::PageResult<PathMappingEntity>, SqlError> {
    pg_page::<Self, _, _>(mm, query.filters, query.page).await
  }
}
