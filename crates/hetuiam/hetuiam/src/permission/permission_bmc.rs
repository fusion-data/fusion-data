use modelsql::{
  ModelManager, Result,
  base::{self, DbBmc, compute_page},
  filter::FilterGroups,
  generate_pg_bmc_common, generate_pg_bmc_filter,
};
use sea_query::{Condition, Expr, Query, SelectStatement};

use hetuiam_core::{
  infra::tables::TABLE_PERMISSION,
  types::{
    Permission, PermissionFilter, PermissionForCreate, PermissionForPage, PermissionForUpdate, PermissionIden,
    RolePermissionIden,
  },
};

use crate::role::RolePermissionBmc;

pub struct PermissionBmc;
impl DbBmc for PermissionBmc {
  const TABLE: &'static str = TABLE_PERMISSION;
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
        q.from(RolePermissionBmc::table_ref()).column(RolePermissionIden::PermissionId);
        q.cond_where(sub_cond);
        q
      }));
    }

    let list_options = compute_page::<Self>(Some(req.page))?;
    list_options.apply_to_sea_query(stmt);

    Ok(())
  }
}
