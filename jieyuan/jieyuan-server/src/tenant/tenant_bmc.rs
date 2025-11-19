use std::sync::OnceLock;

use fusionsql::{
  ModelManager, SqlError,
  base::{BmcConfig, DbBmc},
  filter::{OpValInt32, OpValInt64, OpValString},
  generate_pg_bmc_common, generate_pg_bmc_filter,
};

use jieyuan_core::model::{TABLE_TENANT, Tenant, TenantFilter, TenantForCreate, TenantForUpdate, TenantStatus};

pub struct TenantBmc;

impl DbBmc for TenantBmc {
  fn _bmc_config() -> &'static BmcConfig {
    static CONFIG: OnceLock<BmcConfig> = OnceLock::new();
    CONFIG.get_or_init(|| {
      BmcConfig::new_table(TABLE_TENANT).with_use_logical_deletion(false) // 不使用逻辑删除，使用状态管理
    })
  }
}

// Generate common BMC functions (create, update, get, list, delete, etc.)
generate_pg_bmc_common!(
  Bmc: TenantBmc,
  Entity: Tenant,
  ForCreate: TenantForCreate,
  ForUpdate: TenantForUpdate,
);

// Generate filter functions for query operations
generate_pg_bmc_filter!(
  Bmc: TenantBmc,
  Entity: Tenant,
  Filter: TenantFilter,
);

// Extended BMC methods for tenant-specific operations
impl TenantBmc {
  /// Check if tenant name exists (only active tenants)
  pub async fn exists_by_name(mm: &ModelManager, name: &str) -> Result<bool, SqlError> {
    let filter = TenantFilter {
      name: Some(OpValString::eq(name)),
      status: Some(OpValInt32::eq(TenantStatus::Active as i32)),
      ..Default::default()
    };
    let count = Self::count(mm, vec![filter]).await?;
    Ok(count > 0)
  }

  /// Get tenant by name (only active tenants)
  pub async fn get_by_name(mm: &ModelManager, name: &str) -> Result<Option<Tenant>, SqlError> {
    let filter = TenantFilter {
      name: Some(OpValString::eq(name)),
      status: Some(OpValInt32::eq(TenantStatus::Active as i32)),
      ..Default::default()
    };
    Self::find_unique(mm, vec![filter]).await
  }

  /// Check if tenant name exists excluding a specific tenant (for update operations)
  pub async fn name_exists_excluding_id(mm: &ModelManager, name: &str, exclude_id: i64) -> Result<bool, SqlError> {
    let filter = vec![TenantFilter {
      name: Some(OpValString::eq(name.to_string())),
      id: Some(OpValInt64::not(exclude_id)),
      status: Some(OpValInt32::eq(TenantStatus::Active as i32)),
      ..Default::default()
    }];

    let count = Self::count(mm, filter).await?;
    Ok(count > 0)
  }

  /// Get active tenant count
  pub async fn count_active(mm: &ModelManager) -> Result<u64, SqlError> {
    let filter = TenantFilter { status: Some(OpValInt32::eq(TenantStatus::Active as i32)), ..Default::default() };
    let count = Self::count(mm, vec![filter]).await?;
    Ok(count)
  }

  /// List all active tenants (without pagination)
  pub async fn list_active(mm: &ModelManager) -> Result<Vec<Tenant>, SqlError> {
    let filter = TenantFilter { status: Some(OpValInt32::eq(TenantStatus::Active as i32)), ..Default::default() };
    let entities = Self::find_many(mm, vec![filter], None).await?;
    Ok(entities)
  }

  /// Get active tenant by ID
  pub async fn get_active(mm: &ModelManager, id: i64) -> Result<Option<Tenant>, SqlError> {
    let filter = vec![TenantFilter {
      id: Some(OpValInt64::eq(id)),
      status: Some(OpValInt32::eq(TenantStatus::Active as i32)),
      ..Default::default()
    }];
    Self::find_unique(mm, filter).await
  }

  /// List all tenants including inactive ones
  pub async fn list_all(mm: &ModelManager) -> Result<Vec<Tenant>, SqlError> {
    let entities = Self::find_many(mm, vec![], None).await?;
    Ok(entities)
  }
}
