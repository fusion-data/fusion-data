use std::sync::OnceLock;

use fusionsql::{
  ModelManager, Result,
  base::{self, BmcConfig, DbBmc, compute_page},
  filter::{FilterGroups, apply_to_sea_query},
  generate_pg_bmc_common, generate_pg_bmc_filter,
};
use sea_query::{Condition, Expr, Query, SelectStatement};

use jieyuan_core::model::{
  Permission, PermissionFilter, PermissionForCreate, PermissionForPage, PermissionForUpdate, PermissionIden,
  RolePermissionIden, TABLE_PERMISSION,
};

use crate::role::RolePermissionBmc;

pub struct PermissionBmc;
impl DbBmc for PermissionBmc {
  fn _static_config() -> &'static BmcConfig {
    static CONFIG: OnceLock<BmcConfig> = OnceLock::new();
    CONFIG.get_or_init(|| BmcConfig::new_table(TABLE_PERMISSION).with_use_logical_deletion(true))
  }
}

generate_pg_bmc_common!(
  Bmc: PermissionBmc,
  Entity: Permission,
  ForCreate: PermissionForCreate,
  ForUpdate: PermissionForUpdate,
);
generate_pg_bmc_filter!(
  Bmc: PermissionBmc,
  Entity: Permission,
  Filter: PermissionFilter,
);

impl PermissionBmc {
  pub async fn count_by(mm: &ModelManager, req: PermissionForPage) -> Result<u64> {
    let count = base::count_on::<Self, _>(mm, |query| Self::make_select_statement(query, req)).await?;
    Ok(count)
  }

  fn make_select_statement(stmt: &mut SelectStatement, req: PermissionForPage) -> Result<()> {
    // condition from filter
    let filters: FilterGroups = req.filters.into();
    let cond: Condition = filters.try_into()?;
    if !cond.is_empty() {
      stmt.cond_where(cond);
    }

    let sub_cond: Condition = req.role_perm_filter.try_into()?;
    if !sub_cond.is_empty() {
      stmt.and_where(Expr::col(PermissionIden::Id).in_subquery({
        let mut q = Query::select();
        q.from(RolePermissionBmc::_static_config().table_ref()).column(RolePermissionIden::PermissionId);
        q.cond_where(sub_cond);
        q
      }));
    }

    let list_options = compute_page(Self::_static_config(), Some(req.page))?;
    apply_to_sea_query(&list_options, stmt);

    Ok(())
  }
}
