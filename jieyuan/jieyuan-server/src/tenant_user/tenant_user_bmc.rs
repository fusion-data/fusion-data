use std::sync::OnceLock;

use fusionsql::{
  ModelManager, SqlError,
  base::{self, BmcConfig, DbBmc, compute_page},
  filter::OpValInt64,
  filter::{FilterGroups, apply_to_sea_query},
  generate_pg_bmc_common, generate_pg_bmc_filter,
};
use sea_query::{Condition, SelectStatement};

use jieyuan_core::model::{
  TABLE_TENANT_USER, TABLE_USER, TenantUser, TenantUserChangeQueryReq, TenantUserChangeQueryResp, TenantUserFilter,
  TenantUserForCreate, TenantUserForUpdate, TenantUserStatus, UserForQuery, UserWithTenant,
};

pub struct TenantUserBmc;
impl DbBmc for TenantUserBmc {
  fn _static_config() -> &'static BmcConfig {
    static CONFIG: OnceLock<BmcConfig> = OnceLock::new();
    CONFIG.get_or_init(|| BmcConfig::new_table(TABLE_TENANT_USER).with_has_updated_by(false).with_has_created_by(false))
  }
}

generate_pg_bmc_common!(
  Bmc: TenantUserBmc,
  Entity: TenantUser,
  ForCreate: TenantUserForCreate,
  ForUpdate: TenantUserForUpdate,
);

generate_pg_bmc_filter!(
  Bmc: TenantUserBmc,
  Entity: TenantUser,
  Filter: TenantUserFilter,
);

impl TenantUserBmc {
  /// Link user to tenant with upsert logic
  pub async fn link_user_to_tenant(
    mm: &ModelManager,
    user_id: i64,
    tenant_id: i64,
    status: TenantUserStatus,
  ) -> Result<(), SqlError> {
    let mm = mm.get_txn_clone();
    mm.dbx().begin_txn().await?;

    // First try to find existing association
    let filters = vec![
      TenantUserFilter { tenant_id: Some(OpValInt64::eq(tenant_id)), ..Default::default() },
      TenantUserFilter { user_id: Some(OpValInt64::eq(user_id)), ..Default::default() },
    ];

    // Check if association exists
    if let Ok(Some(_)) = base::pg_find_first::<Self, TenantUser, _>(&mm, filters.clone()).await {
      // Update existing association
      let update_data = TenantUserForUpdate { status: Some(status) };
      base::update::<Self, _, _>(&mm, filters, update_data).await.map(|_| ())?;
    } else {
      // Create new association
      let create_data = TenantUserForCreate { tenant_id, user_id, status: Some(status) };
      Self::create(&mm, create_data).await?;
    }

    mm.dbx().commit_txn().await?;
    Ok(())
  }

  /// Unlink user from tenant
  pub async fn unlink_user_from_tenant(mm: &ModelManager, user_id: i64, tenant_id: i64) -> Result<u64, SqlError> {
    let filters = vec![
      TenantUserFilter { tenant_id: Some(OpValInt64::eq(tenant_id)), ..Default::default() },
      TenantUserFilter { user_id: Some(OpValInt64::eq(user_id)), ..Default::default() },
    ];

    base::delete::<Self, _>(mm, filters).await
  }

  /// Update tenant user status
  pub async fn update_tenant_user_status(
    mm: &ModelManager,
    user_id: i64,
    tenant_id: i64,
    status: TenantUserStatus,
  ) -> Result<u64, SqlError> {
    let filters = vec![
      TenantUserFilter { tenant_id: Some(OpValInt64::eq(tenant_id)), ..Default::default() },
      TenantUserFilter { user_id: Some(OpValInt64::eq(user_id)), ..Default::default() },
    ];

    let update_data = TenantUserForUpdate { status: Some(status) };

    base::update::<Self, _, _>(mm, filters, update_data).await
  }

  /// Check if user has active association with tenant
  pub async fn is_user_active_in_tenant(mm: &ModelManager, user_id: i64, tenant_id: i64) -> Result<bool, SqlError> {
    let filters = vec![
      TenantUserFilter { tenant_id: Some(OpValInt64::eq(tenant_id)), ..Default::default() },
      TenantUserFilter { user_id: Some(OpValInt64::eq(user_id)), ..Default::default() },
      TenantUserFilter { status: Some(OpValInt64::eq(TenantUserStatus::Active as i64)), ..Default::default() },
    ];

    let result = base::pg_find_first::<Self, TenantUser, _>(mm, filters).await?;
    Ok(result.is_some())
  }

  /// Count tenant users by query
  pub async fn count_by(mm: &ModelManager, req: UserForQuery) -> Result<u64, SqlError> {
    let count = base::count_on::<Self, _>(mm, |query| Self::make_select_statement(query, req)).await?;
    Ok(count)
  }

  /// Get user's active tenant count
  pub async fn get_user_active_tenant_count(mm: &ModelManager, user_id: i64) -> Result<i64, SqlError> {
    let req = UserForQuery {
      page: fusionsql::page::Page::default(),
      filters: vec![
        TenantUserFilter { user_id: Some(OpValInt64::eq(user_id)), ..Default::default() },
        TenantUserFilter { status: Some(OpValInt64::eq(TenantUserStatus::Active as i64)), ..Default::default() },
      ],
    };

    let count = Self::count_by(mm, req).await?;
    Ok(count as i64)
  }

  /// Make select statement for queries
  fn make_select_statement(stmt: &mut SelectStatement, req: UserForQuery) -> Result<(), SqlError> {
    // condition from filter
    let filters: FilterGroups = req.filters.into();
    let cond: Condition = filters.try_into()?;
    if !cond.is_empty() {
      stmt.cond_where(cond);
    }

    let list_options = compute_page(Self::_static_config(), Some(req.page))?;
    apply_to_sea_query(&list_options, stmt);

    Ok(())
  }

  /// Get user's active tenant associations
  pub async fn get_user_active_tenants(mm: &ModelManager, user_id: i64) -> Result<Vec<TenantUser>, SqlError> {
    let filters = vec![
      TenantUserFilter { user_id: Some(OpValInt64::eq(user_id)), ..Default::default() },
      TenantUserFilter { status: Some(OpValInt64::eq(TenantUserStatus::Active as i64)), ..Default::default() },
    ];

    base::pg_find_many::<Self, TenantUser, _>(mm, filters, None).await
  }

  /// Get user with tenant information for login
  pub async fn get_user_with_tenant(
    mm: &ModelManager,
    user_id: i64,
    tenant_id: i64,
  ) -> Result<Option<UserWithTenant>, SqlError> {
    let db = mm.dbx().db_postgres().map_err(SqlError::from)?;

    let query = format!(
      r#"
      SELECT
        u.id, u.email, u.phone, u.name, u.status, u.gender,
        tu.tenant_id, tu.status as tenant_status,
        u.created_by, u.created_at, u.updated_by, u.updated_at
      FROM {} u
      INNER JOIN {} tu ON u.id = tu.user_id
      WHERE u.id = $1 AND tu.tenant_id = $2
      LIMIT 1
      "#,
      TABLE_USER, TABLE_TENANT_USER
    );

    let mut rows = db
      .fetch_all(sqlx::query_as::<_, UserWithTenant>(&query).bind(user_id).bind(tenant_id))
      .await
      .map_err(SqlError::from)?;

    let result = rows.pop();

    Ok(result)
  }

  pub async fn query_tenant_user_changes(
    mm: &ModelManager,
    req: TenantUserChangeQueryReq,
  ) -> Result<TenantUserChangeQueryResp, SqlError> {
    let paged = base::pg_page::<Self, _, _>(mm, req.filters, req.page).await?;
    Ok(TenantUserChangeQueryResp { page: paged.page, result: paged.result })
  }
}
