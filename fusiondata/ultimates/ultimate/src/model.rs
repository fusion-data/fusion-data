use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct IdI64Result {
  pub id: i64,
}
impl IdI64Result {
  pub fn new(id: i64) -> Self {
    Self { id }
  }
}

#[cfg(feature = "uuid")]
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct IdUuidResult {
  pub id: uuid::Uuid,
}
#[cfg(feature = "uuid")]
impl IdUuidResult {
  pub fn new(id: uuid::Uuid) -> Self {
    Self { id }
  }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct IdUlidResult {
  pub id: ulid::Ulid,
}
impl IdUlidResult {
  pub fn new(id: ulid::Ulid) -> Self {
    Self { id }
  }
}
