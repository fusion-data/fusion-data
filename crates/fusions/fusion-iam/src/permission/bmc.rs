use modelsql::{
  ModelManager, Result,
  base::{self, DbBmc, compute_list_options},
  filter::{FilterGroups, ListOptions},
  generate_pg_bmc_common,
};
use sea_query::{Condition, Expr, Query, SelectStatement};
use ultimate_api::v1::{Page, PagePayload, Pagination};

use crate::role::role_permission::{RolePermissionBmc, RolePermissionIden};

use super::{Permission, PermissionFilters, PermissionForCreate, PermissionForUpdate, PermissionIden};

pub struct PermissionBmc;
impl DbBmc for PermissionBmc {
  const TABLE: &'static str = "permission";
}

generate_pg_bmc_common!(
  Bmc: PermissionBmc,
  Entity: Permission,
  ForCreate: PermissionForCreate,
  ForUpdate: PermissionForUpdate,
);

impl PermissionBmc {
  pub async fn page(
    mm: &ModelManager,
    filters: PermissionFilters,
    pagination: Pagination,
  ) -> Result<PagePayload<Permission>> {
    let total_size = Self::count(mm, filters.clone()).await?;
    let items = Self::find_many(mm, filters, Some((&pagination).into())).await?;
    Ok(PagePayload::new(Page::new(total_size), items))
  }

  pub async fn count(mm: &ModelManager, filters: PermissionFilters) -> Result<i64> {
    let count = base::count_on::<Self, _>(mm, |query| Self::make_select_statement(query, filters, None)).await?;
    Ok(count)
  }

  pub async fn find_many(
    mm: &ModelManager,
    filters: PermissionFilters,
    list_options: Option<ListOptions>,
  ) -> Result<Vec<Permission>> {
    let items =
      base::pg_find_many_on::<Self, _, _>(mm, |query| Self::make_select_statement(query, filters, list_options))
        .await?;
    Ok(items)
  }

  fn make_select_statement(
    query: &mut SelectStatement,
    filter: PermissionFilters,
    list_options: Option<ListOptions>,
  ) -> Result<()> {
    // condition from filter
    let filters: FilterGroups = filter.filter.into();
    let cond: Condition = filters.try_into()?;
    if !cond.is_empty() {
      query.cond_where(cond);
    }

    let sub_cond: Condition = filter.role_perm_filter.try_into()?;
    if !sub_cond.is_empty() {
      query.and_where(Expr::col(PermissionIden::Id).in_subquery({
        let mut q = Query::select();
        q.from(RolePermissionBmc::table_ref()).column(RolePermissionIden::PermissionId);
        q.cond_where(sub_cond);
        q
      }));
    }

    let list_options = compute_list_options::<Self>(list_options)?;
    list_options.apply_to_sea_query(query);

    Ok(())
  }
}
