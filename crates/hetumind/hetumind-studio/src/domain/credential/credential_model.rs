use modelsql::field::{FieldMask, Fields};
use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize, Fields)]
pub struct CredentialForUpdate {
  pub name: Option<String>,
  pub data: Option<String>,
  pub kind: Option<String>,
  pub is_managed: Option<bool>,
  pub update_mask: Option<FieldMask>,
}

#[cfg(test)]
mod tests {
  use modelsql::field::HasSeaFields;
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
