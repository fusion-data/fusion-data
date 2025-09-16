use fusion_common::time::OffsetDateTime;
use modelsql_core::filter::{OpValsInt32, OpValsInt64, OpValsString, Page};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(sqlx::FromRow, modelsql::field::Fields), sea_query::enum_def)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct Policy {
  pub id: i64,
  pub description: Option<String>,
  pub policy: serde_json::Value,
  pub status: i32,
  pub cid: i64,
  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub ctime: OffsetDateTime,
  pub mid: Option<i64>,
  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub mtime: Option<OffsetDateTime>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::field::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct PolicyForCreate {
  pub id: i64,
  pub description: Option<String>,
  pub policy: serde_json::Value,
  pub status: Option<i32>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::field::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct PolicyForUpdate {
  pub description: Option<String>,
  pub policy: Option<serde_json::Value>,
  pub status: Option<i32>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct PolicyForPage {
  pub page: Page,
  pub filter: Vec<PolicyFilter>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::filter::FilterNodes))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct PolicyFilter {
  pub id: Option<OpValsInt64>,
  pub description: Option<OpValsString>,
  pub status: Option<OpValsInt32>,
}
