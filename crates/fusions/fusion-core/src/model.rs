use serde::{Deserialize, Serialize, de::DeserializeOwned};

/// 能用包装结果，将可 Serialize 的类型包裹在 `data` 字段中
#[derive(Debug, Serialize, Deserialize)]
pub struct WrapperResult<T> {
  pub data: T,
}

impl<T> WrapperResult<T> {
  pub fn new(data: T) -> Self {
    Self { data }
  }
}

impl<T: Serialize> From<T> for WrapperResult<T> {
  fn from(data: T) -> Self {
    Self::new(data)
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdResult {
  pub id: serde_json::Value,
}

impl IdResult {
  pub fn new<T>(id: T) -> Self
  where
    T: Serialize,
  {
    Self { id: serde_json::to_value(id).unwrap() }
  }

  pub fn to<T>(&self) -> Result<T, serde_json::Error>
  where
    T: DeserializeOwned,
  {
    serde_json::from_value(self.id.clone())
  }

  #[cfg(feature = "with-uuid")]
  pub fn to_uuid(&self) -> Result<uuid::Uuid, serde_json::Error> {
    self.to::<uuid::Uuid>()
  }
}

#[derive(Debug, Serialize, Deserialize)]
// #[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct IdI64Result {
  pub id: i64,
}
impl IdI64Result {
  pub fn new(id: i64) -> Self {
    Self { id }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdStringResult {
  pub id: String,
}
impl IdStringResult {
  pub fn new(id: String) -> Self {
    Self { id }
  }
}

#[cfg(feature = "with-uuid")]
#[derive(Debug, Serialize, Deserialize)]
// #[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct IdUuidResult {
  pub id: uuid::Uuid,
}

#[cfg(feature = "with-uuid")]
impl IdUuidResult {
  pub fn new(id: uuid::Uuid) -> Self {
    Self { id }
  }
}

#[cfg(feature = "with-uuid")]
impl From<uuid::Uuid> for IdUuidResult {
  fn from(id: uuid::Uuid) -> Self {
    Self::new(id)
  }
}

#[cfg(feature = "with-ulid")]
#[derive(Debug, Serialize, Deserialize)]
// #[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct IdUlidResult {
  pub id: ulid::Ulid,
}

#[cfg(feature = "with-ulid")]
impl IdUlidResult {
  pub fn new(id: ulid::Ulid) -> Self {
    Self { id }
  }
}
