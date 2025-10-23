use chrono::{DateTime, FixedOffset};
use fusion_common::page::Page;
use fusionsql_core::filter::{OpValInt32, OpValInt64};
use serde::{Deserialize, Serialize};

/// Policy attachment entity representing the relationship between policies and principals (users/roles)
#[derive(Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(sqlx::FromRow, fusionsql::Fields), sea_query::enum_def)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct PolicyAttachmentEntity {
  pub id: i64,
  pub tenant_id: i64,
  pub principal_type: i32, // 1: user, 2: role
  pub principal_id: i64,
  pub policy_id: i64,
  pub attachment_type: i32, // 1: direct, 2: inherited
  pub created_at: DateTime<FixedOffset>,
  pub created_by: i64,
  pub updated_at: Option<DateTime<FixedOffset>>,
  pub updated_by: Option<i64>,
  pub logical_deletion: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct PolicyAttachmentForCreate {
  pub tenant_id: i64,
  pub principal_type: i32,
  pub principal_id: i64,
  pub policy_id: i64,
  pub attachment_type: Option<i32>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct PolicyAttachmentForUpdate {
  pub attachment_type: Option<i32>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct PolicyAttachmentForPage {
  pub page: Page,
  pub filter: Vec<PolicyAttachmentFilter>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::filter::FilterNodes))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct PolicyAttachmentFilter {
  pub id: Option<OpValInt64>,
  pub tenant_id: Option<OpValInt64>,
  pub principal_type: Option<OpValInt32>,
  pub principal_id: Option<OpValInt64>,
  pub policy_id: Option<OpValInt64>,
  pub attachment_type: Option<OpValInt32>,
}
