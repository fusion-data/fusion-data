use fusionsql::{
  FilterNodes,
  field::Fields,
  filter::{OpValBool, OpValDateTime, OpValInt32, OpValInt64, OpValString},
  page::Page,
};
use hetumind_core::workflow::CredentialKind;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Fields)]
pub struct CredentialForUpdate {
  pub namespace_id: Option<String>,
  pub name: Option<String>,
  pub data: Option<String>,
  pub kind: Option<CredentialKind>,
  pub is_managed: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Fields)]
pub struct CredentialForInsert {
  pub namespace_id: String,
  pub name: String,
  pub data: String,
  pub kind: CredentialKind,
  pub is_managed: Option<bool>,
  pub id: Option<Uuid>,
}

#[derive(Debug, Clone, Default, Deserialize, FilterNodes)]
pub struct CredentialFilter {
  pub namespace_id: Option<OpValString>,
  pub name: Option<OpValString>,
  pub data: Option<OpValString>,
  pub kind: Option<OpValInt32>,
  pub is_managed: Option<OpValBool>,
  pub created_at: Option<OpValDateTime>,
  pub created_by: Option<OpValInt64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CredentialForQuery {
  #[serde(default)]
  pub page: Page,
  #[serde(default)]
  pub filters: Vec<CredentialFilter>,
}
