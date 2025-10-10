use fusionsql::{
  FilterNodes,
  field::Fields,
  filter::{OpValBool, OpValDateTime, OpValInt64, OpValString},
  page::Page,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone, Default, Deserialize, Fields)]
pub struct CredentialForUpdate {
  pub namespace_id: Option<String>,
  pub name: Option<String>,
  pub data: Option<String>,
  pub kind: Option<String>,
  pub is_managed: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize, Fields)]
pub struct CredentialForInsert {
  pub namespace_id: String,
  pub name: String,
  pub data: String,
  pub kind: String,
  pub is_managed: Option<bool>,
  pub id: Option<Uuid>,
}

#[derive(Debug, Clone, Default, Deserialize, FilterNodes)]
pub struct CredentialFilter {
  pub namespace_id: Option<OpValString>,
  pub name: Option<OpValString>,
  pub data: Option<OpValString>,
  pub kind: Option<OpValString>,
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

#[cfg(test)]
mod tests {
  use fusionsql::field::HasSeaFields;
  use sea_query::ColumnRef;

  use super::*;

  #[test]
  fn test_credential_model() {
    let credential = CredentialForUpdate { name: Some("test".to_string()), ..Default::default() };
    println!("{:?}", credential);
    let sea_fields_with_mask = credential
      .sea_fields_with_mask()
      .into_iter()
      .map(|f| match f.column_ref {
        ColumnRef::Column(sea_rc) => sea_rc.to_string(),
        _ => todo!(),
      })
      .collect::<Vec<_>>();
    assert_eq!(sea_fields_with_mask, vec!["name"]);
  }
}
