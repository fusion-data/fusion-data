use std::sync::OnceLock;

use fusionsql::{
  ModelManager, SqlError,
  base::{BmcConfig, DbBmc},
  filter::{OpValInt32, OpValInt64, OpValString},
  generate_pg_bmc_common, generate_pg_bmc_filter,
};

use jieyuan_core::model::{NamespaceEntity, NamespaceFilter, NamespaceForCreate, NamespaceForUpdate, TABLE_NAMESPACE};

pub struct NamespaceBmc;
impl DbBmc for NamespaceBmc {
  fn _static_config() -> &'static BmcConfig {
    static CONFIG: OnceLock<BmcConfig> = OnceLock::new();
    CONFIG.get_or_init(|| BmcConfig::new_table(TABLE_NAMESPACE))
  }
}

// Generate common BMC functions (create, update, get, list, delete, etc.)
generate_pg_bmc_common!(
  Bmc: NamespaceBmc,
  Entity: NamespaceEntity,
  ForCreate: NamespaceForCreate,
  ForUpdate: NamespaceForUpdate,
);

// Generate filter functions for query operations
generate_pg_bmc_filter!(
  Bmc: NamespaceBmc,
  Entity: NamespaceEntity,
  Filter: NamespaceFilter,
);

// Extended BMC methods for namespace-specific operations
impl NamespaceBmc {
  /// Check if namespace name exists in the specified tenant (only active namespaces)
  pub async fn exists_by_name(mm: &ModelManager, name: &str, tenant_id: i64) -> Result<bool, SqlError> {
    let filter = NamespaceFilter {
      tenant_id: Some(OpValInt64::eq(tenant_id)),
      name: Some(OpValString::eq(name)),
      status: Some(OpValInt32::eq(jieyuan_core::model::NamespaceStatus::Active as i32)),
      ..Default::default()
    };
    let count = Self::count(mm, vec![filter]).await?;
    Ok(count > 0)
  }

  /// Get namespace by name and tenant_id (only active namespaces)
  pub async fn get_by_name(mm: &ModelManager, name: &str, tenant_id: i64) -> Result<Option<NamespaceEntity>, SqlError> {
    let filter = NamespaceFilter {
      tenant_id: Some(OpValInt64::eq(tenant_id)),
      name: Some(OpValString::eq(name)),
      status: Some(OpValInt32::eq(jieyuan_core::model::NamespaceStatus::Active as i32)),
      ..Default::default()
    };
    Self::find_unique(mm, vec![filter]).await
  }

  /// Get namespace count by tenant
  pub async fn count_by_tenant(mm: &ModelManager, tenant_id: i64) -> Result<u64, SqlError> {
    let filter = NamespaceFilter { tenant_id: Some(OpValInt64::eq(tenant_id)), ..Default::default() };
    let count = Self::count(mm, vec![filter]).await?;
    Ok(count)
  }

  /// List namespaces by tenant (without pagination)
  pub async fn list_by_tenant(mm: &ModelManager, tenant_id: i64) -> Result<Vec<NamespaceEntity>, SqlError> {
    let filter = NamespaceFilter { tenant_id: Some(OpValInt64::eq(tenant_id)), ..Default::default() };
    let entities = Self::find_many(mm, vec![filter], None).await?;
    Ok(entities)
  }

  /// Get namespace with tenant validation
  pub async fn get_with_tenant_validation(
    mm: &ModelManager,
    id: i64,
    tenant_id: i64,
  ) -> Result<Option<NamespaceEntity>, SqlError> {
    let filter = vec![NamespaceFilter {
      id: Some(OpValInt64::eq(id)),
      tenant_id: Some(OpValInt64::eq(tenant_id)),
      status: Some(OpValInt32::eq(jieyuan_core::model::NamespaceStatus::Active as i32)),
      ..Default::default()
    }];
    Self::find_unique(mm, filter).await
  }
}
