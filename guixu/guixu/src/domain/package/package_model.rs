use modelsql::field::Fields;
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ultimate_common::time::UtcDateTime;

/// 已安装包表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "installed_packages")]
pub struct InstalledPackages {
  pub package_name: String,
  pub installed_version: String,
  pub author_name: Option<String>,
  pub author_email: Option<String>,
  pub ctime: UtcDateTime,
  pub mtime: UtcDateTime,
}

/// 已安装节点表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "installed_nodes")]
pub struct InstalledNodes {
  pub name: String,
  pub kind: String,
  pub latest_version: i32,
  pub package: String,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_package_models() {
    assert_eq!(InstalledPackagesIden::Table.as_ref(), "installed_packages");
    assert_eq!(InstalledNodesIden::Table.as_ref(), "installed_nodes");
  }
}
