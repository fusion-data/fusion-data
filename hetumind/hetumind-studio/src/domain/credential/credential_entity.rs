use fusion_common::time::OffsetDateTime;
use fusionsql::field::Fields;
use hetumind_core::{credential::CredentialId, workflow::CredentialKind};
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 凭证实体表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "credential_entity")]
pub struct CredentialEntity {
  pub id: CredentialId,
  pub namespace_id: String,
  pub name: String,
  /// encrypted credential data with jwe
  pub data: String,
  pub kind: CredentialKind,
  pub is_managed: bool,
  pub created_at: OffsetDateTime,
  pub updated_at: Option<OffsetDateTime>,
  pub created_by: i64,
  pub updated_by: Option<i64>,
  pub logical_deletion: Option<OffsetDateTime>,
}
