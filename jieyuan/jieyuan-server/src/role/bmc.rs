use std::sync::OnceLock;

use fusionsql::page::{Page, PageResult};
use fusionsql::{
  ModelManager, Result,
  base::{self, BmcConfig, DbBmc, compute_page},
  filter::{FilterGroups, apply_to_sea_query},
  generate_pg_bmc_common,
};
use sea_query::{Condition, Expr, Query, SelectStatement};

use jieyuan_core::model::{CreateRoleDto, Role, RoleFilters, RoleForUpdate, RoleIden, RolePermissionIden};

use super::RolePermissionBmc;

pub struct RoleBmc;
impl DbBmc for RoleBmc {
  fn _bmc_config() -> &'static BmcConfig {
    static CONFIG: OnceLock<BmcConfig> = OnceLock::new();
    CONFIG.get_or_init(|| BmcConfig::new_table("iam_role").with_use_logical_deletion(true))
  }
}

generate_pg_bmc_common!(
  Bmc: RoleBmc,
  Entity: Role,
  ForCreate: CreateRoleDto,
  ForUpdate: RoleForUpdate,
);

impl RoleBmc {
  pub async fn page(mm: &ModelManager, filters: RoleFilters, pagination: Page) -> Result<PageResult<Role>> {
    let total_size = Self::count(mm, filters.clone()).await?;
    let items = Self::find_many(mm, filters, Some(pagination)).await?;
    Ok(PageResult::new(total_size, items))
  }

  async fn count(mm: &ModelManager, filters: RoleFilters) -> Result<u64> {
    let count = base::count_on::<Self, _>(mm, move |query| Self::select_statement(query, filters, None)).await?;
    Ok(count)
  }

  async fn find_many(mm: &ModelManager, filters: RoleFilters, list_options: Option<Page>) -> Result<Vec<Role>> {
    let items =
      base::pg_find_many_on::<Self, Role, _>(mm, |query| Self::select_statement(query, filters, list_options)).await?;
    Ok(items)
  }

  fn select_statement(query: &mut SelectStatement, filters: RoleFilters, list_options: Option<Page>) -> Result<()> {
    // condition from filter
    {
      let group: FilterGroups = filters.filter.into();
      let cond: Condition = group.try_into()?;
      query.cond_where(cond);
    }

    {
      let sub_cond: Condition = filters.role_perm_filter.try_into()?;
      if !sub_cond.is_empty() {
        query.and_where(Expr::col(RoleIden::Id).in_subquery({
          let mut q = Query::select();
          q.from(RolePermissionBmc::_bmc_config().table_ref()).column(RolePermissionIden::RoleId);
          q.cond_where(sub_cond);
          q
        }));
      }
    }

    let list_options = compute_page(RoleBmc::_bmc_config(), list_options)?;
    apply_to_sea_query(&list_options, query);

    Ok(())
  }
}
