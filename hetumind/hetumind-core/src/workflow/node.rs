use std::ops::Deref;

use serde::{Deserialize, Serialize};

/// 节点唯一标识符，工作流配置中唯一。用于标识工作流定义中配置的节点
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, derive_more::From, derive_more::Into)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type), sqlx(transparent))]
#[serde(transparent)]
pub struct NodeName(String);

impl NodeName {
  pub fn as_str(&self) -> &str {
    self.0.as_str()
  }
}

impl AsRef<str> for NodeName {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

impl Deref for NodeName {
  type Target = str;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<&str> for NodeName {
  fn from(id: &str) -> Self {
    Self(id.to_string())
  }
}

impl std::fmt::Display for NodeName {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.0)
  }
}

/// 节点类型，用于唯一标识一个节点，相同类型的不同版本节点使用相同的 NodeKind
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, derive_more::Into)]
#[serde(transparent)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type), sqlx(transparent))]
pub struct NodeKind(String);

impl NodeKind {
  pub fn as_str(&self) -> &str {
    self.0.as_str()
  }
}

impl Deref for NodeKind {
  type Target = str;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl AsRef<str> for NodeKind {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

impl From<String> for NodeKind {
  fn from(id: String) -> Self {
    Self(id)
  }
}

impl From<&str> for NodeKind {
  fn from(id: &str) -> Self {
    Self(id.to_string())
  }
}

impl std::fmt::Display for NodeKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.0)
  }
}

#[cfg(feature = "with-db")]
modelsql::generate_string_newtype_to_sea_query_value!(Struct: NodeName, Struct: NodeKind);
