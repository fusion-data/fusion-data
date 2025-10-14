use std::ops::Deref;

use derive_more::Display;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::generate_uuid_newtype;

/// 凭证唯一标识符
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Display)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type), sqlx(transparent))]
#[serde(transparent)]
pub struct CredentialId(pub(crate) Uuid);

impl Deref for CredentialId {
  type Target = Uuid;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

generate_uuid_newtype!(Struct: CredentialId);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialInfo {
  pub id: CredentialId,
  pub name: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
  Bearer,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_credential_id() {
    let id = CredentialId::now_v7();
    println!("id is {}", id);

    let id_ts = id.get_timestamp();
    assert!(id_ts.is_some());
  }
}
